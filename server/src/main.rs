use axum::{extract::State, routing::get, Router};
use dotenvy::dotenv;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::services::ServeDir;

mod auth;
mod db_worker;
mod fallback;
mod fhir;
mod fhir_analytics;
mod login;
mod models;
mod replay;
mod serial;
mod signup;
mod sse;
mod state;
mod websocket;

use auth::AuthUser;
use state::AppState;

#[tokio::main]
async fn main() {
    dotenv().ok();

    // Initialize Logging
    tracing_subscriber::fmt::init();
    println!("Server initializing...");

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    println!("Connecting to database...");
    let pool = db::get_db_pool(&database_url)
        .await
        .expect("Failed to connect to database");
    println!("Database connection established.");

    //  Redis Connection
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set");
    let redis_client = redis::Client::open(redis_url.as_str()).expect("Invalid Redis URL");
    println!("Redis client connected");

    //  Create the Broadcast Channel
    let (tx, _rx) = broadcast::channel(100);

    // Fallback Monitor - backfills from DB when hardware is unavailable
    let fallback_state = Arc::new(fallback::FallbackState::new());

    //  Start Background Tasks/Data Pipeline
    let serial_port = env::var("SERIAL_PORT").expect("SERIAL_PORT must be set");
    let baud_rate: u32 = env::var("BAUD_RATE")
        .expect("BAUD_RATE must be set")
        .parse()
        .expect("BAUD_RATE must be a valid number");
    serial::spawn_serial_listener(
        tx.clone(),
        redis_client.clone(),
        serial_port,
        baud_rate,
        fallback_state.clone(),
    );

    // Start fallback monitor (watches for data gaps and backfills from DB)
    // Can be disabled with DISABLE_FALLBACK=true for local/replay mode
    if env::var("DISABLE_FALLBACK")
        .map(|v| v != "true")
        .unwrap_or(true)
    {
        fallback::spawn_fallback_monitor(
            pool.clone(),
            tx.clone(),
            redis_client.clone(),
            fallback_state,
        );
    } else {
        println!("Fallback monitor disabled");
    }

    // DB Worker/Storage
    db_worker::spawn_db_worker(pool.clone(), tx.subscribe()).await;

    //  Build the Application State
    let app_state = AppState {
        db: pool,
        tx,
        redis: redis_client,
    };

    //  Define Routes
    let app = Router::new()
        // Real-Time Streaming (SSE primary, WebSocket fallback)
        .route("/events", get(sse::sse_handler))
        .route("/ws", get(websocket::ws_handler))
        // FHIR Compliance API
        .route(
            "/api/fhir/observation/latest",
            get(fhir::get_latest_observation),
        )
        // FHIR Analytics API (LOINC 87705-0)
        .route(
            "/api/fhir/analytics/user/:user_id",
            get(fhir_analytics::get_user_analytics),
        )
        .route(
            "/api/fhir/analytics/latest",
            get(fhir_analytics::get_latest_analytics),
        )
        // Signup form + handler
        .route(
            "/signup",
            get(signup::show_signup_form).post(signup::signup_handler),
        )
        // Login form + handler
        .route(
            "/login",
            get(login::show_login_form).post(login::login_handler),
        )
        // Protected stats endpoint
        .route("/stats", get(get_user_stats))
        // Health Check
        .route("/health", get(|| async { "Status: Healthy" }))
        // Replay log data for testing/demo
        .route("/api/replay", get(start_replay))
        // Frontend Hosting
        .nest_service(
            "/",
            ServeDir::new(env::var("FRONTEND_DIR").unwrap_or_else(|_| {
                concat!(env!("CARGO_MANIFEST_DIR"), "/../frontend").to_string()
            })),
        )
        .with_state(app_state);

    // Start the Server
    let server_addr = env::var("SERVER_ADDRESS").unwrap_or_else(|_| "0.0.0.0:8000".to_string());
    let addr: SocketAddr = server_addr.parse().expect("Invalid SERVER_ADDRESS");
    println!("Sedentary Tracker listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_user_stats(user: AuthUser) -> impl axum::response::IntoResponse {
    format!(
        "Fetching secret stats for {} (User ID: {})",
        user.name, user.user_id
    )
}

async fn start_replay(State(state): State<AppState>) -> impl axum::response::IntoResponse {
    let log_path = env::var("REPLAY_LOG_PATH").unwrap_or_else(|_| "arduino_data.log".to_string());
    let replay_speed: u64 = env::var("REPLAY_SPEED_MS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(50); // 50ms between readings for ~20x speed

    replay::spawn_replay_task(
        state.tx.clone(),
        state.redis.clone(),
        log_path.clone(),
        replay_speed,
    );

    format!(
        "Replay started from: {} (speed: {}ms per reading)",
        log_path, replay_speed
    )
}
