use crate::fallback::FallbackState;
use crate::models::{ProcessedState, RawReading};
use chrono::{NaiveTime, Utc};
use redis::AsyncCommands;
use std::collections::VecDeque;
use std::env;
use std::io::{BufRead, BufReader};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tokio::sync::broadcast;

// CLASSIFICATION THRESHOLDS - Load from environment
fn thresh_fidget() -> f32 {
    env::var("THRESH_FIDGET")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.020)
}

fn thresh_active() -> f32 {
    env::var("THRESH_ACTIVE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.040)
}

pub fn alert_limit_sec() -> u64 {
    env::var("ALERT_LIMIT_SECONDS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1200)
}

fn sensor_history_limit() -> isize {
    env::var("SENSOR_HISTORY_LIMIT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(500)
}

const SMOOTHING_WINDOW: usize = 10; // Number of samples for smoothing buffer

/// Classifies activity state based on PIR and smoothed acceleration
fn classify_state(pir: i32, smoothed_acc: f32) -> String {
    if pir == 1 || smoothed_acc > thresh_active() {
        "ACTIVE".to_string()
    } else if smoothed_acc > thresh_fidget() {
        "FIDGET".to_string()
    } else {
        "SEDENTARY".to_string()
    }
}

pub fn spawn_serial_listener(
    tx: broadcast::Sender<String>,
    redis_client: redis::Client,
    port_name: String,
    baud_rate: u32,
    fallback_state: Arc<FallbackState>,
) {
    thread::spawn(move || {
        println!("Connecting to serial device...");

        let port = serialport::new(&port_name, baud_rate)
            .timeout(Duration::from_millis(1000))
            .open();

        // Create a dedicated async runtime for the serial thread
        let rt = tokio::runtime::Runtime::new().unwrap();

        // State tracking
        let mut acc_buffer: VecDeque<f32> = VecDeque::with_capacity(SMOOTHING_WINDOW);
        let mut sedentary_timer: u64 = 0;
        let mut last_second: Option<String> = None;

        match port {
            Ok(p) => {
                println!("Serial Connected! Processing raw sensor data...");
                let mut reader = BufReader::new(p);
                let mut line = String::new();

                loop {
                    line.clear();
                    if let Ok(bytes_read) = reader.read_line(&mut line) {
                        if bytes_read == 0 {
                            continue;
                        }

                        let clean_line = line.trim();
                        if clean_line.starts_with('{') {
                            // Parse raw Arduino data
                            if let Ok(reading) = serde_json::from_str::<RawReading>(clean_line) {
                                // Notify fallback monitor that real hardware data is arriving
                                fallback_state.record_data_received();
                                // Add to smoothing buffer
                                if acc_buffer.len() >= SMOOTHING_WINDOW {
                                    acc_buffer.pop_front();
                                }
                                acc_buffer.push_back(reading.acc);

                                // Calculate smoothed acceleration (mean of buffer)
                                let smoothed_acc: f32 = if acc_buffer.is_empty() {
                                    0.0
                                } else {
                                    acc_buffer.iter().sum::<f32>() / acc_buffer.len() as f32
                                };

                                // Classify state
                                let state = classify_state(reading.pir, smoothed_acc);

                                // Update sedentary timer (once per second based on timestamp)
                                let current_second = reading.ts.clone();
                                if last_second.as_ref() != Some(&current_second) {
                                    last_second = Some(current_second);

                                    match state.as_str() {
                                        "ACTIVE" => sedentary_timer = 0,     // Reset on activity
                                        "FIDGET" => {}                       // Pause
                                        "SEDENTARY" => sedentary_timer += 1, // Increment
                                        _ => {}
                                    }
                                }

                                // Build processed output with full UTC timestamp
                                let timestamp = NaiveTime::parse_from_str(&reading.ts, "%H:%M:%S")
                                    .map(|time| Utc::now().date_naive().and_time(time).and_utc())
                                    .unwrap_or_else(|_| Utc::now());

                                let output = ProcessedState {
                                    state: state.clone(),
                                    timer: sedentary_timer,
                                    val: smoothed_acc,
                                    alert: sedentary_timer >= alert_limit_sec(),
                                    timestamp,
                                };

                                let json_out = serde_json::to_string(&output).unwrap();

                                // Broadcast to WebSocket and cache in Redis
                                rt.block_on(async {
                                    // Redis cache for reconnection
                                    if let Ok(mut con) =
                                        redis_client.get_multiplexed_async_connection().await
                                    {
                                        let _: () = con
                                            .lpush("sensor_history", &json_out)
                                            .await
                                            .unwrap_or(());
                                        let _: () = con
                                            .ltrim("sensor_history", 0, sensor_history_limit() - 1)
                                            .await
                                            .unwrap_or(());
                                    }
                                    // Push to WebSocket
                                    let _ = tx.send(json_out);
                                });
                            }
                        }
                    }
                }
            }
            Err(e) => eprintln!("Serial Error: {}", e),
        }
    });
}
