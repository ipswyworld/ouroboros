// simple tcp-proxy.rs (tokio)
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{copy_bidirectional};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listen = std::env::var("LISTEN").unwrap_or("0.0.0.0:9001".into());
    let target = std::env::var("TARGET").expect("TARGET env required, e.g. 127.0.0.1:9001");
    let listener = TcpListener::bind(listen).await?;
    println!("relay listening; forwarding to {}", target);
    loop {
        let (inbound, _) = listener.accept().await?;
        let target = target.clone();
        tokio::spawn(async move {
            if let Ok(outbound) = TcpStream::connect(&target).await {
                let _ = copy_bidirectional(&mut inbound.into_split().0, &mut outbound.into_split().1).await;
            } else {
                println!("relay: connect to {} failed", target);
            }
        });
    }
}
