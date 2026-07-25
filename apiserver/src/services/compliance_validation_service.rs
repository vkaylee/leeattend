use std::collections::HashMap;
use std::sync::Arc;

use chrono::{Datelike, NaiveDate, NaiveTime, Timelike};
use uuid::Uuid;

use crate::diesel_pool::TenantAuthToken;
use crate::errors::{AppError, AppResult};
use crate::models::compliance::{
    ComplianceCheckRequest, ComplianceCheckResponse, ComplianceViolation,
    ComplianceViolationDetails, ViolationSeverity, ViolationType,
};
use crate::models::ShiftRoster;
use crate::repositories::compliance_repository::ComplianceRepository;
use crate::repositories::roster::RosterRepository;
use crate::utils::i18n::I18nKey;

// ============================================================================
// ComplianceValidationService
// ============================================================================

/// Validates shift assignments against workspace compliance rules.
///
/// Called from handlers as a pre-mutation gate before persisting any roster or
/// pattern assignment.
pub struct ComplianceValidationService {
    compliance_repo: Arc<dyn ComplianceRepository>,
    roster_repo: Arc<dyn RosterRepository>,
}

impl ComplianceValidationService {
    pub fn new(
        compliance_repo: Arc<dyn ComplianceRepository>,
        roster_repo: Arc<dyn RosterRepository>,
    ) -> Self {
        Self {
            compliance_repo,
            roster_repo,
        }
    }

    /// Validate a set of shift assignments against the workspace's compliance rules.
    ///
    /// # Arguments
    /// * `workspace_id` — the workspace to validate within
    /// * `request` — contains employment_id, shift_id, dates, and override flags
    ///
    /// # Returns
    /// * `Ok(ComplianceCheckResponse)` — if compliant or soft violations accepted
    /// * `Err(AppError::Validation)` — if hard violations exist (cannot override)
    /// * `Err(AppError::Conflict)` — if soft violations exist without override_reason
    #[tracing::instrument(skip_all, err)]
    pub async fn validate_assignments(
        &self,
        workspace_id: Uuid,
        auth_token: &TenantAuthToken,
        request: &ComplianceCheckRequest,
    ) -> AppResult<ComplianceCheckResponse> {
        // 1. Load rules
        let rules = self.compliance_repo.get_rules(auth_token).await?;

        // 2. If compliance is disabled, skip all checks
        if !rules.is_enabled {
            return Ok(ComplianceCheckResponse {
                compliant: true,
                violations: vec![],
                requires_override: false,
            });
        }

        // 3. Load existing rosters for this employment across the date range ±1 day
        let min_date = request
            .dates
            .iter()
            .min()
            .copied()
            .ok_or_else(|| AppError::validation(I18nKey::Validation))?;
        let max_date = request
            .dates
            .iter()
            .max()
            .copied()
            .ok_or_else(|| AppError::validation(I18nKey::Validation))?;
        let query_start = min_date - chrono::Duration::days(1);
        let query_end = max_date + chrono::Duration::days(1);

        let existing_map = self
            .roster_repo
            .get_employment_rosters_batch(
                auth_token,
                &[request.employment_id],
                query_start,
                query_end,
            )
            .await?;

        let existing_rosters: Vec<&ShiftRoster> = existing_map
            .get(&request.employment_id)
            .map(|v| v.iter().collect())
            .unwrap_or_default();

        // 4. Load roster data for interval info
        let roster_data = self.roster_repo.get_raw_roster_data(workspace_id).await?;

        // 5. Run the four compliance checks
        let mut violations: Vec<ComplianceViolation> = Vec::new();

        // Check A: Overlap detection (HARD)
        Self::check_overlaps(
            request,
            &existing_rosters,
            &roster_data.shift_intervals,
            &mut violations,
        );

        // Check B: Rest period (SOFT)
        Self::check_rest_period(
            request,
            &existing_rosters,
            &roster_data.shift_intervals,
            rules.min_rest_period_hours,
            &mut violations,
        );

        // Check C: Weekly hours (SOFT)
        Self::check_weekly_hours(
            request,
            &existing_rosters,
            &roster_data.shift_intervals,
            rules.max_weekly_hours,
            &mut violations,
        );

        // Check D: Monthly OT (SOFT)
        Self::check_monthly_ot(
            request,
            &existing_rosters,
            &roster_data.shift_intervals,
            rules.max_monthly_ot_hours,
            &mut violations,
        );

        // 6. Resolve
        let has_hard = violations
            .iter()
            .any(|v| v.severity == ViolationSeverity::Hard);
        let has_soft = violations
            .iter()
            .any(|v| v.severity == ViolationSeverity::Soft);

        if has_hard {
            return Err(AppError::validation(I18nKey::ShiftOverlapDetected));
        }

        if has_soft {
            if request.allow_warnings && request.override_reason.is_some() {
                // Manager overrode with reason — proceed
                return Ok(ComplianceCheckResponse {
                    compliant: true,
                    violations, // include for audit
                    requires_override: false,
                });
            }

            // Soft violations without override → conflict with structured payload
            let violations_json = serde_json::to_value(&violations).unwrap_or_default();
            return Err(AppError::conflict_with_retry(
                I18nKey::ComplianceOverrideRequired,
                Option::<(&str, &str)>::None,
                serde_json::json!({
                    "violations": violations_json,
                    "requires_override": true,
                }),
            ));
        }

        Ok(ComplianceCheckResponse {
            compliant: true,
            violations: vec![],
            requires_override: false,
        })
    }

