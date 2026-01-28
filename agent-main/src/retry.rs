//! Retry logic with exponential backoff and jitter for connection resilience.
//!
//! Provides configurable retry delays for sidecar and WebSocket connections.

use rand::Rng;
use std::time::Duration;

/// Sidecar retry configuration
pub const SIDECAR_MAX_RETRY_DELAY_SECS: u64 = 60;
pub const SIDECAR_MAX_RETRY_COUNT: u32 = 10;
pub const SIDECAR_STABILITY_THRESHOLD_SECS: u64 = 30;

/// Calculates exponential backoff delay with jitter for sidecar retries.
///
/// # Arguments
/// * `retry_count` - Current retry attempt number (0 = first attempt)
///
/// # Returns
/// Duration to wait before next retry attempt.
///
/// # Behavior
/// - retry_count=0: Returns 2s (initial delay)
/// - retry_count=1-6: Exponential backoff (2s, 4s, 8s, 16s, 32s, 64s)
/// - retry_count>6: Capped at 60s
/// - All delays have ±25% jitter applied
pub fn get_sidecar_retry_delay(retry_count: u32) -> Duration {
    if retry_count == 0 {
        return Duration::from_secs(2); // Initial delay
    }

    let base_delay = std::cmp::min(
        2u64.pow(std::cmp::min(retry_count, 6)), // 2, 4, 8, 16, 32, 64
        SIDECAR_MAX_RETRY_DELAY_SECS,
    );

    // Apply ±25% jitter to prevent thundering herd
    let mut rng = rand::thread_rng();
    let jitter_factor = rng.gen_range(0.75..=1.25);
    let jittered = (base_delay as f64 * jitter_factor) as u64;

    Duration::from_secs(std::cmp::max(2, std::cmp::min(jittered, SIDECAR_MAX_RETRY_DELAY_SECS)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_retry_returns_2_seconds() {
        let delay = get_sidecar_retry_delay(0);
        assert_eq!(delay, Duration::from_secs(2));
    }

    #[test]
    fn test_first_retry_is_around_2_seconds() {
        // First retry (count=1): base is 2^1 = 2s, with jitter 1.5s-2.5s
        let delay = get_sidecar_retry_delay(1);
        assert!(delay.as_secs() >= 1, "Delay should be >= 1s (2s - 25% jitter, min 2s)");
        assert!(delay.as_secs() <= 2, "Delay should be <= 2s");
    }

    #[test]
    fn test_exponential_growth() {
        // Test that higher retry counts produce larger delays (on average)
        // We'll run multiple samples to account for jitter
        let mut sum_low: u64 = 0;
        let mut sum_high: u64 = 0;

        for _ in 0..10 {
            sum_low += get_sidecar_retry_delay(2).as_secs();  // base 4s
            sum_high += get_sidecar_retry_delay(5).as_secs(); // base 32s
        }

        // Average of higher retry should be significantly larger
        assert!(sum_high > sum_low, "Higher retry count should produce larger delays");
    }

    #[test]
    fn test_max_delay_is_capped_at_60_seconds() {
        // Even with high retry count, delay should not exceed 60s (+25% jitter = 75s max)
        for _ in 0..20 {
            let delay = get_sidecar_retry_delay(100);
            assert!(delay.as_secs() <= 75, "Delay should be <= 75s (60s + 25% jitter)");
            assert!(delay.as_secs() >= 45, "Delay should be >= 45s (60s - 25% jitter)");
        }
    }

    #[test]
    fn test_delay_never_below_2_seconds() {
        // Minimum delay should be 2s due to clamping
        for retry_count in 0..20 {
            let delay = get_sidecar_retry_delay(retry_count);
            assert!(delay.as_secs() >= 2, "Delay should never be below 2s");
        }
    }

    #[test]
    fn test_jitter_produces_variation() {
        // Same retry count should produce different delays due to jitter
        let mut delays: Vec<u64> = Vec::new();
        for _ in 0..10 {
            delays.push(get_sidecar_retry_delay(4).as_millis() as u64);
        }

        // Check that not all delays are identical (jitter is working)
        let first = delays[0];
        let has_variation = delays.iter().any(|&d| d != first);
        // Note: This could theoretically fail if RNG produces same values, but very unlikely
        assert!(has_variation || delays.len() < 2, "Jitter should produce variation in delays");
    }

    #[test]
    fn test_retry_count_progression() {
        // Verify the expected base delays for each retry count
        // Base delays: 0->2s, 1->2s, 2->4s, 3->8s, 4->16s, 5->32s, 6->64s(capped to 60s)
        let expected_bases = [2, 2, 4, 8, 16, 32, 60];

        for (count, &expected_base) in expected_bases.iter().enumerate() {
            let delay = get_sidecar_retry_delay(count as u32);
            let min_expected = (expected_base as f64 * 0.75) as u64;
            let max_expected = std::cmp::min((expected_base as f64 * 1.25) as u64, 60);

            // Account for the minimum clamp of 2s
            let min_expected = std::cmp::max(min_expected, 2);

            assert!(
                delay.as_secs() >= min_expected && delay.as_secs() <= max_expected,
                "Retry count {} should produce delay in range [{}, {}], got {}",
                count,
                min_expected,
                max_expected,
                delay.as_secs()
            );
        }
    }
}
