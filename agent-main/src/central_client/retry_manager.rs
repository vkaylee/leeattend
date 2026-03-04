use std::time::{Duration, Instant};
use log::{info, warn};
use rand::Rng;

use super::connection::{MAX_RETRY_DELAY_SECS, MAX_RETRY_COUNT, STABILITY_THRESHOLD_SECS};

/// Parse composite key "device_id:user_id:timestamp_unix:punch:status" back to components
/// Uses rsplit to safely handle device_id containing colons
/// Format: device_id can contain colons, user_id, timestamp_unix, punch, and status cannot
/// Returns timestamp as i64 (Unix timestamp in UTC seconds)
pub(super) fn parse_composite_key(key: &str) -> Option<(String, String, i64, Option<i32>, Option<i32>)> {
    let mut parts = key.rsplitn(4, ':');
    
    // 1. Extract status, punch, timestamp
    let status_str = parts.next()?;
    let punch_str = parts.next()?;
    let timestamp_str = parts.next()?;
    let rest = parts.next()?; // device_id:user_id

    let status: Option<i32> = status_str.parse().ok().filter(|&v| v != -1);
    let punch: Option<i32> = punch_str.parse().ok().filter(|&v| v != -1);
    let timestamp_unix = timestamp_str.parse::<i64>().ok()?;

    // 2. Extract Device ID and User ID
    // Strategy: If device_id looks like a UUID (36 chars), split from left.
    // Otherwise, assume strict "no colons in user_id" rule (legacy MAC address support).
    let (device_id, user_id) = if rest.len() > 36 && rest.as_bytes()[36] == b':' {
        // Assume UUID device_id
        (&rest[..36], &rest[37..]) // Skip the colon
    } else {
        // Fallback: Split from right (preserves colons in device_id, assumes none in user_id)
        match rest.rsplit_once(':') {
            Some((d, u)) => (d, u),
            None => return None,
        }
    };

    Some((device_id.trim().to_string(), user_id.trim().to_string(), timestamp_unix, punch, status))
}

pub(super) struct RetryManager {
    pub(super) retry_count: u32,
    pub(super) stable_since: Option<Instant>,
}

impl RetryManager {
    pub(super) fn new() -> Self {
        Self {
            retry_count: 0,
            stable_since: None,
        }
    }

    pub(super) fn get_delay(&self) -> Duration {
        if self.retry_count == 0 {
            return Duration::from_secs(0);
        }

        // Exponential backoff: 1s, 2s, 4s, 8s, 16s, 32s, 60s (capped)
        let base_delay = std::cmp::min(
            2u64.pow(std::cmp::min(self.retry_count - 1, 31)),
            MAX_RETRY_DELAY_SECS
        );

        // Apply ±25% jitter to prevent thundering herd
        let mut rng = rand::thread_rng();
        let jitter_factor = rng.gen_range(0.75..=1.25);
        let jittered = (base_delay as f64 * jitter_factor) as u64;

        // Ensure minimum 1s and maximum 60s after jitter
        let final_delay = jittered.clamp(1, MAX_RETRY_DELAY_SECS);
        Duration::from_secs(final_delay)
    }

    pub(super) fn record_connection_established(&mut self) {
        // Don't reset retry_count immediately - wait for stability
        // Start the stability timer
        self.stable_since = Some(Instant::now());
        info!("Connection established, starting stability timer");
    }

    pub(super) fn record_connection_failed(&mut self) {
        self.retry_count = std::cmp::min(self.retry_count + 1, MAX_RETRY_COUNT);
        self.stable_since = None;
        warn!("Connection failed, retry count: {}", self.retry_count);
    }

    pub(super) fn record_dropped_prematurely(&mut self) {
        self.record_connection_failed();
    }

    #[allow(dead_code)]
    pub(super) fn record_stable(&mut self) {
        if let Some(since) = self.stable_since {
            if since.elapsed() >= Duration::from_secs(STABILITY_THRESHOLD_SECS)
                && self.retry_count > 0 {
                    info!("Connection stable for {STABILITY_THRESHOLD_SECS}s, resetting retry count");
                    self.retry_count = 0;
                }
                // Keep stable_since as-is to avoid repeated reset logs
        } else {
            // First stable call after connection - start the timer
            self.stable_since = Some(Instant::now());
        }
    }
}

