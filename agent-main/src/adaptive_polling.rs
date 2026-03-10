//! Adaptive polling interval calculator
//! Reduces polling frequency when no new data is received to save resources

/// Polling interval configuration
pub const BASE_POLL_INTERVAL_SECS: u64 = 5;
pub const MAX_POLL_INTERVAL_SECS: u64 = 45;
const ESCALATION_THRESHOLD: u32 = 3;

/// Default number of poll cycles to skip after a successful fetch with new data.
/// This acts as a TTL cooldown — after fetching new logs, the agent skips
/// this many subsequent polls before fetching again.
/// Configurable via `SUCCESS_SKIP_POLLS` env var.
pub const DEFAULT_SUCCESS_SKIP_POLLS: u32 = 60;

/// State for adaptive polling
#[derive(Debug, Clone)]
pub struct AdaptivePollingState {
    pub current_interval_secs: u64,
    pub consecutive_no_new_data: u32,
    /// TTL skip countdown: when > 0, the poll cycle should be skipped
    pub skip_remaining: u32,
    /// Configured number of polls to skip (from env or default)
    skip_polls_config: u32,
}

impl Default for AdaptivePollingState {
    fn default() -> Self {
        Self {
            current_interval_secs: BASE_POLL_INTERVAL_SECS,
            consecutive_no_new_data: 0,
            skip_remaining: 0,
            skip_polls_config: DEFAULT_SUCCESS_SKIP_POLLS,
        }
    }
}

impl AdaptivePollingState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with a custom skip_polls value (from config/env)
    pub fn with_skip_polls(skip_polls: u32) -> Self {
        Self {
            skip_polls_config: skip_polls,
            ..Self::default()
        }
    }

    /// Update state based on whether new data was found
    /// Returns the new interval and whether it changed
    pub fn update(&mut self, found_new_data: bool) -> (u64, bool) {
        let old_interval = self.current_interval_secs;

        if found_new_data {
            // New data found - reset to base interval and set skip cooldown
            self.current_interval_secs = BASE_POLL_INTERVAL_SECS;
            self.consecutive_no_new_data = 0;
            self.skip_remaining = self.skip_polls_config;
        } else {
            // No new data - potentially escalate interval
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

    /// Check if this poll cycle should be skipped (TTL cooldown active).
    /// Returns `true` if skip is active, and decrements the counter.
    pub fn should_skip(&mut self) -> bool {
        if self.skip_remaining > 0 {
            self.skip_remaining -= 1;
            true
        } else {
            false
        }
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
    }

    #[test]
    fn test_new_data_resets_interval() {
        let mut state = AdaptivePollingState::new();

        // Simulate escalation first
        for _ in 0..10 {
            state.update(false);
        }
        assert!(state.interval() > BASE_POLL_INTERVAL_SECS);

        // New data should reset
        let (interval, changed) = state.update(true);
        assert_eq!(interval, BASE_POLL_INTERVAL_SECS);
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
    }

    #[test]
    fn test_reset_after_long_idle_then_new_data() {
        let mut state = AdaptivePollingState::new();

        // Escalate to max
        for _ in 0..50 {
            state.update(false);
        }
        assert_eq!(state.interval(), MAX_POLL_INTERVAL_SECS);

        // Single new data event resets everything
        state.update(true);
        assert_eq!(state.interval(), BASE_POLL_INTERVAL_SECS);
        assert_eq!(state.consecutive_no_new_data, 0);
    }

    #[test]
    fn test_intermittent_new_data_prevents_escalation() {
        let mut state = AdaptivePollingState::new();

        // Alternate: 2 no-data, 1 new-data (never reaches threshold of 3)
        for _ in 0..10 {
            state.update(false);
            state.update(false);
            state.update(true);
        }

        // Should always be at base interval
        assert_eq!(state.interval(), BASE_POLL_INTERVAL_SECS);
    }

    // === TTL Skip Tests ===

    #[test]
    fn test_skip_after_new_data() {
        let mut state = AdaptivePollingState::with_skip_polls(3);

        // No skip initially
        assert!(!state.should_skip());

        // New data sets skip_remaining
        state.update(true);
        assert_eq!(state.skip_remaining, 3);

        // should_skip returns true and decrements
        assert!(state.should_skip());
        assert_eq!(state.skip_remaining, 2);
    }

    #[test]
    fn test_skip_countdown_to_zero() {
        let mut state = AdaptivePollingState::with_skip_polls(3);
        state.update(true);

        // Drain all skip counts
        for _ in 0..3 {
            assert!(state.should_skip());
        }

        // Now should_skip returns false
        assert!(!state.should_skip());
        assert_eq!(state.skip_remaining, 0);
    }

    #[test]
    fn test_no_skip_when_no_new_data() {
        let mut state = AdaptivePollingState::new();

        // update(false) should NOT set skip
        state.update(false);
        assert_eq!(state.skip_remaining, 0);
        assert!(!state.should_skip());
    }

    #[test]
    fn test_skip_resets_on_new_data() {
        let mut state = AdaptivePollingState::with_skip_polls(3);
        state.update(true);

        // Consume one skip
        assert!(state.should_skip());

        // Another update(true) resets the counter
        state.update(true);
        assert_eq!(state.skip_remaining, 3);
    }

    #[test]
    fn test_skip_does_not_interfere_with_escalation() {
        let mut state = AdaptivePollingState::with_skip_polls(2);

        // Get new data (sets skip)
        state.update(true);
        assert_eq!(state.skip_remaining, 2);

        // Drain skips
        while state.should_skip() {}

        // Now escalate with no-data
        for _ in 0..ESCALATION_THRESHOLD {
            state.update(false);
        }
        assert!(state.interval() > BASE_POLL_INTERVAL_SECS);
        assert_eq!(state.skip_remaining, 0); // no skip set by no-data
    }

    #[test]
    fn test_default_skip_polls_value() {
        let state = AdaptivePollingState::new();
        assert_eq!(state.skip_polls_config, DEFAULT_SUCCESS_SKIP_POLLS);
        assert_eq!(state.skip_polls_config, 60);
    }

    #[test]
    fn test_custom_skip_polls_value() {
        let state = AdaptivePollingState::with_skip_polls(5);
        assert_eq!(state.skip_polls_config, 5);
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
    /// This is critical for preventing UDP session timeouts on ZK devices.
    #[test]
    fn test_escalation_never_exceeds_session_safe_limit() {
        let mut state = AdaptivePollingState::new();

        // Run many no-data polls
        for _ in 0..50 {
            state.update(false);
        }

        // Must cap at MAX_POLL_INTERVAL_SECS = 45
        assert_eq!(state.interval(), 45, "Interval must cap at 45s, not higher");
    }
}
