use chrono::{DateTime, Utc};
use log::{debug, warn};
use std::sync::atomic::{AtomicI64, Ordering};

/// Thread-safe server-time synchronization.
///
/// Computes and stores the offset between the server clock and the local
/// system clock.  All critical timestamps in the agent should use
/// [`corrected_now`] / [`corrected_timestamp`] instead of raw `Utc::now()`.
///
/// The offset is recalculated on every heartbeat ack from the server,
/// so even if the local system clock drifts or is changed mid-session,
/// timestamps will self-correct within one heartbeat interval (~30 s).
pub struct TimeSync {
    /// offset_ms = server_utc_ms − local_utc_ms
    offset_ms: AtomicI64,
}

impl TimeSync {
    pub const fn new() -> Self {
        Self {
            offset_ms: AtomicI64::new(0),
        }
    }

    /// Update the offset from a server-provided RFC 3339 timestamp.
    ///
    /// Called every time the agent receives a `heartbeat_ack` message.
    pub fn update_offset(&self, server_time_rfc3339: &str) {
        let server_dt = match DateTime::parse_from_rfc3339(server_time_rfc3339) {
            Ok(dt) => dt.with_timezone(&Utc),
            Err(e) => {
                warn!("TimeSync: failed to parse server_time '{server_time_rfc3339}': {e}");
                return;
            }
        };

        let local_now = Utc::now();
        let offset = server_dt
            .signed_duration_since(local_now)
            .num_milliseconds();

        let prev = self.offset_ms.swap(offset, Ordering::Relaxed);

        if (offset - prev).abs() > 1000 {
            debug!(
                "TimeSync: offset updated {prev}ms → {offset}ms (server={}, local={})",
                server_dt.to_rfc3339(),
                local_now.to_rfc3339()
            );
        }
    }

    /// Returns `Utc::now()` corrected by the server-time offset.
    pub fn corrected_now(&self) -> DateTime<Utc> {
        let offset_ms = self.offset_ms.load(Ordering::Relaxed);
        Utc::now() + chrono::Duration::milliseconds(offset_ms)
    }

    /// Returns a corrected Unix timestamp (seconds since epoch).
    pub fn corrected_timestamp(&self) -> i64 {
        self.corrected_now().timestamp()
    }

    /// Returns the current offset in milliseconds (for diagnostics).
    #[allow(dead_code)]
    pub fn offset_ms(&self) -> i64 {
        self.offset_ms.load(Ordering::Relaxed)
    }
}

impl Default for TimeSync {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_has_zero_offset() {
        let ts = TimeSync::new();
        assert_eq!(ts.offset_ms(), 0);
    }

    #[test]
    fn test_corrected_now_with_zero_offset() {
        let ts = TimeSync::new();
        let before = Utc::now();
        let corrected = ts.corrected_now();
        let after = Utc::now();
        // corrected should be within [before, after]
        assert!(corrected >= before && corrected <= after);
    }

    #[test]
    fn test_update_offset_with_valid_rfc3339() {
        let ts = TimeSync::new();
        // Simulate server being 5 seconds ahead
        let server_time = (Utc::now() + chrono::Duration::seconds(5)).to_rfc3339();
        ts.update_offset(&server_time);
        // offset should be approximately +5000ms (allow 500ms tolerance for test execution)
        let offset = ts.offset_ms();
        assert!(
            offset > 4500 && offset < 5500,
            "offset was {offset}ms, expected ~5000ms"
        );
    }

    #[test]
    fn test_update_offset_with_invalid_string() {
        let ts = TimeSync::new();
        ts.update_offset("not-a-date");
        // offset should remain 0
        assert_eq!(ts.offset_ms(), 0);
    }

    #[test]
    fn test_corrected_timestamp_with_positive_offset() {
        let ts = TimeSync::new();
        let server_time = (Utc::now() + chrono::Duration::seconds(10)).to_rfc3339();
        ts.update_offset(&server_time);

        let raw = Utc::now().timestamp();
        let corrected = ts.corrected_timestamp();
        // corrected should be ~10 seconds ahead of raw
        let diff = corrected - raw;
        assert!((9..=11).contains(&diff), "diff was {diff}s, expected ~10s");
    }

    #[test]
    fn test_corrected_timestamp_with_negative_offset() {
        let ts = TimeSync::new();
        let server_time = (Utc::now() - chrono::Duration::seconds(10)).to_rfc3339();
        ts.update_offset(&server_time);

        let raw = Utc::now().timestamp();
        let corrected = ts.corrected_timestamp();
        let diff = raw - corrected;
        assert!((9..=11).contains(&diff), "diff was {diff}s, expected ~10s");
    }

    #[test]
    fn test_offset_updates_on_re_sync() {
        let ts = TimeSync::new();

        // First sync: server 5s ahead
        let t1 = (Utc::now() + chrono::Duration::seconds(5)).to_rfc3339();
        ts.update_offset(&t1);
        let offset1 = ts.offset_ms();
        assert!(offset1 > 4000, "first offset was {offset1}ms");

        // Second sync: server 2s ahead (clock corrected)
        let t2 = (Utc::now() + chrono::Duration::seconds(2)).to_rfc3339();
        ts.update_offset(&t2);
        let offset2 = ts.offset_ms();
        assert!(
            offset2 > 1000 && offset2 < 3000,
            "second offset was {offset2}ms"
        );
    }
}
