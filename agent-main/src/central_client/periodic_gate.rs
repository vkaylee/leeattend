use std::time::{Duration, Instant};

/// Time gate for periodic maintenance tasks inside hot loops.
///
/// A task guarded by this gate runs once immediately, then at most once per
/// `interval`. Sync-loop maintenance (DLQ sweeps, stale in-flight resets) is
/// O(table size); without a gate it re-runs on every iteration, which at the
/// backlogged sleep tier means ~20 full table scans per second.
pub(super) struct PeriodicGate {
    interval: Duration,
    last_run: Option<Instant>,
}

impl PeriodicGate {
    /// Create a gate that is due immediately, then every `interval` thereafter.
    pub(super) const fn new(interval: Duration) -> Self {
        Self {
            interval,
            last_run: None,
        }
    }

    /// Returns true if the task is due now, recording this instant as the last run.
    pub(super) fn is_due(&mut self) -> bool {
        self.is_due_at(Instant::now())
    }

    /// Deterministic variant of [`Self::is_due`] used by tests to avoid sleeping.
    pub(super) fn is_due_at(&mut self, now: Instant) -> bool {
        let due = match self.last_run {
            None => true,
            Some(last) => now.duration_since(last) >= self.interval,
        };
        if due {
            self.last_run = Some(now);
        }
        due
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first_call_is_due() {
        let mut gate = PeriodicGate::new(Duration::from_secs(300));
        assert!(gate.is_due(), "gate must run once at startup");
    }

    #[test]
    fn test_second_call_before_interval_is_not_due() {
        let base = Instant::now();
        let mut gate = PeriodicGate::new(Duration::from_secs(300));

        assert!(gate.is_due_at(base));
        // Second sync-loop iteration, 50ms later (backlogged interval tier).
        assert!(
            !gate.is_due_at(base + Duration::from_millis(50)),
            "gate must not re-run on the next loop iteration"
        );
    }

    #[test]
    fn test_not_due_until_interval_elapses() {
        let base = Instant::now();
        let interval = Duration::from_secs(300);
        let mut gate = PeriodicGate::new(interval);

        assert!(gate.is_due_at(base));

        // Simulate 20 iterations/sec for just under the full interval.
        let mut elapsed = Duration::from_millis(0);
        while elapsed < interval - Duration::from_millis(50) {
            elapsed += Duration::from_millis(50);
            assert!(
                !gate.is_due_at(base + elapsed),
                "gate fired early at {elapsed:?}"
            );
        }

        // Exactly at the interval boundary it becomes due again.
        assert!(gate.is_due_at(base + interval));
    }

    #[test]
    fn test_interval_measured_from_last_run_not_start() {
        let base = Instant::now();
        let interval = Duration::from_secs(60);
        let mut gate = PeriodicGate::new(interval);

        assert!(gate.is_due_at(base));
        assert!(gate.is_due_at(base + interval));
        // Only 1s after the previous run — not due, even though 61s since start.
        assert!(!gate.is_due_at(base + interval + Duration::from_secs(1)));
        assert!(gate.is_due_at(base + interval + interval));
    }

    #[test]
    fn test_zero_interval_always_due() {
        let base = Instant::now();
        let mut gate = PeriodicGate::new(Duration::from_secs(0));
        assert!(gate.is_due_at(base));
        assert!(gate.is_due_at(base));
    }
}
