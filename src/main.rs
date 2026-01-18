use anyhow::Context;
use tokio::fs::read_to_string;

use crate::{config::Config, proxy::Proxy};

mod config;
mod proxy;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config: Config = toml::from_str(&read_to_string("./lucifer.toml").await?)
        .context("Failed to read config")?;

    let proxy = Proxy::new(config.proxy);
    proxy.start().await?;

    Ok(())
}
