use super::*;
use chrono::{TimeZone, Utc};

// RawReading Tests
// Tests for Arduino input format: {"ts":"12:34:56","pir":0,"acc":0.045}

#[test]
fn test_raw_reading_deserialization() {
    let json = r#"{"ts": "12:34:56", "pir": 1, "acc": 0.045}"#;
    let reading: RawReading = serde_json::from_str(json).unwrap();

    assert_eq!(reading.ts, "12:34:56");
    assert_eq!(reading.pir, 1);
    assert!((reading.acc - 0.045).abs() < 0.001);
}

#[test]
fn test_raw_reading_pir_inactive() {
    let json = r#"{"ts": "10:00:00", "pir": 0, "acc": 0.01}"#;
    let reading: RawReading = serde_json::from_str(json).unwrap();

    assert_eq!(reading.pir, 0);
}

#[test]
fn test_raw_reading_zero_acceleration() {
    let json = r#"{"ts": "00:00:00", "pir": 0, "acc": 0.0}"#;
    let reading: RawReading = serde_json::from_str(json).unwrap();

    assert_eq!(reading.ts, "00:00:00");
    assert_eq!(reading.pir, 0);
    assert_eq!(reading.acc, 0.0);
}

#[test]
fn test_raw_reading_high_acceleration() {
    let json = r#"{"ts": "12:00:00", "pir": 1, "acc": 2.5}"#;
    let reading: RawReading = serde_json::from_str(json).unwrap();

    assert!((reading.acc - 2.5).abs() < 0.001);
}

// ProcessedState Tests

#[test]
fn test_processed_state_serialization() {
    let state = ProcessedState {
        state: "SEDENTARY".to_string(),
        timer: 600,
        val: 0.02,
        alert: true,
        timestamp: Utc.with_ymd_and_hms(2026, 1, 6, 10, 0, 0).unwrap(),
    };

    let json = serde_json::to_string(&state).unwrap();
    assert!(json.contains("\"state\":\"SEDENTARY\""));
    assert!(json.contains("\"timer\":600"));
    assert!(json.contains("\"alert\":true"));
}

#[test]
fn test_processed_state_deserialization() {
    let json = r#"{
        "state": "ACTIVE",
        "timer": 0,
        "val": 1.0,
        "alert": false,
        "timestamp": "2026-01-06T10:00:00Z"
    }"#;

    let state: ProcessedState = serde_json::from_str(json).unwrap();
    assert_eq!(state.state, "ACTIVE");
    assert_eq!(state.timer, 0);
    assert!(!state.alert);
}

#[test]
fn test_processed_state_alert_threshold() {
    // Test case for alert being true (sedentary for too long)
    let state = ProcessedState {
        state: "SEDENTARY".to_string(),
        timer: 1800, // 30 minutes
        val: 0.01,
        alert: true,
        timestamp: Utc.with_ymd_and_hms(2026, 1, 6, 10, 30, 0).unwrap(),
    };

    assert!(state.alert);
    assert!(state.timer >= 1800);
}

#[test]
fn test_processed_state_no_alert() {
    let state = ProcessedState {
        state: "FIDGET".to_string(),
        timer: 60,
        val: 0.2,
        alert: false,
        timestamp: Utc.with_ymd_and_hms(2026, 1, 6, 10, 1, 0).unwrap(),
    };

    assert!(!state.alert);
}

#[test]
fn test_processed_state_clone() {
    let state = ProcessedState {
        state: "ACTIVE".to_string(),
        timer: 0,
        val: 1.5,
        alert: false,
        timestamp: Utc.with_ymd_and_hms(2026, 1, 6, 10, 0, 0).unwrap(),
    };

    let cloned = state.clone();
    assert_eq!(state, cloned);
}

#[test]
fn test_processed_state_roundtrip() {
    let original = ProcessedState {
        state: "SEDENTARY".to_string(),
        timer: 900,
        val: 0.05,
        alert: false,
        timestamp: Utc.with_ymd_and_hms(2026, 1, 6, 10, 15, 0).unwrap(),
    };

    let json = serde_json::to_string(&original).unwrap();
    let restored: ProcessedState = serde_json::from_str(&json).unwrap();

    assert_eq!(original, restored);
}
