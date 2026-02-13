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
