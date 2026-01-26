use super::*;
// Addition Tests
#[test]
fn test_add_basic() {
    let result = add(2, 2);
    assert_eq!(result, 4);
}

#[test]
fn test_add_zeros() {
    let result = add(0, 0);
    assert_eq!(result, 0);
}

#[test]
fn test_add_with_zero() {
    assert_eq!(add(5, 0), 5);
    assert_eq!(add(0, 5), 5);
}

#[test]
fn test_add_large_numbers() {
    let result = add(1_000_000, 2_000_000);
    assert_eq!(result, 3_000_000);
}

#[test]
fn test_add_max_value() {
    let result = add(u64::MAX - 1, 1);
    assert_eq!(result, u64::MAX);
}

// Subtraction Tests

#[test]
fn test_checked_sub_basic() {
    assert_eq!(checked_sub(5, 3), Some(2));
}

#[test]
fn test_checked_sub_equal() {
    assert_eq!(checked_sub(5, 5), Some(0));
}

#[test]
fn test_checked_sub_underflow() {
    assert_eq!(checked_sub(3, 5), None);
}

#[test]
fn test_checked_sub_zero() {
    assert_eq!(checked_sub(10, 0), Some(10));
}

// Checked Multiplication Tests

#[test]
fn test_checked_mul_basic() {
    assert_eq!(checked_mul(3, 4), Some(12));
}

#[test]
fn test_checked_mul_by_zero() {
    assert_eq!(checked_mul(100, 0), Some(0));
    assert_eq!(checked_mul(0, 100), Some(0));
}

#[test]
fn test_checked_mul_by_one() {
    assert_eq!(checked_mul(42, 1), Some(42));
}

#[test]
fn test_checked_mul_overflow() {
    assert_eq!(checked_mul(u64::MAX, 2), None);
}

// Checked Division Tests

#[test]
fn test_checked_div_basic() {
    assert_eq!(checked_div(10, 2), Some(5));
}

#[test]
fn test_checked_div_by_zero() {
    assert_eq!(checked_div(10, 0), None);
}

#[test]
fn test_checked_div_by_one() {
    assert_eq!(checked_div(42, 1), Some(42));
}

#[test]
fn test_checked_div_truncation() {
    assert_eq!(checked_div(7, 2), Some(3)); // Integer division truncates
}

#[test]
fn test_checked_div_zero_dividend() {
    assert_eq!(checked_div(0, 5), Some(0));
}
