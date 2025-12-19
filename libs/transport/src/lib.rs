mod http;
pub use http::HttpTransport;

use tokio::io::{AsyncRead, AsyncWrite};
use std::fmt::Debug;
use std::net::SocketAddr;


pub trait Transport1: Debug + Send + Sync {

    fn proxy(&self, proxy: &str) -> anyhow::Result<()>;

    fn send(&self, headers: &str, data: Option<Vec<u8>>) -> anyhow::Result<Vec<u8>>;
}


#[async_trait::async_trait]
pub trait Transport2: Debug + Send + Sync {

    type Acceptor: Send + Sync;

    type Stream: 'static + AsyncRead + AsyncWrite + Unpin + Send + Sync;

    fn new() -> Self where Self: Sized;

    async fn listen(&self, addr: &str) -> anyhow::Result<Self::Acceptor>;

    async fn accept(&mut self, a: &mut Self::Acceptor) -> anyhow::Result<(Self::Stream, SocketAddr)>;
    
    async fn dial(&self,addr: &str) -> anyhow::Result<Self::Stream>;

}
