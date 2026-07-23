use super::*;
use crate::cache::{RosterData, RosterShift};
use crate::models::enums::EarlyArrivalStrategy;
use crate::models::{AttendanceRules, Holiday, TimeEntry, Workspace};
use crate::repositories::overtime_policy_repository::OvertimePolicyRepositoryTrait;
use chrono::{NaiveDate, NaiveTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

// ── Mock repositories ──────────────────────────────────────────────────

struct MockOvertimePolicyRepository;

#[async_trait::async_trait]
impl OvertimePolicyRepositoryTrait for MockOvertimePolicyRepository {
    async fn create_policy_definition(
        &self,
        _workspace_id: Uuid,
        _name: &str,
        _description: Option<&str>,
    ) -> Result<crate::models::OvertimePolicy, crate::errors::AppError> {
        unimplemented!()
    }

    async fn get_policy_definitions(
        &self,
        _workspace_id: Uuid,
        _page: i64,
        _limit: i64,
    ) -> Result<(Vec<crate::models::OvertimePolicy>, i64), crate::errors::AppError> {
        unimplemented!()
    }

    async fn get_policy_with_rules(
        &self,
        _workspace_id: Uuid,
        _id: Uuid,
    ) -> Result<
        Option<(
            crate::models::OvertimePolicy,
            Vec<crate::models::OvertimeRule>,
        )>,
        crate::errors::AppError,
    > {
        Ok(None)
    }

    async fn add_rule(
        &self,
        _workspace_id: Uuid,
        _rule: &crate::models::OvertimeRule,
    ) -> Result<crate::models::OvertimeRule, crate::errors::AppError> {
        unimplemented!()
    }

    async fn resolve_policy_for_employee(
        &self,
        _employee_id: Uuid,
        _workspace_id: Uuid,
        _date: NaiveDate,
    ) -> Result<Option<Uuid>, crate::errors::AppError> {
        Ok(None)
    }

    async fn create_policy_assignment(
        &self,
        _workspace_id: Uuid,
        _assignee_type: &str,
        _assignee_id: Option<Uuid>,
        _policy_id: Uuid,
        _effective_from: NaiveDate,
    ) -> Result<(), crate::errors::AppError> {
        unimplemented!()
    }

    async fn get_workspace_overtime_data(
        &self,
        _workspace_id: Uuid,
    ) -> Result<
        crate::repositories::overtime_policy_repository::WorkspaceOvertimeData,
        crate::errors::AppError,
    > {
        Ok(
            crate::repositories::overtime_policy_repository::WorkspaceOvertimeData {
                policies: std::collections::HashMap::new(),
                assignments: Vec::new(),
            },
        )
    }
}

struct MockOvertimePolicyWithRulesRepository {
    policy_id: Uuid,
    rules: Vec<crate::models::OvertimeRule>,
}

#[async_trait::async_trait]
impl OvertimePolicyRepositoryTrait for MockOvertimePolicyWithRulesRepository {
    async fn create_policy_definition(
        &self,
        _workspace_id: Uuid,
        _name: &str,
        _description: Option<&str>,
    ) -> Result<crate::models::OvertimePolicy, crate::errors::AppError> {
        unimplemented!()
    }

    async fn get_policy_definitions(
        &self,
        _workspace_id: Uuid,
        _page: i64,
        _limit: i64,
    ) -> Result<(Vec<crate::models::OvertimePolicy>, i64), crate::errors::AppError> {
        unimplemented!()
    }

    async fn get_policy_with_rules(
        &self,
        _workspace_id: Uuid,
        id: Uuid,
    ) -> Result<
        Option<(
            crate::models::OvertimePolicy,
            Vec<crate::models::OvertimeRule>,
        )>,
        crate::errors::AppError,
    > {
        if id == self.policy_id {
            let policy = crate::models::OvertimePolicy::new(
                id,
                _workspace_id,
                "Test Policy".to_string(),
                None,
                None,
                None,
            );
            Ok(Some((policy, self.rules.clone())))
        } else {
            Ok(None)
        }
    }

    async fn add_rule(
        &self,
        _workspace_id: Uuid,
        _rule: &crate::models::OvertimeRule,
    ) -> Result<crate::models::OvertimeRule, crate::errors::AppError> {
        unimplemented!()
    }

    async fn resolve_policy_for_employee(
        &self,
        _employee_id: Uuid,
        _workspace_id: Uuid,
        _date: NaiveDate,
    ) -> Result<Option<Uuid>, crate::errors::AppError> {
        Ok(Some(self.policy_id))
    }

    async fn create_policy_assignment(
        &self,
        _workspace_id: Uuid,
        _assignee_type: &str,
        _assignee_id: Option<Uuid>,
        _policy_id: Uuid,
        _effective_from: NaiveDate,
    ) -> Result<(), crate::errors::AppError> {
        unimplemented!()
    }

    async fn get_workspace_overtime_data(
        &self,
        _workspace_id: Uuid,
    ) -> Result<
        crate::repositories::overtime_policy_repository::WorkspaceOvertimeData,
        crate::errors::AppError,
    > {
        Ok(
            crate::repositories::overtime_policy_repository::WorkspaceOvertimeData {
                policies: std::collections::HashMap::new(),
                assignments: Vec::new(),
            },
        )
    }
}

fn test_workspace(ws_id: Uuid) -> Workspace {
    Workspace::new(
        ws_id,
        "Test Workspace".to_string(),
        "YYYY-MM-DD".to_string(),
        "HH:mm:ss".to_string(),
        "UTC".to_string(),
        "#000".to_string(),
        "en".to_string(),
        None,
        None,
        "SplitAtMidnight".to_string(),
        Decimal::new(30, 1),
        Utc::now(),
        None,
    )
}

// ── Tests ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_process_segment_attendance_no_shift_fallback() {
    let employee_id = Uuid::new_v4();
    let ws_id = Uuid::new_v4();

    let workspace = test_workspace(ws_id);

    // Create time entry (punch-in: 08:00, punch-out: 12:00) => 4 hours (8 units of 30 mins)
    let punch_in = Utc::now()
        .date_naive()
        .and_hms_opt(8, 0, 0)
        .unwrap()
        .and_utc();
    let punch_out = Utc::now()
        .date_naive()
        .and_hms_opt(12, 0, 0)
        .unwrap()
        .and_utc();

    let entry = TimeEntry::new(
        Uuid::new_v4(),
        employee_id,
        ws_id,
        None,
        None,
        punch_in,
        Some(punch_out),
        Some(240),
        false,
        "work".to_string(),
        None,
        None,
        Utc::now(),
        Utc::now(),
    );

    let mut daily_entries_map = HashMap::new();
    daily_entries_map.insert(Utc::now().date_naive(), vec![entry]);

    let attendance_rules = AttendanceRules {
        work_unit_minutes: 30,
        rounding_policy: "floor".to_string(),
        early_arrival_strategy: EarlyArrivalStrategy::Block,
        grace_period_minutes: None,
    };

    let ctx = PayrollContext {
        employee_id,
        workspace,
        tz: chrono_tz::UTC,
        start_date: Utc::now().date_naive(),
        end_date: Utc::now().date_naive(),
        salary_history: vec![],
        all_dept_history: vec![],
        daily_entries_map,
        holidays_map: Arc::new(HashMap::new()),
        roster_data: Arc::new(RosterData::default()),
        emp_id_number: "EMP001".to_string(),
        emp_tags: None,
        payroll_roster_map: HashMap::new(),
        working_conditions: vec![],
        condition_type_catalog: Arc::new(HashMap::new()),
        attendance_policy: attendance_rules.clone(),
        emp_payroll_configs: vec![],
        ws_payroll_settings_history: Arc::new(vec![]),
        department_payroll_configs: HashMap::new(),
        overtime_data: Arc::new(
            crate::repositories::overtime_policy_repository::WorkspaceOvertimeData::default(),
        ),
        active_contracts: vec![],
        active_assignments: vec![],
        active_employments: vec![],
        primary_location_id: Uuid::new_v4(),
        ot_base_component_ids: std::collections::HashSet::new(),
        month_end_payroll_settings: Arc::new(None),
        month_end_attendance_policy: crate::models::AttendanceRules {
            work_unit_minutes: 30,
            rounding_policy: "floor".to_string(),
            early_arrival_strategy: EarlyArrivalStrategy::Block,
            grace_period_minutes: None,
        },
        leave_breakdown: crate::repositories::leave_request_repository::LeaveBreakdown {
            total_days: 0.0,
            unpaid_days: 0.0,
            sick_days: 0.0,
        },
        active_policies_with_tiers: Arc::new(vec![]),
        salary_base_groups: Arc::new(vec![]),
        unpaid_holiday_days: 0,
        rule_assignments_with_tiers: vec![],
    };

    let std_config = crate::domain::policy::resolver::StandardMonthConfig::new(
        Decimal::new(22, 0),
        Decimal::new(8, 0),
        "workspace".to_string(),
    );

    let mut policy_cache = HashMap::new();
    let overtime_policy_repo = MockOvertimePolicyRepository;

    // resolver doesn't get called because day_shifts is empty
    let args = ProcessSegmentAttendanceArgs {
        ctx: &ctx,
        seg_start: Utc::now().date_naive(),
        seg_end: Utc::now().date_naive(),
        segment_total_base: Decimal::new(1000, 0),
        segment_ot_base: Decimal::new(1000, 0),
        std_config: &std_config,
        attendance_rules: &attendance_rules,
        seg_units_per_day: Decimal::new(16, 0),
        policy_cache: &mut policy_cache,
        overtime_policy_repo: &overtime_policy_repo,
    };

    let result = AttendanceMatcher::process_segment_attendance(args).await;
    // 240 mins / 30 mins = 8 units
    assert_eq!(result.std_units, 8);
    assert_eq!(result.ot_units, 0);
}