    // -----------------------------------------------------------------------
    // Check A: Overlap (HARD)
    // -----------------------------------------------------------------------
    fn check_overlaps(
        request: &ComplianceCheckRequest,
        existing: &[&ShiftRoster],
        shift_intervals: &HashMap<Uuid, Vec<crate::cache::RosterInterval>>,
        violations: &mut Vec<ComplianceViolation>,
    ) {
        let new_intervals = shift_intervals.get(&request.shift_id);

        for date in &request.dates {
            // Find existing rosters for this date
            for roster in existing.iter().filter(|r| r.date == *date) {
                // Skip if it's the same shift (idempotent upsert)
                if roster.shift_id == Some(request.shift_id) {
                    continue;
                }

                let Some(existing_shift_id) = roster.shift_id else {
                    continue; // day off — no overlap
                };

                let existing_intervals = shift_intervals.get(&existing_shift_id);

                // If either shift has no intervals, treat any different shift on
                // the same date as overlapping (conservative).
                match (new_intervals, existing_intervals) {
                    (Some(ni), Some(ei)) => {
                        // Check interval-by-interval for overlap
                        let has_overlap = ni.iter().any(|n| {
                            ei.iter().any(|e| {
                                times_overlap(n.start_time, n.end_time, e.start_time, e.end_time)
                            })
                        });
                        if has_overlap {
                            violations.push(ComplianceViolation {
                                violation_type: ViolationType::Overlap,
                                severity: ViolationSeverity::Hard,
                                message: "Shift overlap detected".to_string(),
                                details: ComplianceViolationDetails {
                                    employment_id: request.employment_id,
                                    shift_ids: vec![request.shift_id, existing_shift_id],
                                    conflicting_dates: vec![*date],
                                    total_hours: None,
                                    limit_hours: None,
                                },
                            });
                        }
                    }
                    _ => {
                        // Conservative: any different shift on same day = overlap
                        violations.push(ComplianceViolation {
                            violation_type: ViolationType::Overlap,
                            severity: ViolationSeverity::Hard,
                            message: "Shift overlap detected".to_string(),
                            details: ComplianceViolationDetails {
                                employment_id: request.employment_id,
                                shift_ids: vec![request.shift_id, existing_shift_id],
                                conflicting_dates: vec![*date],
                                total_hours: None,
                                limit_hours: None,
                            },
                        });
                    }
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // Check B: Rest Period (SOFT)
    // -----------------------------------------------------------------------
    #[allow(clippy::expect_used)]
    fn check_rest_period(
        request: &ComplianceCheckRequest,
        existing: &[&ShiftRoster],
        shift_intervals: &HashMap<Uuid, Vec<crate::cache::RosterInterval>>,
        min_rest_hours: i32,
        violations: &mut Vec<ComplianceViolation>,
    ) {
        let new_intervals = shift_intervals.get(&request.shift_id);

        for date in &request.dates {
            let prev_date = *date - chrono::Duration::days(1);
            let next_date = *date + chrono::Duration::days(1);

            // Check gap after previous day's shift
            if let Some(prev_roster) = existing.iter().find(|r| r.date == prev_date) {
                if let Some(prev_shift_id) = prev_roster.shift_id {
                    if let Some(prev_intervals) = shift_intervals.get(&prev_shift_id) {
                        if let Some(new_ints) = new_intervals {
                            // Last end time of prev shift vs first start time of new shift
                            let prev_end = prev_intervals
                                .iter()
                                .map(|i| i.end_time)
                                .max_by_key(|t| seconds_from_midnight_or_wrap(*t))
                                .unwrap_or(
                                    NaiveTime::from_hms_opt(0, 0, 0).expect("0:0:0 is valid"),
                                );

                            let new_start = new_ints
                                .iter()
                                .map(|i| i.start_time)
                                .min_by_key(|t| seconds_from_midnight_or_wrap(*t))
                                .unwrap_or(
                                    NaiveTime::from_hms_opt(23, 59, 59).expect("23:59:59 is valid"),
                                );

                            let gap_hours = rest_gap_hours(prev_end, new_start);
                            if gap_hours < f64::from(min_rest_hours) {
                                violations.push(ComplianceViolation {
                                    violation_type: ViolationType::RestPeriod,
                                    severity: ViolationSeverity::Soft,
                                    message: format!(
                                        "Insufficient rest: {gap_hours:.0}h gap, {min_rest_hours}h required"
                                    ),
                                    details: ComplianceViolationDetails {
                                        employment_id: request.employment_id,
                                        shift_ids: vec![request.shift_id, prev_shift_id],
                                        conflicting_dates: vec![prev_date, *date],
                                        total_hours: Some(gap_hours),
                                        limit_hours: Some(min_rest_hours),
                                    },
                                });
                            }
                        }
                    }
                }
            }

            // Check gap before next day's shift
            if let Some(next_roster) = existing.iter().find(|r| r.date == next_date) {
                if let Some(next_shift_id) = next_roster.shift_id {
                    if let Some(next_intervals) = shift_intervals.get(&next_shift_id) {
                        if let Some(new_ints) = new_intervals {
                            let new_end = new_ints
                                .iter()
                                .map(|i| i.end_time)
                                .max_by_key(|t| seconds_from_midnight_or_wrap(*t))
                                .unwrap_or(
                                    NaiveTime::from_hms_opt(0, 0, 0).expect("0:0:0 is valid"),
                                );

                            let next_start = next_intervals
                                .iter()
                                .map(|i| i.start_time)
                                .min_by_key(|t| seconds_from_midnight_or_wrap(*t))
                                .unwrap_or(
                                    NaiveTime::from_hms_opt(23, 59, 59).expect("23:59:59 is valid"),
                                );

                            let gap_hours = rest_gap_hours(new_end, next_start);
                            if gap_hours < f64::from(min_rest_hours) {
                                violations.push(ComplianceViolation {
                                    violation_type: ViolationType::RestPeriod,
                                    severity: ViolationSeverity::Soft,
                                    message: format!(
                                        "Insufficient rest: {gap_hours:.0}h gap, {min_rest_hours}h required"
                                    ),
                                    details: ComplianceViolationDetails {
                                        employment_id: request.employment_id,
                                        shift_ids: vec![request.shift_id, next_shift_id],
                                        conflicting_dates: vec![*date, next_date],
                                        total_hours: Some(gap_hours),
                                        limit_hours: Some(min_rest_hours),
                                    },
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // Check C: Weekly Hours (SOFT)
    // -----------------------------------------------------------------------
    fn check_weekly_hours(
        request: &ComplianceCheckRequest,
        existing: &[&ShiftRoster],
        shift_intervals: &HashMap<Uuid, Vec<crate::cache::RosterInterval>>,
        max_weekly_hours: i32,
        violations: &mut Vec<ComplianceViolation>,
    ) {
        // Determine the week (Mon-Sun) for each assignment date
        for date in &request.dates {
            let week_start = monday_of_week(*date);
            let week_end = week_start + chrono::Duration::days(6);

            // Count hours from existing rosters in this week
            let mut total_hours: f64 = 0.0;
            for roster in existing
                .iter()
                .filter(|r| r.date >= week_start && r.date <= week_end)
            {
                if let Some(shift_id) = roster.shift_id {
                    if let Some(intervals) = shift_intervals.get(&shift_id) {
                        total_hours += intervals.iter().map(interval_hours).sum::<f64>();
                    }
                }
            }

            // Add hours from the new assignment dates within this week
            for _d in request
                .dates
                .iter()
                .filter(|d| **d >= week_start && **d <= week_end)
            {
                if let Some(intervals) = shift_intervals.get(&request.shift_id) {
                    total_hours += intervals.iter().map(interval_hours).sum::<f64>();
                }
            }

            if total_hours > f64::from(max_weekly_hours) {
                violations.push(ComplianceViolation {
                    violation_type: ViolationType::WeeklyHours,
                    severity: ViolationSeverity::Soft,
                    message: format!(
                        "Weekly hours exceeded: {total_hours:.1}h, limit: {max_weekly_hours}h"
                    ),
                    details: ComplianceViolationDetails {
                        employment_id: request.employment_id,
                        shift_ids: vec![request.shift_id],
                        conflicting_dates: vec![week_start, week_end],
                        total_hours: Some(total_hours),
                        limit_hours: Some(max_weekly_hours),
                    },
                });
                break; // Report once per check
            }
        }
    }

    // -----------------------------------------------------------------------
    // Check D: Monthly OT (SOFT)
    // -----------------------------------------------------------------------
    #[allow(clippy::expect_used)]
    fn check_monthly_ot(
        request: &ComplianceCheckRequest,
        existing: &[&ShiftRoster],
        shift_intervals: &HashMap<Uuid, Vec<crate::cache::RosterInterval>>,
        max_monthly_ot_hours: i32,
        violations: &mut Vec<ComplianceViolation>,
    ) {
        const STANDARD_DAILY_HOURS: f64 = 8.0;

        for date in &request.dates {
            let month_start = NaiveDate::from_ymd_opt(date.year(), date.month(), 1)
                .expect("year/month from a valid NaiveDate is valid");
            let month_end = last_day_of_month(*date);

            // Count OT from existing rosters in this month
            let mut total_ot: f64 = 0.0;
            for roster in existing
                .iter()
                .filter(|r| r.date >= month_start && r.date <= month_end)
            {
                if let Some(shift_id) = roster.shift_id {
                    if let Some(intervals) = shift_intervals.get(&shift_id) {
                        let daily_hours: f64 = intervals.iter().map(interval_hours).sum();
                        if daily_hours > STANDARD_DAILY_HOURS {
                            total_ot += daily_hours - STANDARD_DAILY_HOURS;
                        }
                    }
                }
            }

            // Add OT from new assignment dates in this month
            for _d in request
                .dates
                .iter()
                .filter(|d| **d >= month_start && **d <= month_end)
            {
                if let Some(intervals) = shift_intervals.get(&request.shift_id) {
                    let daily_hours: f64 = intervals.iter().map(interval_hours).sum();
                    if daily_hours > STANDARD_DAILY_HOURS {
                        total_ot += daily_hours - STANDARD_DAILY_HOURS;
                    }
                }
            }

            if total_ot > f64::from(max_monthly_ot_hours) {
                violations.push(ComplianceViolation {
                    violation_type: ViolationType::MonthlyOvertime,
                    severity: ViolationSeverity::Soft,
                    message: format!(
                        "Monthly OT exceeded: {total_ot:.1}h OT, limit: {max_monthly_ot_hours}h"
                    ),
                    details: ComplianceViolationDetails {
                        employment_id: request.employment_id,
                        shift_ids: vec![request.shift_id],
                        conflicting_dates: vec![month_start, month_end],
                        total_hours: Some(total_ot),
                        limit_hours: Some(max_monthly_ot_hours),
                    },
                });
                break; // Report once per check
            }
        }
    }
}

// ============================================================================
// Helper functions
// ============================================================================

/// Check whether two time ranges overlap.
/// Handles overnight shifts (end < start) by treating the range as wrapping past midnight.
fn times_overlap(s1: NaiveTime, e1: NaiveTime, s2: NaiveTime, e2: NaiveTime) -> bool {
    let to_range = |s: NaiveTime, e: NaiveTime| -> (i64, i64) {
        let sm = i64::from(s.num_seconds_from_midnight());
        let em = i64::from(e.num_seconds_from_midnight());
        if em > sm {
            (sm, em)
        } else {
            // Overnight: treat end as next day
            (sm, em + 86_400)
        }
    };
    let (a_start, a_end) = to_range(s1, e1);
    let (b_start, b_end) = to_range(s2, e2);
    // Basic overlap check
    if a_start < b_end && b_start < a_end {
        return true;
    }
    // Also check with b shifted by +24h to catch cross-midnight cases where
    // one shift is overnight and the other is a morning shift the next day.
    a_start < (b_end + 86_400) && (b_start + 86_400) < a_end
}

/// Compute seconds from midnight, wrapping overnight times to next day for comparison.
fn seconds_from_midnight_or_wrap(t: NaiveTime) -> i64 {
    i64::from(t.num_seconds_from_midnight())
}

/// Calculate the rest gap in hours between end of one shift and start of the next.
/// Handles overnight end times.
#[allow(clippy::cast_precision_loss)]
fn rest_gap_hours(prev_end: NaiveTime, next_start: NaiveTime) -> f64 {
    let pe = i64::from(prev_end.num_seconds_from_midnight());
    let ns = i64::from(next_start.num_seconds_from_midnight());
    // If prev_end is after midnight (e.g., 02:00), wrap it
    let pe_adjusted = if pe < 43_200 {
        // prev_end is in the early morning (e.g., 02:00) → treat as next day
        pe + 86_400
    } else {
        pe
    };
    // If next_start is before prev_end (same-day), next_start is actually next day
    let ns_adjusted = if ns <= pe_adjusted && ns < 43_200 {
        ns + 86_400
    } else {
        ns
    };
    let gap_seconds = ns_adjusted - pe_adjusted;
    if gap_seconds < 0 {
        0.0
    } else {
        (gap_seconds as f64) / 3600.0
    }
}

/// Calculate total hours for an interval.
#[allow(clippy::cast_lossless, clippy::cast_precision_loss)]
fn interval_hours(interval: &crate::cache::RosterInterval) -> f64 {
    let start_secs = i64::from(interval.start_time.num_seconds_from_midnight());
    let end_secs = i64::from(interval.end_time.num_seconds_from_midnight());
    let duration = if end_secs > start_secs {
        end_secs - start_secs
    } else {
        // Overnight
        end_secs + 86_400 - start_secs
    };
    // Subtract break if auto-deduct
    let break_secs = if interval.is_break_auto_deduct {
        (interval.break_duration_minutes * 60) as i64
    } else {
        0
    };
    let net_secs = (duration - break_secs).max(0);
    (net_secs as f64) / 3600.0
}

/// Get the Monday of the week containing the given date.
fn monday_of_week(date: NaiveDate) -> NaiveDate {
    let weekday = date.weekday().num_days_from_monday();
    date - chrono::Duration::days(i64::from(weekday))
}

/// Get the last day of the month containing the given date.
#[allow(clippy::expect_used)]
fn last_day_of_month(date: NaiveDate) -> NaiveDate {
    let next_month = if date.month() == 12 {
        NaiveDate::from_ymd_opt(date.year() + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(date.year(), date.month() + 1, 1)
    };
    next_month.expect("year/month from a valid NaiveDate is valid") - chrono::Duration::days(1)
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveTime;

    fn t(h: u32, m: u32) -> NaiveTime {
        NaiveTime::from_hms_opt(h, m, 0).unwrap()
    }

    #[test]
    fn test_times_overlap_non_overlapping() {
        // 08:00-12:00 vs 12:00-17:00: adjacent but not overlapping
        assert!(!times_overlap(t(8, 0), t(12, 0), t(12, 0), t(17, 0)));
        // 08:00-12:00 vs 13:00-17:00: gap between
        assert!(!times_overlap(t(8, 0), t(12, 0), t(13, 0), t(17, 0)));
    }

    #[test]
    fn test_times_overlap_overlapping() {
        // 08:00-13:00 vs 12:00-17:00
        assert!(times_overlap(t(8, 0), t(13, 0), t(12, 0), t(17, 0)));
        // 08:00-17:00 vs 10:00-15:00 (contained)
        assert!(times_overlap(t(8, 0), t(17, 0), t(10, 0), t(15, 0)));
        // Identical
        assert!(times_overlap(t(8, 0), t(17, 0), t(8, 0), t(17, 0)));
    }

    #[test]
    fn test_times_overlap_overnight() {
        // 22:00-06:00 vs 08:00-12:00: no overlap
        assert!(!times_overlap(t(22, 0), t(6, 0), t(8, 0), t(12, 0)));
        // 22:00-06:00 vs 04:00-08:00: overlap
        assert!(times_overlap(t(22, 0), t(6, 0), t(4, 0), t(8, 0)));
        // Two overnight shifts overlapping
        assert!(times_overlap(t(22, 0), t(6, 0), t(23, 0), t(7, 0)));
    }

    #[test]
    fn test_rest_gap_hours_same_day() {
        // End at 17:00, start next at 08:00 → 15h gap
        let gap = rest_gap_hours(t(17, 0), t(8, 0));
        assert!((gap - 15.0).abs() < 0.01, "Expected 15h gap, got {}", gap);
    }

    #[test]
    fn test_rest_gap_hours_overnight() {
        // End at 22:00, start at 06:00 next day → 8h gap
        let gap = rest_gap_hours(t(22, 0), t(6, 0));
        assert!((gap - 8.0).abs() < 0.01, "Expected 8h gap, got {}", gap);
    }

    #[test]
    fn test_rest_gap_hours_short_rest() {
        // End at 22:00, start at 05:00 → 7h gap (insufficient for 12h rule)
        let gap = rest_gap_hours(t(22, 0), t(5, 0));
        assert!((gap - 7.0).abs() < 0.01, "Expected 7h gap, got {}", gap);
    }

    #[test]
    fn test_interval_hours_standard() {
        let interval = crate::cache::RosterInterval {
            interval_id: Uuid::new_v4(),
            sort_order: 0,
            interval_name: None,
            start_time: t(8, 0),
            end_time: t(17, 0),
            break_duration_minutes: 60,
            break_start: None,
            break_end: None,
            is_break_auto_deduct: true,
            break_is_overtime: false,
            early_punch_in_grace_period_minutes: 0,
            allow_early_ot: false,
            max_early_ot_minutes: None,
            ot_aggregation_mode: "aggregate".to_string(),
            is_night_shift: false,
        };
        // 9h - 1h break = 8h
        assert!((interval_hours(&interval) - 8.0).abs() < 0.01);
    }

    #[test]
    fn test_interval_hours_overnight() {
        let interval = crate::cache::RosterInterval {
            interval_id: Uuid::new_v4(),
            sort_order: 0,
            interval_name: None,
            start_time: t(22, 0),
            end_time: t(6, 0),
            break_duration_minutes: 30,
            break_start: None,
            break_end: None,
            is_break_auto_deduct: true,
            break_is_overtime: false,
            early_punch_in_grace_period_minutes: 0,
            allow_early_ot: false,
            max_early_ot_minutes: None,
            ot_aggregation_mode: "aggregate".to_string(),
            is_night_shift: true,
        };
        // 8h - 0.5h break = 7.5h
        assert!((interval_hours(&interval) - 7.5).abs() < 0.01);
    }

    #[test]
    fn test_monday_of_week() {
        // 2026-07-23 is a Thursday
        let thu = NaiveDate::from_ymd_opt(2026, 7, 23).unwrap();
        let mon = monday_of_week(thu);
        assert_eq!(mon, NaiveDate::from_ymd_opt(2026, 7, 20).unwrap());
        // Monday itself
        let mon2 = NaiveDate::from_ymd_opt(2026, 7, 20).unwrap();
        assert_eq!(monday_of_week(mon2), mon2);
    }

    #[test]
    fn test_last_day_of_month() {
        let jul = NaiveDate::from_ymd_opt(2026, 7, 15).unwrap();
        let last = last_day_of_month(jul);
        assert_eq!(last, NaiveDate::from_ymd_opt(2026, 7, 31).unwrap());
        let feb = NaiveDate::from_ymd_opt(2026, 2, 1).unwrap();
        let last_feb = last_day_of_month(feb);
        assert_eq!(last_feb, NaiveDate::from_ymd_opt(2026, 2, 28).unwrap());
    }
}
