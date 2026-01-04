mod config;
pub use config::UserAddr;

#[cfg(feature = "http")]
mod http;
#[cfg(feature = "http")]
pub use http::HttpTransport;

#[cfg(feature = "tcp")]
mod tcp;
#[cfg(feature = "tcp")]
pub use tcp::TcpTransport;

#[cfg(feature = "kcp")]
mod kcp;
#[cfg(feature = "kcp")]
pub use kcp::KcpTransport;

#[cfg(feature = "ws")]
mod websocket;
#[cfg(feature = "ws")]
pub use websocket::WebsocketTransport;



use std::fmt::Debug;
use std::net::SocketAddr;

use tokio::io::{AsyncRead, AsyncWrite};
use tokio_rustls::rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use tokio_rustls::rustls::{pki_types,DigitallySignedStruct, Error, SignatureScheme};


pub trait Transport1: Debug + Send + Sync {

    fn proxy(&self, proxy: &str) -> anyhow::Result<()>;

    fn send(&self, headers: &str, data: Option<Vec<u8>>) -> anyhow::Result<Vec<u8>>;
    
}


#[async_trait::async_trait]
pub trait Transport2: Debug + Send + Sync {

    type Acceptor: Send + Sync;

    type Stream: 'static + AsyncRead + AsyncWrite + Unpin + Send + Sync;

    fn new() -> Self where Self: Sized;

    async fn listen(&self, addr: &SocketAddr) -> anyhow::Result<Self::Acceptor>;

    async fn accept(&mut self, a: &mut Self::Acceptor) -> anyhow::Result<(Self::Stream, SocketAddr)>;
    
    async fn dial(&self,addr: &UserAddr) -> anyhow::Result<Self::Stream>;

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

