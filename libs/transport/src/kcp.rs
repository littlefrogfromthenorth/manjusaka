use super::Transport;
use crate::config::UserAddr;

use anyhow::Result;
use async_trait::async_trait;
use tokio_kcp::{KcpListener, KcpStream};
pub use tokio_kcp::{KcpConfig, KcpNoDelayConfig};

use std::net::SocketAddr;

#[derive(Debug,Clone)]
pub struct KcpTransport {
    pub config: KcpConfig
}

#[async_trait]
impl Transport for KcpTransport {
    type Acceptor = KcpListener;
    type Stream = KcpStream;

    fn new() 
    -> Self {
        Self {
            config: KcpConfig {
                nodelay: KcpNoDelayConfig::fastest(),
                ..KcpConfig::default()
            }
        }
    }

    async fn listen(&self, addr: &SocketAddr) 
    -> Result<Self::Acceptor> {
        Ok(KcpListener::bind(self.config, addr).await?)
    }

    async fn accept(&mut self, a: &mut Self::Acceptor) 
    -> Result<(Self::Stream, SocketAddr)> {
        Ok(a.accept().await?)
    }

    async fn dial(
        &self, 
        addr: &UserAddr, 
    ) -> Result<Self::Stream> {
        let saddr = addr.get_socketaddr()?;
        let stream = KcpStream::connect(&self.config, saddr).await?;
        Ok(stream)
    }
}
