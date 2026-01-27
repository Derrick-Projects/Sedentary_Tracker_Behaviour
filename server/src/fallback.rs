use crate::models::ProcessedState;
use chrono::{DateTime, Utc};
use redis::AsyncCommands;
use sqlx::PgPool;
use std::env;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::broadcast;
use tokio::time::interval;

// Configuration for fallback behavior
fn fallback_timeout_seconds() -> u64 {
    env::var("FALLBACK_TIMEOUT_SECONDS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10)
}

fn fallback_batch_size() -> i64 {
    env::var("FALLBACK_BATCH_SIZE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(500)
}

fn fallback_replay_interval_ms() -> u64 {
    env::var("FALLBACK_REPLAY_INTERVAL_MS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100)
}

// Shared state for tracking last data received
pub struct FallbackState {
    last_data_time: AtomicU64,
    is_fallback_active: AtomicBool,
}

impl FallbackState {
    pub fn new() -> Self {
        Self {
            last_data_time: AtomicU64::new(current_timestamp()),
            is_fallback_active: AtomicBool::new(false),
        }
    }

    pub fn record_data_received(&self) {
        self.last_data_time
            .store(current_timestamp(), Ordering::SeqCst);
        if self.is_fallback_active.load(Ordering::SeqCst) {
            self.is_fallback_active.store(false, Ordering::SeqCst);
            println!("Hardware reconnected - exiting fallback mode");
        }
    }

    pub fn seconds_since_last_data(&self) -> u64 {
        let last = self.last_data_time.load(Ordering::SeqCst);
        current_timestamp().saturating_sub(last)
    }

    pub fn is_in_fallback(&self) -> bool {
        self.is_fallback_active.load(Ordering::SeqCst)
    }

    pub fn enter_fallback(&self) {
        if !self.is_fallback_active.swap(true, Ordering::SeqCst) {
            println!("Hardware unavailable - entering fallback mode");
        }
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// Spawns the fallback monitor that watches for data gaps
// and backfills from the database when hardware is unavailable
pub fn spawn_fallback_monitor(
    pool: PgPool,
    tx: broadcast::Sender<String>,
    redis_client: redis::Client,
    fallback_state: Arc<FallbackState>,
) {
    let timeout = fallback_timeout_seconds();
    let batch_size = fallback_batch_size();
    let replay_interval = fallback_replay_interval_ms();

    println!(
        "Fallback monitor started (timeout: {}s, batch: {} rows, replay: {}ms)",
        timeout, batch_size, replay_interval
    );

    tokio::spawn(async move {
        let mut check_interval = interval(Duration::from_secs(1));

        loop {
            check_interval.tick().await;

            let seconds_idle = fallback_state.seconds_since_last_data();

            if seconds_idle >= timeout && !fallback_state.is_in_fallback() {
                fallback_state.enter_fallback();

                // Fetch historical data from database
                if let Err(e) = backfill_from_database(
                    &pool,
                    &tx,
                    &redis_client,
                    batch_size,
                    replay_interval,
                    &fallback_state,
                )
                .await
                {
                    eprintln!("Fallback backfill error: {}", e);
                }
            }
        }
    });
}

/// Fetches the last N rows from sedentary_log and broadcasts them
async fn backfill_from_database(
    pool: &PgPool,
    tx: &broadcast::Sender<String>,
    redis_client: &redis::Client,
    batch_size: i64,
    replay_interval_ms: u64,
    fallback_state: &Arc<FallbackState>,
) -> Result<(), sqlx::Error> {
    println!("Backfilling {} rows from database...", batch_size);

    // Get Redis connection for caching
    let redis_conn = redis_client.get_multiplexed_async_connection().await.ok();

    // Fetch last N rows, ordered by created_at ascending (oldest first for replay)
    let rows = sqlx::query!(
        r#"
        SELECT id, state, timer_seconds, acceleration_val, created_at
        FROM sedentary_log
        ORDER BY created_at DESC
        LIMIT $1
        "#,
        batch_size
    )
    .fetch_all(pool)
    .await?;

    if rows.is_empty() {
        println!("No historical data available for backfill");
        return Ok(());
    }

    println!("Retrieved {} rows for backfill", rows.len());

    // Reverse to replay in chronological order (oldest to newest)
    let rows_chronological: Vec<_> = rows.into_iter().rev().collect();

    let replay_delay = Duration::from_millis(replay_interval_ms);

    for row in rows_chronological {
        // Check if real hardware data arrived - exit fallback early
        if !fallback_state.is_in_fallback() {
            println!("Hardware reconnected during backfill - stopping replay");
            break;
        }

        // Convert DB row to ProcessedState
        let timestamp: DateTime<Utc> = row.created_at.unwrap_or_else(Utc::now);

        let timer = row.timer_seconds.unwrap_or(0) as u64;
        let alert_threshold = crate::serial::alert_limit_sec();

        let processed = ProcessedState {
            state: row.state,
            timer,
            val: row.acceleration_val.unwrap_or(0.0),
            alert: timer >= alert_threshold,
            timestamp,
        };

        // Serialize and broadcast + cache to Redis
        if let Ok(json) = serde_json::to_string(&processed) {
            // Broadcast to connected clients
            let _ = tx.send(json.clone());

            // Cache in Redis for new clients
            if let Some(ref mut con) = redis_conn.clone() {
                let _: Result<(), _> = con.lpush("sensor_history", &json).await;
                let _: Result<(), _> = con.ltrim("sensor_history", 0, 99).await;
            }
        }

        // Small delay between replays to avoid flooding the frontend
        tokio::time::sleep(replay_delay).await;
    }

    println!("Backfill complete");
    Ok(())
}