pub(super) fn calculate_sync_stats(logs: &[crate::redb_store::StoredLog]) -> String {
    let new_count = logs.iter().filter(|l| l.retry_count == 0).count();
    let resync_logs: Vec<&crate::redb_store::StoredLog> = logs.iter().filter(|l| l.retry_count > 0).collect();
    let resync_count = resync_logs.len();

    if resync_count > 0 {
        let min_retry = resync_logs.iter().map(|l| l.retry_count).min().unwrap_or(0);
        let max_retry = resync_logs.iter().map(|l| l.retry_count).max().unwrap_or(0);
        if min_retry == max_retry {
            format!("{new_count} New, {resync_count} Resyncs (Attempt #{min_retry})")
        } else {
            format!("{new_count} New, {resync_count} Resyncs (Attempt #{min_retry}-{max_retry})")
        }
    } else {
        format!("{new_count} New")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::redb_store::StoredLog;

    #[test]
    fn test_parse_composite_key_normal() {
        let key = "DEV1:U001:1738314000:0:1";
        let result = parse_composite_key(key);
        assert!(result.is_some());
        let (device_id, user_id, timestamp_unix, punch, status) = result.unwrap();
        assert_eq!(device_id, "DEV1");
        assert_eq!(user_id, "U001");
        assert_eq!(timestamp_unix, 1738314000);
        assert_eq!(punch, Some(0));
        assert_eq!(status, Some(1));
    }

    #[test]
    fn test_parse_composite_key_with_colon_in_device_id() {
        let key = "Office:Main:Gate:U001:1738314000:-1:-1";
        let result = parse_composite_key(key);
        assert!(result.is_some());
        let (device_id, user_id, timestamp_unix, punch, status) = result.unwrap();
        assert_eq!(device_id, "Office:Main:Gate");
        assert_eq!(user_id, "U001");
        assert_eq!(timestamp_unix, 1738314000);
        assert_eq!(punch, None);
        assert_eq!(status, None);
    }

    #[test]
    fn test_parse_composite_key_logic() {
        let k1 = "DEV1:U1:1234567890:0:1";
        assert_eq!(parse_composite_key(k1), Some(("DEV1".into(), "U1".into(), 1234567890, Some(0), Some(1))));

        let k2 = "AA:BB:CC:DD:EE:FF:U1:1234567890:-1:-1";
        assert_eq!(parse_composite_key(k2), Some(("AA:BB:CC:DD:EE:FF".into(), "U1".into(), 1234567890, None, None)));

        let k3 = "D1:U1:INVALID:0:1";
        assert_eq!(parse_composite_key(k3), None);
    }

    #[test]
    fn test_retry_manager_initial_delay_is_zero() {
        let manager = RetryManager::new();
        assert_eq!(manager.get_delay(), Duration::from_secs(0));
    }

    #[test]
    fn test_retry_manager_exponential_backoff_with_jitter() {
        let mut manager = RetryManager::new();
        manager.record_connection_failed();
        let delay1 = manager.get_delay();
        assert!(delay1.as_secs() >= 1);
        assert_eq!(manager.retry_count, 1);
    }

    #[test]
    fn test_retry_manager_max_delay_is_60_seconds() {
        let mut manager = RetryManager::new();
        for _ in 0..15 {
            manager.record_connection_failed();
        }
        assert_eq!(manager.retry_count, MAX_RETRY_COUNT);
        let delay = manager.get_delay();
        assert!(delay.as_secs() <= 75);
    }

    fn make_stored_log(id: i64, retry_count: i64) -> StoredLog {
        StoredLog {
            id,
            device_id: "D1".into(),
            user_id: "U1".into(),
            timestamp: 1672567200,
            status: Some(1),
            punch: Some(1),
            is_synced: 0,
            created_at: chrono::Utc::now().timestamp(),
            updated_at: if retry_count > 0 { Some(chrono::Utc::now().timestamp()) } else { None },
            retry_count,
        }
    }

    #[test]
    fn test_calculate_sync_stats_new_only() {
        let logs = vec![make_stored_log(1, 0)];
        assert_eq!(calculate_sync_stats(&logs), "1 New");
    }

    #[test]
    fn test_calculate_sync_stats_mixed() {
        let logs = vec![make_stored_log(1, 0), make_stored_log(2, 3), make_stored_log(3, 5)];
        assert_eq!(calculate_sync_stats(&logs), "1 New, 2 Resyncs (Attempt #3-5)");
    }

    #[test]
    fn test_calculate_sync_stats_empty() {
        let logs: Vec<StoredLog> = vec![];
        assert_eq!(calculate_sync_stats(&logs), "0 New");
    }
}
