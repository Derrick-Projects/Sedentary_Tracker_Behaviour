use crate::models::{ProcessedState, RawReading};
use crate::serial::alert_limit_sec;
use chrono::{NaiveTime, Utc};
use std::collections::VecDeque;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::time::sleep;

const SMOOTHING_WINDOW: usize = 10;

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

fn classify_state(pir: i32, smoothed_acc: f32) -> String {
    if pir == 1 || smoothed_acc > thresh_active() {
        "ACTIVE".to_string()
    } else if smoothed_acc > thresh_fidget() {
        "FIDGET".to_string()
    } else {
        "SEDENTARY".to_string()
    }
}

pub async fn replay_log_file(
    tx: broadcast::Sender<String>,
    log_path: &Path,
    replay_speed_ms: u64,
) -> Result<usize, String> {
    let file = File::open(log_path).map_err(|e| format!("Failed to open log file: {}", e))?;
    let reader = BufReader::new(file);

    let mut acc_buffer: VecDeque<f32> = VecDeque::with_capacity(SMOOTHING_WINDOW);
    let mut sedentary_timer: u64 = 0;
    let mut last_second: Option<String> = None;
    let mut count = 0;

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };

        let clean_line = line.trim();

        // Skip timestamp prefixes if present (e.g., "[2026-01-23 16:12:03.123] {...}")
        let json_start = clean_line.find('{');
        let json_str = match json_start {
            Some(idx) => &clean_line[idx..],
            None => continue,
        };

        if let Ok(reading) = serde_json::from_str::<RawReading>(json_str) {
            // Add to smoothing buffer
            if acc_buffer.len() >= SMOOTHING_WINDOW {
                acc_buffer.pop_front();
            }
            acc_buffer.push_back(reading.acc);

            // Calculate smoothed acceleration
            let smoothed_acc: f32 = if acc_buffer.is_empty() {
                0.0
            } else {
                acc_buffer.iter().sum::<f32>() / acc_buffer.len() as f32
            };

            // Classify state
            let state = classify_state(reading.pir, smoothed_acc);

            // Update sedentary timer (once per second)
            let current_second = reading.ts.clone();
            if last_second.as_ref() != Some(&current_second) {
                last_second = Some(current_second);

                match state.as_str() {
                    "ACTIVE" => sedentary_timer = 0,
                    "FIDGET" => {}
                    "SEDENTARY" => sedentary_timer += 1,
                    _ => {}
                }
            }

            // Build processed output
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

            // Broadcast to connected clients
            let _ = tx.send(json_out);
            count += 1;

            // Replay delay
            if replay_speed_ms > 0 {
                sleep(Duration::from_millis(replay_speed_ms)).await;
            }
        }
    }

    Ok(count)
}

/// Spawns a background task to replay log data
pub fn spawn_replay_task(
    tx: broadcast::Sender<String>,
    log_path: String,
    replay_speed_ms: u64,
) {
    tokio::spawn(async move {
        let path = Path::new(&log_path);
        println!("Starting replay from: {}", log_path);

        match replay_log_file(tx, path, replay_speed_ms).await {
            Ok(count) => println!("Replay complete: {} records processed", count),
            Err(e) => eprintln!("Replay error: {}", e),
        }
    });
}
