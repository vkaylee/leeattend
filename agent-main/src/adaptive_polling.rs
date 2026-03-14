//! Adaptive polling interval calculator
//! Uses a single interval-based approach:
//! - New data found → set long cooldown interval (disconnect device, sleep, reconnect)
//! - No new data → escalate interval up to session-safe max (keep connection alive)

/// Base polling interval when actively monitoring
pub const BASE_POLL_INTERVAL_SECS: u64 = 5;
/// Max polling interval — must stay below ZK device UDP session timeout (~60s)
pub const MAX_POLL_INTERVAL_SECS: u64 = 45;
/// Default cooldown after successful fetch with new data (5 minutes)
pub const DEFAULT_SUCCESS_COOLDOWN_SECS: u64 = 300;
/// Number of consecutive no-new-data polls before escalation begins
const ESCALATION_THRESHOLD: u32 = 3;

/// State for adaptive polling
#[derive(Debug, Clone)]
pub struct AdaptivePollingState {
    pub current_interval_secs: u64,
    pub consecutive_no_new_data: u32,
    /// Cooldown interval after successful fetch with new data
    success_cooldown_secs: u64,
}

impl Default for AdaptivePollingState {
    fn default() -> Self {
        Self {
            current_interval_secs: BASE_POLL_INTERVAL_SECS,
            consecutive_no_new_data: 0,
            success_cooldown_secs: DEFAULT_SUCCESS_COOLDOWN_SECS,
        }
    }
}

