use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, command};
use tokio::fs::read_to_string;

use crate::{config::Config, proxy::Proxy};

mod config;
pub mod dns;
mod proxy;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, value_name = "FILE")]
    config: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let config: Config =
        toml::from_str(&read_to_string(args.config.unwrap_or("./lucifer.toml".into())).await?)
            .context("Failed to read config")?;

    tracing_subscriber::fmt::init();

    let proxy = Proxy::new(config.proxy);
    proxy.start().await?;

    Ok(())
}
