use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use warp::Filter;
use warp::ws::{Message, WebSocket};

#[tokio::main]
async fn main() {
    let peers = Arc::new(Mutex::new(HashSet::new()));
    let peers_filter = warp::any().map(move || peers.clone());

    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(peers_filter)
        .map(|ws: warp::ws::Ws, peers| {
            ws.on_upgrade(move |socket| client_connected(socket, peers))
        });

    warp::serve(ws_route)
        .run(([0, 0, 0, 0], 3030))
        .await;
}

async fn client_connected(ws: WebSocket, peers: Arc<Mutex<HashSet<tokio::sync::mpsc::UnboundedSender<Message>>>>) {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    peers.lock().unwrap().insert(tx.clone());

    let (ws_tx, mut ws_rx) = ws.split();
    let recv_to_bcast = async move {
        while let Some(Ok(msg)) = ws_rx.next().await {
            for peer in peers.lock().unwrap().iter() {
                let _ = peer.send(msg.clone());
            }
        }
    };
    let send_to_ws = tokio::task::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let _ = ws_tx.send(msg).await;
        }
    });

    tokio::select! {
        _ = recv_to_bcast => (),
        _ = send_to_ws => (),
    }
}