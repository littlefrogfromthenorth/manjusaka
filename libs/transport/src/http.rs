use super::Transport1;
use std::time::Duration;
use parking_lot::Mutex;

const REQ_TIME_OUT:u64 = 60;


static CLIENT: once_cell::sync::Lazy<Mutex<reqwest::blocking::Client>> = once_cell::sync::Lazy::new(|| {
    let client = reqwest::blocking::Client::builder()
        .tcp_keepalive(Duration::from_secs(REQ_TIME_OUT))
        .timeout(Duration::from_secs(REQ_TIME_OUT))
        .cookie_store(true)
        .build()
        .unwrap();
    Mutex::new(client)
});

#[derive(Debug,Clone)]
pub struct HttpTransport {
    baseurl: String,
}

impl HttpTransport {
    pub fn new (baseurl: &str, _c: &str) -> anyhow::Result<Self> {
        Ok(Self{
            baseurl:baseurl.to_string(), 
        })
    }
}

impl Transport1 for HttpTransport{
    fn proxy(&self, proxy: &str) -> anyhow::Result<()>{
        let proxy = reqwest::Proxy::all(proxy)?;
        let client = reqwest::blocking::Client::builder()
            .tcp_keepalive(Duration::from_secs(REQ_TIME_OUT))
            .timeout(Duration::from_secs(REQ_TIME_OUT))
            .cookie_store(true)
            .proxy(proxy)
            .build()?;
        *CLIENT.lock() = client;
        Ok(())
    }

    fn send(&self, _host:&str, data:Option<Vec<u8>>) -> anyhow::Result<Vec<u8>>{
        let client = CLIENT.lock();
        let res = if let Some(s) = data {
            client.post(self.baseurl.clone()).body(s)
        }else{
            client.get(self.baseurl.clone())
        }.send()?;

        Ok(res.bytes()?.to_vec())
    }
}