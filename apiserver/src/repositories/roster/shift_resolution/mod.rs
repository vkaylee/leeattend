//! Pure shift resolution logic — no DB access.
//!
//! The `resolve_effective_shift_from_data` function implements the 5-tier
//! priority chain for resolving which shift applies to an employee on a date.
//!
//! The `resolve_work_segments_from_data` function extends this by expanding
//! resolved shift IDs into full `ResolvedSegment` structs, supporting
//! split-shift patterns where a single day can have multiple work segments.
//!
//! ## Priority Chain (highest → lowest)
//!
//! | Tier | Source       | Priority | Description                                              |
//! |------|-------------|----------|----------------------------------------------------------|
//! | P1   | `roster`    | 100      | Manual daily override (strongest)                        |
//! | P2   | `pattern`   | 80       | Individual working pattern (weekly / rotation)           |
//! | P3   | `rule`      | 60       | Dynamic assignment rules (dept + tags + location + employer) |
//! | P4   | `fixed`     | 20       | Fixed individual shift assignment (fallback)             |
//! | P5   | `none`      | 0        | No shift resolved (day off)                              |
//!
//! ### Why no separate "Department Pattern" or "Department Fixed" tiers?
//!
//! Legacy designs (e.g. Deputy, BambooHR) use dedicated P4/P6 tiers for
//! department-level scheduling. LeeAttend intentionally **subsumes** both
//! into the Dynamic Rules engine (P3), which accepts `department_id`,
//! `tags`, `location_id`, and `employer_id` conditions.  This is strictly
//! more flexible: a single rule can express "department X on Monday" (P4
//! equivalent) or "department X always" (P6 equivalent) without requiring
//! dedicated code paths.  See `rule_evaluator.rs` for the match logic.

use crate::cache::RosterData;
use crate::models::{ResolvedSegment, ScheduleTrace};
use chrono::NaiveDate;
use std::collections::HashMap;
use uuid::Uuid;

/// Resolve the effective shift for a single employee on a single date using
/// pre-loaded [`RosterData`] and a roster override map.
///
/// Priority chain (highest wins):
///   P1 Roster (manual daily override)
///   P2 Individual Pattern (weekly / rotation)
///   P3 Dynamic Assignment Rules (dept + tags + location + employer)
///   P5 Fixed Assignment (permanent individual fallback)
///   — (no shift → day off)
///
/// Returns `(shift_ids, source_label, traces)` where `shift_ids` may contain
/// multiple entries for split-shift patterns.
///
/// This is a **pure function** — no DB access — making it cheap to call in a
/// tight loop and easy to unit-test.
///
/// ## Source values
///
/// | source              | origin tier                                            |
/// |---------------------|--------------------------------------------------------|
/// | `"roster"`          | P1 — manual roster override for this date              |
/// | `"pattern"`         | P2 — matched via `employee_working_patterns`           |
/// | `"assignment_rule"` | P3 — matched via dynamic `assignment_rules` engine     |
/// | `"fixed"`           | P4 — fallback via `employee_shifts` (permanent assign) |
/// | `"none"`            | No shift resolved (day off / no active assignment)     |
#[allow(clippy::too_many_lines)]
pub fn resolve_effective_shift_from_data<S: ::std::hash::BuildHasher>(
    roster_data: &RosterData,
    roster_map: &HashMap<(Uuid, NaiveDate), Option<Uuid>, S>,
    employment_id: Uuid,
    employee_id: Uuid,
    employee_id_number: &str,
    employee_tags: Option<&Vec<String>>,
    date: NaiveDate,
    active_location_id: Option<Uuid>,
    active_employer_id: Option<Uuid>,
) -> (Vec<Option<Uuid>>, String, Vec<ScheduleTrace>) {
    let mut traces = Vec::new();

    // P1: Roster override
    if let Some(shift_opt) = roster_map.get(&(employment_id, date)) {
        traces.push(ScheduleTrace {
            rule_id: None,
            rule_name: "Manual Roster".to_string(),
            priority: 100,
            status: "applied".to_string(),
            reason: Some("Directly assigned via roster".to_string()),
        });
        return (vec![*shift_opt], "roster".to_string(), traces);
    }

    let dow = i32::try_from(chrono::Datelike::weekday(&date).num_days_from_sunday()).unwrap_or(0);

    // P2: Individual Working Pattern — collect ALL matching shifts for the day
    let patterns: Vec<_> = roster_data
        .pattern_assignments
        .get(&employment_id)
        .map(|v| {
            v.iter()
                .filter(|a| {
                    a.start_date <= date
                        && a.end_date.is_none_or(|ed| ed >= date)
                        && match a.pattern_type.as_str() {
                            "rotation" => {
                                let anchor = a.anchor_date.unwrap_or(a.start_date);
                                let cycle_len = a.cycle_length_days.unwrap_or(7);
                                let days_since =
                                    (date - anchor).num_days().rem_euclid(i64::from(cycle_len));
                                // Safety: days_since is in [0, cycle_len) where cycle_len <= 366
                                #[allow(clippy::cast_possible_truncation)]
                                let cycle_day = days_since as i32;
                                a.schedule_day == cycle_day
                            }
                            _ => a.schedule_day == dow, // weekly (unchanged)
                        }
                })
                .collect()
        })
        .unwrap_or_default();

    if !patterns.is_empty() {
        traces.push(ScheduleTrace {
            rule_id: Some(patterns[0].pattern_id),
            rule_name: "Individual Pattern".to_string(),
            priority: 80,
            status: "applied".to_string(),
            reason: Some("Assigned via individual working pattern".to_string()),
        });
        return (
            patterns.iter().map(|p| p.shift_id).collect(),
            "pattern".to_string(),
            traces,
        );
    }

    // P3: Dynamic Assignment Rules
    let emp_dept_id = roster_data
        .dept_history
        .get(&employee_id)
        .and_then(|v| v.iter().find(|h| h.effective_date <= date))
        .map(|h| h.department_id);

    let match_ctx = crate::services::rule_evaluator::EmployeeMatchContext {
        date,
        department_id: emp_dept_id,
        employee_id_number: employee_id_number.to_string(),
        tags: employee_tags.cloned().unwrap_or_default(),
        location_id: active_location_id,
        employer_id: active_employer_id,
    };

    let (matched_rule, rule_traces) =
        crate::services::rule_evaluator::RuleEvaluator::resolve_with_trace(
            &match_ctx,
            &roster_data.active_rules,
        );
    traces.extend(rule_traces);

    if let Some(rule) = matched_rule {
        if let Some(shift_opts) = roster_data.rule_patterns.get(&(rule.pattern_id(), dow)) {
            return (shift_opts.clone(), "assignment_rule".to_string(), traces);
        }
    }

    // P5: Fixed Assignment (Individual)
    let fixed = roster_data
        .fixed_assignments
        .get(&employment_id)
        .and_then(|v| {
            v.iter()
                .find(|a| a.start_date <= date && a.end_date.is_none_or(|ed| ed >= date))
        });

    if let Some(f) = fixed {
        traces.push(ScheduleTrace {
            rule_id: None,
            rule_name: "Fixed Assignment".to_string(),
            priority: 20,
            status: "applied".to_string(),
            reason: Some("Directly assigned shift".to_string()),
        });
        return (vec![Some(f.shift_id)], "fixed".to_string(), traces);
    }

    (vec![None], "none".to_string(), traces)
}

