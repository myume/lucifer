use std::sync::Arc;

use tokio::{net::UdpSocket, time::Instant};

use crate::{
    config::ProxyConfig,
    dns::{read_domain, write_sinkhole_response},
};

pub struct Proxy {
    config: ProxyConfig,
}

impl Proxy {
    pub fn new(config: ProxyConfig) -> Self {
        Self { config }
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        let addr = format!("127.0.0.1:{}", self.config.port.unwrap_or(53));
        let sock = Arc::new(UdpSocket::bind(&addr).await?);
        let upstream_sock = Arc::new(UdpSocket::bind("0.0.0.0:0").await?);
        upstream_sock
            .connect(format!(
                "{}:53",
                self.config
                    .nameservers
                    .first()
                    .expect("nameservers not found")
            ))
            .await?;
        println!("DNS proxy running on {addr}");

        loop {
            let mut buf = [0; 512];
            let start = Instant::now();
            let (len, client_addr) = sock.recv_from(&mut buf).await?;
            println!("Received request from {}", client_addr);

            let domain = read_domain(&buf[12..]);
            println!("{domain}");
            if self.config.blocklist.contains(&domain) {
                println!("accessing blocked domain: {domain}");
                write_sinkhole_response(&mut buf);
                sock.send_to(&buf[..len], client_addr).await.unwrap();
                continue;
            }

            let upstream_sock = upstream_sock.clone();
            let sock = sock.clone();
            tokio::spawn(async move {
                upstream_sock.send(&buf[..len]).await.unwrap();
                let (reply_len, _) = upstream_sock.recv_from(&mut buf).await.unwrap();
                sock.send_to(&buf[..reply_len], client_addr).await.unwrap();
                println!("DNS request took {}ms", start.elapsed().as_millis())
            });
        }
    }
}
