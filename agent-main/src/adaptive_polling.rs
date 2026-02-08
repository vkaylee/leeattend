//! Adaptive polling interval calculator
//! Reduces polling frequency when no new data is received to save resources

/// Polling interval configuration
pub const BASE_POLL_INTERVAL_SECS: u64 = 5;
pub const MAX_POLL_INTERVAL_SECS: u64 = 60;
const ESCALATION_THRESHOLD: u32 = 3;

/// State for adaptive polling
#[derive(Debug, Clone)]
pub struct AdaptivePollingState {
    pub current_interval_secs: u64,
    pub consecutive_no_new_data: u32,
}

impl Default for AdaptivePollingState {
    fn default() -> Self {
        Self {
            current_interval_secs: BASE_POLL_INTERVAL_SECS,
            consecutive_no_new_data: 0,
        }
    }
}

impl AdaptivePollingState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update state based on whether new data was found
    /// Returns the new interval and whether it changed
    pub fn update(&mut self, found_new_data: bool) -> (u64, bool) {
        let old_interval = self.current_interval_secs;

        if found_new_data {
            // New data found - reset to base interval
            self.current_interval_secs = BASE_POLL_INTERVAL_SECS;
            self.consecutive_no_new_data = 0;
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
        // Poll 6: escalate 40->60 (consecutive=6, >= threshold, hit max)
        // Poll 7+: stay at 60 (max capped)
        let expected_intervals = [5, 5, 10, 20, 40, 60, 60, 60];

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
}
