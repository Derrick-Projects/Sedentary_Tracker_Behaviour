use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// 1. RAW INPUT From Arduino
// Format: {"ts":"12:34:56","pir":0,"acc":0.045}
#[derive(Debug, Deserialize, PartialEq)]
pub struct RawReading {
    pub ts: String, // Timestamp from RTC (HH:MM:SS)
    pub pir: i32,   // PIR sensor (0 or 1)
    pub acc: f32,   // Acceleration delta magnitude
}

// 2. PROCESSED OUTPUT (To Frontend & DB)
// Classification is also done server-side in serial.rs
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ProcessedState {
    pub state: String,            // "ACTIVE", "FIDGET", "SEDENTARY"
    pub timer: u64,               // Inactive seconds
    pub val: f32,                 // Smoothed acceleration value
    pub alert: bool,              // Trigger alert?
    pub timestamp: DateTime<Utc>, // Full timestamp (UTC)
}

#[cfg(test)]
#[path = "models_tests.rs"]
mod tests;