#[tokio::test]
async fn test_process_segment_attendance_with_shift_matching() {
    let employee_id = Uuid::new_v4();
    let ws_id = Uuid::new_v4();
    let shift_id = Uuid::new_v4();

    let workspace = test_workspace(ws_id);

    let date = Utc::now().date_naive();
    // Time entry: 09:00 to 18:00 (matching 9-18 shift)
    let punch_in = date.and_hms_opt(9, 0, 0).unwrap().and_utc();
    let punch_out = date.and_hms_opt(18, 0, 0).unwrap().and_utc();

    let entry = TimeEntry::new(
        Uuid::new_v4(),
        employee_id,
        ws_id,
        None,
        None,
        punch_in,
        Some(punch_out),
        Some(540),
        false,
        "work".to_string(),
        None,
        None,
        Utc::now(),
        Utc::now(),
    );

    let mut daily_entries_map = HashMap::new();
    daily_entries_map.insert(date, vec![entry]);

    // Define shift: 09:00 to 18:00, 1 hour (60 min) break
    let shift = RosterShift::new("Day Shift".to_string(), Some("D".to_string()));

    let mut shifts = HashMap::new();
    shifts.insert(shift_id, shift);

    let interval_id = Uuid::new_v4();
    let mut shift_intervals = HashMap::new();
    shift_intervals.insert(
        shift_id,
        vec![crate::cache::RosterInterval {
            interval_id,
            sort_order: 0,
            interval_name: None,
            start_time: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            end_time: NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
            break_duration_minutes: 60,
            break_start: Some(NaiveTime::from_hms_opt(12, 0, 0).unwrap()),
            break_end: Some(NaiveTime::from_hms_opt(13, 0, 0).unwrap()),
            is_break_auto_deduct: true,
            break_is_overtime: false,
            early_punch_in_grace_period_minutes: 0,
            allow_early_ot: false,
            max_early_ot_minutes: None,
            ot_aggregation_mode: "aggregate".to_string(),
            is_night_shift: false,
        }],
    );

    let roster_data = RosterData {
        shifts,
        shift_intervals,
        ..Default::default()
    };

    let employment_id = Uuid::new_v4();
    let employer_id = Uuid::new_v4();
    let employment = crate::models::employment::Employment::new(
        employment_id,
        ws_id,
        employee_id,
        employer_id,
        None,
        None,
        "active".to_string(),
        date - chrono::Duration::days(10),
        None,
        None,
        "full_time".to_string(),
        None,
        Utc::now(),
        Utc::now(),
        None,
        None,
    );

    let mut payroll_roster_map = HashMap::new();
    payroll_roster_map.insert((employment_id, date), Some(shift_id));

    let attendance_rules = AttendanceRules {
        work_unit_minutes: 30,
        rounding_policy: "floor".to_string(),
        early_arrival_strategy: EarlyArrivalStrategy::Block,
        grace_period_minutes: None,
    };

    let ctx = PayrollContext {
        employee_id,
        workspace,
        tz: chrono_tz::UTC,
        start_date: date,
        end_date: date,
        salary_history: vec![],
        all_dept_history: vec![],
        daily_entries_map,
        holidays_map: Arc::new(HashMap::new()),
        roster_data: Arc::new(roster_data),
        emp_id_number: "EMP001".to_string(),
        emp_tags: None,
        payroll_roster_map,
        working_conditions: vec![],
        condition_type_catalog: Arc::new(HashMap::new()),
        attendance_policy: attendance_rules.clone(),
        emp_payroll_configs: vec![],
        ws_payroll_settings_history: Arc::new(vec![]),
        department_payroll_configs: HashMap::new(),
        overtime_data: Arc::new(
            crate::repositories::overtime_policy_repository::WorkspaceOvertimeData::default(),
        ),
        active_contracts: vec![],
        active_assignments: vec![],
        active_employments: vec![employment],
        primary_location_id: Uuid::new_v4(),
        ot_base_component_ids: std::collections::HashSet::new(),
        month_end_payroll_settings: Arc::new(None),
        month_end_attendance_policy: crate::models::AttendanceRules {
            work_unit_minutes: 30,
            rounding_policy: "floor".to_string(),
            early_arrival_strategy: EarlyArrivalStrategy::Block,
            grace_period_minutes: None,
        },
        leave_breakdown: crate::repositories::leave_request_repository::LeaveBreakdown {
            total_days: 0.0,
            unpaid_days: 0.0,
            sick_days: 0.0,
        },
        active_policies_with_tiers: Arc::new(vec![]),
        salary_base_groups: Arc::new(vec![]),
        unpaid_holiday_days: 0,
        rule_assignments_with_tiers: vec![],
    };

    let std_config = crate::domain::policy::resolver::StandardMonthConfig::new(
        Decimal::new(22, 0),
        Decimal::new(8, 0),
        "workspace".to_string(),
    );

    let mut policy_cache = HashMap::new();
    let overtime_policy_repo = MockOvertimePolicyRepository;

    let args = ProcessSegmentAttendanceArgs {
        ctx: &ctx,
        seg_start: date,
        seg_end: date,
        segment_total_base: Decimal::new(1000, 0),
        segment_ot_base: Decimal::new(1000, 0),
        std_config: &std_config,
        attendance_rules: &attendance_rules,
        seg_units_per_day: Decimal::new(16, 0),
        policy_cache: &mut policy_cache,
        overtime_policy_repo: &overtime_policy_repo,
    };

    let result = AttendanceMatcher::process_segment_attendance(args).await;
    // Total work duration = 9 hours (540 mins) minus 60 min break = 8 hours (480 mins) = 16 units of 30 mins.
    assert_eq!(result.std_units, 16);
}

