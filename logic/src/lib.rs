use serde::{Deserialize, Serialize};

// Holds a window of accelerometer data
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SignalWindow {
    pub data_points: Vec<f64>,
}

impl SignalWindow {
    pub fn new() -> Self {
        SignalWindow {
            data_points: Vec::new(),
        }
    }

    // Add a new magnitude value (sqrt(x^2 + y^2 + z^2))
    pub fn add(&mut self, magnitude: f64) {
        self.data_points.push(magnitude);
        // Keep buffer size manageable
        if self.data_points.len() > 2000 {
            self.data_points.remove(0);
        }
    }
}

// Scientific Features extracted from the signal
#[derive(Debug, Serialize, Deserialize)]
pub struct SignalFeatures {
    pub mean: f64,
    pub variance: f64,
    pub stationarity_passed: bool,
    pub hjorth_activity: f64,
    pub hjorth_mobility: f64,
    pub hjorth_complexity: f64,
}

// 1. Stationarity Test mathcal{S}
// Checks if the signal's statistical properties (mean/variance) are constant over time.
// We divide the signal into M segments and compare them.
pub fn check_stationarity(data: &[f64], segments: usize) -> bool {
    if data.len() < segments {
        return false;
    }

    let chunk_size = data.len() / segments;
    let mut segment_variances = Vec::new();

    for chunk in data.chunks(chunk_size) {
        let mean: f64 = chunk.iter().sum::<f64>() / chunk.len() as f64;
        let variance: f64 =
            chunk.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / chunk.len() as f64;
        segment_variances.push(variance);
    }

    // Calculate variance OF the variances
    let mean_var: f64 = segment_variances.iter().sum::<f64>() / segment_variances.len() as f64;
    let var_of_vars: f64 = segment_variances
        .iter()
        .map(|v| (v - mean_var).powi(2))
        .sum::<f64>()
        / segment_variances.len() as f64;

    // If the variance fluctuates too much, the signal is NOT stationary
    // Threshold needs calibration, but 0.05 is a standard starting point for normalized data.
    var_of_vars < 0.05
}

// 2. Hjorth Parameters
// - Activity: Variance of the signal
// - Mobility: sqrt(Var(deriv) / Var(signal))
// - Complexity: Mobility(deriv) / Mobility(signal)
pub fn calculate_hjorth_params(data: &[f64]) -> SignalFeatures {
    let n = data.len() as f64;

    // 0th Derivative
    let mean: f64 = data.iter().sum::<f64>() / n;
    let variance: f64 = data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
    let activity = variance;

    // 1st Derivative (Discrete difference)
    let mut deriv1 = Vec::new();
    for i in 1..data.len() {
        deriv1.push(data[i] - data[i - 1]);
    }
    let n1 = deriv1.len() as f64;
    let mean1: f64 = deriv1.iter().sum::<f64>() / n1;
    let var1: f64 = deriv1.iter().map(|x| (x - mean1).powi(2)).sum::<f64>() / n1;

    let mobility = (var1 / activity).sqrt();

    // 2nd Derivative (Difference of the difference)
    let mut deriv2 = Vec::new();
    for i in 1..deriv1.len() {
        deriv2.push(deriv1[i] - deriv1[i - 1]);
    }
    let n2 = deriv2.len() as f64;
    let mean2: f64 = deriv2.iter().sum::<f64>() / n2;
    let var2: f64 = deriv2.iter().map(|x| (x - mean2).powi(2)).sum::<f64>() / n2;

    let mobility_deriv = (var2 / var1).sqrt();
    let complexity = mobility_deriv / mobility;

    // Run Stationarity Test on the raw data
    let is_stationary = check_stationarity(data, 16);

    SignalFeatures {
        mean,
        variance,
        stationarity_passed: is_stationary,
        hjorth_activity: activity,
        hjorth_mobility: if mobility.is_nan() { 0.0 } else { mobility },
        hjorth_complexity: if complexity.is_nan() { 0.0 } else { complexity },
    }
}

#[cfg(test)]
mod tests;
