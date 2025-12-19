use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Nonce
};
use flate2::{read::DeflateDecoder, write::DeflateEncoder, Compression};
use anyhow::Result;
use std::io::{Read, Write};


#[derive(Clone)]
pub struct AesCrypt {
    cipher: Aes256Gcm,
}

impl AesCrypt {
    pub fn new(key: &str) -> Result<AesCrypt> {
        let mut key_bytes = key.as_bytes().to_vec();
        if key_bytes.len() < 32 {
            key_bytes.resize(32, 0);
        } else if key_bytes.len() > 32 {
            key_bytes.truncate(32);
        }
        let keya = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(keya);

        Ok(AesCrypt { cipher })
    }

    pub fn encrypt(&self, data: &[u8]) -> anyhow::Result<Vec<u8>> {
        let mut compressor = DeflateEncoder::new(Vec::new(), Compression::default());
        compressor.write_all(data)?;
        let compressed_data = compressor.finish()?;

        // Fixed: Use AeadCore trait's generate_nonce method
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng); // 12位

        // Fixed: Map aes_gcm::Error to anyhow::Error
        let ciphertext = self.cipher
            .encrypt(&nonce, compressed_data.as_ref())
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        let mut result = Vec::with_capacity(nonce.len() + ciphertext.len());
        result.extend_from_slice(&nonce);
        result.extend_from_slice(&ciphertext);

        // Fixed: Return the complete result (nonce + ciphertext), not just ciphertext
        Ok(result)
    }

    pub fn decrypt(&self, data: &[u8]) -> anyhow::Result<Vec<u8>> {
        // Ensure data is long enough to contain nonce
        if data.len() < 12 {
            return Err(anyhow::anyhow!("DE"));
        }

        let nonce = Nonce::from_slice(&data[..12]);
        let ciphertext = &data[12..];

        // Fixed: Map aes_gcm::Error to anyhow::Error
        let compressed_data = self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        let mut decompressor = DeflateDecoder::new(&compressed_data[..]);
        let mut result = Vec::new();
        decompressor.read_to_end(&mut result)?;

        Ok(result)
    }
}