#[tokio::test]
async fn test_process_segment_attendance_paid_holiday_no_work() {
    let employee_id = Uuid::new_v4();
    let ws_id = Uuid::new_v4();

    let workspace = test_workspace(ws_id);

    let date = Utc::now().date_naive();

    // Holiday map with paid holiday
    let mut holidays_map = HashMap::new();
    holidays_map.insert(
        date,
        Holiday::new(
            Uuid::new_v4(),
            ws_id,
            "New Year".to_string(),
            date,
            "full_day".to_string(),
            true,
            None,
            Some(Decimal::new(30, 1)),
            None,
            None,
            false,
            Utc::now(),
            Utc::now(),
            None,
        ),
    );

    let attendance_rules = AttendanceRules {
        work_unit_minutes: 30,
        rounding_policy: "floor".to_string(),
        early_arrival_strategy: EarlyArrivalStrategy::Block,
        grace_period_minutes: None,
    };

    let ctx = PayrollContext {
        employee_id,
        workspace,
        tz: chrono_tz::UTC,
        start_date: date,
        end_date: date,
        salary_history: vec![],
        all_dept_history: vec![],
        daily_entries_map: HashMap::new(),
        holidays_map: Arc::new(holidays_map),
        roster_data: Arc::new(RosterData::default()),
        emp_id_number: "EMP001".to_string(),
        emp_tags: None,
        payroll_roster_map: HashMap::new(),
        working_conditions: vec![],
        condition_type_catalog: Arc::new(HashMap::new()),
        attendance_policy: attendance_rules.clone(),
        emp_payroll_configs: vec![],
        ws_payroll_settings_history: Arc::new(vec![]),
        department_payroll_configs: HashMap::new(),
        overtime_data: Arc::new(
            crate::repositories::overtime_policy_repository::WorkspaceOvertimeData::default(),
        ),
        active_contracts: vec![],
        active_assignments: vec![],
        active_employments: vec![],
        primary_location_id: Uuid::new_v4(),
        ot_base_component_ids: std::collections::HashSet::new(),
        month_end_payroll_settings: Arc::new(None),
        month_end_attendance_policy: crate::models::AttendanceRules {
            work_unit_minutes: 30,
            rounding_policy: "floor".to_string(),
            early_arrival_strategy: EarlyArrivalStrategy::Block,
            grace_period_minutes: None,
        },
        leave_breakdown: crate::repositories::leave_request_repository::LeaveBreakdown {
            total_days: 0.0,
            unpaid_days: 0.0,
            sick_days: 0.0,
        },
        active_policies_with_tiers: Arc::new(vec![]),
        salary_base_groups: Arc::new(vec![]),
        unpaid_holiday_days: 0,
        rule_assignments_with_tiers: vec![],
    };

    let std_config = crate::domain::policy::resolver::StandardMonthConfig::new(
        Decimal::new(22, 0),
        Decimal::new(8, 0),
        "workspace".to_string(),
    );

    let mut policy_cache = HashMap::new();
    let overtime_policy_repo = MockOvertimePolicyRepository;

    let args = ProcessSegmentAttendanceArgs {
        ctx: &ctx,
        seg_start: date,
        seg_end: date,
        segment_total_base: Decimal::new(1000, 0),
        segment_ot_base: Decimal::new(1000, 0),
        std_config: &std_config,
        attendance_rules: &attendance_rules,
        seg_units_per_day: Decimal::new(16, 0), // 8 hours * 2 units/hr = 16 units
        policy_cache: &mut policy_cache,
        overtime_policy_repo: &overtime_policy_repo,
    };

    let result = AttendanceMatcher::process_segment_attendance(args).await;
    // Auto-credit standard day (16 units) because it's a paid holiday with no attendance
    assert_eq!(result.std_units, 16);
}

