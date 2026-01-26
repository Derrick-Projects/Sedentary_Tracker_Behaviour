pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

// Subtracts two unsigned 64-bit integers
// Returns None if the result would underflow
pub fn checked_sub(left: u64, right: u64) -> Option<u64> {
    left.checked_sub(right)
}

// Multiplies two unsigned 64-bit integers
// Returns None if the result would overflow
pub fn checked_mul(left: u64, right: u64) -> Option<u64> {
    left.checked_mul(right)
}

// Divides two unsigned 64-bit integers
// Returns None if divisor is zero
pub fn checked_div(left: u64, right: u64) -> Option<u64> {
    left.checked_div(right)
}

#[cfg(test)]
mod tests;
