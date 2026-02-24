/// Maximum retry delay cap in seconds (5 minutes)
pub(super) const MAX_RETRY_DELAY_SECS: i64 = 300;

/// Maximum bit shift for exponential backoff (2^8 = 256s, then capped to 300s)
pub(super) const MAX_RETRY_SHIFT: i64 = 8;

/// Jitter percentage (±10%)
pub(super) const JITTER_PERCENT: f64 = 0.10;

/// Calculate exponential backoff delay with jitter.
/// Formula: min(2^retry_count, MAX_RETRY_DELAY_SECS) ± 10% jitter
///
/// Delay progression: 1s → 2s → 4s → 8s → 16s → 32s → 64s → 128s → 256s → 300s (capped)
pub(super) fn calculate_backoff_with_jitter(retry_count: i64) -> i64 {
    use rand::Rng;

    // Base delay: 2^retry_count, capped at MAX_RETRY_SHIFT bits
    let base_delay = (1i64 << retry_count.min(MAX_RETRY_SHIFT)).min(MAX_RETRY_DELAY_SECS);

    // Apply ±10% jitter to prevent thundering herd
    let jitter_range = (base_delay as f64 * JITTER_PERCENT) as i64;
    if jitter_range > 0 {
        let jitter = rand::thread_rng().gen_range(-jitter_range..=jitter_range);
        // Ensure delay is at least 1 second
        (base_delay + jitter).max(1)
    } else {
        base_delay
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backoff_first_attempt_is_one_second() {
        // 2^0 = 1. No jitter (range = 0), returns 1.
        assert_eq!(calculate_backoff_with_jitter(0), 1);
    }

    #[test]
    fn test_backoff_exponential_growth() {
        // Base delays: 2^1=2, 2^4=16, 2^8=256 (capped to 300)
        for _ in 0..10 {
            let d1 = calculate_backoff_with_jitter(1);
            let d4 = calculate_backoff_with_jitter(4);
            assert!(d4 > d1, "Higher retry should produce larger delay");
        }
    }

    #[test]
    fn test_backoff_cap_at_max() {
        // At high retry counts: 2^MAX_RETRY_SHIFT = 2^8 = 256, which is < MAX_RETRY_DELAY_SECS (300).
        // With ±10% jitter the exact value varies, but must always satisfy these invariants:
        //   1. delay >= 1 (never zero or negative)
        //   2. delay <= MAX_RETRY_DELAY_SECS (hard cap)
        for _ in 0..30 {
            let delay = calculate_backoff_with_jitter(20); // Well past MAX_RETRY_SHIFT
            assert!(
                delay >= 1,
                "Delay must be >= 1s, got {delay}"
            );
            assert!(
                delay <= MAX_RETRY_DELAY_SECS,
                "Delay must not exceed MAX_RETRY_DELAY_SECS ({MAX_RETRY_DELAY_SECS}s), got {delay}"
            );
        }
    }

    #[test]
    fn test_backoff_never_below_one() {
        for count in 0..15 {
            let delay = calculate_backoff_with_jitter(count);
            assert!(delay >= 1, "Delay must never be below 1s, got {delay} for count={count}");
        }
    }

    #[test]
    fn test_backoff_jitter_produces_variation() {
        // Run multiple times at a count where jitter should apply (base=16 → jitter_range=1)
        let delays: Vec<i64> = (0..30).map(|_| calculate_backoff_with_jitter(4)).collect();
        let has_variation = delays.windows(2).any(|w| w[0] != w[1]);
        // Jitter_range at count=4 is 1 (floor(16*0.10)=1), so values vary by ±1
        assert!(
            has_variation,
            "Expected jitter to produce variation in delays: {:?}",
            &delays[..5]
        );
    }
}
