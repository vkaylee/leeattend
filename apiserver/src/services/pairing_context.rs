use crate::errors::AppError;
use crate::models::{RawAttendanceLog, Shift, TimeEntry};
use crate::services::pairing_algo::{LogEvent, PairedSession, PairingShiftConfig};
use chrono::{DateTime, Duration, NaiveDate, Timelike, Utc};
use chrono_tz::Tz;
use uuid::Uuid;

pub struct PairingContext {
    date: NaiveDate,
    tz: Tz,
}
impl PairingContext {
    pub fn new(date: NaiveDate, tz: Tz) -> Self {
        Self { date, tz }
    }

    pub fn date(&self) -> NaiveDate {
        self.date
    }

    pub fn tz(&self) -> Tz {
        self.tz
    }

    #[allow(clippy::cast_possible_wrap)]
    pub fn build_shift_config(&self, shift: Option<&Shift>) -> Option<PairingShiftConfig> {
        let s = shift?;
        let start_m = s.start_time.hour() * 60 + s.start_time.minute();
        let mut end_m = s.end_time.hour() * 60 + s.end_time.minute();

        // Handle Overnight: if end < start, it's next day
        if s.end_time < s.start_time {
            end_m += 1440;
        }

        let mut break_start_mins = s
            .break_start
            .map(|t: chrono::NaiveTime| t.hour() * 60 + t.minute());
        let mut break_end_mins = s
            .break_end
            .map(|t: chrono::NaiveTime| t.hour() * 60 + t.minute());

        // Handle break overnight transition if needed
        if let (Some(bs), Some(be)) = (break_start_mins, break_end_mins) {
            let mut break_start_val = bs;
            let mut break_end_val = be;

            if break_start_val < start_m {
                break_start_val += 1440;
            }
            if break_end_val < break_start_val {
                break_end_val += 1440;
            }

            break_start_mins = Some(break_start_val);
            break_end_mins = Some(break_end_val);
        }

        Some(PairingShiftConfig::new(
            start_m as i32,
            end_m as i32,
            break_start_mins.map(|m| m as i32),
            break_end_mins.map(|m| m as i32),
            if s.early_punch_in_grace_period_minutes > 0 {
                Some(s.early_punch_in_grace_period_minutes)
            } else {
                None
            },
            None,
        ))
    }

    pub fn calculate_fetch_window(
        &self,
        shift_config: Option<&PairingShiftConfig>,
    ) -> Result<(DateTime<Utc>, DateTime<Utc>), AppError> {
        let start_of_day_local = crate::utils::tz::local_to_tz(&self.tz, self.date, 0, 0, 0)?;

        let (window_start_utc, window_end_utc) = if let Some(s) = shift_config {
            const LOG_SEARCH_BUFFER_MINUTES: i64 = 360;
            let start_mins = i64::from(s.start_minutes()) - LOG_SEARCH_BUFFER_MINUTES;
            let end_mins =
                std::cmp::max(1440, i64::from(s.end_minutes()) + LOG_SEARCH_BUFFER_MINUTES);

            let s_utc = (start_of_day_local + Duration::minutes(start_mins)).with_timezone(&Utc);
            let e_utc = (start_of_day_local + Duration::minutes(end_mins)).with_timezone(&Utc);
            (s_utc, e_utc)
        } else {
            let s_utc = start_of_day_local.with_timezone(&Utc);
            let e_utc = s_utc + Duration::hours(24);
            (s_utc, e_utc)
        };

        Ok((window_start_utc, window_end_utc))
    }

    pub fn build_log_events(&self, logs: &[RawAttendanceLog]) -> Result<Vec<LogEvent>, AppError> {
        let mut filtered_log_events = Vec::new();
        if logs.is_empty() {
            return Ok(filtered_log_events);
        }

        let start_of_day_local = crate::utils::tz::local_to_tz(&self.tz, self.date, 0, 0, 0)?;

        for log in logs {
            let log_local = log.punch_time().with_timezone(&self.tz);
            let duration_since_start = log_local.signed_duration_since(start_of_day_local);
            let minutes = i32::try_from(duration_since_start.num_minutes()).map_err(|_err| {
                AppError::Internal("Log minute offset overflow during pairing".to_string())
            })?;

            if let Some(existing) = filtered_log_events
                .iter_mut()
                .find(|e| e.minutes() == minutes)
            {
                existing.push_id(log.id());
            } else {
                filtered_log_events.push(LogEvent::new(
                    log.id(),
                    minutes,
                    log.raw_data().cloned(),
                    vec![log.id()],
                ));
            }
        }

        Ok(filtered_log_events)
    }

