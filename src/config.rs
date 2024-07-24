use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    path::PathBuf,
};

use data_encoding::BASE64;
use rand::RngCore as _;

use crate::error::{Error, Result};

fn default_public_address() -> SocketAddr {
    SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 3000))
}

fn default_private_address() -> SocketAddr {
    SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 3001))
}

fn default_key() -> String {
    let mut key = [0; 64];

    rand::rngs::OsRng.fill_bytes(&mut key);

    BASE64.encode(&key)
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Config {
    #[serde(rename = "public-address", default = "default_public_address")]
    pub public_address: SocketAddr,
    #[serde(rename = "private-address", default = "default_private_address")]
    pub private_address: SocketAddr,
    #[serde(rename = "cookie-key", default = "default_key")]
    pub cookie_key: String,
    #[serde(rename = "session-key", default = "default_key")]
    pub session_key: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            public_address: default_public_address(),
            private_address: default_private_address(),
            cookie_key: default_key(),
            session_key: default_key(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = PathBuf::from("pod-sync.toml");

        if !path.exists() {
            let config = Self::default();
            let content = toml::to_string_pretty(&config)?;

            std::fs::write(&path, content)?;

            return Ok(config);
        }

        let content = std::fs::read_to_string(&path)?;
        let config = toml::from_str(&content)?;

        let content = toml::to_string_pretty(&config)?;
        std::fs::write(&path, content)?;

        Ok(config)
    }

    pub fn cookie_key(&self) -> Result<Vec<u8>> {
        BASE64
            .decode(self.cookie_key.as_bytes())
            .map_err(Error::from)
    }
}
