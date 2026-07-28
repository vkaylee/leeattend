use super::*;
use crate::repositories::attendance_daily_summary_repository::MockAttendanceDailySummaryRepository;
use crate::repositories::attendance_policy_repository::MockAttendancePolicyRepository;
use crate::repositories::employee::MockEmployeeRepository;
use crate::repositories::employment_repository::MockEmploymentRepository;
use crate::repositories::holiday_repository::MockHolidayRepository;
use crate::repositories::leave_request_repository::MockLeaveRequestRepository;
use crate::repositories::roster::MockRosterRepository;
use crate::repositories::time_entry_repository::MockTimeEntryRepository;
use crate::repositories::workspace_repository::MockWorkspaceRepository;

fn dummy_date() -> NaiveDate {
    NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()
}

fn empty_condition_type_repo() -> Arc<dyn crate::repositories::ConditionTypeRepository> {
    use crate::repositories::condition_type_repository::MockConditionTypeRepository;

    let mut repo = MockConditionTypeRepository::new();
    repo.expect_list_condition_types()
        .returning(|_, _| Box::pin(async move { Ok(vec![]) }));
    Arc::new(repo)
}

fn mock_employment_repo() -> MockEmploymentRepository {
    let mut repo = MockEmploymentRepository::new();
    repo.expect_get_active_employment_on_date()
        .returning(|_, _, _| Ok(None));
    repo.expect_get_employment_by_id()
        .returning(|_, employment_id| {
            Ok(Some(crate::models::employment::Employment::new(
                employment_id,
                Uuid::new_v4(),
                employment_id,
                Uuid::new_v4(),
                None,
                None,
                "active".to_string(),
                NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                None,
                None,
                "full_time".to_string(),
                None,
                chrono::Utc::now(),
                chrono::Utc::now(),
                None,
                None,
            )))
        });
    repo
}

fn create_dummy_summary(sys_ot: i32, status: OtStatus) -> AttendanceDailySummary {
    AttendanceDailySummary {
        id: Uuid::new_v4(),
        workspace_id: Uuid::new_v4(),
        employment_id: Uuid::new_v4(),
        date: dummy_date(),
        actual_work_minutes: 0,
        system_detected_ot_minutes: sys_ot,
        claimed_ot_minutes: 0,
        approved_ot_minutes: 0,
        ot_status: status,
        ot_reason: None,
        manager_note: None,
        claim_entered_by_user_id: None,
        claim_entered_at: None,
        paper_form_photo_url: None,
        entered_via: None,
        is_holiday: false,
        holiday_name: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        policy_version_id: None,
        effective_rules_snapshot: None,
        is_superseded: false,
        superseded_by_id: None,
        superseded_at: None,
    }
}

fn make_service(
    daily_summary_repo: MockAttendanceDailySummaryRepository,
) -> AttendanceSummaryService {
    AttendanceSummaryService::new(AttendanceSummaryServiceArgs {
        time_entry_repo: Arc::new(MockTimeEntryRepository::new()),
        roster_repo: Arc::new(MockRosterRepository::new()),
        policy_repo: Arc::new(MockAttendancePolicyRepository::new()),
        condition_type_repo: empty_condition_type_repo(),
        workspace_repo: Arc::new(MockWorkspaceRepository::new()),
        employee_repo: Arc::new(MockEmployeeRepository::new()),
        employment_repo: Arc::new(mock_employment_repo()),
        holiday_repo: Arc::new(MockHolidayRepository::new()),
        leave_request_repo: Arc::new(MockLeaveRequestRepository::new()),
        daily_summary_repo: Arc::new(daily_summary_repo),
    })
}

// =========================================================================
// determine_ot_status
// =========================================================================

mod determine_ot_status_tests {
    use super::*;

    #[tokio::test]
    async fn status_none_when_no_existing_and_no_ot() {
        let mut mock = MockAttendanceDailySummaryRepository::new();
        mock.expect_get_summary().returning(|_, _, _| Ok(None));

        let svc = make_service(mock);
        let status = svc
            .determine_ot_status(
                &TenantAuthToken::system_override(Uuid::new_v4()),
                Uuid::new_v4(),
                dummy_date(),
                0,
            )
            .await
            .unwrap();

        assert_eq!(status, OtStatus::None);
    }

    #[tokio::test]
    async fn status_pending_when_no_existing_and_has_ot() {
        let mut mock = MockAttendanceDailySummaryRepository::new();
        mock.expect_get_summary().returning(|_, _, _| Ok(None));

        let svc = make_service(mock);
        let status = svc
            .determine_ot_status(
                &TenantAuthToken::system_override(Uuid::new_v4()),
                Uuid::new_v4(),
                dummy_date(),
                60,
            )
            .await
            .unwrap();

        assert_eq!(status, OtStatus::PendingEmployee);
    }

