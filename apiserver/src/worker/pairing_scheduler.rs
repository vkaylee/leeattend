//! Pairing Scheduler — deferred batch reprocessing for attendance logs.
//!
//! ## Problem
//! Every incoming attendance log triggers `process_new_raw_log` which calls
//! `reprocess_daily_attendance` → `replace_time_entries` with a
//! `pg_advisory_xact_lock(employment_id, date)`. For 400k+ logs concentrated on
//! a few employees, this lock serialises ALL workers, capping throughput at
//! ~10 logs/second regardless of `DB_WORKER_MAX_CONN` or `DB_TENANT_MAX_CONN`.
//!
//! ## Solution
//! 1. `process_log` does only the INSERT (fast, no lock contention).
//! 2. After INSERT it sends a lightweight `PairingNotification` via an mpsc
//!    channel.
//! 3. `PairingScheduler` drains the channel every 2 s, deduplicates by
//!    (employment, local_date), and calls `reprocess_daily_attendance` **once**
//!    per group instead of once per log.
//!
//! The advisory lock still protects against concurrent same-(emp, date)
//! mutations, but the contention drops from thousands of serial calls to
//! exactly 1 call per sweep interval.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, NaiveDate, Timelike, Utc};
use chrono_tz::Tz;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::diesel_pool::TenantAuthToken;
use crate::errors::AppError;
use crate::repositories::employee::EmployeeRepository;
use crate::services::pairing::PairingProcessor;

// ---------------------------------------------------------------------------
// PairingNotification — lightweight fire-and-forget signal
// ---------------------------------------------------------------------------

/// Sent by `process_log` after a successful INSERT. Carries just enough to let
/// the scheduler determine which (employment, local_date) needs reprocessing.
#[derive(Debug, Clone)]
pub struct PairingNotification {
    pub workspace_id: Uuid,
    pub employment_id: Uuid,
    pub punch_time: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// PairingNotifier — sender half (held inside AttendanceLogService)
// ---------------------------------------------------------------------------

/// Cloneable sender for the pairing notification channel.
///
/// Kept inside [`AttendanceLogService`] so `process_log` / `process_parsed_log`
/// can fire a notification after each successful INSERT without blocking.
#[derive(Clone)]
pub struct PairingNotifier {
    tx: mpsc::UnboundedSender<PairingNotification>,
}

impl PairingNotifier {
    /// Fire-and-forget notification. If the channel is closed (scheduler
    /// stopped), the notification is silently dropped — the next sweep will
    /// catch up via the logs already in the database.
    pub fn notify(&self, workspace_id: Uuid, employment_id: Uuid, punch_time: DateTime<Utc>) {
        let _ = self.tx.send(PairingNotification {
            workspace_id,
            employment_id,
            punch_time,
        });
    }
}

/// Create a paired (notifier, receiver) connected by an unbounded mpsc channel.
pub fn pairing_channel() -> (
    PairingNotifier,
    mpsc::UnboundedReceiver<PairingNotification>,
) {
    let (tx, rx) = mpsc::unbounded_channel();
    (PairingNotifier { tx }, rx)
}

// ---------------------------------------------------------------------------
// PairingScheduler — consumer half (background task)
// ---------------------------------------------------------------------------

/// Same cutoff value as [`crate::services::pairing::types::OVERNIGHT_REPROCESS_CUTOFF_HOUR`].
/// Duplicated here to avoid making the pairing internals public.
const OVERNIGHT_CUTOFF_HOUR: u32 = 12;

/// Sweep interval when no `PAIRING_SWEEP_INTERVAL_SECS` env var is set.
const DEFAULT_SWEEP_INTERVAL_SECS: u64 = 2;

pub struct PairingScheduler {
    rx: mpsc::UnboundedReceiver<PairingNotification>,
    sweep_interval: Duration,
    pairing_service: Arc<dyn PairingProcessor>,
    employee_repo: Arc<dyn EmployeeRepository>,
}

impl PairingScheduler {
    pub fn new(
        rx: mpsc::UnboundedReceiver<PairingNotification>,
        pairing_service: Arc<dyn PairingProcessor>,
        employee_repo: Arc<dyn EmployeeRepository>,
        registry: &crate::utils::env_registry::EnvRegistry,
    ) -> Self {
        let interval_secs = registry
            .get_parsed::<u64>("PAIRING_SWEEP_INTERVAL_SECS")
            .unwrap_or(DEFAULT_SWEEP_INTERVAL_SECS);

        Self {
            rx,
            sweep_interval: Duration::from_secs(interval_secs),
            pairing_service,
            employee_repo,
        }
    }

