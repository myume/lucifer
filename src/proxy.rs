use std::{sync::Arc, time::Duration};

use anyhow::anyhow;
use tokio::{net::UdpSocket, time::Instant};
use tracing::{debug, error, info, trace};

use crate::{
    config::ProxyConfig,
    dns::{DNS_HEADER_SIZE, read_domain, write_sinkhole_response},
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
        info!("DNS proxy running on {addr}");

        loop {
            let mut buf = [0; 512];
            let (len, client_addr) = sock.recv_from(&mut buf).await?;
            debug!("Received request from {}", client_addr);
            let start = Instant::now();

            let domain = read_domain(&buf[DNS_HEADER_SIZE..]);
            debug!("DNS query for {domain}");
            if self.config.blocklist.contains(&domain) {
                info!("Accessing blocked domain: {domain}");
                write_sinkhole_response(&mut buf);
                if let Err(e) = sock.send_to(&buf[..DNS_HEADER_SIZE], client_addr).await {
                    error!("Failed to send response: {e}");
                };
                continue;
            }

            let sock = sock.clone();
            let nameserver = nameserver.clone();
            tokio::spawn(async move {
                let Ok(upstream_sock) = UdpSocket::bind("0.0.0.0:0").await else {
                    error!("Failed to create upstream socket");
                    return;
                };
                if let Err(e) = upstream_sock.connect(format!("{}:53", nameserver)).await {
                    error!("Failed to connect to upstream socket: {e}");
                    return;
                };
                if let Err(e) = upstream_sock.send(&buf[..len]).await {
                    error!("Failed to send request to upstream: {e}");
                }

                match tokio::time::timeout(Duration::from_secs(5), upstream_sock.recv(&mut buf))
                    .await
                {
                    Ok(Ok(reply_len)) => {
                        if let Err(e) = sock.send_to(&buf[..reply_len], client_addr).await {
                            error!("Failed to send response: {e}");
                        }
                        trace!("DNS request took {}ms", start.elapsed().as_millis());
                    }
                    Ok(Err(e)) => error!("Failed to get upstream response: {e}"),
                    Err(_) => error!("Upstream timed out for {client_addr}"),
                }
            });
        }
    }
}
