use std::sync::Arc;

use anyhow::Context;
use tokio::{fs::read_to_string, net::UdpSocket, time::Instant};

use crate::config::Config;

mod config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config: Config = toml::from_str(&read_to_string("./lucifer.toml").await?)
        .context("Failed to read config")?;

    let addr = format!("127.0.0.1:{}", config.proxy.port.unwrap_or(53));
    let sock = Arc::new(UdpSocket::bind(&addr).await?);
    let upstream_sock = Arc::new(UdpSocket::bind("0.0.0.0:0").await?);
    upstream_sock
        .connect(format!(
            "{}:53",
            config
                .proxy
                .nameservers
                .first()
                .expect("nameservers not found")
        ))
        .await?;
    println!("DNS proxy running on {addr}");

    loop {
        let mut buf = [0; 4096];
        let start = Instant::now();
        let (len, client_addr) = sock.recv_from(&mut buf).await?;
        println!("Received request from {}", client_addr);

        let cloudflare = upstream_sock.clone();
        let proxy = sock.clone();
        tokio::spawn(async move {
            cloudflare.send(&buf[..len]).await.unwrap();
            let (reply_len, _) = cloudflare.recv_from(&mut buf).await.unwrap();
            proxy.send_to(&buf[..reply_len], client_addr).await.unwrap();
            println!("DNS request took {}ms", start.elapsed().as_millis())
        });
    }
}
