use std::sync::Arc;

use anyhow::anyhow;
use tokio::{net::UdpSocket, time::Instant};
use tracing::{debug, error, info, trace};

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
        let Some(nameserver) = self.config.nameservers.first() else {
            return Err(anyhow!("No nameservers configured."));
        };

        let addr = format!("127.0.0.1:{}", self.config.port.unwrap_or(53));
        let sock = Arc::new(UdpSocket::bind(&addr).await?);
        let upstream_sock = Arc::new(UdpSocket::bind("0.0.0.0:0").await?);
        upstream_sock.connect(format!("{}:53", nameserver)).await?;
        info!("DNS proxy running on {addr}");

        loop {
            let mut buf = [0; 512];
            let (len, client_addr) = sock.recv_from(&mut buf).await?;
            debug!("Received request from {}", client_addr);
            let start = Instant::now();

            let domain = read_domain(&buf[12..]);
            debug!("DNS query for {domain}");
            if self.config.blocklist.contains(&domain) {
                info!("Accessing blocked domain: {domain}");
                write_sinkhole_response(&mut buf);
                if let Err(e) = sock.send_to(&buf[..len], client_addr).await {
                    error!("Failed to send response: {e}");
                };
                continue;
            }

            let upstream_sock = upstream_sock.clone();
            let sock = sock.clone();
            tokio::spawn(async move {
                if let Err(e) = upstream_sock.send(&buf[..len]).await {
                    error!("Failed to send request to upstream: {e}");
                }
                match upstream_sock.recv_from(&mut buf).await {
                    Ok((reply_len, _)) => {
                        if let Err(e) = sock.send_to(&buf[..reply_len], client_addr).await {
                            error!("Failed to send response: {e}");
                        }
                        trace!("DNS request took {}ms", start.elapsed().as_millis())
                    }
                    Err(e) => {
                        error!("Failed to get upstream response: {e}");
                    }
                }
            });
        }
    }
}
