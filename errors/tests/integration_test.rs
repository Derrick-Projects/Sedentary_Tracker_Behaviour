// Integration tests for the errors crate
// These test the public API as an external consumer would use it

use errors::{add, checked_div, checked_mul, checked_sub};

#[test]
fn test_math_operations_workflow() {
    let a = 100u64;
    let b = 25u64;

    let sum = add(a, b);
    let product = checked_mul(sum, 2);
    let quotient = checked_div(product.unwrap(), 5);
    let difference = checked_sub(quotient.unwrap(), 10);

    assert_eq!(difference, Some(40));
}

#[test]
fn test_safe_division_by_user_input() {
    // Simulate handling potentially dangerous user input
    let numerator = 100u64;
    let user_inputs = [0, 1, 5, 10, 0, 20];

    let results: Vec<Option<u64>> = user_inputs
        .iter()
        .map(|&divisor| checked_div(numerator, divisor))
        .collect();

    // Division by zero should return None, not panic
    assert_eq!(results[0], None); // 100 / 0
    assert_eq!(results[1], Some(100)); // 100 / 1
    assert_eq!(results[2], Some(20)); // 100 / 5
    assert_eq!(results[4], None); // 100 / 0 again
}

#[test]
fn test_overflow_protection() {
    // Test that handle overflow safely
    let large = u64::MAX;

    // panic without checked operations
    assert_eq!(checked_mul(large, 2), None);
    assert_eq!(checked_sub(0, 1), None);

    // But valid operations still work
    assert_eq!(checked_mul(large, 1), Some(large));
    assert_eq!(checked_sub(large, 0), Some(large));
}

#[test]
fn test_accumulator_pattern() {
    // Simulate accumulating values
    let mut accumulator = 0u64;
    let increments = [10, 20, 30, 40, 50];

    for inc in increments {
        accumulator = add(accumulator, inc);
    }

    assert_eq!(accumulator, 150);
}

#[test]
fn test_timer_calculations() {
    // Simulate timer calculations for sedentary tracking
    let session_seconds = 3600u64; // 1 hour
    let break_seconds = 300u64; // 5 minutes

    // Calculate active time
    let active_time = checked_sub(session_seconds, break_seconds);
    assert_eq!(active_time, Some(3300));

    // Calculate average per segment (12 segments)
    let segments = 12u64;
    let per_segment = checked_div(session_seconds, segments);
    assert_eq!(per_segment, Some(300)); // 5 minutes per segment
}
