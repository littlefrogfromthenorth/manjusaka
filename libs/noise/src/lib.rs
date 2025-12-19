use anyhow::Result;
use tokio::io::{AsyncRead, AsyncWrite};
use rand::{rngs::StdRng, SeedableRng, RngCore, CryptoRng};
use snowstorm::{Builder, NoiseParams, NoiseStream};
use sha1::{Sha1, Digest};


pub fn calc_keyptr(key: &[u8]) -> u64 {
    let mut sh = Sha1::default();
    sh.update(key);
    let hash = sh.finalize(); //20位
      ((hash[0] as u64) << 0)
    | ((hash[1] as u64) << 8)
    | ((hash[2] as u64) << 16)
    | ((hash[3] as u64) << 24)
    | ((hash[4] as u64) << 32)
    | ((hash[5] as u64) << 40)
    | ((hash[6] as u64) << 48)
    | ((hash[7] as u64) << 56)
}

pub struct PublicKey([u8; 32]);
impl PublicKey {
    fn to_bytes(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

pub struct PrivateKey([u8; 32]);
impl PrivateKey {
    pub fn public_key(&self) -> PublicKey {
        let private_key: x25519_dalek::StaticSecret = self.0.into();
        let public_key: x25519_dalek::PublicKey = (&private_key).into();
        PublicKey(public_key.as_bytes().to_owned())
    }
    pub fn for_test(rng: &mut (impl RngCore + CryptoRng)) -> Self {
        Self(x25519_dalek::StaticSecret::new(rng).to_bytes())
    }
}

impl PrivateKey {
    fn to_bytes(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

#[derive(Clone, Debug)]
pub struct NoiseConfig {
    parameters: NoiseParams,
    private_key: Vec<u8>,
    public_key: Vec<u8>,
}

impl NoiseConfig {

    pub fn new(key: &[u8], noiseparams: &str) -> Result<Self> {
        let mut rng = StdRng::seed_from_u64(calc_keyptr(key));
        let private_key = PrivateKey::for_test(&mut rng);
        let public_key = private_key.public_key();
        let parameters: NoiseParams = noiseparams.parse()?;

        log::info!("private_key {:?}",private_key.to_bytes());
        
        Ok(Self {
            parameters,
            private_key: private_key.to_bytes(),
            public_key: public_key.to_bytes(),
        })
    }

    pub fn set_public_key(&mut self, key: &[u8]) 
    -> &mut Self {
        self.public_key = key.to_vec();
        self
    }

    fn builder(&self) -> Builder<'_> {
        let builder = Builder::new(self.parameters.clone())
            .local_private_key(&self.private_key)
            .remote_public_key(&self.public_key);
        builder
    }

//应答
    pub async fn responder<T>(&self, socket: T) 
    -> Result<NoiseStream<T>>
    where
        T: AsyncRead + AsyncWrite + Unpin,
    {
        let noise_stream =
            NoiseStream::handshake(
                socket, 
                self.builder().build_responder()?
            ).await?;
        Ok(noise_stream)
    }

// 发起
    pub async fn initiator<T>(&self, socket: T) 
    -> Result<NoiseStream<T>>
    where
        T: AsyncRead + AsyncWrite + Unpin,
    {
        let noise_stream =
            NoiseStream::handshake(
                socket, 
                self.builder().build_initiator()?
            ).await?;
        Ok(noise_stream)
    }

}
