use super::*;
//SignalWindow Tests

#[test]
fn test_signal_window_new() {
    let window = SignalWindow::new();
    assert!(window.data_points.is_empty());
}

#[test]
fn test_signal_window_add_single_value() {
    let mut window = SignalWindow::new();
    window.add(1.5);
    assert_eq!(window.data_points.len(), 1);
    assert_eq!(window.data_points[0], 1.5);
}

#[test]
fn test_signal_window_add_multiple_values() {
    let mut window = SignalWindow::new();
    for i in 0..100 {
        window.add(i as f64);
    }
    assert_eq!(window.data_points.len(), 100);
    assert_eq!(window.data_points[0], 0.0);
    assert_eq!(window.data_points[99], 99.0);
}

#[test]
fn test_signal_window_buffer_limit() {
    let mut window = SignalWindow::new();
    // Add more than 2000 points to trigger buffer limit
    for i in 0..2100 {
        window.add(i as f64);
    }
    // Buffer capped at 2000
    assert_eq!(window.data_points.len(), 2000);
    // First value should be 100 (oldest values removed)
    assert_eq!(window.data_points[0], 100.0);
    // Last value should be 2099
    assert_eq!(window.data_points[1999], 2099.0);
}

//Stationarity Tests

#[test]
fn test_stationarity_constant_signal() {
    // A constant signal should be stationary
    let data: Vec<f64> = vec![1.0; 100];
    assert!(check_stationarity(&data, 4));
}

#[test]
fn test_stationarity_uniform_noise() {
    // Uniform small noise should be stationary
    let data: Vec<f64> = (0..100).map(|i| 1.0 + (i % 2) as f64 * 0.01).collect();
    assert!(check_stationarity(&data, 4));
}

#[test]
fn test_stationarity_non_stationary_trend() {
    // A signal with a strong trend is NOT stationary
    let data: Vec<f64> = (0..100).map(|i| (i * i) as f64).collect();
    assert!(!check_stationarity(&data, 4));
}

#[test]
fn test_stationarity_too_few_segments() {
    // If data is too short for segments, return false
    let data: Vec<f64> = vec![1.0, 2.0];
    assert!(!check_stationarity(&data, 10));
}

#[test]
fn test_stationarity_abrupt_change() {
    // Signal with high variance changes between segments
    // Using a signal that oscillates between very different variance levels
    let mut data: Vec<f64> = (0..50).map(|i| (i as f64 * 10.0).sin() * 100.0).collect();
    data.extend((0..50).map(|_| 0.0)); // Flat section with zero variance
    assert!(!check_stationarity(&data, 4));
}

//Hjorth Parameters Tests

#[test]
fn test_hjorth_constant_signal() {
    // Constant signal: variance/activity should be 0
    let data: Vec<f64> = vec![5.0; 100];
    let features = calculate_hjorth_params(&data);

    assert_eq!(features.mean, 5.0);
    assert_eq!(features.variance, 0.0);
    assert_eq!(features.hjorth_activity, 0.0);

    assert_eq!(features.hjorth_mobility, 0.0);
    assert_eq!(features.hjorth_complexity, 0.0);
}

#[test]
fn test_hjorth_simple_signal() {
    // Simple oscillating signal
    let data: Vec<f64> = (0..100).map(|i| (i as f64).sin()).collect();
    let features = calculate_hjorth_params(&data);

    // Mean should be close to 0 for a full sine wave
    assert!(features.mean.abs() < 0.1);
    // Variance should be positive
    assert!(features.variance > 0.0);
    // Activity equals variance
    assert_eq!(features.hjorth_activity, features.variance);
    // Mobility should be positive
    assert!(features.hjorth_mobility > 0.0);
    // Complexity should be positive
    assert!(features.hjorth_complexity > 0.0);
}

#[test]
fn test_hjorth_linear_signal() {
    // Linear increasing signal
    let data: Vec<f64> = (0..100).map(|i| i as f64).collect();
    let features = calculate_hjorth_params(&data);

    // Mean should be 49.5 (average of 0..99)
    assert!((features.mean - 49.5).abs() < 0.01);
    // Should have positive variance
    assert!(features.variance > 0.0);
    // Stationarity should fail for trending data
    assert!(!features.stationarity_passed);
}

#[test]
fn test_hjorth_high_frequency_noise() {
    // High frequency alternating signal
    let data: Vec<f64> = (0..100)
        .map(|i| if i % 2 == 0 { 1.0 } else { -1.0 })
        .collect();
    let features = calculate_hjorth_params(&data);

    // Mean should be close to 0
    assert!(features.mean.abs() < 0.1);
    // Variance should be 1.0
    assert!((features.variance - 1.0).abs() < 0.01);
    // High frequency = high mobility
    assert!(features.hjorth_mobility > 1.0);
}

// SignalFeatures Serialization Tests

#[test]
fn test_signal_features_serialization() {
    let features = SignalFeatures {
        mean: 1.5,
        variance: 0.5,
        stationarity_passed: true,
        hjorth_activity: 0.5,
        hjorth_mobility: 0.3,
        hjorth_complexity: 1.2,
    };

    let json = serde_json::to_string(&features).unwrap();
    assert!(json.contains("\"mean\":1.5"));
    assert!(json.contains("\"stationarity_passed\":true"));
}

#[test]
fn test_signal_features_deserialization() {
    let json = r#"{
        "mean": 2.0,
        "variance": 1.0,
        "stationarity_passed": false,
        "hjorth_activity": 1.0,
        "hjorth_mobility": 0.5,
        "hjorth_complexity": 0.8
    }"#;

    let features: SignalFeatures = serde_json::from_str(json).unwrap();
    assert_eq!(features.mean, 2.0);
    assert_eq!(features.variance, 1.0);
    assert!(!features.stationarity_passed);
}

#[test]
fn test_signal_window_clone() {
    let mut window = SignalWindow::new();
    window.add(1.0);
    window.add(2.0);

    let cloned = window.clone();
    assert_eq!(cloned.data_points.len(), 2);
    assert_eq!(cloned.data_points[0], 1.0);
}