    #[tokio::test]
    async fn status_pending_when_existing_none_and_has_ot() {
        let mut mock = MockAttendanceDailySummaryRepository::new();
        mock.expect_get_summary()
            .returning(|_, _, _| Ok(Some(create_dummy_summary(0, OtStatus::None))));

        let svc = make_service(mock);
        let status = svc
            .determine_ot_status(
                &TenantAuthToken::system_override(Uuid::new_v4()),
                Uuid::new_v4(),
                dummy_date(),
                60,
            )
            .await
            .unwrap();

        assert_eq!(status, OtStatus::PendingEmployee);
    }

    #[tokio::test]
    async fn status_preserved_when_already_claimed() {
        let mut mock = MockAttendanceDailySummaryRepository::new();
        mock.expect_get_summary()
            .returning(|_, _, _| Ok(Some(create_dummy_summary(60, OtStatus::PendingManager))));

        let svc = make_service(mock);
        let token = TenantAuthToken::system_override(Uuid::new_v4());
        let status = svc
            .determine_ot_status(&token, Uuid::new_v4(), dummy_date(), 120)
            .await
            .unwrap();

        assert_eq!(status, OtStatus::PendingManager);
    }
}

// =========================================================================
// claim_ot
// =========================================================================

mod claim_ot_tests {
    use super::*;

    #[tokio::test]
    async fn claim_ot_success() {
        let mut mock = MockAttendanceDailySummaryRepository::new();
        mock.expect_claim_ot().returning(|_, _, _, _, _| Ok(1));

        let svc = make_service(mock);
        let token = TenantAuthToken::system_override(Uuid::new_v4());
        let result = svc
            .claim_ot(&token, Uuid::new_v4(), dummy_date(), 60, None)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn claim_ot_not_found() {
        let mut mock = MockAttendanceDailySummaryRepository::new();
        mock.expect_claim_ot().returning(|_, _, _, _, _| Ok(0));

        let svc = make_service(mock);
        let token = TenantAuthToken::system_override(Uuid::new_v4());
        let result = svc
            .claim_ot(&token, Uuid::new_v4(), dummy_date(), 60, None)
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }
}

// =========================================================================
// approve_ot
// =========================================================================

mod approve_ot_tests {
    use super::*;