/// Resolve full work segments for a single employee on a single date.
///
/// Wraps [`resolve_effective_shift_from_data`] and expands each resolved
/// shift ID into one or more [`ResolvedSegment`] structs.
///
/// When a shift has intervals defined in `shift_default_intervals`, each
/// interval produces its own segment.  Otherwise a single segment is created
/// from the shift itself.
///
/// Returns segments sorted by `scheduled_start`.
#[allow(clippy::too_many_lines)]
pub fn resolve_work_segments_from_data<S: ::std::hash::BuildHasher>(
    roster_data: &RosterData,
    roster_map: &HashMap<(Uuid, NaiveDate), Option<Uuid>, S>,
    employment_id: Uuid,
    employee_id: Uuid,
    employee_id_number: &str,
    employee_tags: Option<&Vec<String>>,
    date: NaiveDate,
    active_location_id: Option<Uuid>,
    active_employer_id: Option<Uuid>,
) -> Vec<ResolvedSegment> {
    let (shift_ids, source, traces) = resolve_effective_shift_from_data(
        roster_data,
        roster_map,
        employment_id,
        employee_id,
        employee_id_number,
        employee_tags,
        date,
        active_location_id,
        active_employer_id,
    );

    let mut segments: Vec<ResolvedSegment> = Vec::new();

    for (idx, shift_opt) in shift_ids.iter().enumerate() {
        let Some(shift_id) = *shift_opt else { continue };
        let Some(shift) = roster_data.shifts.get(&shift_id) else {
            continue;
        };

        // Shift identity metadata
        let shift_name = Some(shift.name().to_string());
        let shift_symbol = shift.report_symbol().map(String::from);

        // Every shift must have at least 1 interval (guaranteed by DB migration).
        // Build segments directly from intervals — no fallback needed.
        if let Some(intervals) = roster_data.shift_intervals.get(&shift_id) {
            for interval in intervals {
                segments.push(ResolvedSegment {
                    shift_id,
                    interval_id: Some(interval.interval_id),
                    sort_order: i32::try_from(idx).unwrap_or(i32::MAX),
                    scheduled_start: interval.start_time,
                    scheduled_end: interval.end_time,
                    source: source.clone(),
                    date: Some(date),
                    shift_name: shift_name.clone(),
                    shift_symbol: shift_symbol.clone(),
                    interval_name: interval.interval_name.clone(),
                    segment_type: "work".to_string(),
                    break_duration_minutes: Some(interval.break_duration_minutes),
                    break_start: interval.break_start,
                    break_end: interval.break_end,
                    is_break_auto_deduct: Some(interval.is_break_auto_deduct),
                    break_is_overtime: Some(interval.break_is_overtime),
                    allow_early_ot: interval.allow_early_ot,
                    max_early_ot_minutes: interval.max_early_ot_minutes,
                    early_punch_in_grace_period_minutes: interval
                        .early_punch_in_grace_period_minutes,
                    ot_aggregation_mode: interval.ot_aggregation_mode.clone(),
                    is_night_shift: interval.is_night_shift,
                    traces: traces.clone(),
                    is_holiday: false,
                    is_holiday_paid: false,
                    holiday_name: None,
                });
            }
        }
    }

    // Sort by scheduled start time for correct segment ordering
    segments.sort_by_key(|s| s.scheduled_start);

    // Re-assign sort_order after sorting
    for (i, seg) in segments.iter_mut().enumerate() {
        seg.sort_order = i32::try_from(i).unwrap_or(i32::MAX);
    }

    segments
}
