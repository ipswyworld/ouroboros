// tests/p2p_handshake_and_relay.rs
use ouro_dag::network::handshake::{self, Envelope};
use tokio_util::codec::LengthDelimitedCodec;
use tokio::net::{TcpListener, TcpStream};
use futures::{SinkExt, StreamExt};
use serde_json::json;

/// Test that server_handshake_and_upgrade and client_handshake_over_framed perform the hello/challenge/signature
/// dance and the client receives the optional PeerList (server returns empty if none configured).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_handshake_roundtrip() {
    // bind listener to ephemeral port
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().unwrap();

    // spawn server accept loop that handles a single connection using server_handshake_and_upgrade
    let server_task = tokio::spawn(async move {
        let (stream, _peer) = listener.accept().await.expect("accept");
        // call server handshake which will send a PeerList automatically
        let res = handshake::server_handshake_and_upgrade(stream).await;
        assert!(res.is_ok(), "server handshake failed: {:?}", res.err());
        let (_peer_info, mut framed) = res.unwrap();

        // after handshake, read one optional frame (peer_list already sent by server function)
        // we will then send a test envelope to client to ensure framed works
        let env = Envelope::new("test_from_server", &json!({"hello":"client"})).unwrap();
        let bytes = serde_json::to_vec(&env).unwrap();
        let _ = framed.send(bytes.into()).await;
    });

    // client side: connect and perform client handshake
    let stream = TcpStream::connect(addr).await.expect("connect");
    let framed = tokio_util::codec::Framed::new(stream, LengthDelimitedCodec::new());
    let (framed_after, discovered) = handshake::client_handshake_over_framed(framed, "node-test-client", None)
        .await
        .expect("client handshake");
    // server sent a peer_list (likely empty) as part of server_handshake_and_upgrade
    assert!(discovered.is_empty() || discovered.iter().all(|s| !s.is_empty()));
    // read the extra test frame from server
    let mut framed = framed_after;
    if let Some(frame) = framed.next().await {
        let bytes = frame.expect("frame");
        let env: Envelope = serde_json::from_slice(&bytes).expect("parse env");
        assert_eq!(env.typ, "test_from_server");
        assert_eq!(env.payload["hello"], "client");
    } else {
        panic!("client did not receive server test frame");
    }

    let _ = server_task.await;
}

/// Test the framed relay: if two clients connect to a framed relay that re-broadcasts frames,
/// a frame sent by client A should be received by client B.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_framed_relay_forwarding() {
    // small in-test framed relay similar to src/bin/relay.rs
    use tokio::sync::Mutex;
    use std::sync::Arc;

    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let relay_addr = listener.local_addr().unwrap();

    let clients = Arc::new(Mutex::new(Vec::<tokio::sync::mpsc::Sender<Vec<u8>>>::new()));
    // spawn relay accept loop
    let relay_task = {
        let clients = clients.clone();
        tokio::spawn(async move {
            loop {
                let (socket, _peer) = listener.accept().await.expect("accept");
                let clients_inner = clients.clone();
                tokio::spawn(async move {
                    let (mut sink, mut stream) = tokio_util::codec::Framed::new(socket, LengthDelimitedCodec::new()).split();
                    let (tx, mut rx) = tokio::sync::mpsc::channel::<Vec<u8>>(8);
                    {
                        let mut g = clients_inner.lock().await;
                        g.push(tx.clone());
                    }
                    // writer task
                    let writer = tokio::spawn(async move {
                        while let Some(msg) = rx.recv().await {
                            if sink.send(msg.into()).await.is_err() { break; }
                        }
                    });

                    // read loop: broadcast frames to all clients
                    while let Some(frame) = stream.next().await {
                        match frame {
                            Ok(bytes) => {
                                let mut g = clients_inner.lock().await;
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
                    let _ = writer.await;
                });
            }
        })
    };

    // client A connects
    let s1 = TcpStream::connect(relay_addr).await.expect("connect1");
    let mut f1 = tokio_util::codec::Framed::new(s1, LengthDelimitedCodec::new());

    // client B connects
    let s2 = TcpStream::connect(relay_addr).await.expect("connect2");
    let mut f2 = tokio_util::codec::Framed::new(s2, LengthDelimitedCodec::new());

    // client1 sends an Envelope
    let env = Envelope::new("relay_test", &json!({"msg":"hello-from-A"})).unwrap();
    let bytes = serde_json::to_vec(&env).unwrap();
    f1.send(bytes.clone().into()).await.expect("send");

    // client2 should receive it (with some small timeout)
    tokio::select! {
        res = f2.next() => {
            let frame = res.expect("frame opt").expect("frame bytes");
            let got: Envelope = serde_json::from_slice(&frame).expect("parse");
            assert_eq!(got.typ, "relay_test");
            assert_eq!(got.payload["msg"], "hello-from-A");
        }
        _ = tokio::time::sleep(std::time::Duration::from_secs(2)) => {
            panic!("client B did not receive relayed frame");
        }
    }

    // shutdown relay (drop)
    relay_task.abort();
}
