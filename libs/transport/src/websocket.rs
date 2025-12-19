
use super::Transport;
use super::NoCertificateVerification;
use crate::config::UserAddr;

use async_trait::async_trait;
use bytes::Bytes;
use futures::{Sink, Stream};

use tokio_util::io::StreamReader;
use tokio::io::{AsyncBufRead, AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::{TcpListener, TcpSocket, TcpStream, ToSocketAddrs};
use tokio_rustls::rustls::pki_types::ServerName;
use tokio_rustls::rustls::{ClientConfig, RootCertStore};
use tokio_rustls::{TlsConnector, TlsStream};
use tokio_tungstenite::tungstenite::protocol::{Message,Role};
use tokio_tungstenite::{
    accept_async_with_config, 
    client_async_with_config, 
    MaybeTlsStream, 
    WebSocketStream
};
use tokio_tungstenite::tungstenite::handshake::client::{generate_key, Request as WscRequest};

pub use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;

use url::Url;
use core::result::Result;
use std::convert::TryFrom;
use std::io::{Error, ErrorKind};
use std::net::{IpAddr, SocketAddr};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{ready, Context, Poll};


#[derive(Debug)]
struct StreamWrapper {
    inner: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl Stream for StreamWrapper {
    type Item = Result<Bytes, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.get_mut().inner).poll_next(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Ready(Some(Err(err))) => {
                Poll::Ready(Some(Err(Error::new(ErrorKind::Other, err))))
            }
            Poll::Ready(Some(Ok(res))) => {
                if let Message::Binary(b) = res {
                    Poll::Ready(Some(Ok(Bytes::from(b))))
                } else {
                    Poll::Ready(Some(Err(Error::new(
                        ErrorKind::InvalidData,
                        "unexpected frame",
                    ))))
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

#[derive(Debug)]
pub struct WebsocketTunnel {
    inner: StreamReader<StreamWrapper, Bytes>,
}

impl AsyncRead for WebsocketTunnel {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.get_mut().inner).poll_read(cx, buf)
    }
}

impl AsyncBufRead for WebsocketTunnel {
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<&[u8]>> {
        Pin::new(&mut self.get_mut().inner).poll_fill_buf(cx)
    }

    fn consume(self: Pin<&mut Self>, amt: usize) {
        Pin::new(&mut self.get_mut().inner).consume(amt)
    }
}

impl AsyncWrite for WebsocketTunnel {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        let sw = self.get_mut().inner.get_mut();
        ready!(Pin::new(&mut sw.inner)
            .poll_ready(cx)
            .map_err(|err| Error::new(ErrorKind::Other, err)))?;

        match Pin::new(&mut sw.inner).start_send(Message::Binary(buf.to_vec().into())) {
            Ok(()) => Poll::Ready(Ok(buf.len())),
            Err(e) => Poll::Ready(Err(Error::new(ErrorKind::Other, e))),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Pin::new(&mut self.get_mut().inner.get_mut().inner)
            .poll_flush(cx)
            .map_err(|err| Error::new(ErrorKind::Other, err))
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Pin::new(&mut self.get_mut().inner.get_mut().inner)
            .poll_close(cx)
            .map_err(|err| Error::new(ErrorKind::Other, err))
    }
}

#[derive(Debug)]
pub struct WebsocketTransport {
    config: WebSocketConfig,
}

#[async_trait]
impl Transport for WebsocketTransport {
    type Acceptor = TcpListener;
    type Stream = WebsocketTunnel;

    fn new() -> Self {
        
        Self{
            config: WebSocketConfig::default()
        }
    }

    async fn listen(
        &self, 
        addr: &SocketAddr
    ) -> anyhow::Result<Self::Acceptor> {
        Ok(TcpListener::bind(addr).await?)
    }

    async fn accept(
        &mut self,
        a: &mut Self::Acceptor,
    ) -> anyhow::Result<(Self::Stream, SocketAddr)> {
        let (tcp,addr) = a.accept().await?;
        let conn = MaybeTlsStream::Plain(tcp);

        let wsstream = accept_async_with_config(conn, Some(self.config)).await?;

        let stream = WebsocketTunnel {
            inner: StreamReader::new(
                StreamWrapper { 
                    inner: wsstream
                }
            ),
        };
        Ok((stream,addr))
    }

    async fn dial(&self, addr: &UserAddr) -> anyhow::Result<Self::Stream> {
        
        let mut req = WscRequest::builder();
        req = req.header("Host", addr.host_str());
        req = req.header("Connection", "Upgrade");
        req = req.header("Upgrade", "websocket");
        req = req.header("Sec-WebSocket-Version", "13");
        req = req.header("Sec-WebSocket-Key", generate_key());

        let saddr = addr.get_socketaddr()?;
        let tcp = TcpStream::connect(saddr).await?;
        let wsstream = if addr.is_tls() {
            let mut config = ClientConfig::builder()
                .with_root_certificates(RootCertStore::empty())
                .with_no_client_auth(); 
            config //忽略证书校验 
                .dangerous()
                .set_certificate_verifier(Arc::new(NoCertificateVerification {}));
            let connector = TlsConnector::from(Arc::new(config));
            let dnsname = ServerName::try_from(addr.host_str())?.to_owned();
            let tls = connector.connect(dnsname, tcp).await?;
            MaybeTlsStream::Rustls(tls)
        }else{
            MaybeTlsStream::Plain(tcp)
        };

        let req = req.method("GET").uri(&addr.0.to_string()).body(())?;

        let (websocketstream, response) =
            client_async_with_config(req, wsstream, Some(self.config)).await?;
        log::info!("ws response {:?}:", response);

        let stream = WebsocketTunnel {
            inner: StreamReader::new(
                StreamWrapper { 
                    inner: websocketstream 
                }
            ),
        };
        Ok(stream)
    }

}