    pub fn build_time_entries(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        shift_id_opt: Option<Uuid>,
        sessions: Vec<PairedSession>,
        traces: &serde_json::Value,
    ) -> Result<Vec<TimeEntry>, AppError> {
        let mut new_entries = Vec::new();
        for session in sessions {
            let start_dt_local = crate::utils::tz::local_to_tz(&self.tz, self.date, 0, 0, 0)?
                + Duration::minutes(i64::from(session.start_minutes()));
            let start_dt_utc = start_dt_local.with_timezone(&Utc);

            if start_dt_local.date_naive() != self.date {
                continue;
            }

            let end_dt_utc = if let Some(m) = session.end_minutes() {
                let end_dt_local = crate::utils::tz::local_to_tz(&self.tz, self.date, 0, 0, 0)?
                    + Duration::minutes(i64::from(m));
                Some(end_dt_local.with_timezone(&Utc))
            } else {
                None
            };

            let duration = end_dt_utc
                .map(|end| i32::try_from((end - start_dt_utc).num_minutes()).unwrap_or(i32::MAX));

            let mut final_metadata = session
                .metadata()
                .cloned()
                .unwrap_or_else(|| serde_json::json!({}));
            if let serde_json::Value::Object(ref mut map) = final_metadata {
                map.insert("schedule_traces".to_string(), traces.clone());
            }

            new_entries.push(TimeEntry::new(
                crate::utils::id::new_id(),
                employment_id,
                workspace_id,
                shift_id_opt,
                start_dt_utc,
                end_dt_utc,
                duration,
                session.is_break_overtime(),
                session.entry_type().to_string(),
                session.note().map(String::from),
                Some(final_metadata),
                Utc::now(),
                Utc::now(),
            ));
        }
        Ok(new_entries)
    }
}
// injected tests

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::enums::LogSource;
    use crate::services::pairing_algo::EntryType;
    use chrono::{NaiveTime, TimeZone};

    #[test]
    fn test_utc_timezone_offset_bug() {
        // Replicate the exact bug for Employee 1877
        let date = NaiveDate::from_ymd_opt(2026, 5, 20).unwrap();

        // Bug scenario: Tz is UTC
        let tz_utc = chrono_tz::UTC;
        let context_utc = PairingContext::new(date, tz_utc);

        // Shift is 07:30 to 16:30
        let shift = Shift {
            id: Uuid::new_v4(),
            workspace_id: Uuid::new_v4(),
            name: "Hành chính".to_string(),
            start_time: NaiveTime::from_hms_opt(7, 30, 0).unwrap(),
            end_time: NaiveTime::from_hms_opt(16, 30, 0).unwrap(),
            break_start: None,
            break_end: None,
            report_symbol: None,
            break_duration_minutes: 60,
            is_break_auto_deduct: false,
            break_is_overtime: false,
            early_punch_in_grace_period_minutes: 0,
            allow_early_ot: false,
            is_active: true,
            parent_shift_id: None,
            location_id: None,
            employer_id: None,
            max_early_ot_minutes: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let shift_config_utc = context_utc.build_shift_config(Some(&shift));
        let (window_start_utc, _window_end_utc) = context_utc
            .calculate_fetch_window(shift_config_utc.as_ref())
            .unwrap();

        // window_start_utc should be 2026-05-20 01:30:00 UTC
        let expected_window_start = Utc.with_ymd_and_hms(2026, 5, 20, 1, 30, 0).unwrap();
        assert_eq!(window_start_utc, expected_window_start);

        // Employee logs 07:08 local time (00:08 UTC)
        let log1_utc = Utc.with_ymd_and_hms(2026, 5, 20, 0, 8, 27).unwrap();
        // Employee logs 21:31 local time (14:31 UTC)
        let log2_utc = Utc.with_ymd_and_hms(2026, 5, 20, 14, 31, 36).unwrap();

        let raw_logs = vec![
            RawAttendanceLog::new(
                Uuid::new_v4(),
                Uuid::new_v4(),
                Uuid::new_v4(),
                log1_utc,
                LogSource::Manual,
                None,
                None,
                1,
                1,
                Utc::now(),
            ),
            RawAttendanceLog::new(
                Uuid::new_v4(),
                Uuid::new_v4(),
                Uuid::new_v4(),
                log2_utc,
                LogSource::Manual,
                None,
                None,
                1,
                1,
                Utc::now(),
            ),
        ];

        let log_events_utc = context_utc.build_log_events(&raw_logs).unwrap();

        // Under UTC context, log1 is 00:08:27 from start of day, which is 8 minutes
        assert_eq!(log_events_utc[0].minutes(), 8);

        // Under UTC context, log2 is 14:31:36 from start of day, which is 871 minutes
        assert_eq!(log_events_utc[1].minutes(), 871);

        // Now test correct timezone scenario (Asia/Ho_Chi_Minh)
        let tz_hcm = chrono_tz::Asia::Ho_Chi_Minh;
        let context_hcm = PairingContext::new(date, tz_hcm);

        let shift_config_hcm = context_hcm.build_shift_config(Some(&shift));
        let (window_start_hcm, _window_end_hcm) = context_hcm
            .calculate_fetch_window(shift_config_hcm.as_ref())
            .unwrap();

        // window_start_hcm should be 2026-05-19 18:30:00 UTC (01:30 HCM time)
        let expected_window_start_hcm = Utc.with_ymd_and_hms(2026, 5, 19, 18, 30, 0).unwrap();
        assert_eq!(window_start_hcm, expected_window_start_hcm);

        let log_events_hcm = context_hcm.build_log_events(&raw_logs).unwrap();

        // Under HCM context, log1 is 07:08:27 from start of day, which is 428 minutes
        assert_eq!(log_events_hcm[0].minutes(), 428);

        // Under HCM context, log2 is 21:31:36 from start of day, which is 1291 minutes
        assert_eq!(log_events_hcm[1].minutes(), 1291);
    }
    // injected tests

    #[test]
    fn test_build_shift_config_overnight() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let tz = chrono_tz::UTC;
        let context = PairingContext::new(date, tz);

        let shift = Shift {
            id: Uuid::new_v4(),
            workspace_id: Uuid::new_v4(),
            name: "Night".to_string(),
            start_time: NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
            end_time: NaiveTime::from_hms_opt(6, 0, 0).unwrap(), // Next day
            break_start: Some(NaiveTime::from_hms_opt(2, 0, 0).unwrap()), // Next day break
            break_end: Some(NaiveTime::from_hms_opt(3, 0, 0).unwrap()),
            report_symbol: None,
            break_duration_minutes: 60,
            is_break_auto_deduct: false,
            break_is_overtime: false,
            early_punch_in_grace_period_minutes: 0,
            allow_early_ot: false,
            is_active: true,
            parent_shift_id: None,
            location_id: None,
            employer_id: None,
            max_early_ot_minutes: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let config = context.build_shift_config(Some(&shift)).unwrap();
        assert_eq!(config.start_minutes(), 1320); // 22 * 60
        assert_eq!(config.end_minutes(), 360 + 1440); // 6 * 60 + 1440 = 1800
        assert_eq!(config.break_start_minutes(), Some(120 + 1440)); // 2 * 60 + 1440
        assert_eq!(config.break_end_minutes(), Some(180 + 1440)); // 3 * 60 + 1440
    }

    #[test]
    fn test_calculate_fetch_window_no_shift() {
        let date = NaiveDate::from_ymd_opt(2026, 5, 20).unwrap();
        let tz = chrono_tz::Asia::Ho_Chi_Minh;
        let context = PairingContext::new(date, tz);

        let (start_utc, end_utc) = context.calculate_fetch_window(None).unwrap();

        // No shift means 00:00 to 24:00 HCM time
        let expected_start = Utc.with_ymd_and_hms(2026, 5, 19, 17, 0, 0).unwrap();
        let expected_end = Utc.with_ymd_and_hms(2026, 5, 20, 17, 0, 0).unwrap();

        assert_eq!(start_utc, expected_start);
        assert_eq!(end_utc, expected_end);
    }

    #[test]
    fn test_build_log_events_deduplication() {
        let date = NaiveDate::from_ymd_opt(2026, 5, 20).unwrap();
        let tz = chrono_tz::UTC;
        let context = PairingContext::new(date, tz);

        let workspace_id = Uuid::new_v4();
        let employee_id = Uuid::new_v4();
        let log1 = RawAttendanceLog::new(
            Uuid::new_v4(),
            workspace_id,
            employee_id,
            Utc.with_ymd_and_hms(2026, 5, 20, 8, 30, 15).unwrap(),
            LogSource::Manual,
            None,
            None,
            1,
            1,
            Utc::now(),
        );

        let log2 = RawAttendanceLog::new(
            Uuid::new_v4(),
            workspace_id,
            employee_id,
            Utc.with_ymd_and_hms(2026, 5, 20, 8, 30, 45).unwrap(), // Same minute
            LogSource::Manual,
            None,
            None,
            1,
            1,
            log1.created_at(),
        );

        let logs = vec![log1, log2];
        let events = context.build_log_events(&logs).unwrap();

        // Should be deduplicated to 1 event because they are in the same minute
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].minutes(), 8 * 60 + 30);
    }

    #[test]
    fn test_build_time_entries() {
        let date = NaiveDate::from_ymd_opt(2026, 5, 20).unwrap();
        let tz = chrono_tz::Asia::Ho_Chi_Minh;
        let context = PairingContext::new(date, tz);

        let workspace_id = Uuid::new_v4();
        let employment_id = Uuid::new_v4();

        // Paired Session from 08:00 to 17:00 (480 mins to 1020 mins)
        let session1 = PairedSession::new(
            480,
            Some(1020),
            EntryType::Work,
            None,
            false,
            Some(serde_json::json!({"algo": "shift_window"})),
        );

        // Paired Session missing checkout at 18:00 (1080 mins)
        let session2 = PairedSession::new(
            1080,
            None,
            EntryType::Work,
            Some("Missing Punch Out".to_string()),
            false,
            None,
        );

        let sessions = vec![session1, session2];
        let traces = serde_json::json!(["trace1"]);

        let entries = context
            .build_time_entries(workspace_id, employment_id, None, sessions, &traces)
            .unwrap();

        assert_eq!(entries.len(), 2);

        let e1 = &entries[0];
        assert_eq!(e1.workspace_id(), workspace_id);
        assert_eq!(e1.employment_id(), employment_id);
        // 08:00 HCM is 01:00 UTC
        assert_eq!(
            e1.punch_in_time(),
            Utc.with_ymd_and_hms(2026, 5, 20, 1, 0, 0).unwrap()
        );
        // 17:00 HCM is 10:00 UTC
        assert_eq!(
            e1.punch_out_time(),
            Some(Utc.with_ymd_and_hms(2026, 5, 20, 10, 0, 0).unwrap())
        );
        assert_eq!(e1.duration_minutes(), Some(540));

        let meta1 = e1.generation_metadata().unwrap();
        assert_eq!(
            meta1.get("schedule_traces").unwrap().as_array().unwrap()[0]
                .as_str()
                .unwrap(),
            "trace1"
        );

        let e2 = &entries[1];
        // 18:00 HCM is 11:00 UTC
        assert_eq!(
            e2.punch_in_time(),
            Utc.with_ymd_and_hms(2026, 5, 20, 11, 0, 0).unwrap()
        );
        assert_eq!(e2.punch_out_time(), None);
        assert_eq!(e2.duration_minutes(), None);
        assert_eq!(e2.note(), Some("Missing Punch Out"));
    }
}