    #[tokio::test]
    async fn approve_ot_success() {
        let mut mock = MockAttendanceDailySummaryRepository::new();
        mock.expect_approve_ot().returning(|_, _, _, _, _, _| Ok(1));

        let svc = make_service(mock);
        let token = TenantAuthToken::system_override(Uuid::new_v4());
        let result = svc
            .approve_ot(
                &token,
                Uuid::new_v4(),
                dummy_date(),
                60,
                None,
                OtStatus::Approved,
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn approve_ot_not_found() {
        let mut mock = MockAttendanceDailySummaryRepository::new();
        mock.expect_approve_ot().returning(|_, _, _, _, _, _| Ok(0));

        let svc = make_service(mock);
        let token = TenantAuthToken::system_override(Uuid::new_v4());
        let result = svc
            .approve_ot(
                &token,
                Uuid::new_v4(),
                dummy_date(),
                60,
                None,
                OtStatus::Approved,
            )
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }
}

// =========================================================================
// enter_paper_ot
// =========================================================================

mod enter_paper_ot_tests {
    use super::*;

    fn paper_ot_args(claimed: i32, allow_exceed: bool) -> EnterPaperOtArgs {
        EnterPaperOtArgs {
            employment_id: Uuid::new_v4(),
            date: dummy_date(),
            claimed_minutes: claimed,
            reason: "OT".to_string(),
            entered_by_user_id: Uuid::new_v4(),
            photo_url: None,
            allow_exceed,
        }
    }

    #[tokio::test]
    async fn paper_ot_success() {
        let mut mock = MockAttendanceDailySummaryRepository::new();
        mock.expect_get_summary().returning(|_, _, _| Ok(None));
        mock.expect_upsert_paper_ot()
            .returning(|_, _, _, _, _, _, _| Ok(create_dummy_summary(0, OtStatus::None)));

        let svc = make_service(mock);
        let token = TenantAuthToken::system_override(Uuid::new_v4());
        let result = svc.enter_paper_ot(&token, paper_ot_args(60, true)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn paper_ot_exceed_fails() {
        let mut mock = MockAttendanceDailySummaryRepository::new();
        mock.expect_get_summary()
            .returning(|_, _, _| Ok(Some(create_dummy_summary(60, OtStatus::None))));
        // upsert_paper_ot should NOT be called — validation rejects first
        mock.expect_upsert_paper_ot().never();

        let svc = make_service(mock);
        let token = TenantAuthToken::system_override(Uuid::new_v4());
        let result = svc.enter_paper_ot(&token, paper_ot_args(120, false)).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::Validation(_)));
    }

    #[tokio::test]
    async fn paper_ot_exceed_allowed_bypasses_validation() {
        let mut mock = MockAttendanceDailySummaryRepository::new();
        mock.expect_get_summary()
            .returning(|_, _, _| Ok(Some(create_dummy_summary(60, OtStatus::None))));
        mock.expect_upsert_paper_ot()
            .returning(|_, _, _, _, _, _, _| Ok(create_dummy_summary(0, OtStatus::None)));

        let svc = make_service(mock);
        // claimed_minutes=120 > system_detected=60, but allow_exceed=true
        let token = TenantAuthToken::system_override(Uuid::new_v4());
        let result = svc.enter_paper_ot(&token, paper_ot_args(120, true)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn paper_ot_no_existing_summary_proceeds() {
        let mut mock = MockAttendanceDailySummaryRepository::new();
        mock.expect_get_summary().returning(|_, _, _| Ok(None));
        mock.expect_upsert_paper_ot()
            .returning(|_, _, _, _, _, _, _| Ok(create_dummy_summary(0, OtStatus::None)));

        let svc = make_service(mock);
        // allow_exceed=false but no existing summary → no validation to fail
        let token = TenantAuthToken::system_override(Uuid::new_v4());
        let result = svc.enter_paper_ot(&token, paper_ot_args(120, false)).await;
        assert!(result.is_ok());
    }
}

// =========================================================================
// recalculate_daily_summary - Leave conflict auto-cancellation tests
// =========================================================================
mod recalculate_daily_summary_leave_tests {
    use super::*;
    use crate::models::enums::LeaveDurationType;
    use crate::models::{LeaveRequest, LeaveStatus, TimeEntry, Workspace};
    use crate::repositories::attendance_daily_summary_repository::MockAttendanceDailySummaryRepository;
    use crate::repositories::attendance_policy_repository::MockAttendancePolicyRepository;
    use crate::repositories::employee::MockEmployeeRepository;
    use crate::repositories::holiday_repository::MockHolidayRepository;
    use crate::repositories::leave_request_repository::MockLeaveRequestRepository;
    use crate::repositories::roster::MockRosterRepository;
    use crate::repositories::time_entry_repository::MockTimeEntryRepository;
    use crate::repositories::workspace_repository::MockWorkspaceRepository;
    use chrono::Utc;

    fn create_approved_leave(
        id: Uuid,
        employee_id: Uuid,
        start: NaiveDate,
        end: NaiveDate,
    ) -> LeaveRequest {
        LeaveRequest::new(
            id,
            employee_id,
            Uuid::new_v4(),
            start,
            end,
            None,
            LeaveStatus::Approved,
            LeaveDurationType::FullDay,
            None,
            None,
            None,
            None,
            chrono::Utc::now(),
            chrono::Utc::now(),
            None,
        )
    }

    fn test_workspace(workspace_id: Uuid) -> Workspace {
        Workspace::new(
            workspace_id,
            "Test".to_string(),
            "YYYY-MM-DD".to_string(),
            "HH:mm".to_string(),
            "UTC".to_string(),
            "indigo".to_string(),
            "vi".to_string(),
            None,
            None,
            "std".to_string(),
            rust_decimal::Decimal::ONE,
            Utc::now(),
            None,
        )
    }

    #[tokio::test]
    async fn test_leave_conflict_auto_cancel_single_day() {
        let employee_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let date = dummy_date();
        let leave_id = Uuid::new_v4();

        let mut time_entry_repo = MockTimeEntryRepository::new();
        let mut roster_repo = MockRosterRepository::new();
        let policy_repo = MockAttendancePolicyRepository::new();
        let mut workspace_repo = MockWorkspaceRepository::new();
        let mut employee_repo = MockEmployeeRepository::new();
        employee_repo
            .expect_get_employee_by_id()
            .returning(move |id, _| {
                Ok(Some(crate::models::Employee::new(
                    id,
                    workspace_id,
                    None,
                    "John Doe".to_string(),
                    "EMP001".to_string(),
                    vec![],
                    chrono::Utc::now(),
                    chrono::Utc::now(),
                    None,
                )))
            });
        let mut holiday_repo = MockHolidayRepository::new();
        let mut leave_request_repo = MockLeaveRequestRepository::new();
        let mut daily_summary_repo = MockAttendanceDailySummaryRepository::new();

        // Mock fetch workspace timezone
        workspace_repo
            .expect_get_workspace_by_id()
            .returning(move |_| Box::pin(async move { Ok(Some(test_workspace(workspace_id))) }));

        // Mock is_holiday
        holiday_repo
            .expect_is_holiday()
            .returning(move |_, _, _| Box::pin(async move { Ok(None) }));

        // Mock fetch schedule: initially we return a schedule with shift_id=Some(shift_id) (original shift)
        let shift_id = Uuid::new_v4();
        roster_repo
            .expect_get_workspace_schedules()
            .returning(move |_, _, _, _, _, _, _| {
                Box::pin(async move {
                    Ok((
                        vec![crate::models::EmployeeSchedule::new(
                            employee_id,
                            employee_id,
                            "EMP001".to_string(),
                            "John Doe".to_string(),
                            vec![crate::models::EffectiveSchedule::new(
                                date,
                                Some(shift_id),
                                "pattern".to_string(),
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                vec![],
                                false,
                                false,
                                None,
                            )],
                        )],
                        1,
                    ))
                })
            });

        // Mock fetch time entries (non-empty, i.e., worked)
        time_entry_repo
            .expect_get_employment_time_entries()
            .returning(move |_, _, _, _| {
                let punch_in = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
                    date.and_hms_opt(8, 0, 0).unwrap(),
                    chrono::Utc,
                );
                let punch_out = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
                    date.and_hms_opt(17, 0, 0).unwrap(),
                    chrono::Utc,
                );
                Ok(vec![TimeEntry::new(
                    Uuid::new_v4(),
                    employee_id,
                    workspace_id,
                    None,
                    None,
                    punch_in,
                    Some(punch_out),
                    Some(480),
                    false,
                    "work".to_string(),
                    None,
                    None,
                    Utc::now(),
                    Utc::now(),
                )])
            });

        // Mock get leave requests: returns one approved leave covering the date
        leave_request_repo
            .expect_get_employee_leave_requests()
            .returning(move |_, _| {
                Box::pin(async move {
                    Ok(vec![create_approved_leave(
                        leave_id,
                        employee_id,
                        date,
                        date,
                    )])
                })
            });

        // Expect cancellation of the leave request
        leave_request_repo
            .expect_cancel_leave_request()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(leave_id),
            )
            .times(1)
            .returning(|_, _| Box::pin(async move { Ok(()) }));

        // Under Option B, we never delete roster overrides because we don't write them on approval
        roster_repo.expect_delete_roster().never();

        // Expect get summary
        daily_summary_repo
            .expect_get_summary()
            .returning(|_, _, _| Ok(None));

        // Expect upsert daily summary
        daily_summary_repo
            .expect_upsert_calculated_summary()
            .times(1)
            .returning(|_, _, _, _, _, _, _, _| Ok(()));

        // Construct service
        let svc = AttendanceSummaryService::new(AttendanceSummaryServiceArgs {
            time_entry_repo: Arc::new(time_entry_repo),
            roster_repo: Arc::new(roster_repo),
            policy_repo: Arc::new(policy_repo),
            condition_type_repo: empty_condition_type_repo(),
            workspace_repo: Arc::new(workspace_repo),
            employee_repo: Arc::new(employee_repo),
            employment_repo: Arc::new(mock_employment_repo()),
            holiday_repo: Arc::new(holiday_repo),
            leave_request_repo: Arc::new(leave_request_repo),
            daily_summary_repo: Arc::new(daily_summary_repo),
        });

        let result = svc
            .recalculate_daily_summary(
                &TenantAuthToken::system_override(workspace_id),
                employee_id,
                date,
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_leave_conflict_auto_cancel_multi_day() {
        let employee_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let date = dummy_date();
        let leave_id = Uuid::new_v4();

        let mut time_entry_repo = MockTimeEntryRepository::new();
        let mut roster_repo = MockRosterRepository::new();
        let policy_repo = MockAttendancePolicyRepository::new();
        let mut workspace_repo = MockWorkspaceRepository::new();
        let mut employee_repo = MockEmployeeRepository::new();
        employee_repo
            .expect_get_employee_by_id()
            .returning(move |id, _| {
                Ok(Some(crate::models::Employee::new(
                    id,
                    workspace_id,
                    None,
                    "John Doe".to_string(),
                    "EMP001".to_string(),
                    vec![],
                    chrono::Utc::now(),
                    chrono::Utc::now(),
                    None,
                )))
            });
        let mut holiday_repo = MockHolidayRepository::new();
        let mut leave_request_repo = MockLeaveRequestRepository::new();
        let mut daily_summary_repo = MockAttendanceDailySummaryRepository::new();

        workspace_repo
            .expect_get_workspace_by_id()
            .returning(move |_| Box::pin(async move { Ok(Some(test_workspace(workspace_id))) }));

        // Mock is_holiday
        holiday_repo
            .expect_is_holiday()
            .returning(move |_, _, _| Box::pin(async move { Ok(None) }));

        roster_repo
            .expect_get_workspace_schedules()
            .returning(move |_, _, _, _, _, _, _| {
                Box::pin(async move {
                    Ok((
                        vec![crate::models::EmployeeSchedule::new(
                            employee_id,
                            employee_id,
                            "EMP001".to_string(),
                            "John Doe".to_string(),
                            vec![crate::models::EffectiveSchedule::new(
                                date,
                                None,
                                "pattern".to_string(),
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                vec![],
                                false,
                                false,
                                None,
                            )],
                        )],
                        1,
                    ))
                })
            });

        time_entry_repo
            .expect_get_employment_time_entries()
            .returning(move |_, _, _, _| {
                let punch_in = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
                    date.and_hms_opt(8, 0, 0).unwrap(),
                    chrono::Utc,
                );
                let punch_out = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
                    date.and_hms_opt(17, 0, 0).unwrap(),
                    chrono::Utc,
                );
                Ok(vec![TimeEntry::new(
                    Uuid::new_v4(),
                    employee_id,
                    workspace_id,
                    None,
                    None,
                    punch_in,
                    Some(punch_out),
                    Some(480),
                    false,
                    "work".to_string(),
                    None,
                    None,
                    Utc::now(),
                    Utc::now(),
                )])
            });

        // Leave request covers Mon to Wed
        let start = date;
        let end = date.succ_opt().unwrap().succ_opt().unwrap();
        leave_request_repo
            .expect_get_employee_leave_requests()
            .returning(move |_, _| {
                Box::pin(async move {
                    Ok(vec![create_approved_leave(
                        leave_id,
                        employee_id,
                        start,
                        end,
                    )])
                })
            });

        leave_request_repo
            .expect_cancel_leave_request()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(leave_id),
            )
            .times(1)
            .returning(|_, _| Box::pin(async move { Ok(()) }));

        // Under Option B, we never delete roster overrides
        roster_repo.expect_delete_roster().never();

        daily_summary_repo
            .expect_get_summary()
            .returning(|_, _, _| Ok(None));

        daily_summary_repo
            .expect_upsert_calculated_summary()
            .times(1)
            .returning(|_, _, _, _, _, _, _, _| Ok(()));

        let svc = AttendanceSummaryService::new(AttendanceSummaryServiceArgs {
            time_entry_repo: Arc::new(time_entry_repo),
            roster_repo: Arc::new(roster_repo),
            policy_repo: Arc::new(policy_repo),
            condition_type_repo: empty_condition_type_repo(),
            workspace_repo: Arc::new(workspace_repo),
            employee_repo: Arc::new(employee_repo),
            employment_repo: Arc::new(mock_employment_repo()),
            holiday_repo: Arc::new(holiday_repo),
            leave_request_repo: Arc::new(leave_request_repo),
            daily_summary_repo: Arc::new(daily_summary_repo),
        });

        let result = svc
            .recalculate_daily_summary(
                &TenantAuthToken::system_override(workspace_id),
                employee_id,
                date,
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_leave_conflict_pending_rejected_not_cancelled() {
        let employee_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let date = dummy_date();

        let mut time_entry_repo = MockTimeEntryRepository::new();
        let mut roster_repo = MockRosterRepository::new();
        let policy_repo = MockAttendancePolicyRepository::new();
        let mut workspace_repo = MockWorkspaceRepository::new();
        let mut employee_repo = MockEmployeeRepository::new();
        employee_repo
            .expect_get_employee_by_id()
            .returning(move |id, _| {
                Ok(Some(crate::models::Employee::new(
                    id,
                    workspace_id,
                    None,
                    "John Doe".to_string(),
                    "EMP001".to_string(),
                    vec![],
                    chrono::Utc::now(),
                    chrono::Utc::now(),
                    None,
                )))
            });
        let mut holiday_repo = MockHolidayRepository::new();
        let mut leave_request_repo = MockLeaveRequestRepository::new();
        let mut daily_summary_repo = MockAttendanceDailySummaryRepository::new();

        workspace_repo
            .expect_get_workspace_by_id()
            .returning(move |_| Box::pin(async move { Ok(Some(test_workspace(workspace_id))) }));

        // Mock is_holiday
        holiday_repo
            .expect_is_holiday()
            .returning(move |_, _, _| Box::pin(async move { Ok(None) }));

        roster_repo
            .expect_get_workspace_schedules()
            .returning(move |_, _, _, _, _, _, _| {
                Box::pin(async move {
                    Ok((
                        vec![crate::models::EmployeeSchedule::new(
                            employee_id,
                            employee_id,
                            "EMP001".to_string(),
                            "John Doe".to_string(),
                            vec![crate::models::EffectiveSchedule::new(
                                date,
                                None,
                                "pattern".to_string(),
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                vec![],
                                false,
                                false,
                                None,
                            )],
                        )],
                        1,
                    ))
                })
            });

        time_entry_repo
            .expect_get_employment_time_entries()
            .returning(move |_, _, _, _| {
                let punch_in = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
                    date.and_hms_opt(8, 0, 0).unwrap(),
                    chrono::Utc,
                );
                let punch_out = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
                    date.and_hms_opt(17, 0, 0).unwrap(),
                    chrono::Utc,
                );
                Ok(vec![TimeEntry::new(
                    Uuid::new_v4(),
                    employee_id,
                    workspace_id,
                    None,
                    None,
                    punch_in,
                    Some(punch_out),
                    Some(480),
                    false,
                    "work".to_string(),
                    None,
                    None,
                    Utc::now(),
                    Utc::now(),
                )])
            });

        // Pending request
        let req = LeaveRequest::new(
            Uuid::new_v4(),
            employee_id,
            Uuid::new_v4(),
            date,
            date,
            None,
            LeaveStatus::Pending,
            LeaveDurationType::FullDay,
            None,
            None,
            None,
            None,
            chrono::Utc::now(),
            chrono::Utc::now(),
            None,
        );
        leave_request_repo
            .expect_get_employee_leave_requests()
            .returning(move |_, _| {
                let req_clone = req.clone();
                Box::pin(async move { Ok(vec![req_clone]) })
            });

        // Should NOT call cancel_leave_request or delete_roster
        leave_request_repo.expect_cancel_leave_request().never();
        roster_repo.expect_delete_roster().never();

        daily_summary_repo
            .expect_get_summary()
            .returning(|_, _, _| Ok(None));

        daily_summary_repo
            .expect_upsert_calculated_summary()
            .times(1)
            .returning(|_, _, _, _, _, _, _, _| Ok(()));

        let svc = AttendanceSummaryService::new(AttendanceSummaryServiceArgs {
            time_entry_repo: Arc::new(time_entry_repo),
            roster_repo: Arc::new(roster_repo),
            policy_repo: Arc::new(policy_repo),
            condition_type_repo: empty_condition_type_repo(),
            workspace_repo: Arc::new(workspace_repo),
            employee_repo: Arc::new(employee_repo),
            employment_repo: Arc::new(mock_employment_repo()),
            holiday_repo: Arc::new(holiday_repo),
            leave_request_repo: Arc::new(leave_request_repo),
            daily_summary_repo: Arc::new(daily_summary_repo),
        });

        let result = svc
            .recalculate_daily_summary(
                &TenantAuthToken::system_override(workspace_id),
                employee_id,
                date,
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_leave_no_punch_not_cancelled() {
        let employee_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let date = dummy_date();

        let mut time_entry_repo = MockTimeEntryRepository::new();
        let mut roster_repo = MockRosterRepository::new();
        let policy_repo = MockAttendancePolicyRepository::new();
        let mut workspace_repo = MockWorkspaceRepository::new();
        let mut employee_repo = MockEmployeeRepository::new();
        employee_repo
            .expect_get_employee_by_id()
            .returning(move |id, _| {
                Ok(Some(crate::models::Employee::new(
                    id,
                    workspace_id,
                    None,
                    "John Doe".to_string(),
                    "EMP001".to_string(),
                    vec![],
                    chrono::Utc::now(),
                    chrono::Utc::now(),
                    None,
                )))
            });
        let mut holiday_repo = MockHolidayRepository::new();
        let mut leave_request_repo = MockLeaveRequestRepository::new();
        let mut daily_summary_repo = MockAttendanceDailySummaryRepository::new();

        workspace_repo
            .expect_get_workspace_by_id()
            .returning(move |_| Box::pin(async move { Ok(Some(test_workspace(workspace_id))) }));

        // Mock is_holiday
        holiday_repo
            .expect_is_holiday()
            .returning(move |_, _, _| Box::pin(async move { Ok(None) }));

        roster_repo
            .expect_get_workspace_schedules()
            .returning(move |_, _, _, _, _, _, _| {
                Box::pin(async move {
                    Ok((
                        vec![crate::models::EmployeeSchedule::new(
                            employee_id,
                            employee_id,
                            "EMP001".to_string(),
                            "John Doe".to_string(),
                            vec![crate::models::EffectiveSchedule::new(
                                date,
                                None,
                                "pattern".to_string(),
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                vec![],
                                false,
                                false,
                                None,
                            )],
                        )],
                        1,
                    ))
                })
            });

        // NO time entries (i.e., did not work)
        time_entry_repo
            .expect_get_employment_time_entries()
            .returning(move |_, _, _, _| Ok(vec![]));

        // Should NOT call get_employee_leave_requests, cancel_leave_request or delete_roster
        leave_request_repo
            .expect_get_employee_leave_requests()
            .never();
        leave_request_repo.expect_cancel_leave_request().never();
        roster_repo.expect_delete_roster().never();

        daily_summary_repo
            .expect_get_summary()
            .returning(|_, _, _| Ok(None));

        daily_summary_repo
            .expect_upsert_calculated_summary()
            .times(1)
            .returning(|_, _, _, _, _, _, _, _| Ok(()));

        let svc = AttendanceSummaryService::new(AttendanceSummaryServiceArgs {
            time_entry_repo: Arc::new(time_entry_repo),
            roster_repo: Arc::new(roster_repo),
            policy_repo: Arc::new(policy_repo),
            condition_type_repo: empty_condition_type_repo(),
            workspace_repo: Arc::new(workspace_repo),
            employee_repo: Arc::new(employee_repo),
            employment_repo: Arc::new(mock_employment_repo()),
            holiday_repo: Arc::new(holiday_repo),
            leave_request_repo: Arc::new(leave_request_repo),
            daily_summary_repo: Arc::new(daily_summary_repo),
        });

        let result = svc
            .recalculate_daily_summary(
                &TenantAuthToken::system_override(workspace_id),
                employee_id,
                date,
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_leave_on_holiday_not_cancelled_by_regular_logic() {
        let employee_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let date = dummy_date();

        let mut time_entry_repo = MockTimeEntryRepository::new();
        let mut roster_repo = MockRosterRepository::new();
        let policy_repo = MockAttendancePolicyRepository::new();
        let mut workspace_repo = MockWorkspaceRepository::new();
        let mut employee_repo = MockEmployeeRepository::new();
        employee_repo
            .expect_get_employee_by_id()
            .returning(move |id, _| {
                Ok(Some(crate::models::Employee::new(
                    id,
                    workspace_id,
                    None,
                    "John Doe".to_string(),
                    "EMP001".to_string(),
                    vec![],
                    chrono::Utc::now(),
                    chrono::Utc::now(),
                    None,
                )))
            });
        let mut holiday_repo = MockHolidayRepository::new();
        let mut leave_request_repo = MockLeaveRequestRepository::new();
        let mut daily_summary_repo = MockAttendanceDailySummaryRepository::new();

        workspace_repo
            .expect_get_workspace_by_id()
            .returning(move |_| Box::pin(async move { Ok(Some(test_workspace(workspace_id))) }));

        // Mock is_holiday for the holiday check inside handle_holiday_leave_conflict
        holiday_repo
            .expect_is_holiday()
            .returning(move |_, _, _| Box::pin(async move { Ok(None) }));

        roster_repo
            .expect_get_workspace_schedules()
            .returning(move |_, _, _, _, _, _, _| {
                Box::pin(async move {
                    Ok((
                        vec![crate::models::EmployeeSchedule::new(
                            employee_id,
                            employee_id,
                            "EMP001".to_string(),
                            "John Doe".to_string(),
                            vec![crate::models::EffectiveSchedule::new(
                                date,
                                None,
                                "pattern".to_string(),
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                vec![],
                                true,
                                false,
                                Some("Test Holiday".to_string()),
                            )],
                        )],
                        1,
                    ))
                })
            });

        // Worked (has time entries)
        time_entry_repo
            .expect_get_employment_time_entries()
            .returning(move |_, _, _, _| {
                let punch_in = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
                    date.and_hms_opt(8, 0, 0).unwrap(),
                    chrono::Utc,
                );
                let punch_out = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
                    date.and_hms_opt(17, 0, 0).unwrap(),
                    chrono::Utc,
                );
                Ok(vec![TimeEntry::new(
                    Uuid::new_v4(),
                    employee_id,
                    workspace_id,
                    None,
                    None,
                    punch_in,
                    Some(punch_out),
                    Some(480),
                    false,
                    "work".to_string(),
                    None,
                    None,
                    Utc::now(),
                    Utc::now(),
                )])
            });

        // Under regular logic, it should NOT try to cancel regular leaves or query them because it's a holiday
        leave_request_repo
            .expect_get_employee_leave_requests()
            .never();
        leave_request_repo.expect_cancel_leave_request().never();
        roster_repo.expect_delete_roster().never();

        daily_summary_repo
            .expect_get_summary()
            .returning(|_, _, _| Ok(None));

        daily_summary_repo
            .expect_upsert_calculated_summary()
            .times(1)
            .returning(|_, _, _, _, _, _, _, _| Ok(()));

        let svc = AttendanceSummaryService::new(AttendanceSummaryServiceArgs {
            time_entry_repo: Arc::new(time_entry_repo),
            roster_repo: Arc::new(roster_repo),
            policy_repo: Arc::new(policy_repo),
            condition_type_repo: empty_condition_type_repo(),
            workspace_repo: Arc::new(workspace_repo),
            employee_repo: Arc::new(employee_repo),
            employment_repo: Arc::new(mock_employment_repo()),
            holiday_repo: Arc::new(holiday_repo),
            leave_request_repo: Arc::new(leave_request_repo),
            daily_summary_repo: Arc::new(daily_summary_repo),
        });

        let result = svc
            .recalculate_daily_summary(
                &TenantAuthToken::system_override(workspace_id),
                employee_id,
                date,
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_recalculate_attendance_range_success() {
        let workspace_id = Uuid::new_v4();
        let employee_id = Uuid::new_v4();
        let date = NaiveDate::from_ymd_opt(2026, 6, 20).unwrap();
        let date2 = date + chrono::Duration::days(1);

        let mut time_entry_repo = MockTimeEntryRepository::new();
        let mut roster_repo = MockRosterRepository::new();
        let policy_repo = MockAttendancePolicyRepository::new();
        let mut workspace_repo = MockWorkspaceRepository::new();
        let employee_repo = MockEmployeeRepository::new();
        let mut holiday_repo = MockHolidayRepository::new();
        let mut leave_request_repo = MockLeaveRequestRepository::new();
        let mut daily_summary_repo = MockAttendanceDailySummaryRepository::new();

        let emp_id = employee_id;
        let ws_id = workspace_id;

        workspace_repo
            .expect_get_workspace_by_id()
            .returning(move |_| Box::pin(async move { Ok(Some(test_workspace(ws_id))) }));

        roster_repo
            .expect_get_workspace_schedules()
            .returning(move |_, _, _, _, _, _, _| {
                Box::pin(async move {
                    Ok((
                        vec![crate::models::EmployeeSchedule::new(
                            emp_id,
                            emp_id,
                            "EMP001".to_string(),
                            "John Doe".to_string(),
                            vec![
                                crate::models::EffectiveSchedule::new(
                                    date,
                                    None,
                                    "pattern".to_string(),
                                    None,
                                    None,
                                    None,
                                    None,
                                    None,
                                    None,
                                    None,
                                    None,
                                    None,
                                    None,
                                    None,
                                    None,
                                    vec![],
                                    false,
                                    false,
                                    None,
                                ),
                                crate::models::EffectiveSchedule::new(
                                    date2,
                                    None,
                                    "pattern".to_string(),
                                    None,
                                    None,
                                    None,
                                    None,
                                    None,
                                    None,
                                    None,
                                    None,
                                    None,
                                    None,
                                    None,
                                    None,
                                    vec![],
                                    false,
                                    false,
                                    None,
                                ),
                            ],
                        )],
                        1,
                    ))
                })
            });

        time_entry_repo
            .expect_get_employment_time_entries()
            .returning(move |_, _, _, _| Ok(vec![]));

        leave_request_repo
            .expect_get_employee_leave_requests()
            .returning(|_, _| Box::pin(async { Ok(vec![]) }));

        holiday_repo
            .expect_get_holidays_in_range()
            .returning(|_, _, _| Box::pin(async { Ok(std::collections::HashMap::new()) }));

        daily_summary_repo
            .expect_get_summaries()
            .returning(|_, _, _, _| Ok(vec![]));

        daily_summary_repo
            .expect_upsert_calculated_summary()
            .times(2)
            .returning(|_, _, _, _, _, _, _, _| Ok(()));

        let svc = AttendanceSummaryService::new(AttendanceSummaryServiceArgs {
            time_entry_repo: Arc::new(time_entry_repo),
            roster_repo: Arc::new(roster_repo),
            policy_repo: Arc::new(policy_repo),
            condition_type_repo: empty_condition_type_repo(),
            workspace_repo: Arc::new(workspace_repo),
            employee_repo: Arc::new(employee_repo),
            employment_repo: Arc::new(mock_employment_repo()),
            holiday_repo: Arc::new(holiday_repo),
            leave_request_repo: Arc::new(leave_request_repo),
            daily_summary_repo: Arc::new(daily_summary_repo),
        });

        let result = svc
            .recalculate_attendance_range(
                &TenantAuthToken::system_override(workspace_id),
                employee_id,
                date,
                date2,
            )
            .await;
        assert!(result.is_ok());
    }
}
