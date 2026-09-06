use std::{net::SocketAddr, path};

use serde::{Deserialize, Serialize};
use tokio::{
    fs,
    io::{self, AsyncWriteExt as _},
};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub admin: AdminConfig,
}

#[derive(Serialize, Deserialize)]
pub struct GeneralConfig {
    /// The socket address to bind to.
    ///
    /// A socket address must be of format
    /// <ip-address>:<port>. IPv6 address must be
    /// enclosed in square brackets ('[', ']').
    pub address: SocketAddr,
}

#[derive(Serialize, Deserialize)]
pub struct AdminConfig {
    /// The username for the admin account.
    pub username: String,
    /// The password for the admin account.
    pub password: String,
}

impl Config {
    /// Attempts to load a configuration file from a given path.
    pub async fn load<P: AsRef<path::Path>>(path: P) -> Result<Self, tokio::io::Error> {
        let toml_str = fs::read_to_string(path).await?;
        let config: Config = toml::from_str(&toml_str)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.message()))?;

        Ok(config)
    }

    /// Write the default configuration to a path.
    pub async fn write_default<P: AsRef<path::Path>>(path: P) -> Result<(), tokio::io::Error> {
        let mut file = fs::File::create(path).await?;
        file.write_all(DEFAULT_CONFIG.as_bytes()).await?;
        Ok(())
    }
}

const DEFAULT_CONFIG: &str = include_str!("../default_config.toml");
