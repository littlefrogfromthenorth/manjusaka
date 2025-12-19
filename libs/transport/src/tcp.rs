use super::Transport;

use crate::config::UserAddr;

use anyhow::Result;
use async_trait::async_trait;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio_rustls::{TlsConnector, TlsStream};
use tokio_rustls::rustls::{pki_types, ClientConfig, RootCertStore,DigitallySignedStruct, Error, SignatureScheme};
use tokio_rustls::rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};

use std::net::SocketAddr;
use std::sync::Arc;
use std::convert::TryFrom;

#[derive(Debug,Clone)]
pub struct TcpConfig {
    pub nodelay: bool,
    pub proxy: Option<String>,
}

impl Default for TcpConfig {
    fn default() 
    -> Self {
        Self {
            nodelay: true,
            proxy: None,
        }
    }
}


#[derive(Debug,Clone)]
pub struct TcpTransport {
    pub config: TcpConfig,
}

#[async_trait]
impl Transport for TcpTransport {
    type Acceptor = TcpListener;
    type Stream = TcpOrTlsStream;

    fn new()
    -> Self {
        Self {
            config: TcpConfig::default(),
        }
    }

    async fn listen(
        &self, 
        addr: &SocketAddr
    ) -> Result<Self::Acceptor> {
        Ok(TcpListener::bind(addr).await?)
    }

    async fn accept(
        &mut self, 
        a: &mut Self::Acceptor
    ) -> Result<(Self::Stream, SocketAddr)> {
        let (tcp,addr) = a.accept().await?;
        Ok((TcpOrTlsStream::Tcp(tcp),addr)) // todo tls未实现
    }

    async fn dial(
        &self, 
        addr: &UserAddr, 
    ) -> Result<Self::Stream> {
        let saddr = addr.get_socketaddr()?;

        let tcp = if let Some(_proxy) = &self.config.proxy{
            TcpStream::connect(saddr).await? // todo tcp 代理
        }else{
            TcpStream::connect(saddr).await?  
        };

        tcp.set_nodelay(self.config.nodelay)?;

        let stream = if addr.is_tls() {
            let mut config = ClientConfig::builder()
                .with_root_certificates(RootCertStore::empty())
                .with_no_client_auth(); 

            config //忽略证书校验 
                .dangerous()
                .set_certificate_verifier(Arc::new(NoCertificateVerification {}));

            let connector = TlsConnector::from(Arc::new(config));

            let dnsname = pki_types::ServerName::try_from(addr.host_str())?.to_owned();

            let tls = TlsStream::Client(connector.connect(dnsname, tcp).await?);
            
            TcpOrTlsStream::Tls(tls)
        }else{
            TcpOrTlsStream::Tcp(tcp)
        };
        Ok(stream)
    }
}


#[derive(Debug)]
pub enum TcpOrTlsStream {
    Tcp(TcpStream),
    Tls(TlsStream<TcpStream>),
}

impl AsyncRead for TcpOrTlsStream {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        ctx: &mut std::task::Context<'_>,
        buf: &mut ReadBuf<'_>) 
    -> std::task::Poll<std::io::Result<()>> {
        match self.get_mut() {
            TcpOrTlsStream::Tcp(s) => std::pin::Pin::new(s).poll_read(ctx, buf),
            TcpOrTlsStream::Tls(s) => std::pin::Pin::new(s).poll_read(ctx, buf),
        }
    }
}

impl AsyncWrite for TcpOrTlsStream {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        ctx: &mut std::task::Context<'_>,
        buf: &[u8]) 
    -> std::task::Poll<core::result::Result<usize, std::io::Error>> {
        match self.get_mut() {
            TcpOrTlsStream::Tcp(s) => std::pin::Pin::new(s).poll_write(ctx, buf),
            TcpOrTlsStream::Tls(s) => std::pin::Pin::new(s).poll_write(ctx, buf),
        }
    }

    fn poll_flush(self: std::pin::Pin<&mut Self>, ctx: &mut std::task::Context<'_>) 
    -> std::task::Poll<core::result::Result<(), std::io::Error>> {
        match self.get_mut() {
            TcpOrTlsStream::Tcp(s) => std::pin::Pin::new(s).poll_flush(ctx),
            TcpOrTlsStream::Tls(s) => std::pin::Pin::new(s).poll_flush(ctx),
        }
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        ctx: &mut std::task::Context<'_>) 
    -> std::task::Poll<core::result::Result<(), std::io::Error>> {
        match self.get_mut() {
            TcpOrTlsStream::Tcp(s) => std::pin::Pin::new(s).poll_shutdown(ctx),
            TcpOrTlsStream::Tls(s) => std::pin::Pin::new(s).poll_shutdown(ctx),
        }
    }
}


#[derive(Debug)]
pub struct NoCertificateVerification {}

impl ServerCertVerifier for NoCertificateVerification {
    fn verify_server_cert(
        &self,
        _: &pki_types::CertificateDer<'_>,
        _: &[pki_types::CertificateDer<'_>],
        _: &pki_types::ServerName<'_>,
        _: &[u8],
        _: pki_types::UnixTime) 
    -> Result<ServerCertVerified, Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _: &[u8],
        _: &pki_types::CertificateDer<'_>,
        _: &DigitallySignedStruct) 
    -> Result<HandshakeSignatureValid, Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _: &[u8],
        _: &pki_types::CertificateDer<'_>,
        _: &DigitallySignedStruct)
    -> Result<HandshakeSignatureValid, Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) 
    -> Vec<SignatureScheme> {
        vec![
            SignatureScheme::RSA_PKCS1_SHA1,
            SignatureScheme::ECDSA_SHA1_Legacy,
            SignatureScheme::RSA_PKCS1_SHA256,
            SignatureScheme::ECDSA_NISTP256_SHA256,
            SignatureScheme::RSA_PKCS1_SHA384,
            SignatureScheme::ECDSA_NISTP384_SHA384,
            SignatureScheme::RSA_PKCS1_SHA512,
            SignatureScheme::ECDSA_NISTP521_SHA512,
            SignatureScheme::RSA_PSS_SHA256,
            SignatureScheme::RSA_PSS_SHA384,
            SignatureScheme::RSA_PSS_SHA512,
            SignatureScheme::ED25519,
            SignatureScheme::ED448,
        ]
    }
}

