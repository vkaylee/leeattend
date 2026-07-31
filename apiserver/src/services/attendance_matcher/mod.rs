//! AttendanceMatcher — Shift resolution, entry-to-shift matching,
//! daily unit calculation, and holiday credit logic.
//!
//! Extracted from `PayrollService::calculate_monthly_payslip` to satisfy SRP.

use chrono::NaiveDate;
use rust_decimal::Decimal;

use super::attendance_context::AttendanceMatcherContext;

use std::collections::HashMap;
use uuid::Uuid;

// ── Sub-modules ─────────────────────────────────────────────────────────

mod pairing;
mod scoring;

#[cfg(test)]
mod tests_mod;

// ── Public types ────────────────────────────────────────────────────────

/// Result of processing attendance for a single payroll segment.
pub struct SegmentAttendanceResult {
    pub ot_pay: Decimal,
    pub holiday_work_premium: Decimal,
    pub late_early_count: i32,
}

/// Arguments for [`AttendanceMatcher::process_segment_attendance`].
pub struct ProcessSegmentAttendanceArgs<'a> {
    pub ctx: &'a AttendanceMatcherContext,
    pub seg_start: NaiveDate,
    pub seg_end: NaiveDate,
    pub segment_total_base: Decimal,
    pub segment_ot_base: Decimal,
    pub std_config: &'a crate::domain::policy::resolver::StandardMonthConfig,
    pub attendance_rules: &'a crate::models::AttendanceRules,
    pub seg_units_per_day: Decimal,
    pub policy_cache: &'a mut HashMap<Uuid, Vec<crate::models::OvertimeRule>>,
}

// ── Public API ──────────────────────────────────────────────────────────

pub struct AttendanceMatcher;

impl AttendanceMatcher {
    /// Process daily attendance for a payroll segment date range.
    ///
    /// Iterates each calendar day in `[seg_start, seg_end]`, resolves shifts,
    /// matches time‑entries to shifts, calculates standard/OT units, holiday
    /// premiums, and OT pay.
    #[tracing::instrument(skip_all)]
    pub async fn process_segment_attendance(
        args: ProcessSegmentAttendanceArgs<'_>,
    ) -> SegmentAttendanceResult {
        let employee_id = args.ctx.employee_id();

        let mut segment_ot_pay = Decimal::ZERO;
        let mut holiday_work_premium = Decimal::ZERO;
        let mut late_early_count = 0i32;

        let mut current_date = args.seg_start;
        while current_date <= args.seg_end {
            // Resolve active employment, employer, and location for this date
            let Some((active_employer_id, active_employment_id, active_location_id)) =
                Self::resolve_day_context(args.ctx, current_date)
            else {
                current_date = match current_date.succ_opt() {
                    Some(d) => d,
                    None => break,
                };
                continue;
            };

            // Collect and sort shifts from pre-loaded roster data
            let day_shifts = Self::collect_sorted_day_shifts(
                args.ctx,
                active_employment_id,
                employee_id,
                active_location_id,
                active_employer_id,
                current_date,
            );

            if let Some(entries) = args.ctx.daily_entries_map().get(&current_date) {
                let scoring_ctx = scoring::ScoringContext::new(
                    args.ctx.tz(),
                    args.ctx.working_conditions(),
                    args.ctx.holidays_map(),
                    args.ctx.condition_type_catalog(),
                );
                let (_day_std, _day_ot, day_ot_pay, day_premium, day_violations) =
                    Self::process_daily_entries(
                        &scoring_ctx,
                        current_date,
                        &day_shifts,
                        entries,
                        args.attendance_rules,
                    );
                segment_ot_pay += day_ot_pay;
                holiday_work_premium += day_premium;
                late_early_count += day_violations;
            }

            current_date = match current_date.succ_opt() {
                Some(d) => d,
                None => break,
            };
        }

        SegmentAttendanceResult {
            ot_pay: segment_ot_pay,
            holiday_work_premium,
            late_early_count,
        }
    }
}