#[tokio::test]
async fn test_process_segment_attendance_holiday_work_compensation() {
    let employee_id = Uuid::new_v4();
    let ws_id = Uuid::new_v4();
    let shift_id = Uuid::new_v4();

    let workspace = test_workspace(ws_id);

    let date = Utc::now().date_naive();
    // Time entry: 09:00 to 18:00 (matching 9-18 shift)
    let punch_in = date.and_hms_opt(9, 0, 0).unwrap().and_utc();
    let punch_out = date.and_hms_opt(18, 0, 0).unwrap().and_utc();

    let entry = TimeEntry::new(
        Uuid::new_v4(),
        employee_id,
        ws_id,
        None,
        None,
        punch_in,
        Some(punch_out),
        Some(540),
        false,
        "work".to_string(),
        None,
        None,
        Utc::now(),
        Utc::now(),
    );

    let mut daily_entries_map = HashMap::new();
    daily_entries_map.insert(date, vec![entry]);

    // Define shift: 09:00 to 18:00, 1 hour break
    let shift = RosterShift::new("Day Shift".to_string(), Some("D".to_string()));

    let mut shifts = HashMap::new();
    shifts.insert(shift_id, shift);

    let interval_id = Uuid::new_v4();
    let mut shift_intervals = HashMap::new();
    shift_intervals.insert(
        shift_id,
        vec![crate::cache::RosterInterval {
            interval_id,
            sort_order: 0,
            interval_name: None,
            start_time: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            end_time: NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
            break_duration_minutes: 60,
            break_start: Some(NaiveTime::from_hms_opt(12, 0, 0).unwrap()),
            break_end: Some(NaiveTime::from_hms_opt(13, 0, 0).unwrap()),
            is_break_auto_deduct: true,
            break_is_overtime: false,
            early_punch_in_grace_period_minutes: 0,
            allow_early_ot: false,
            max_early_ot_minutes: None,
            ot_aggregation_mode: "aggregate".to_string(),
            is_night_shift: false,
        }],
    );

    let roster_data = RosterData {
        shifts,
        shift_intervals,
        ..Default::default()
    };

    let employment_id = Uuid::new_v4();
    let employer_id = Uuid::new_v4();
    let employment = crate::models::employment::Employment::new(
        employment_id,
        ws_id,
        employee_id,
        employer_id,
        None,
        None,
        "active".to_string(),
        date - chrono::Duration::days(10),
        None,
        None,
        "full_time".to_string(),
        None,
        Utc::now(),
        Utc::now(),
        None,
        None,
    );

    let mut payroll_roster_map = HashMap::new();
    payroll_roster_map.insert((employment_id, date), Some(shift_id));

    let mut holidays_map = HashMap::new();
    holidays_map.insert(
        date,
        Holiday::new(
            Uuid::new_v4(),
            ws_id,
            "New Year".to_string(),
            date,
            "full_day".to_string(),
            true,
            None,
            Some(Decimal::new(30, 1)),
            None,
            None,
            true,
            Utc::now(),
            Utc::now(),
            None,
        ),
    );

    let attendance_rules = AttendanceRules {
        work_unit_minutes: 30,
        rounding_policy: "floor".to_string(),
        early_arrival_strategy: EarlyArrivalStrategy::Block,
        grace_period_minutes: None,
    };

    let ctx = PayrollContext {
        employee_id,
        workspace,
        tz: chrono_tz::UTC,
        start_date: date,
        end_date: date,
        salary_history: vec![],
        all_dept_history: vec![],
        daily_entries_map,
        holidays_map: Arc::new(holidays_map),
        roster_data: Arc::new(roster_data),
        emp_id_number: "EMP001".to_string(),
        emp_tags: None,
        payroll_roster_map,
        working_conditions: vec![],
        condition_type_catalog: Arc::new(HashMap::new()),
        attendance_policy: attendance_rules.clone(),
        emp_payroll_configs: vec![],
        ws_payroll_settings_history: Arc::new(vec![]),
        department_payroll_configs: HashMap::new(),
        overtime_data: Arc::new(
            crate::repositories::overtime_policy_repository::WorkspaceOvertimeData::default(),
        ),
        active_contracts: vec![],
        active_assignments: vec![],
        active_employments: vec![employment],
        primary_location_id: Uuid::new_v4(),
        ot_base_component_ids: std::collections::HashSet::new(),
        month_end_payroll_settings: Arc::new(None),
        month_end_attendance_policy: crate::models::AttendanceRules {
            work_unit_minutes: 30,
            rounding_policy: "floor".to_string(),
            early_arrival_strategy: EarlyArrivalStrategy::Block,
            grace_period_minutes: None,
        },
        leave_breakdown: crate::repositories::leave_request_repository::LeaveBreakdown {
            total_days: 0.0,
            unpaid_days: 0.0,
            sick_days: 0.0,
        },
        active_policies_with_tiers: Arc::new(vec![]),
        salary_base_groups: Arc::new(vec![]),
        unpaid_holiday_days: 0,
        rule_assignments_with_tiers: vec![],
    };

    let std_config = crate::domain::policy::resolver::StandardMonthConfig::new(
        Decimal::new(22, 0),
        Decimal::new(8, 0),
        "workspace".to_string(),
    );

    let mut policy_cache = HashMap::new();
    let overtime_policy_repo = MockOvertimePolicyRepository;

    let args = ProcessSegmentAttendanceArgs {
        ctx: &ctx,
        seg_start: date,
        seg_end: date,
        segment_total_base: Decimal::new(1000, 0),
        segment_ot_base: Decimal::new(1000, 0),
        std_config: &std_config,
        attendance_rules: &attendance_rules,
        seg_units_per_day: Decimal::new(16, 0), // 8 hours = 16 units
        policy_cache: &mut policy_cache,
        overtime_policy_repo: &overtime_policy_repo,
    };

    let result = AttendanceMatcher::process_segment_attendance(args).await;
    // worked units: 16 (8 hours)
    // is_credit_on_work adds seg_units_per_day (16) => 32 std units
    assert_eq!(result.std_units, 32);
    // worked fraction = 16 / 16 = 1.0
    // daily rate = 1000 / 22 = 45.4545...
    // multiplier - 1.0 = 2.0
    // holiday premium = 1.0 * (1000 / 22) * 2.0 = 90.90909090909090909090909091
    let expected_premium = Decimal::new(1000, 0) / Decimal::new(22, 0) * Decimal::new(2, 0);
    assert_eq!(result.holiday_work_premium, expected_premium);
}