    /// Start the scheduler as a background tokio task.
    pub fn start(mut self, cancel_token: CancellationToken) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            info!(
                interval_secs = self.sweep_interval.as_secs(),
                "Pairing scheduler started"
            );

            let mut tick = tokio::time::interval(self.sweep_interval);
            // Skip the first immediate tick so the channel has time to
            // accumulate notifications after startup.
            tick.tick().await;

            loop {
                tokio::select! {
                    () = cancel_token.cancelled() => break,
                    _ = tick.tick() => {
                        if let Err(e) = self.sweep().await {
                            error!(error = ?e, "Pairing sweep failed");
                        }
                    }
                }
            }

            info!("Pairing scheduler stopped");
        })
    }

    /// Drain all pending notifications, deduplicate by (employment, local_date),
    /// and call `reprocess_daily_attendance` once per group.
    async fn sweep(&mut self) -> Result<(), AppError> {
        // 1. Drain the channel
        let mut notifications: Vec<PairingNotification> = Vec::new();
        while let Ok(n) = self.rx.try_recv() {
            notifications.push(n);
        }

        if notifications.is_empty() {
            return Ok(());
        }

        let total = notifications.len();

        // 2. Group by (workspace_id, employment_id)
        let mut by_employment: HashMap<(Uuid, Uuid), Vec<DateTime<Utc>>> = HashMap::new();
        for n in &notifications {
            by_employment
                .entry((n.workspace_id, n.employment_id))
                .or_default()
                .push(n.punch_time);
        }

        let employment_count = by_employment.len();
        let mut date_groups = 0u64;
        let mut skipped = 0u64;

        // 3. For each employment, resolve timezone and determine local dates
        for ((workspace_id, employment_id), punch_times) in &by_employment {
            let auth_token = TenantAuthToken::system_override(*workspace_id);

            // Resolve fallback timezone for this employment (1 query per employment)
            let tz: Tz = match self
                .employee_repo
                .get_employee_timezone_fallback(&auth_token, *employment_id)
                .await
            {
                Ok(tz) => tz,
                Err(e) => {
                    warn!(
                        workspace_id = %workspace_id,
                        employment_id = %employment_id,
                        error = %e,
                        "Failed to resolve timezone, skipping pairing sweep"
                    );
                    skipped += punch_times.len() as u64;
                    continue;
                }
            };

            // Group punch_times by local_date
            let mut dates: HashSet<NaiveDate> = HashSet::new();
            let mut prev_dates: HashSet<NaiveDate> = HashSet::new();

            for pt in punch_times {
                let local_dt = pt.with_timezone(&tz);
                let local_date = local_dt.date_naive();
                dates.insert(local_date);

                // Overnight cutoff: if any punch is early morning, also
                // reprocess the previous calendar day.
                if local_dt.hour() < OVERNIGHT_CUTOFF_HOUR {
                    prev_dates.insert(local_date - chrono::Duration::days(1));
                }
            }

            // Merge overnight-triggered previous dates
            dates.extend(prev_dates);

            // 4. Reprocess each (employment, date) group exactly once
            for date in &dates {
                date_groups += 1;
                if let Err(e) = self
                    .pairing_service
                    .reprocess_daily_attendance(*workspace_id, *employment_id, *date, tz)
                    .await
                {
                    warn!(
                        workspace_id = %workspace_id,
                        employment_id = %employment_id,
                        date = %date,
                        error = %e,
                        "Pairing reprocess failed for group"
                    );
                }
            }
        }

        if total > 0 {
            info!(
                notifications = total,
                employments = employment_count,
                date_groups = date_groups,
                skipped = skipped,
                "Pairing sweep complete"
            );
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    use chrono::TimeZone;
    use chrono_tz::Asia;

    // -----------------------------------------------------------------------
    // Test doubles
    // -----------------------------------------------------------------------

    /// Minimal mock for [`PairingProcessor`] that records every call so we can
    /// assert deduplication and correct parameters.
    struct SpyPairingProcessor {
        calls: Mutex<Vec<(Uuid, Uuid, NaiveDate)>>,
    }

    impl SpyPairingProcessor {
        fn new() -> Self {
            Self {
                calls: Mutex::new(Vec::new()),
            }
        }

        fn take_calls(&self) -> Vec<(Uuid, Uuid, NaiveDate)> {
            std::mem::take(&mut *self.calls.lock().unwrap())
        }
    }

    #[async_trait::async_trait]
    impl PairingProcessor for SpyPairingProcessor {
        async fn process_new_raw_log(
            &self,
            _log: &crate::models::RawAttendanceLog,
        ) -> Result<(), AppError> {
            Ok(())
        }

        async fn reprocess_daily_attendance(
            &self,
            workspace_id: Uuid,
            employment_id: Uuid,
            date: NaiveDate,
            _tz: Tz,
        ) -> Result<(), AppError> {
            self.calls
                .lock()
                .unwrap()
                .push((workspace_id, employment_id, date));
            Ok(())
        }

        async fn reprocess_attendance_range(
            &self,
            _workspace_id: Uuid,
            _employment_id: Uuid,
            _start_date: NaiveDate,
            _end_date: NaiveDate,
            _tz: Tz,
        ) -> Result<(), AppError> {
            Ok(())
        }
    }

    // -----------------------------------------------------------------------
    // Helper: build a scheduler with the spy + a mocked employee repo
    // -----------------------------------------------------------------------

    fn make_scheduler(
        spy: Arc<SpyPairingProcessor>,
        tz: Tz,
    ) -> (PairingScheduler, PairingNotifier) {
        let mut emp_repo = crate::repositories::employee::MockEmployeeRepository::new();
        emp_repo
            .expect_get_employee_timezone_fallback()
            .returning(move |_, _| Ok(tz));

        let (notifier, rx) = pairing_channel();
        let scheduler = PairingScheduler {
            rx,
            sweep_interval: Duration::from_millis(50),
            pairing_service: spy,
            employee_repo: Arc::new(emp_repo),
        };
        (scheduler, notifier)
    }

    // -----------------------------------------------------------------------
    // Tests
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_empty_channel_no_reprocess() {
        let spy = Arc::new(SpyPairingProcessor::new());
        let (mut scheduler, _notifier) = make_scheduler(spy.clone(), Asia::Ho_Chi_Minh);

        scheduler.sweep().await.unwrap();

        assert!(
            spy.take_calls().is_empty(),
            "no calls when channel is empty"
        );
    }

    #[tokio::test]
    async fn test_single_notification_triggers_reprocess() {
        let spy = Arc::new(SpyPairingProcessor::new());
        let (mut scheduler, notifier) = make_scheduler(spy.clone(), Asia::Ho_Chi_Minh);

        let ws = Uuid::new_v4();
        let emp = Uuid::new_v4();
        // Use a fixed UTC time that is always afternoon in Asia/Ho_Chi_Minh (UTC+7):
        // 07:00 UTC = 14:00 local  →  always >= OVERNIGHT_CUTOFF_HOUR (12).
        let punch = Utc.with_ymd_and_hms(2025, 6, 15, 7, 0, 0).unwrap();

        notifier.notify(ws, emp, punch);
        scheduler.sweep().await.unwrap();

        let calls = spy.take_calls();
        assert_eq!(calls.len(), 1, "one reprocess call expected");
        assert_eq!(calls[0].0, ws);
        assert_eq!(calls[0].1, emp);
    }

    #[tokio::test]
    async fn test_multiple_notifications_same_employee_deduplicated() {
        let spy = Arc::new(SpyPairingProcessor::new());
        let (mut scheduler, notifier) = make_scheduler(spy.clone(), Asia::Ho_Chi_Minh);

        let ws = Uuid::new_v4();
        let emp = Uuid::new_v4();

        // 10 logs, all at 14:00+ local (afternoon → no overnight trigger)
        let base = chrono::NaiveDate::from_ymd_opt(2026, 7, 23)
            .unwrap()
            .and_hms_opt(14, 0, 0)
            .unwrap();
        let base_utc = Asia::Ho_Chi_Minh
            .from_local_datetime(&base)
            .unwrap()
            .with_timezone(&Utc);

        for i in 0..10 {
            notifier.notify(ws, emp, base_utc + chrono::Duration::minutes(i as i64));
        }

        scheduler.sweep().await.unwrap();

        let calls = spy.take_calls();
        assert_eq!(
            calls.len(),
            1,
            "10 logs for same (emp, date) should trigger exactly 1 reprocess"
        );
    }

    #[tokio::test]
    async fn test_different_employees_separate_calls() {
        let spy = Arc::new(SpyPairingProcessor::new());
        let (mut scheduler, notifier) = make_scheduler(spy.clone(), Asia::Ho_Chi_Minh);

        let ws = Uuid::new_v4();
        let emp_a = Uuid::new_v4();
        let emp_b = Uuid::new_v4();
        // 14:00 local → afternoon, no overnight
        let punch = chrono::NaiveDate::from_ymd_opt(2026, 7, 23)
            .unwrap()
            .and_hms_opt(14, 0, 0)
            .unwrap();
        let punch_utc = Asia::Ho_Chi_Minh
            .from_local_datetime(&punch)
            .unwrap()
            .with_timezone(&Utc);

        notifier.notify(ws, emp_a, punch_utc);
        notifier.notify(ws, emp_b, punch_utc);

        scheduler.sweep().await.unwrap();

        let calls = spy.take_calls();
        assert_eq!(
            calls.len(),
            2,
            "two different employees → two reprocess calls"
        );
    }

    #[tokio::test]
    async fn test_overnight_punch_triggers_previous_date() {
        let spy = Arc::new(SpyPairingProcessor::new());
        let (mut scheduler, notifier) = make_scheduler(spy.clone(), Asia::Ho_Chi_Minh);

        let ws = Uuid::new_v4();
        let emp = Uuid::new_v4();

        // Punch at 03:00 local time (Ho Chi Minh = UTC+7 → 20:00 UTC previous day)
        // 03:00 +07 = 20:00 UTC on the day before
        let today_local = chrono::Local::now().date_naive();
        let early_morning_local = today_local.and_hms_opt(3, 0, 0).unwrap();
        let early_morning_utc = Asia::Ho_Chi_Minh
            .from_local_datetime(&early_morning_local)
            .unwrap()
            .with_timezone(&Utc);

        notifier.notify(ws, emp, early_morning_utc);
        scheduler.sweep().await.unwrap();

        let calls = spy.take_calls();
        assert_eq!(
            calls.len(),
            2,
            "early-morning punch should trigger both current and previous date"
        );
    }

    #[tokio::test]
    async fn test_different_local_dates_from_same_employee() {
        let spy = Arc::new(SpyPairingProcessor::new());
        let (mut scheduler, notifier) = make_scheduler(spy.clone(), Asia::Ho_Chi_Minh);

        let ws = Uuid::new_v4();
        let emp = Uuid::new_v4();

        // Use afternoon times (≥12) to avoid overnight cutoff triggering
        // extra previous-date reprocess calls.

        // 2026-07-23 14:00 +07 = 2026-07-23 07:00 UTC
        let day1_local = NaiveDate::from_ymd_opt(2026, 7, 23)
            .unwrap()
            .and_hms_opt(14, 0, 0)
            .unwrap();
        let day1_utc = Asia::Ho_Chi_Minh
            .from_local_datetime(&day1_local)
            .unwrap()
            .with_timezone(&Utc);

        // 2026-07-24 14:00 +07 = 2026-07-24 07:00 UTC
        let day2_local = NaiveDate::from_ymd_opt(2026, 7, 24)
            .unwrap()
            .and_hms_opt(14, 0, 0)
            .unwrap();
        let day2_utc = Asia::Ho_Chi_Minh
            .from_local_datetime(&day2_local)
            .unwrap()
            .with_timezone(&Utc);

        notifier.notify(ws, emp, day1_utc);
        notifier.notify(ws, emp, day2_utc);

        scheduler.sweep().await.unwrap();

        let calls = spy.take_calls();
        assert_eq!(
            calls.len(),
            2,
            "two different local dates → two separate reprocess calls"
        );
    }
}
