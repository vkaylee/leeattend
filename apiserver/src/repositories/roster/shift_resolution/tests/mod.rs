#[cfg(test)]
#[allow(
    clippy::too_many_lines,
    clippy::similar_names,
    clippy::clone_on_ref_ptr
)]
mod tests {
    use super::*;
    use crate::cache::{RosterData, RosterFixedAssignment, RosterPatternEntry};
    use chrono::NaiveTime;
    use std::sync::Arc;

    fn resolve_effective_shift_from_data(
        roster_data: &RosterData,
        roster_map: &HashMap<(Uuid, NaiveDate), Option<Uuid>>,
        employee_id: Uuid,
        employee_id_number: &str,
        employee_tags: Option<&Vec<String>>,
        date: NaiveDate,
    ) -> (Vec<Option<Uuid>>, String, Vec<ScheduleTrace>) {
        super::resolve_effective_shift_from_data(
            roster_data,
            roster_map,
            employee_id,
            employee_id,
            employee_id_number,
            employee_tags,
            date,
            None,
            None,
        )
    }

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).expect("valid date")
    }

    /// Build a weekly `RosterPatternEntry` for tests (rotation fields defaulted).
    fn weekly_entry(
        pattern_id: Uuid,
        start: NaiveDate,
        end: Option<NaiveDate>,
        schedule_day: i32,
        shift_id: Option<Uuid>,
    ) -> RosterPatternEntry {
        RosterPatternEntry {
            pattern_id,
            start_date: start,
            end_date: end,
            schedule_day,
            shift_id,
            pattern_type: "weekly".to_string(),
            cycle_length_days: None,
            anchor_date: None,
        }
    }

    fn empty_roster_data() -> Arc<RosterData> {
        Arc::new(RosterData {
            shifts: HashMap::new(),
            fixed_assignments: HashMap::new(),
            pattern_assignments: HashMap::new(),
            active_rules: Vec::new(),
            rule_patterns: HashMap::new(),
            dept_history: HashMap::new(),
            shift_intervals: HashMap::new(),
        })
    }

    #[test]

    fn rotation_entry(
        pattern_id: Uuid,
        start: NaiveDate,
        end: Option<NaiveDate>,
        cycle_length_days: i32,
        anchor_date: NaiveDate,
        schedule_day: i32,
        shift_id: Option<Uuid>,
    ) -> RosterPatternEntry {
        RosterPatternEntry {
            pattern_id,
            start_date: start,
            end_date: end,
            schedule_day,
            shift_id,
            pattern_type: "rotation".to_string(),
            cycle_length_days: Some(cycle_length_days),
            anchor_date: Some(anchor_date),
        }
    }


    fn make_test_shift(id: Uuid, name: &str) -> (Uuid, RosterShift) {
        (id, RosterShift::new(name.to_string(), None))
    }

    fn make_night_shift(id: Uuid, name: &str) -> (Uuid, RosterShift) {
        (id, RosterShift::new(name.to_string(), None))
    }

    /// Create a minimal interval for a shift (used when tests need segments but
    /// don't care about interval-specific config).
    fn make_simple_interval(interval_id: Uuid, start: NaiveTime, end: NaiveTime) -> RosterInterval {
        RosterInterval {
            interval_id,
            sort_order: 0,
            interval_name: None,
            start_time: start,
            end_time: end,
            break_duration_minutes: 0,
            break_start: None,
            break_end: None,
            is_break_auto_deduct: false,
            break_is_overtime: false,
            early_punch_in_grace_period_minutes: 0,
            allow_early_ot: false,
            max_early_ot_minutes: None,
            ot_aggregation_mode:
                crate::services::attendance_calculator::OtAggregationMode::Aggregate,
            is_night_shift: false,
        }
    }

    fn make_roster_data_with_shifts(shifts_map: Vec<(Uuid, RosterShift)>) -> Arc<RosterData> {
        let mut data = (*empty_roster_data()).clone();
        data.shifts = shifts_map.into_iter().collect();
        Arc::new(data)
    }

    fn resolve_work_segments_wrapper(
        roster_data: &RosterData,
        roster_map: &HashMap<(Uuid, NaiveDate), Option<Uuid>>,
        emp_id: Uuid,
        date: NaiveDate,
    ) -> Vec<crate::models::ResolvedSegment> {
        super::resolve_work_segments_from_data(
            roster_data,
            roster_map,
            emp_id,
            emp_id,
            "EMP001",
            None,
            date,
            None,
            None,
        )
    }


    mod core;
    mod rotation1;
    mod rotation2;
    mod segments1;
    mod segments2;
    mod segments3;
}
