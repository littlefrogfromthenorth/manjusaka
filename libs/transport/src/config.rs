use url::Url;
use serde::Deserialize;
use serde::Deserializer;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use anyhow::Context;

#[derive(Debug, Clone)]
pub struct UserAddr(pub Url);

impl UserAddr{
    pub fn is_tls(&self) 
    -> bool {
        match self.0.scheme().to_lowercase().as_str(){
            "https"|"wss"|"tls" => true,
            _ => false,
        }
    }

    pub fn host_str(&self) 
    -> &str {
        let host_str = self.0.host_str().unwrap(); // 调用的地方都是客户端 暂时没大问题
        host_str
    }

    pub fn get_socketaddr(&self) 
    -> anyhow::Result<SocketAddr>{
        let port = if let Some(port) = self.0.port() {
            port
        } else {
            if self.is_tls(){443}else{80}
        };
        to_addr((self.host_str(),port))
    }
}

impl From<String> for UserAddr {
    fn from(s: String) 
    -> UserAddr {
        let uri = if s.contains("://"){
            Url::parse(&s).expect("地址错误")
        }else{
            Url::parse(&format!("http://{}",s)).expect("地址错误")
        };
        UserAddr(uri)
    }
}

impl<'de> Deserialize<'de> for UserAddr {
    fn deserialize<D>(deserializer: D) 
    -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(UserAddr::from(String::deserialize(deserializer)?))
    }
}

pub fn to_addr<S: ToSocketAddrs>(addr: S) 
-> anyhow::Result<SocketAddr> {
    let addr = addr
        .to_socket_addrs()?
        .next()
        .with_context(|| "转换地址错误")?;
    Ok(addr)
}