use crate::errors::AppError;
use crate::models::{NewTimeEntryParams, RawAttendanceLog, ResolvedSegment, TimeEntry};
use crate::services::pairing::types::{
    LogEvent, MultiSegmentConfig, PairedSession, PairingContext, PairingShiftConfig,
};
use chrono::{DateTime, Duration, NaiveDate, NaiveTime, Timelike, Utc};
use chrono_tz::Tz;
use uuid::Uuid;

const MINUTES_PER_DAY: u32 = 1440;

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

    pub fn build_shift_config(
        &self,
        start_time: NaiveTime,
        end_time: NaiveTime,
        break_start: Option<NaiveTime>,
        break_end: Option<NaiveTime>,
        grace_minutes: i32,
    ) -> Option<PairingShiftConfig> {
        Some(build_config_from_times(
            start_time,
            end_time,
            break_start,
            break_end,
            grace_minutes,
        ))
    }

    pub fn build_shift_config_from_schedule(
        &self,
        sched: &crate::models::EffectiveSchedule,
    ) -> Option<PairingShiftConfig> {
        let start_time = sched.start_time()?;
        let end_time = sched.end_time()?;
        Some(build_config_from_times(
            start_time,
            end_time,
            sched.break_start(),
            sched.break_end(),
            sched.early_punch_in_grace_period_minutes().unwrap_or(0),
        ))
    }

    /// Build a MultiSegmentConfig from resolved work segments.
    /// Each segment produces a PairingShiftConfig with its own time window and break config.
    pub fn build_multi_segment_config(
        &self,
        segments: &[ResolvedSegment],
    ) -> Option<MultiSegmentConfig> {
        if segments.is_empty() {
            return None;
        }

        let configs: Vec<PairingShiftConfig> = segments
            .iter()
            .map(|seg| {
                let mut cfg = build_config_from_times(
                    seg.scheduled_start,
                    seg.scheduled_end,
                    seg.break_start,
                    seg.break_end,
                    seg.early_punch_in_grace_period_minutes,
                );
                if let Some(ref sid) = seg.interval_id {
                    cfg.segment_id = Some(*sid);
                }
                cfg
            })
            .collect();

        Some(MultiSegmentConfig::new(configs))
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
            let e_utc = crate::utils::tz::start_of_next_day_utc(&self.tz, self.date)
                .unwrap_or_else(|_| s_utc + Duration::hours(24));
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
                .find(|e| (e.punch_time() - log.punch_time()).num_seconds().abs() < 5)
            {
                existing.push_id(log.id());
            } else {
                filtered_log_events.push(LogEvent::new(
                    log.id(),
                    minutes,
                    log.punch_time(),
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
            let start_dt_utc = session.start_time();
            let start_dt_local = start_dt_utc.with_timezone(&self.tz);

            if start_dt_local.date_naive() != self.date {
                continue;
            }

            let end_dt_utc = session.end_time();

            let duration = end_dt_utc.map(|end| {
                let mins = (end - start_dt_utc).num_minutes();
                i32::try_from(mins).unwrap_or_else(|_| {
                    tracing::warn!(
                        employment_id = %employment_id,
                        minutes = mins,
                        "Duration overflow in time entry calculation, clamping to i32::MAX"
                    );
                    i32::MAX
                })
            });

            let mut final_metadata = session
                .metadata()
                .cloned()
                .unwrap_or_else(|| serde_json::json!({}));
            if let serde_json::Value::Object(ref mut map) = final_metadata {
                map.insert("schedule_traces".to_string(), traces.clone());
            }

            new_entries.push(TimeEntry::new(NewTimeEntryParams {
                id: crate::utils::id::new_id(),
                employment_id,
                workspace_id,
                shift_id: shift_id_opt,
                segment_id: None,
                punch_in_time: start_dt_utc,
                punch_out_time: end_dt_utc,
                duration_minutes: duration,
                is_break_overtime: session.is_break_overtime(),
                entry_type: *session.entry_type(),
                note: session.note().map(std::string::ToString::to_string),
                generation_metadata: Some(final_metadata),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }));
        }
        Ok(new_entries)
    }
}

#[allow(clippy::cast_possible_wrap)]
fn build_config_from_times(
    start_time: NaiveTime,
    end_time: NaiveTime,
    break_start: Option<NaiveTime>,
    break_end: Option<NaiveTime>,
    grace_minutes: i32,
) -> PairingShiftConfig {
    let start_m = start_time.hour() * 60 + start_time.minute();
    let mut end_m = end_time.hour() * 60 + end_time.minute();

    if end_time < start_time {
        end_m += MINUTES_PER_DAY;
    }

    let mut break_start_mins = break_start.map(|t| t.hour() * 60 + t.minute());
    let mut break_end_mins = break_end.map(|t| t.hour() * 60 + t.minute());

    if let (Some(bs), Some(be)) = (break_start_mins, break_end_mins) {
        let mut bsv = bs;
        let mut bev = be;
        if bsv < start_m {
            bsv += MINUTES_PER_DAY;
        }
        if bev < bsv {
            bev += MINUTES_PER_DAY;
        }
        break_start_mins = Some(bsv);
        break_end_mins = Some(bev);
    }

    PairingShiftConfig::new(
        start_m as i32,
        end_m as i32,
        break_start_mins.map(|m| m as i32),
        break_end_mins.map(|m| m as i32),
        if grace_minutes > 0 {
            Some(grace_minutes)
        } else {
            None
        },
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fetch_window_no_shift_is_full_day_spring_forward() {
        // No shift config → window should be [midnight, next-midnight).
        // On spring-forward day (23h), end should NOT be start + 24h.
        let tz: chrono_tz::Tz = "America/New_York".parse().unwrap();
        let date = NaiveDate::from_ymd_opt(2026, 3, 8).unwrap();
        let ctx = PairingContext::new(date, tz);

        let (s, e) = ctx.calculate_fetch_window(None).unwrap();

        // Duration should be 23h (spring-forward)
        let hours = e.signed_duration_since(s).num_hours();
        assert_eq!(
            hours, 23,
            "Spring-forward fetch window should be 23h, got {hours}h"
        );
    }

    #[test]
    fn fetch_window_no_shift_is_full_day_fall_back() {
        // On fall-back day (25h), window should span 25h.
        let tz: chrono_tz::Tz = "America/New_York".parse().unwrap();
        let date = NaiveDate::from_ymd_opt(2026, 11, 1).unwrap();
        let ctx = PairingContext::new(date, tz);

        let (s, e) = ctx.calculate_fetch_window(None).unwrap();

        // Duration should be 25h (fall-back)
        let hours = e.signed_duration_since(s).num_hours();
        assert_eq!(
            hours, 25,
            "Fall-back fetch window should be 25h, got {hours}h"
        );
    }

    #[test]
    fn fetch_window_no_shift_normal_day() {
        let tz: chrono_tz::Tz = "America/New_York".parse().unwrap();
        let date = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
        let ctx = PairingContext::new(date, tz);

        let (s, e) = ctx.calculate_fetch_window(None).unwrap();

        // Duration should be 24h
        let hours = e.signed_duration_since(s).num_hours();
        assert_eq!(
            hours, 24,
            "Normal day fetch window should be 24h, got {hours}h"
        );
    }
}
