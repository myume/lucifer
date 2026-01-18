use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub proxy: ProxyConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub port: Option<u16>,
    pub nameservers: Vec<String>,
}
