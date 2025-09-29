// src/bin/relay.rs
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use futures::{SinkExt, StreamExt};
use ouro_dag::network::handshake::Envelope;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The port to listen on
    #[arg(short, long, default_value_t = 9009)]
    port: u16,
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let port = cli.port;
    let addr = format!("0.0.0.0:{}", port);
    println!("Starting framed relay on {}", addr);
    let listener = TcpListener::bind(&addr).await?;
    let clients = Arc::new(Mutex::new(Vec::<tokio::sync::mpsc::Sender<Vec<u8>>>::new()));

    loop {
        let (socket, remote) = listener.accept().await?;
        println!("Relay: client connected {}", remote);
        let clients_inner = clients.clone();

        tokio::spawn(async move {
            let (mut sink, mut stream) = Framed::new(socket, LengthDelimitedCodec::new()).split();
            // per-client outgoing queue
            let (tx, mut rx) = tokio::sync::mpsc::channel::<Vec<u8>>(64);

            // register sender
            {
                let mut g = clients_inner.lock().await;
                g.push(tx.clone());
            }

            // sender task
            let send_task = tokio::spawn(async move {
                while let Some(msg) = rx.recv().await {
                    // send to the client (already length-delimited frames)
                    if sink.send(msg.into()).await.is_err() {
                        break;
                    }
                }
            });

            // read loop and broadcast to all other clients
            while let Some(frame) = stream.next().await {
                match frame {
                    Ok(bytes) => {
                        if bytes.is_empty() { continue; }
                        // minimal sanity: try to parse envelope (non-fatal)
                        let _ = serde_json::from_slice::<Envelope>(&bytes);
                        // broadcast to all clients
                        let mut g = clients_inner.lock().await;
                        // clone current list so writes don't borrow
                        let mut i = 0usize;
                        while i < g.len() {
                            if g[i].try_send(bytes.to_vec()).is_err() {
                                g.remove(i);
                            } else {
                                i += 1;
                            }
                        }
                    }
                    Err(_) => break,
                }
            }

            // cleanup
            println!("Relay: client disconnected {}", remote);
            // drop the tx by removing it
            {
                let mut g = clients_inner.lock().await;
                g.retain(|s| !s.same_channel(&tx));
            }
            let _ = send_task.await;
        });
    }
}