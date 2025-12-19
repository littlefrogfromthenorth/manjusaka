use super::Transport;
use std::time::Duration;
use std::sync::Arc;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::time::{SystemTime, UNIX_EPOCH};

use parking_lot::Mutex;
use hmac::{Hmac, Mac};
use sha1::Sha1;

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

type HmacSha1 = Hmac<Sha1>;

pub struct AliyunOssClient {
    access_key_id: String,
    access_key_secret: String,
    endpoint: String,
    bucket: String,
}

impl AliyunOssClient {
    pub fn new(access_key_id: &str, access_key_secret: &str, endpoint: &str, bucket: &str) -> Self {
        AliyunOssClient {
            access_key_id: access_key_id.to_string(),
            access_key_secret: access_key_secret.to_string(),
            endpoint: endpoint.to_string(),
            bucket: bucket.to_string(),
        }
    }

    // 生成签名
    fn generate_signature(&self, method: &str, content_md5: &str, content_type: &str, date: &str, canonicalized_oss_headers: &str, canonicalized_resource: &str) -> String {
        let string_to_sign = format!("{}\n{}\n{}\n{}\n{}{}", 
            method, 
            content_md5, 
            content_type, 
            date, 
            canonicalized_oss_headers, 
            canonicalized_resource
        );

        let mut mac = HmacSha1::new_from_slice(self.access_key_secret.as_bytes()).expect("HMAC can be created");
        mac.update(string_to_sign.as_bytes());
        let result = mac.finalize();
        let code_bytes = result.into_bytes();
        base64::encode(&code_bytes)
    }

    // 上传文件
    pub async fn upload_file(&self, object_name: &str, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::open(file_path)?;
        let mut content = Vec::new();
        file.read_to_end(&mut content)?;

        let content_type = mime_guess::from_path(file_path).first_or_octet_stream().to_string();
        let date = chrono::Utc::now().to_rfc2822();
        let canonicalized_resource = format!("/{}/{}", self.bucket, object_name);
        
        // 计算Content-MD5
        use md5;
        let digest = md5::compute(&content);
        let content_md5 = base64::encode(&digest.0);

        let signature = self.generate_signature(
            "PUT", 
            &content_md5, 
            &content_type, 
            &date, 
            "", 
            &canonicalized_resource
        );

        let url = format!("https://{}.{}{}", self.bucket, self.endpoint, canonicalized_resource);

        let client = reqwest::Client::new();
        let response = client
            .put(&url)
            .header("Authorization", format!("OSS {}:{}", self.access_key_id, signature))
            .header("Content-Type", &content_type)
            .header("Content-MD5", &content_md5)
            .header("Date", &date)
            .body(content)
            .send()
            .await?;

        if response.status().is_success() {
            println!("文件上传成功: {}", object_name);
            Ok(())
        } else {
            let error_text = response.text().await?;
            Err(format!("上传失败: {}", error_text).into())
        }
    }

    // 下载文件
    pub async fn download_file(&self, object_name: &str, local_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let date = chrono::Utc::now().to_rfc2822();
        let canonicalized_resource = format!("/{}/{}", self.bucket, object_name);
        
        let signature = self.generate_signature(
            "GET", 
            "", 
            "", 
            &date, 
            "", 
            &canonicalized_resource
        );

        let url = format!("https://{}.{}{}", self.bucket, self.endpoint, canonicalized_resource);

        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header("Authorization", format!("OSS {}:{}", self.access_key_id, signature))
            .header("Date", &date)
            .send()
            .await?;

        if response.status().is_success() {
            let content = response.bytes().await?;
            let mut file = File::create(local_path)?;
            file.write_all(&content)?;
            println!("文件下载成功: {}", local_path);
            Ok(())
        } else {
            let error_text = response.text().await?;
            Err(format!("下载失败: {}", error_text).into())
        }
    }

    // 列出文件
    pub async fn list_objects(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let date = chrono::Utc::now().to_rfc2822();
        let canonicalized_resource = format!("/{}/", self.bucket);
        
        let signature = self.generate_signature(
            "GET", 
            "", 
            "", 
            &date, 
            "", 
            &canonicalized_resource
        );

        let url = format!("https://{}.{}{}", self.bucket, self.endpoint, canonicalized_resource);

        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header("Authorization", format!("OSS {}:{}", self.access_key_id, signature))
            .header("Date", &date)
            .send()
            .await?;

        if response.status().is_success() {
            let xml_content = response.text().await?;
            // 简单解析XML响应
            let mut objects = Vec::new();
            for line in xml_content.lines() {
                if line.trim().starts_with("<Key>") && line.trim().ends_with("</Key>") {
                    let object_name = line.trim()
                        .strip_prefix("<Key>")
                        .unwrap()
                        .strip_suffix("</Key>")
                        .unwrap();
                    objects.push(object_name.to_string());
                }
            }
            Ok(objects)
        } else {
            let error_text = response.text().await?;
            Err(format!("列出文件失败: {}", error_text).into())
        }
    }
}


#[derive(Debug,Clone)]
pub struct OssTransport {
    baseurl: String,
}

impl OssTransport {
    pub fn new (baseurl:&str) -> anyhow::Result<Self> {
        Ok(Self{
            baseurl:baseurl.to_string(), 
        })
    }
}

impl Transport for OssTransport{
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

    fn send(&self, _action:u16, data:Option<Vec<u8>>) -> anyhow::Result<Vec<u8>>{
        let client = CLIENT.lock();
        let res = if let Some(s) = data {
            client.post(self.baseurl.clone()).body(s)
        }else{
            client.get(self.baseurl.clone())
        }.send()?;

        Ok(res.bytes()?.to_vec())
    }
}