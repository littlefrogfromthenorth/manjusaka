
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;
use sha1::{Sha1,Digest as _};
use sha2::Sha256;
use base64::{engine::general_purpose, Engine as _};
use chrono::Local;


use std::time::{SystemTime, UNIX_EPOCH};
use std::net::UdpSocket;


pub fn pid() -> String {
    std::process::id().to_string()
}

pub fn username() -> String {
    whoami::username().trim_end_matches('\0').to_owned()
}

pub fn hostname() -> String {
    whoami::fallible::hostname().unwrap_or("unknown".to_string())
}

pub fn uuid() -> String {
    str::replace(&format!("{:?}", uuid::Uuid::new_v4()), "-", "").to_uppercase()
}

pub fn randstr(n: u16) -> String {
    let mut rng = thread_rng();
    (0..n).map(|_| rng.sample(Alphanumeric) as char).collect()
}

pub fn getstr(n: u16) -> String {
    if n > 9999 {
        return format!("{}",n)
    }
    loop{
        let s = randstr(6);
        let mut p = 1u64;
        for c in s.chars() {
            p = (p * (c as u64)) % 10000;
        }
        if p == n as u64 {
            return s;
        }
    }
}

pub fn getstrmod(s: &str) -> u16 {
    let mut p = 1u64;
    for c in s.chars() {
        p = (p * (c as u64)) % 10000;
    }
    p as u16
}

pub fn sha1(s: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(s);
    format!("{:x}",hasher.finalize())
}

pub fn sha256(s: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(s);
    format!("{:x}",hasher.finalize())
}

pub fn get_local_ipaddr() -> String {
    let mut default = "127.0.0.1".to_string();
    if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
        if let Ok(_) = socket.connect("8.8.8.8:53") {
            if let Ok(addr) = socket.local_addr() {
                default = addr.ip().to_string();
            }
        }
    }
    default
}

pub fn b64encode(s: &[u8]) -> String {
    general_purpose::STANDARD.encode(s)
}

pub fn b64decode(s: &str) -> anyhow::Result<Vec<u8>> {
    Ok(general_purpose::STANDARD.decode(s)?)
}

pub fn timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as i64
}

pub fn format_timestamp() -> String {
    let local_time = Local::now();
    local_time.format("%Y-%m-%d-%H-%M-%S").to_string()
}


// 简单的字节数组替换函数
pub fn replace_bytes(data: &[u8], pattern: &[u8], replacement: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    let mut i = 0;
    
    while i < data.len() {
        if i + pattern.len() <= data.len() && &data[i..i + pattern.len()] == pattern {
            result.extend_from_slice(replacement);
            i += pattern.len();
        } else {
            result.push(data[i]);
            i += 1;
        }
    }
    
    result
}