#[tokio::test]
async fn test_process_segment_attendance_with_overtime_pay() {
    let employee_id = Uuid::new_v4();
    let ws_id = Uuid::new_v4();
    let shift_id = Uuid::new_v4();
    let policy_id = Uuid::new_v4();

    let workspace = test_workspace(ws_id);

    let date = NaiveDate::from_ymd_opt(2026, 1, 26).unwrap(); // Monday
                                                              // Time entry: 09:00 to 20:00 (matching 9-18 shift) -> 2 hours late OT
    let punch_in = date.and_hms_opt(9, 0, 0).unwrap().and_utc();
    let punch_out = date.and_hms_opt(20, 0, 0).unwrap().and_utc();

    let entry = TimeEntry::new(
        Uuid::new_v4(),
        employee_id,
        ws_id,
        None,
        None,
        punch_in,
        Some(punch_out),
        Some(660),
        false,
        "work".to_string(),
        None,
        None,
        Utc::now(),
        Utc::now(),
    );

    let mut daily_entries_map = HashMap::new();
    daily_entries_map.insert(date, vec![entry]);

    // Define shift: 09:00 to 18:00, 1 hour break
    let shift = RosterShift::new("Day Shift".to_string(), Some("D".to_string()));

    let mut shifts = HashMap::new();
    shifts.insert(shift_id, shift);

    let interval_id = Uuid::new_v4();
    let mut shift_intervals = HashMap::new();
    shift_intervals.insert(
        shift_id,
        vec![crate::cache::RosterInterval {
            interval_id,
            sort_order: 0,
            interval_name: None,
            start_time: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            end_time: NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
            break_duration_minutes: 60,
            break_start: Some(NaiveTime::from_hms_opt(12, 0, 0).unwrap()),
            break_end: Some(NaiveTime::from_hms_opt(13, 0, 0).unwrap()),
            is_break_auto_deduct: true,
            break_is_overtime: false,
            early_punch_in_grace_period_minutes: 0,
            allow_early_ot: false,
            max_early_ot_minutes: None,
            ot_aggregation_mode: "aggregate".to_string(),
            is_night_shift: false,
        }],
    );

    let roster_data = RosterData {
        shifts,
        shift_intervals,
        ..Default::default()
    };

    let employment_id = Uuid::new_v4();
    let employer_id = Uuid::new_v4();
    let employment = crate::models::employment::Employment::new(
        employment_id,
        ws_id,
        employee_id,
        employer_id,
        None,
        None,
        "active".to_string(),
        date - chrono::Duration::days(10),
        None,
        None,
        "full_time".to_string(),
        None,
        Utc::now(),
        Utc::now(),
        None,
        None,
    );

    let mut payroll_roster_map = HashMap::new();
    payroll_roster_map.insert((employment_id, date), Some(shift_id));

    let attendance_rules = AttendanceRules {
        work_unit_minutes: 30,
        rounding_policy: "floor".to_string(),
        early_arrival_strategy: EarlyArrivalStrategy::Block,
        grace_period_minutes: None,
    };

    let rules = vec![crate::models::OvertimeRule::new(
        Uuid::new_v4(),
        policy_id,
        1,
        Decimal::new(20, 1), // 2.0x
        Some(NaiveTime::from_hms_opt(18, 0, 0).unwrap()),
        Some(NaiveTime::from_hms_opt(22, 0, 0).unwrap()),
        127,
        false,
        None,
        None,
    )];

    let mut policies = HashMap::new();
    let overtime_policy = crate::models::OvertimePolicy::new(
        policy_id,
        Uuid::new_v4(),
        "Test Policy".to_string(),
        None,
        None,
        None,
    );
    policies.insert(policy_id, (overtime_policy, rules.clone()));

    let assignments = vec![crate::models::OvertimePolicyAssignment::new(
        Uuid::new_v4(),
        Uuid::new_v4(),
        "employee".to_string(),
        Some(employee_id),
        policy_id,
        date,
        None,
        Some(chrono::Utc::now()),
    )];

    let overtime_data =
        crate::repositories::overtime_policy_repository::WorkspaceOvertimeData {
            policies,
            assignments,
        };

    let ctx = PayrollContext {
        employee_id,
        workspace,
        tz: chrono_tz::UTC,
        start_date: date,
        end_date: date,
        salary_history: vec![],
        all_dept_history: vec![],
        daily_entries_map,
        holidays_map: Arc::new(HashMap::new()),
        roster_data: Arc::new(roster_data),
        emp_id_number: "EMP001".to_string(),
        emp_tags: None,
        payroll_roster_map,
        working_conditions: vec![],
        condition_type_catalog: Arc::new(HashMap::new()),
        attendance_policy: attendance_rules.clone(),
        emp_payroll_configs: vec![],
        ws_payroll_settings_history: Arc::new(vec![]),
        department_payroll_configs: HashMap::new(),
        overtime_data: Arc::new(overtime_data),
        active_contracts: vec![],
        active_assignments: vec![],
        active_employments: vec![employment],
        primary_location_id: Uuid::new_v4(),
        ot_base_component_ids: std::collections::HashSet::new(),
        month_end_payroll_settings: Arc::new(None),
        month_end_attendance_policy: crate::models::AttendanceRules {
            work_unit_minutes: 30,
            rounding_policy: "floor".to_string(),
            early_arrival_strategy: EarlyArrivalStrategy::Block,
            grace_period_minutes: None,
        },
        leave_breakdown: crate::repositories::leave_request_repository::LeaveBreakdown {
            total_days: 0.0,
            unpaid_days: 0.0,
            sick_days: 0.0,
        },
        active_policies_with_tiers: Arc::new(vec![]),
        salary_base_groups: Arc::new(vec![]),
        unpaid_holiday_days: 0,
        rule_assignments_with_tiers: vec![],
    };

    let std_config = crate::domain::policy::resolver::StandardMonthConfig::new(
        Decimal::new(22, 0),
        Decimal::new(8, 0),
        "workspace".to_string(),
    );

    let overtime_policy_repo = MockOvertimePolicyWithRulesRepository {
        policy_id,
        rules: rules.clone(),
    };

    let mut policy_cache = HashMap::new();
    let args = ProcessSegmentAttendanceArgs {
        ctx: &ctx,
        seg_start: date,
        seg_end: date,
        segment_total_base: Decimal::new(1760, 0),
        segment_ot_base: Decimal::new(1760, 0), // hourly_rate = 1760 / (22*8) = 10
        std_config: &std_config,
        attendance_rules: &attendance_rules,
        seg_units_per_day: Decimal::new(16, 0),
        policy_cache: &mut policy_cache,
        overtime_policy_repo: &overtime_policy_repo,
    };

    let result = AttendanceMatcher::process_segment_attendance(args).await;
    // 8 hours std + 2 hours OT
    assert_eq!(result.std_units, 16);
    assert_eq!(result.ot_units, 4); // 2 hours = 4 units
                                    // 2 hours OT * 10 hourly_rate * 2.0 multiplier = 40.0
    assert_eq!(result.ot_pay, Decimal::new(40, 0));
}

