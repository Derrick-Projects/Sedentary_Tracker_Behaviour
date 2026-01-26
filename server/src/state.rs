use sqlx::PgPool;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    // The "Hub" that broadcasts JSON strings to everyone (WebSocket + DB Worker)
    pub tx: broadcast::Sender<String>,
    // Redis client for caching and pub/sub
    pub redis: redis::Client,
}
