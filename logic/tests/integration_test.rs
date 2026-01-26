// Integration tests for the logic crate
// These test the public API as an external consumer would use it

use logic::{SignalFeatures, SignalWindow, calculate_hjorth_params, check_stationarity};

#[test]
fn test_signal_processing_workflow() {
    // Simulate a real-world workflow: collect data, then analyze it
    let mut window = SignalWindow::new();

    // Simulate 100 accelerometer readings sedentary behavior
    for i in 0..100 {
        let magnitude = 1.0 + (i as f64 * 0.01).sin() * 0.1; // Small oscillations
        window.add(magnitude);
    }

    // Analyze the collected data
    let features = calculate_hjorth_params(&window.data_points);

    // Sedentary behavior should show low activity and be stationary
    assert!(
        features.hjorth_activity < 0.1,
        "Sedentary signal should have low activity"
    );
    assert!(
        features.stationarity_passed,
        "Sedentary signal should be stationary"
    );
}

#[test]
fn test_active_vs_sedentary_classification() {
    // Test that we can distinguish between active and sedentary signals

    // Sedentary signal: low variance, stationary
    let sedentary_data: Vec<f64> = (0..200)
        .map(|i| 1.0 + (i as f64 * 0.05).sin() * 0.05)
        .collect();
    let sedentary_features = calculate_hjorth_params(&sedentary_data);

    // Active signal: high variance
    let active_data: Vec<f64> = (0..200).map(|i| (i as f64 * 0.3).sin() * 5.0).collect();
    let active_features = calculate_hjorth_params(&active_data);

    // Active signal should have higher activity than sedentary
    assert!(
        active_features.hjorth_activity > sedentary_features.hjorth_activity,
        "Active signal should have higher activity than sedentary"
    );
}

#[test]
fn test_signal_window_data_collection() {
    // Test the data collection buffer behavior
    let mut window = SignalWindow::new();

    // Collect data points
    for i in 0..500 {
        window.add(i as f64);
    }

    assert_eq!(window.data_points.len(), 500);

    // Verify we can analyze the collected data
    let features = calculate_hjorth_params(&window.data_points);

    // Linear increasing data should have positive variance
    assert!(
        features.variance > 0.0,
        "Trending data should have positive variance"
    );
    assert!(
        features.hjorth_activity > 0.0,
        "Trending data should have activity"
    );
}

#[test]
fn test_stationarity_detection() {
    // Test stationarity detection with different signal types

    // Constant signal (stationary)
    let constant: Vec<f64> = vec![42.0; 100];
    assert!(
        check_stationarity(&constant, 4),
        "Constant signal should be stationary"
    );

    // Signal with varying variance levels (non-stationary)
    let mut varying: Vec<f64> = (0..50).map(|i| (i as f64 * 10.0).sin() * 100.0).collect();
    varying.extend(vec![0.0; 50]); // Add flat section
    assert!(
        !check_stationarity(&varying, 4),
        "Signal with varying variance should not be stationary"
    );
}

#[test]
fn test_hjorth_parameters_edge_cases() {
    // Test with minimum viable data
    let small_data: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let features = calculate_hjorth_params(&small_data);

    // Should compute without panicking
    assert!(features.mean > 0.0);
    assert!(features.variance > 0.0);
}

#[test]
fn test_signal_features_serialization_roundtrip() {
    // Test that features can be serialized for sending to frontend/DB
    let features = SignalFeatures {
        mean: 1.5,
        variance: 0.25,
        stationarity_passed: true,
        hjorth_activity: 0.25,
        hjorth_mobility: 0.1,
        hjorth_complexity: 1.5,
    };

    // Serialize to JSON
    let json = serde_json::to_string(&features).expect("Should serialize");

    // Deserialize back
    let restored: SignalFeatures = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(restored.mean, features.mean);
    assert_eq!(restored.stationarity_passed, features.stationarity_passed);
}