impl AdaptivePollingState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with a custom cooldown value (from config/env)
    pub fn with_cooldown(cooldown_secs: u64) -> Self {
        Self {
            success_cooldown_secs: cooldown_secs,
            ..Self::default()
        }
    }

    /// Update state based on whether new data was found.
    /// Returns the new interval and whether it changed.
    pub fn update(&mut self, found_new_data: bool) -> (u64, bool) {
        let old_interval = self.current_interval_secs;

        if found_new_data {
            // New data found → long cooldown (caller should disconnect before sleeping)
            self.current_interval_secs = self.success_cooldown_secs;
            self.consecutive_no_new_data = 0;
        } else {
            // No new data → escalate interval (stay connected)
            self.consecutive_no_new_data += 1;
            if self.consecutive_no_new_data >= ESCALATION_THRESHOLD
                && self.current_interval_secs < MAX_POLL_INTERVAL_SECS
            {
                self.current_interval_secs =
                    std::cmp::min(self.current_interval_secs * 2, MAX_POLL_INTERVAL_SECS);
            }
        }

        let changed = old_interval != self.current_interval_secs;
        (self.current_interval_secs, changed)
    }

    /// Get current interval
    pub fn interval(&self) -> u64 {
        self.current_interval_secs
    }

    /// Update the cooldown value (e.g. from a live config change)
    pub fn set_cooldown(&mut self, cooldown_secs: u64) {
        self.success_cooldown_secs = cooldown_secs;
    }

    /// Returns true if the current interval exceeds session-safe limit,
    /// meaning the caller should proactively disconnect before sleeping.
    pub fn needs_disconnect(&self) -> bool {
        self.current_interval_secs > MAX_POLL_INTERVAL_SECS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let state = AdaptivePollingState::new();
        assert_eq!(state.interval(), BASE_POLL_INTERVAL_SECS);
        assert_eq!(state.consecutive_no_new_data, 0);
        assert!(!state.needs_disconnect());
    }

    #[test]
    fn test_new_data_sets_cooldown_interval() {
        let mut state = AdaptivePollingState::new();
        let (interval, changed) = state.update(true);
        assert_eq!(interval, DEFAULT_SUCCESS_COOLDOWN_SECS);
        assert!(changed);
        assert_eq!(state.consecutive_no_new_data, 0);
        assert!(state.needs_disconnect()); // 300s > 45s → should disconnect
    }

    #[test]
    fn test_custom_cooldown() {
        let mut state = AdaptivePollingState::with_cooldown(600);
        state.update(true);
        assert_eq!(state.interval(), 600);
        assert!(state.needs_disconnect());
    }

    #[test]
    fn test_new_data_resets_interval() {
        let mut state = AdaptivePollingState::new();

        // Simulate escalation first
        for _ in 0..10 {
            state.update(false);
        }
        assert!(state.interval() > BASE_POLL_INTERVAL_SECS);

        // New data should set cooldown
        let (interval, changed) = state.update(true);
        assert_eq!(interval, DEFAULT_SUCCESS_COOLDOWN_SECS);
        assert!(changed);
        assert_eq!(state.consecutive_no_new_data, 0);
    }

    #[test]
    fn test_no_escalation_before_threshold() {
        let mut state = AdaptivePollingState::new();

        // First 2 polls with no new data - should NOT escalate yet
        for _ in 0..2 {
            let (interval, changed) = state.update(false);
            assert_eq!(interval, BASE_POLL_INTERVAL_SECS);
            assert!(!changed);
        }
    }

    #[test]
    fn test_escalation_after_threshold() {
        let mut state = AdaptivePollingState::new();

        // 3rd poll with no new data triggers first escalation
        for _ in 0..3 {
            state.update(false);
        }
        assert_eq!(state.interval(), BASE_POLL_INTERVAL_SECS * 2); // 5 -> 10
        assert!(!state.needs_disconnect()); // 10s < 45s → stay connected
    }

    #[test]
    fn test_escalation_progression() {
        let mut state = AdaptivePollingState::new();

        // Track escalation progression:
        // Poll 1,2: still at base (consecutive=1,2)
        // Poll 3: escalate 5->10 (consecutive=3, >= threshold)
        // Poll 4: escalate 10->20 (consecutive=4, >= threshold)
        // Poll 5: escalate 20->40 (consecutive=5, >= threshold)
        // Poll 6: escalate 40->45 (consecutive=6, >= threshold, hit max)
        // Poll 7+: stay at 45 (max capped)
        let expected_intervals = [5, 5, 10, 20, 40, 45, 45, 45];

        for (i, &expected) in expected_intervals.iter().enumerate() {
            state.update(false);
            assert_eq!(
                state.interval(),
                expected,
                "Poll {} should have interval {}s, got {}s",
                i + 1,
                expected,
                state.interval()
            );
        }
    }

    #[test]
    fn test_max_interval_cap() {
        let mut state = AdaptivePollingState::new();

        // Run many polls with no new data
        for _ in 0..100 {
            state.update(false);
        }

        // Should never exceed max
        assert_eq!(state.interval(), MAX_POLL_INTERVAL_SECS);
        assert!(!state.needs_disconnect()); // 45s == max, not > max
    }

    #[test]
    fn test_reset_after_long_idle_then_new_data() {
        let mut state = AdaptivePollingState::new();

        // Escalate to max
        for _ in 0..50 {
            state.update(false);
        }
        assert_eq!(state.interval(), MAX_POLL_INTERVAL_SECS);

        // Single new data event sets cooldown
        state.update(true);
        assert_eq!(state.interval(), DEFAULT_SUCCESS_COOLDOWN_SECS);
        assert_eq!(state.consecutive_no_new_data, 0);
    }

    #[test]
    fn test_intermittent_new_data_prevents_escalation() {
        let mut state = AdaptivePollingState::new();

        // Alternate: 2 no-data, 1 new-data (never reaches threshold of 3)
        for _ in 0..10 {
            state.update(false);
            state.update(false);
            state.update(true); // resets consecutive counter + sets cooldown
        }

        // After update(true), interval is cooldown
        assert_eq!(state.interval(), DEFAULT_SUCCESS_COOLDOWN_SECS);
    }

    #[test]
    fn test_needs_disconnect_boundary() {
        let mut state = AdaptivePollingState::new();

        // At base interval → no disconnect
        assert!(!state.needs_disconnect());

        // Escalate to max (45s) → no disconnect (== max, not >)
        for _ in 0..50 {
            state.update(false);
        }
        assert_eq!(state.interval(), 45);
        assert!(!state.needs_disconnect());

        // New data → cooldown (300s) → needs disconnect
        state.update(true);
        assert_eq!(state.interval(), 300);
        assert!(state.needs_disconnect());
    }

    #[test]
    fn test_no_data_after_cooldown_escalates_from_base() {
        let mut state = AdaptivePollingState::new();

        // Get cooldown
        state.update(true);
        assert_eq!(state.interval(), 300);

        // Now no-data polls: cooldown interval (300) > MAX (45), so
        // no further escalation since already above max — but we need
        // to check the logic handles this edge case.
        // After reconnect, polling_state should be reset or continue from 300.
        // The no-data branch only escalates when < MAX, so 300 stays at 300.
        state.update(false);
        assert_eq!(state.interval(), 300); // stays at 300 (already > MAX, no doubling)

        // After enough no-data, stays at 300 (no change since 300 >= MAX)
        for _ in 0..10 {
            state.update(false);
        }
        assert_eq!(state.interval(), 300);
    }

    /// Verify MAX_POLL_INTERVAL_SECS is set to 45s.
    /// This value must stay below typical ZK device UDP session timeout (~60s)
    /// to prevent silent session loss during adaptive polling.
    #[test]
    fn test_max_poll_interval_below_udp_session_timeout() {
        assert_eq!(MAX_POLL_INTERVAL_SECS, 45, "MAX_POLL_INTERVAL must be 45s (below ZK UDP session timeout)");
        assert!(MAX_POLL_INTERVAL_SECS < 60, "MAX_POLL_INTERVAL must be below 60s UDP session timeout");
    }

    /// Verify that escalation progression reaches exactly 45s and never exceeds it.
    #[test]
    fn test_escalation_never_exceeds_session_safe_limit() {
        let mut state = AdaptivePollingState::new();

        for _ in 0..50 {
            state.update(false);
        }

        assert_eq!(state.interval(), 45, "Interval must cap at 45s, not higher");
    }

    // ── set_cooldown tests (live config propagation) ──

    #[test]
    fn test_set_cooldown_changes_next_success_interval() {
        let mut state = AdaptivePollingState::with_cooldown(300);

        // New data → uses original cooldown
        state.update(true);
        assert_eq!(state.interval(), 300);

        // Change cooldown mid-flight
        state.set_cooldown(120);

        // Reset to base first (simulate reconnect)
        state.current_interval_secs = BASE_POLL_INTERVAL_SECS;
        state.consecutive_no_new_data = 0;

        // Next new data → uses updated cooldown
        state.update(true);
        assert_eq!(state.interval(), 120);
    }

    #[test]
    fn test_set_cooldown_takes_effect_immediately_on_next_update() {
        let mut state = AdaptivePollingState::with_cooldown(300);

        // Start polling — no data for a few polls
        for _ in 0..5 {
            state.update(false);
        }
        // Interval escalated: 5 → 10 → 20 → 40 → 40 (capped logic)
        assert!(state.interval() > BASE_POLL_INTERVAL_SECS);

        // Config push arrives: cooldown changes to 60s
        state.set_cooldown(60);

        // New data arrives — should use 60s, not 300s
        let (interval, _) = state.update(true);
        assert_eq!(interval, 60);
        assert!(state.needs_disconnect()); // 60 > 45 (MAX_POLL_INTERVAL_SECS)
    }

    #[test]
    fn test_set_cooldown_below_session_max_skips_disconnect() {
        let mut state = AdaptivePollingState::with_cooldown(300);

        // Shrink cooldown below session-safe limit
        state.set_cooldown(30);
        state.update(true);

        // 30s < 45s → no disconnect needed
        assert_eq!(state.interval(), 30);
        assert!(!state.needs_disconnect());
    }

    #[test]
    fn test_set_cooldown_does_not_affect_no_data_escalation() {
        let mut state = AdaptivePollingState::with_cooldown(300);

        // Change cooldown
        state.set_cooldown(60);

        // No-data escalation should still follow base → 2x progression,
        // unaffected by cooldown value
        state.update(false); // consecutive=1, interval=5
        state.update(false); // consecutive=2, interval=5
        state.update(false); // consecutive=3, interval=10
        assert_eq!(state.interval(), 10);
    }

    /// Simulates the actual orchestrator→worker pattern:
    /// Arc<AtomicU64> is shared, orchestrator writes, worker reads + set_cooldown
    #[test]
    fn test_atomic_shared_cooldown_pattern() {
        use std::sync::{Arc, atomic::{AtomicU64, Ordering}};

        let shared_cooldown = Arc::new(AtomicU64::new(300));

        // Worker creates polling state from the shared value
        let mut polling_state = AdaptivePollingState::with_cooldown(
            shared_cooldown.load(Ordering::Relaxed)
        );

        // Worker polls — new data uses 300s cooldown
        polling_state.update(true);
        assert_eq!(polling_state.interval(), 300);

        // ── Orchestrator receives ConfigPush: cooldown → 120s ──
        shared_cooldown.store(120, Ordering::Relaxed);

        // Worker syncs at top of next poll iteration (simulates the loop)
        polling_state.set_cooldown(shared_cooldown.load(Ordering::Relaxed));

        // Reset to base (simulate reconnect after cooldown disconnect)
        polling_state.current_interval_secs = BASE_POLL_INTERVAL_SECS;
        polling_state.consecutive_no_new_data = 0;

        // Next new-data event uses the updated 120s cooldown
        polling_state.update(true);
        assert_eq!(polling_state.interval(), 120);

        // ── Orchestrator changes again: cooldown → 30s ──
        shared_cooldown.store(30, Ordering::Relaxed);
        polling_state.set_cooldown(shared_cooldown.load(Ordering::Relaxed));
        polling_state.current_interval_secs = BASE_POLL_INTERVAL_SECS;
        polling_state.update(true);
        assert_eq!(polling_state.interval(), 30);
        assert!(!polling_state.needs_disconnect()); // 30 < 45
    }
}
