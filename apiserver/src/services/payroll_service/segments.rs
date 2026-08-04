//! Payroll segment builder — determines salary/dept change boundaries.
//!
//! Extracted from `payroll_service/mod.rs` to satisfy SRP.

use chrono::NaiveDate;
use std::collections::BTreeSet;

use super::context::PayrollContext;

/// Determine salary/department change segment boundaries within a payroll period.
///
/// Scans salary history and department history for effective-date changes
/// and splits the payroll period into sub-segments where salary and department
/// are stable.
pub fn build_segments(ctx: &PayrollContext) -> Vec<(NaiveDate, NaiveDate)> {
    let mut change_points = BTreeSet::new();
    change_points.insert(ctx.start_date);

    for (comp, _) in &ctx.salary_history {
        if comp.effective_date > ctx.start_date && comp.effective_date <= ctx.end_date {
            change_points.insert(comp.effective_date);
        }
    }
    for dept in &ctx.all_dept_history {
        if dept.effective_date() > ctx.start_date && dept.effective_date() <= ctx.end_date {
            change_points.insert(dept.effective_date());
        }
    }

    let sorted_dates: Vec<NaiveDate> = change_points.into_iter().collect();
    let mut segments = Vec::new();

    for i in 0..sorted_dates.len() {
        let seg_start = sorted_dates[i];
        let seg_end = if i + 1 < sorted_dates.len() {
            sorted_dates[i + 1].pred_opt().unwrap_or(seg_start)
        } else {
            ctx.end_date
        };
        segments.push((seg_start, seg_end));
    }
    segments
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::*;
    use chrono::Utc;
    use rust_decimal::Decimal;
    use std::collections::HashMap;
    use std::sync::Arc;
    use uuid::Uuid;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).expect("valid date")
    }

    fn make_salary_entry(
        effective_date: NaiveDate,
    ) -> (
        crate::models::EmployeeSalaryComponent,
        crate::models::SalaryComponentDefinition,
    ) {
        (
            crate::models::EmployeeSalaryComponent {
                id: Uuid::new_v4(),
                employment_id: Uuid::new_v4(),
                component_definition_id: Uuid::new_v4(),
                amount: Decimal::from(1000),
                effective_date,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            crate::models::SalaryComponentDefinition {
                id: Uuid::new_v4(),
                workspace_id: Uuid::new_v4(),
                name: "Base".into(),
                code: "BASE".into(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        )
    }

    fn make_dept_entry(effective_date: NaiveDate) -> crate::models::EmployeeDepartmentHistory {
        crate::models::EmployeeDepartmentHistory::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            effective_date,
            None,
            Utc::now(),
        )
    }

    fn make_ctx(
        start_date: NaiveDate,
        end_date: NaiveDate,
        salary_history: Vec<(
            crate::models::EmployeeSalaryComponent,
            crate::models::SalaryComponentDefinition,
        )>,
        all_dept_history: Vec<crate::models::EmployeeDepartmentHistory>,
    ) -> PayrollContext {
        PayrollContext {
            employee_id: Uuid::new_v4(),
            workspace: crate::models::Workspace::new(NewWorkspaceParams {
            id: Uuid::new_v4(),
            name: "Test".into(),
            date_format: "YYYY-MM-DD".into(),
            time_format: "HH:mm".into(),
            timezone: "UTC".into(),
            theme_primary_color: "#000".into(),
            default_language: "en".into(),
            logo_url: None,
            tagline: None,
            holiday_overnight_strategy: "split".into(),
            default_holiday_ot_multiplier: Decimal::from(2),
            created_at: Utc::now(),
            custom_db_url_encrypted: None,
        }),
            tz: chrono_tz::UTC,
            start_date,
            end_date,
            salary_history,
            all_dept_history,
            daily_entries_map: HashMap::new(),
            holidays_map: Arc::new(HashMap::new()),
            roster_data: Arc::new(crate::cache::RosterData {
                shifts: HashMap::new(),
                fixed_assignments: HashMap::new(),
                pattern_assignments: HashMap::new(),
                active_rules: Vec::new(),
                rule_patterns: HashMap::new(),
                dept_history: HashMap::new(),
                shift_intervals: HashMap::new(),
            }),
            emp_id_number: "EMP001".into(),
            emp_tags: None,
            payroll_roster_map: HashMap::new(),
            working_conditions: Vec::new(),
            condition_type_catalog: Arc::new(HashMap::new()),
            attendance_policy: crate::models::AttendanceRules::default(),
            emp_payroll_configs: Vec::new(),
            ws_payroll_settings_history: Arc::new(Vec::new()),
            department_payroll_configs: HashMap::new(),
            overtime_data: Arc::new(
                crate::repositories::overtime_policy_repository::WorkspaceOvertimeData {
                    policies: HashMap::new(),
                    assignments: Vec::new(),
                },
            ),
            ot_base_component_ids: std::collections::HashSet::new(),
            active_assignments: Vec::new(),
            active_employments: Vec::new(),
            primary_location_id: Uuid::new_v4(),
            month_end_payroll_settings: Arc::new(None),
            month_end_attendance_policy: crate::models::AttendanceRules::default(),
            leave_breakdown: crate::repositories::leave_request_repository::LeaveBreakdown {
                total_days: 0.0,
                unpaid_days: 0.0,
                sick_days: 0.0,
            },
            active_policies_with_tiers: Arc::new(vec![]),
            salary_base_groups: Arc::new(vec![]),
            unpaid_holiday_days: 0,
            rule_assignments_with_tiers: vec![],
            daily_summaries: HashMap::new(),
        }
    }

    #[test]
    fn test_no_changes() {
        let ctx = make_ctx(d(2026, 3, 1), d(2026, 3, 31), vec![], vec![]);
        let segments = build_segments(&ctx);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0], (d(2026, 3, 1), d(2026, 3, 31)));
    }

    #[test]
    fn test_salary_change_mid_month() {
        let ctx = make_ctx(
            d(2026, 3, 1),
            d(2026, 3, 31),
            vec![make_salary_entry(d(2026, 3, 16))],
            vec![],
        );
        let segments = build_segments(&ctx);
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0], (d(2026, 3, 1), d(2026, 3, 15)));
        assert_eq!(segments[1], (d(2026, 3, 16), d(2026, 3, 31)));
    }

    #[test]
    fn test_dept_change_mid_month() {
        let ctx = make_ctx(
            d(2026, 3, 1),
            d(2026, 3, 31),
            vec![],
            vec![make_dept_entry(d(2026, 3, 10))],
        );
        let segments = build_segments(&ctx);
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0], (d(2026, 3, 1), d(2026, 3, 9)));
        assert_eq!(segments[1], (d(2026, 3, 10), d(2026, 3, 31)));
    }

    #[test]
    fn test_multiple_changes() {
        let ctx = make_ctx(
            d(2026, 3, 1),
            d(2026, 3, 31),
            vec![make_salary_entry(d(2026, 3, 10))],
            vec![make_dept_entry(d(2026, 3, 20))],
        );
        let segments = build_segments(&ctx);
        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0], (d(2026, 3, 1), d(2026, 3, 9)));
        assert_eq!(segments[1], (d(2026, 3, 10), d(2026, 3, 19)));
        assert_eq!(segments[2], (d(2026, 3, 20), d(2026, 3, 31)));
    }

    #[test]
    fn test_change_on_first_day_no_extra_segment() {
        let ctx = make_ctx(
            d(2026, 3, 1),
            d(2026, 3, 31),
            vec![make_salary_entry(d(2026, 3, 1))],
            vec![],
        );
        let segments = build_segments(&ctx);
        assert_eq!(segments.len(), 1);
    }

    #[test]
    fn test_change_after_end_ignored() {
        let ctx = make_ctx(
            d(2026, 3, 1),
            d(2026, 3, 31),
            vec![make_salary_entry(d(2026, 4, 5))],
            vec![],
        );
        let segments = build_segments(&ctx);
        assert_eq!(segments.len(), 1);
    }

    #[test]
    fn test_same_day_salary_and_dept_deduplicated() {
        let ctx = make_ctx(
            d(2026, 3, 1),
            d(2026, 3, 31),
            vec![make_salary_entry(d(2026, 3, 15))],
            vec![make_dept_entry(d(2026, 3, 15))],
        );
        let segments = build_segments(&ctx);
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0], (d(2026, 3, 1), d(2026, 3, 14)));
        assert_eq!(segments[1], (d(2026, 3, 15), d(2026, 3, 31)));
    }
}
