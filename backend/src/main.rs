use axum::{
    extract::ws::{Message, WebSocketUpgrade, WebSocket},
    routing::get,
    Router, Extension,
};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{Mutex, broadcast};
use ulid::Ulid;

type Clients = Arc<Mutex<HashMap<String, broadcast::Sender<String>>>>;

#[tokio::main]
async fn main() {
    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .layer(Extension(clients));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Extension(clients): Extension<Clients>,
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, clients))
}

async fn handle_socket(mut socket: WebSocket, clients: Clients) {
    let id = Ulid::new().to_string();
    let (tx, mut rx) = broadcast::channel(16);
    clients.lock().await.insert(id.clone(), tx.clone());

    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if socket.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    let recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(msg))) = socket.recv().await {
            if let Ok(json): serde_json::Value = serde_json::from_str(&msg) {
                if let Some(to) = json.get("to").and_then(|v| v.as_str()) {
                    let clients = clients.lock().await;
                    if let Some(target_tx) = clients.get(to) {
                        let _ = target_tx.send(msg.clone());
                    }
                }
            }
        }
    });

    let _ = tokio::join!(send_task, recv_task);
    clients.lock().await.remove(&id);
}