#[tokio::test]
async fn test_process_segment_attendance_with_shift_break_is_overtime(
) -> Result<(), Box<dyn std::error::Error>> {
    let employee_id = Uuid::new_v4();
    let ws_id = Uuid::new_v4();
    let shift_id = Uuid::new_v4();

    let workspace = test_workspace(ws_id);

    let date = Utc::now().date_naive();
    // Time entry: 09:00 to 18:00 (matching 9-18 shift)
    // Employee took no break (actual break minutes = 0)
    let punch_in = date.and_hms_opt(9, 0, 0).ok_or("invalid time")?.and_utc();
    let punch_out = date.and_hms_opt(18, 0, 0).ok_or("invalid time")?.and_utc();

    let entry = TimeEntry::new(
        Uuid::new_v4(),
        employee_id,
        ws_id,
        None,
        None,
        punch_in,
        Some(punch_out),
        Some(540),
        false,
        "work".to_string(),
        None,
        None,
        Utc::now(),
        Utc::now(),
    );

    let mut daily_entries_map = HashMap::new();
    daily_entries_map.insert(date, vec![entry]);

    // Define shift: 09:00 to 18:00, 1 hour (60 min) break, break_is_overtime = true
    let shift = RosterShift::new("Day Shift".to_string(), Some("D".to_string()));

    let mut shifts = HashMap::new();
    shifts.insert(shift_id, shift);

    let interval_id = Uuid::new_v4();
    let mut shift_intervals = HashMap::new();
    shift_intervals.insert(
        shift_id,
        vec![crate::cache::RosterInterval {
            interval_id,
            sort_order: 0,
            interval_name: None,
            start_time: NaiveTime::from_hms_opt(9, 0, 0).ok_or("invalid start time")?,
            end_time: NaiveTime::from_hms_opt(18, 0, 0).ok_or("invalid end time")?,
            break_duration_minutes: 60,
            break_start: Some(NaiveTime::from_hms_opt(12, 0, 0).ok_or("invalid break start")?),
            break_end: Some(NaiveTime::from_hms_opt(13, 0, 0).ok_or("invalid break end")?),
            is_break_auto_deduct: true,
            break_is_overtime: true,
            early_punch_in_grace_period_minutes: 0,
            allow_early_ot: false,
            max_early_ot_minutes: None,
            ot_aggregation_mode: "aggregate".to_string(),
            is_night_shift: false,
        }],
    );

    let roster_data = RosterData {
        shifts,
        shift_intervals,
        ..Default::default()
    };

    let employment_id = Uuid::new_v4();
    let employer_id = Uuid::new_v4();
    let employment = crate::models::employment::Employment::new(
        employment_id,
        ws_id,
        employee_id,
        employer_id,
        None,
        None,
        "active".to_string(),
        date - chrono::Duration::days(10),
        None,
        None,
        "full_time".to_string(),
        None,
        Utc::now(),
        Utc::now(),
        None,
        None,
    );

    let mut payroll_roster_map = HashMap::new();
    payroll_roster_map.insert((employment_id, date), Some(shift_id));

    let attendance_rules = AttendanceRules {
        work_unit_minutes: 30,
        rounding_policy: "floor".to_string(),
        early_arrival_strategy: EarlyArrivalStrategy::Block,
        grace_period_minutes: None,
    };

    let ctx = PayrollContext {
        employee_id,
        workspace,
        tz: chrono_tz::UTC,
        start_date: date,
        end_date: date,
        salary_history: vec![],
        all_dept_history: vec![],
        daily_entries_map,
        holidays_map: Arc::new(HashMap::new()),
        roster_data: Arc::new(roster_data),
        emp_id_number: "EMP001".to_string(),
        emp_tags: None,
        payroll_roster_map,
        working_conditions: vec![],
        condition_type_catalog: Arc::new(HashMap::new()),
        attendance_policy: attendance_rules.clone(),
        emp_payroll_configs: vec![],
        ws_payroll_settings_history: Arc::new(vec![]),
        department_payroll_configs: HashMap::new(),
        overtime_data: Arc::new(
            crate::repositories::overtime_policy_repository::WorkspaceOvertimeData::default(),
        ),
        active_contracts: vec![],
        active_assignments: vec![],
        active_employments: vec![employment],
        primary_location_id: Uuid::new_v4(),
        ot_base_component_ids: std::collections::HashSet::new(),
        month_end_payroll_settings: Arc::new(None),
        month_end_attendance_policy: crate::models::AttendanceRules {
            work_unit_minutes: 30,
            rounding_policy: "floor".to_string(),
            early_arrival_strategy: EarlyArrivalStrategy::Block,
            grace_period_minutes: None,
        },
        leave_breakdown: crate::repositories::leave_request_repository::LeaveBreakdown {
            total_days: 0.0,
            unpaid_days: 0.0,
            sick_days: 0.0,
        },
        active_policies_with_tiers: Arc::new(vec![]),
        salary_base_groups: Arc::new(vec![]),
        unpaid_holiday_days: 0,
        rule_assignments_with_tiers: vec![],
    };

    let std_config = crate::domain::policy::resolver::StandardMonthConfig::new(
        Decimal::new(22, 0),
        Decimal::new(8, 0),
        "workspace".to_string(),
    );

    let mut policy_cache = HashMap::new();
    let overtime_policy_repo = MockOvertimePolicyRepository;

    let args = ProcessSegmentAttendanceArgs {
        ctx: &ctx,
        seg_start: date,
        seg_end: date,
        segment_total_base: Decimal::new(1000, 0),
        segment_ot_base: Decimal::new(1000, 0),
        std_config: &std_config,
        attendance_rules: &attendance_rules,
        seg_units_per_day: Decimal::new(16, 0),
        policy_cache: &mut policy_cache,
        overtime_policy_repo: &overtime_policy_repo,
    };

    let result = AttendanceMatcher::process_segment_attendance(args).await;
    assert_eq!(result.std_units, 16);
    assert_eq!(result.ot_units, 2);
    Ok(())
}
