use crate::state::AppState;
use axum::{
    extract::State,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
};
use futures::Stream;
use redis::AsyncCommands;
use std::convert::Infallible;
use std::time::Duration;

/// Server-Sent Events handler for real-time sensor data streaming

pub async fn sse_handler(State(state): State<AppState>) -> impl IntoResponse {
    let stream = create_sensor_stream(state);

    Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keepalive"),
    )
}

/// Creates a stream of sensor data events
///
/// Flow:
/// 1. Optionally fetch historical data from Redis (disabled with SKIP_HISTORY=true)
/// 2. Stream live updates from broadcast channel
fn create_sensor_stream(state: AppState) -> impl Stream<Item = Result<Event, Infallible>> {
    async_stream::stream! {
        // Step 1: Fetch historical data from Redis (skip if SKIP_HISTORY=true)
        let skip_history = std::env::var("SKIP_HISTORY")
            .map(|v| v == "true")
            .unwrap_or(false);

        if !skip_history {
            if let Ok(mut con) = state.redis.get_multiplexed_async_connection().await {
                let limit: isize = std::env::var("SENSOR_HISTORY_LIMIT")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(500);
                let history: Vec<String> = con
                    .lrange("sensor_history", 0, limit - 1)
                    .await
                    .unwrap_or_else(|e| {
                        eprintln!("Redis error fetching history: {:?}", e);
                        vec![]
                    });

                // Send history to client (reversed because lpush stores newest first)
                for msg in history.into_iter().rev() {
                    yield Ok::<_, Infallible>(
                        Event::default()
                            .event("sensor-data")
                            .data(msg)
                    );
                }
            } else {
                eprintln!("Failed to connect to Redis for SSE history");
            }
        }

        // Step 2: Live stream from broadcast channel
        let mut rx = state.tx.subscribe();

        while let Ok(msg) = rx.recv().await {
            yield Ok::<_, Infallible>(
                Event::default()
                    .event("sensor-data")
                    .data(msg)
            );
        }
    }
}
