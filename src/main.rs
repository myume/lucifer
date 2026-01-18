use std::sync::Arc;

use tokio::{net::UdpSocket, time::Instant};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr = "127.0.0.1:53";
    let sock = Arc::new(UdpSocket::bind(addr).await?);
    let upstream_sock = Arc::new(UdpSocket::bind("0.0.0.0:0").await?);
    upstream_sock.connect("1.1.1.3:53").await?;
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
