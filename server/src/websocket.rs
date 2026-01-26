use crate::state::AppState;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use redis::AsyncCommands;

pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    // 1. RECONNECTION BACKUP (Fetch from Redis)
    // This fills the graph immediately upon connection
    if let Ok(mut con) = state.redis.get_multiplexed_async_connection().await {
        
        let history: Vec<String> = con.lrange("sensor_history", 0, 99).await.unwrap_or(vec![]);

        // Send history to frontend (reversed because lpush stores newest first)
        for msg in history.into_iter().rev() {
            let _ = socket.send(Message::Text(msg)).await;
        }
    }

    // 2. LIVE STREAM Zero Latency
    let mut rx = state.tx.subscribe();
    while let Ok(msg) = rx.recv().await {
        if socket.send(Message::Text(msg)).await.is_err() {
            break;
        }
    }
}
