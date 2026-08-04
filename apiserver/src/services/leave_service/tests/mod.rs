use super::*;
use crate::models::{
    Employee, EmployeeLeaveBalance, LeaveRequest, NewLeaveRequestParams, ShiftRoster,
    WorkspaceLeaveBalance,
};
use crate::repositories::leave_accrual_log_repository::{ExistingAccrualLog, LeaveHistoryLogItem};
use crate::repositories::leave_request_repository::{LeaveBreakdown, LeaveHistoryRequestItem};
use async_trait::async_trait;
use chrono::{NaiveDate, NaiveTime};
use rust_decimal::Decimal;
use std::sync::Mutex;

struct MockLeaveBalanceRepo {
    workspace_balances: Mutex<Vec<WorkspaceLeaveBalance>>,
    employee_balances: Mutex<Vec<EmployeeLeaveBalance>>,
    adjust_called: Mutex<bool>,
}
#[async_trait]
impl LeaveBalanceRepository for MockLeaveBalanceRepo {
    async fn get_workspace_balances(
        &self,
        _auth_token: &crate::diesel_pool::TenantAuthToken,
        _y: i32,
        _l: i64,
        _o: i64,
        _s: Option<String>,
    ) -> Result<(Vec<WorkspaceLeaveBalance>, i64), AppError> {
        let data = self.workspace_balances.lock().unwrap().clone();
        let total = data.len() as i64;
        Ok((data, total))
    }
    async fn adjust_balance(
        &self,
        _auth_token: &crate::diesel_pool::TenantAuthToken,
        _e: Uuid,
        _t: Uuid,
        _a: Decimal,
        _d: NaiveDate,
        _r: &str,
    ) -> Result<(), AppError> {
        *self.adjust_called.lock().unwrap() = true;
        Ok(())
    }
    async fn get_employee_balances(
        &self,
        _auth_token: &crate::diesel_pool::TenantAuthToken,
        _e: Uuid,
        _y: i32,
    ) -> Result<Vec<EmployeeLeaveBalance>, AppError> {
        Ok(self.employee_balances.lock().unwrap().clone())
    }
    async fn get_employee_balances_batch(
        &self,
        _auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_ids: &[Uuid],
        _year: i32,
    ) -> Result<std::collections::HashMap<Uuid, Vec<EmployeeLeaveBalance>>, AppError> {
        let mut map = std::collections::HashMap::new();
        let balances = self.employee_balances.lock().unwrap().clone();
        for &id in employee_ids {
            map.insert(id, balances.clone());
        }
        Ok(map)
    }
    async fn get_employee_balances_by_years(
        &self,
        _auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_ids: &[Uuid],
        years: &[i32],
    ) -> Result<
        std::collections::HashMap<i32, std::collections::HashMap<Uuid, Vec<EmployeeLeaveBalance>>>,
        AppError,
    > {
        let balances = self.employee_balances.lock().unwrap().clone();
        let mut outer = std::collections::HashMap::new();
        for &year in years {
            let mut inner = std::collections::HashMap::new();
            for &id in employee_ids {
                inner.insert(id, balances.clone());
            }
            outer.insert(year, inner);
        }
        Ok(outer)
    }
    async fn upsert_leave_balance(
        &self,
        _auth_token: &crate::diesel_pool::TenantAuthToken,
        _b: &crate::models::LeaveBalance,
    ) -> Result<(), AppError> {
        unimplemented!()
    }
}

struct MockLeaveRequestRepo {
    exists: Mutex<bool>,
    leave_request: Mutex<Option<LeaveRequest>>,
    update_called: Mutex<bool>,
    history_requests: Mutex<Vec<LeaveHistoryRequestItem>>,
}
#[async_trait]
impl LeaveRequestRepository for MockLeaveRequestRepo {
    async fn create_leave_request(
        &self,
        _auth_token: &crate::diesel_pool::TenantAuthToken,
        _r: &LeaveRequest,
    ) -> Result<(), AppError> {
        Ok(())
    }
    async fn check_leave_type_exists(
        &self,
        _auth_token: &crate::diesel_pool::TenantAuthToken,
        _t: Uuid,
    ) -> Result<bool, AppError> {
        Ok(*self.exists.lock().unwrap())
    }
    async fn get_leave_type_name(
        &self,
        _auth_token: &crate::diesel_pool::TenantAuthToken,
        _t: Uuid,
    ) -> Result<Option<String>, AppError> {
        Ok(Some("Annual Leave".into()))
    }
    async fn get_leave_request_by_id(
        &self,
        _auth_token: &crate::diesel_pool::TenantAuthToken,
        _i: Uuid,
    ) -> Result<Option<LeaveRequest>, AppError> {
        Ok(self.leave_request.lock().unwrap().clone())
    }
    async fn update_leave_request_status(
        &self,
        _auth_token: &crate::diesel_pool::TenantAuthToken,
        _i: Uuid,
        _s: LeaveStatus,
        _a: Option<Uuid>,
    ) -> Result<(), AppError> {
        *self.update_called.lock().unwrap() = true;
        Ok(())
    }
    async fn get_employee_leave_requests(
        &self,
        _auth_token: &crate::diesel_pool::TenantAuthToken,
        _e: Uuid,
    ) -> Result<Vec<LeaveRequest>, AppError> {
        Ok(vec![])
    }
    async fn get_workspace_leave_requests(
        &self,
        _auth_token: &crate::diesel_pool::TenantAuthToken,
        _s: Option<LeaveStatus>,
    ) -> Result<Vec<LeaveRequest>, AppError> {
        Ok(vec![])
    }
    async fn get_paginated_workspace_leave_requests(
        &self,
        _auth_token: &crate::diesel_pool::TenantAuthToken,
        _status: Option<LeaveStatus>,
        _employee_id: Option<Uuid>,
        _limit: i64,
        _offset: i64,
    ) -> Result<(Vec<LeaveRequest>, i64), AppError> {
        Ok((vec![], 0))
    }
    async fn get_employee_leave_history_requests(
        &self,
        _auth_token: &crate::diesel_pool::TenantAuthToken,
        _e: Uuid,
        _y: i32,
        _t: Option<Uuid>,
    ) -> Result<Vec<LeaveHistoryRequestItem>, AppError> {
        Ok(self.history_requests.lock().unwrap().clone())
    }

    // Unused in tests but required by trait
    async fn get_approved_leave_breakdown(
        &self,
        _e: Uuid,
        _s: NaiveDate,
        _en: NaiveDate,
        _auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<LeaveBreakdown, AppError> {
        unimplemented!()
    }
    async fn get_employee_department_for_request(
        &self,
        _r: Uuid,
        _auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Option<Uuid>, AppError> {
        unimplemented!()
    }
    async fn get_leave_types(
        &self,
        _auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<(String, String)>, AppError> {
        unimplemented!()
    }
    async fn get_system_generated_holiday_leave(
        &self,
        _auth_token: &crate::diesel_pool::TenantAuthToken,
        _e: Uuid,
        _d: NaiveDate,
    ) -> Result<Option<(Uuid, Uuid, String)>, AppError> {
        unimplemented!()
    }
    async fn cancel_leave_request(
        &self,
        _auth_token: &crate::diesel_pool::TenantAuthToken,
        _i: Uuid,
    ) -> Result<(), AppError> {
        unimplemented!()
    }
    async fn bulk_cancel_leave_requests(
        &self,
        _auth_token: &crate::diesel_pool::TenantAuthToken,
        _ids: &[Uuid],
    ) -> Result<u64, AppError> {
        Ok(0)
    }
    async fn create_leave_request_tx(
        &self,
        _conn: &mut diesel_async::AsyncPgConnection,
        _auth_token: &crate::diesel_pool::TenantAuthToken,
        __r: &LeaveRequest,
    ) -> Result<(), AppError> {
        unimplemented!()
    }
    async fn bulk_cancel_leave_requests_tx(
        &self,
        _conn: &mut diesel_async::AsyncPgConnection,
        _auth_token: &crate::diesel_pool::TenantAuthToken,
        _ids: &[Uuid],
    ) -> Result<u64, AppError> {
        unimplemented!()
    }
    async fn update_leave_request_status_tx(
        &self,
        _conn: &mut diesel_async::AsyncPgConnection,
        _auth_token: &crate::diesel_pool::TenantAuthToken,
        _id: Uuid,
        _status: LeaveStatus,
        _approved_by: Option<Uuid>,
    ) -> Result<(), AppError> {
        unimplemented!()
    }
    async fn get_unpaid_holiday_days_without_leave(
        &self,
        _auth_token: &crate::diesel_pool::TenantAuthToken,
        _e: Uuid,
        _s: NaiveDate,
        _en: NaiveDate,
    ) -> Result<i64, AppError> {
        unimplemented!()
    }
    async fn get_batch_leave_days(
        &self,
        _auth_token: &crate::diesel_pool::TenantAuthToken,
        _e: &[Uuid],
        _s: NaiveDate,
        _en: NaiveDate,
    ) -> Result<std::collections::HashMap<Uuid, i64>, AppError> {
        unimplemented!()
    }
    async fn get_approved_leave_dates(
        &self,
        _auth_token: &crate::diesel_pool::TenantAuthToken,
        _e: &[Uuid],
        _s: NaiveDate,
        _en: NaiveDate,
    ) -> Result<std::collections::HashSet<(Uuid, NaiveDate)>, AppError> {
        unimplemented!()
    }
    async fn get_approved_leave_details(
        &self,
        _auth_token: &crate::diesel_pool::TenantAuthToken,
        _employee_ids: &[Uuid],
        _start_date: NaiveDate,
        _end_date: NaiveDate,
    ) -> Result<std::collections::HashMap<(Uuid, NaiveDate), bool>, AppError> {
        Ok(std::collections::HashMap::new())
    }
    async fn get_approved_leave_periods(
        &self,
        _auth_token: &crate::diesel_pool::TenantAuthToken,
        _e: Uuid,
        _s: NaiveDate,
        _en: NaiveDate,
    ) -> Result<Vec<(NaiveDate, NaiveDate)>, AppError> {
        Ok(vec![])
    }
    async fn get_approved_leave_breakdowns_batch(
        &self,
        _employee_ids: &[Uuid],
        _start_date: NaiveDate,
        _end_date: NaiveDate,
        _auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<
        std::collections::HashMap<
            Uuid,
            crate::repositories::leave_request_repository::LeaveBreakdown,
        >,
        AppError,
    > {
        Ok(std::collections::HashMap::new())
    }
    async fn get_unpaid_holiday_days_without_leave_batch(
        &self,
        _auth_token: &crate::diesel_pool::TenantAuthToken,
        _employee_ids: &[Uuid],
        _start_date: NaiveDate,
        _end_date: NaiveDate,
    ) -> Result<std::collections::HashMap<Uuid, i64>, AppError> {
        Ok(std::collections::HashMap::new())
    }
    async fn get_leave_requests_by_ids(
        &self,
        _auth_token: &crate::diesel_pool::TenantAuthToken,
        _ids: &[Uuid],
    ) -> Result<Vec<LeaveRequest>, AppError> {
        unimplemented!()
    }
}

struct MockRosterRepo {
    upserts: Mutex<Vec<ShiftRoster>>,
}
#[async_trait]
impl RosterRepository for MockRosterRepo {
    async fn get_employment_rosters_batch(
        &self,
        _auth_token: &crate::diesel_pool::TenantAuthToken,
        _employment_ids: &[uuid::Uuid],
        _start_date: chrono::NaiveDate,
        _end_date: chrono::NaiveDate,
    ) -> Result<
        std::collections::HashMap<uuid::Uuid, Vec<crate::models::ShiftRoster>>,
        crate::errors::AppError,
    > {
        Ok(std::collections::HashMap::new())
    }

    async fn get_raw_roster_data(
        &self,
        _workspace_id: Uuid,
    ) -> Result<std::sync::Arc<crate::cache::RosterData>, AppError> {
        Ok(std::sync::Arc::new(crate::cache::RosterData::default()))
    }

    async fn upsert_roster(
        &self,
        _auth_token: &crate::diesel_pool::TenantAuthToken,
        _r: &ShiftRoster,
    ) -> Result<ShiftRoster, AppError> {
        self.upserts.lock().unwrap().push(_r.clone());
        Ok(_r.clone())
    }
    async fn bulk_upsert_roster(
        &self,
        _auth_token: &crate::diesel_pool::TenantAuthToken,
        _e: Uuid,
        _i: Vec<crate::models::RosterItem>,
    ) -> Result<(), AppError> {
        Ok(())
    }
    async fn get_effective_shift(
        &self,
        _auth_token: &crate::diesel_pool::TenantAuthToken,
        _e: Uuid,
        _d: NaiveDate,
    ) -> Result<
        (
            Option<Uuid>,
            Vec<crate::models::ScheduleTrace>,
            std::sync::Arc<crate::cache::RosterData>,
        ),
        AppError,
    > {
        Ok((
            None,
            vec![],
            std::sync::Arc::new(crate::cache::RosterData::default()),
        ))
    }
    async fn delete_roster(
        &self,
        _auth_token: &crate::diesel_pool::TenantAuthToken,
        _e: Uuid,
        _d: NaiveDate,
    ) -> Result<(), AppError> {
        Ok(())
    }
    // Unused
    async fn get_workspace_schedules(
        &self,
        _c: Option<&crate::cache::RosterCache>,
        _auth_token: &crate::diesel_pool::TenantAuthToken,
        _s: NaiveDate,
        _en: NaiveDate,
        _e: Option<&Vec<Uuid>>,
        _page: Option<i64>,
        _limit: Option<i64>,
    ) -> Result<(Vec<crate::models::EmployeeSchedule>, i64), AppError> {
        unimplemented!()
    }
    async fn get_assignment_history(
        &self,
        _auth_token: &crate::diesel_pool::TenantAuthToken,
        _page: Option<i64>,
        _limit: Option<i64>,
    ) -> Result<(Vec<crate::models::AssignmentHistory>, i64), AppError> {
        unimplemented!()
    }
    async fn get_employment_rosters(
        &self,
        _auth_token: &crate::diesel_pool::TenantAuthToken,
        _employment_id: Uuid,
        _s: NaiveDate,
        _en: NaiveDate,
        _page: Option<i64>,
        _limit: Option<i64>,
    ) -> Result<(Vec<ShiftRoster>, i64), AppError> {
        Ok((vec![], 0))
    }
}

struct MockLeaveAccrualLogRepo {
    history_logs: Mutex<Vec<LeaveHistoryLogItem>>,
}
#[async_trait]
impl crate::repositories::leave_accrual_log_repository::LeaveAccrualLogRepository
    for MockLeaveAccrualLogRepo
{
    async fn get_employment_leave_history_logs(
        &self,
        _w: Uuid,
        _e: Uuid,
        _y: i32,
        _t: Option<Uuid>,
    ) -> Result<Vec<LeaveHistoryLogItem>, AppError> {
        Ok(self.history_logs.lock().unwrap().clone())
    }

    async fn get_employment_leave_history_logs_batch(
        &self,
        _w: Uuid,
        _employment_ids: &[Uuid],
        _y: i32,
        _t: Option<Uuid>,
    ) -> Result<Vec<LeaveHistoryLogItem>, AppError> {
        Ok(self.history_logs.lock().unwrap().clone())
    }

    async fn persist_accrual(
        &self,
        _auth_token: &crate::diesel_pool::TenantAuthToken,
        _workspace_id: Uuid,
        _employment_id: Uuid,
        _assignment_id: Uuid,
        _leave_type_id: Uuid,
        _accrual_date: NaiveDate,
        _amount: f64,
        _force: bool,
    ) -> Result<Option<crate::models::LeaveAccrualLog>, AppError> {
        Ok(None)
    }
    // Unused
    async fn find_by_employment_date_type(
        &self,
        _e: Uuid,
        _l: Uuid,
        _a: NaiveDate,
        _w: Uuid,
    ) -> Result<Option<ExistingAccrualLog>, AppError> {
        unimplemented!()
    }
    async fn get_assignment_effective_from(
        &self,
        _a: Uuid,
        _w: Uuid,
    ) -> Result<NaiveDate, AppError> {
        unimplemented!()
    }
    async fn get_workspace_timezone(&self, _w: Uuid) -> Result<String, AppError> {
        unimplemented!()
    }
    async fn update_accrual_amount(
        &self,
        _l: Uuid,
        _w: Uuid,
        _n: Decimal,
    ) -> Result<crate::models::LeaveAccrualLog, AppError> {
        unimplemented!()
    }
    async fn insert_accrual_log(
        &self,
        _w: Uuid,
        _e: Uuid,
        _a: Option<Uuid>,
        _d: NaiveDate,
        _m: Decimal,
        _t: Uuid,
    ) -> Result<Option<crate::models::LeaveAccrualLog>, AppError> {
        unimplemented!()
    }
    async fn upsert_balance_accrual(
        &self,
        _ws: Uuid,
        _e: Uuid,
        _y: i32,
        _l: Uuid,
        _a: Decimal,
    ) -> Result<(), AppError> {
        unimplemented!()
    }
    async fn adjust_balance_accrual(
        &self,
        _ws: Uuid,
        _e: Uuid,
        _y: i32,
        _l: Uuid,
        _d: Decimal,
    ) -> Result<(), AppError> {
        unimplemented!()
    }
    async fn persist_accruals_batch(
        &self,
        _auth_token: &crate::diesel_pool::TenantAuthToken,
        _workspace_id: Uuid,
        _entries: &[crate::repositories::leave_accrual_log_repository::AccrualBatchEntry],
        _force: bool,
    ) -> Result<Vec<crate::models::LeaveAccrualLog>, AppError> {
        Ok(vec![])
    }
}

fn build_service(
    mock_emp: Option<Employee>,
    type_exists: bool,
) -> (
    LeaveService,
    Arc<MockLeaveBalanceRepo>,
    Arc<MockLeaveRequestRepo>,
    Arc<MockRosterRepo>,
    Arc<MockLeaveAccrualLogRepo>,
) {
    let bal_repo = Arc::new(MockLeaveBalanceRepo {
        workspace_balances: Mutex::new(vec![]),
        employee_balances: Mutex::new(vec![]),
        adjust_called: Mutex::new(false),
    });
    let req_repo = Arc::new(MockLeaveRequestRepo {
        exists: Mutex::new(type_exists),
        leave_request: Mutex::new(None),
        update_called: Mutex::new(false),
        history_requests: Mutex::new(vec![]),
    });
    use crate::repositories::employee::MockEmployeeRepository;
    let mut mock_emp_repo = MockEmployeeRepository::new();
    let has_emp = mock_emp.is_some();
    if let Some(emp) = mock_emp {
        mock_emp_repo
            .expect_get_employee_by_id()
            .returning(move |_, _| Ok(Some(emp.clone())));
    } else {
        mock_emp_repo
            .expect_get_employee_by_id()
            .returning(|_, _| Ok(None));
    }
    let emp_repo = Arc::new(mock_emp_repo);
    let ros_repo = Arc::new(MockRosterRepo {
        upserts: Mutex::new(vec![]),
    });
    let log_repo = Arc::new(MockLeaveAccrualLogRepo {
        history_logs: Mutex::new(vec![]),
    });

    let mut mock_employment_repo =
        crate::repositories::employment_repository::MockEmploymentRepository::new();
    mock_employment_repo
        .expect_list_employee_employments()
        .returning(|_, employee_id| {
            Ok(vec![crate::models::employment::Employment::new(
                Uuid::new_v4(),
                employee_id,
                Uuid::new_v4(),
                Uuid::new_v4(),
                None,
                None,
                "active".to_string(),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                None,
                None,
                "full_time".to_string(),
                None,
                chrono::Utc::now(),
                chrono::Utc::now(),
                None,
                None,
            )])
        });
    if has_emp {
        mock_employment_repo
            .expect_get_employment_by_id()
            .returning(|_, _| {
                Ok(Some(crate::models::employment::Employment::new(
                    Uuid::new_v4(),
                    Uuid::new_v4(),
                    Uuid::new_v4(),
                    Uuid::new_v4(),
                    None,
                    None,
                    "active".to_string(),
                    NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
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
    } else {
        mock_employment_repo
            .expect_get_employment_by_id()
            .returning(|_, _| Ok(None));
    }

    let mut mock_workspace_config_repo =
        crate::repositories::workspace_config_repository::MockWorkspaceConfigRepository::new();
    mock_workspace_config_repo
        .expect_get_effective_config()
        .returning(|_, _, _, _, _, _| Box::pin(async move { Ok(None) }));

    let mock_user_repo = crate::repositories::user_repository::MockUserRepository::new();
    let mock_workspace_repo =
        crate::repositories::workspace_repository::MockWorkspaceRepository::new();

    (
        LeaveService::new(
            bal_repo.clone(),
            req_repo.clone(),
            emp_repo,
            Arc::new(mock_employment_repo),
            ros_repo.clone(),
            log_repo.clone(),
            Arc::new(mock_workspace_config_repo),
            Arc::new(mock_user_repo),
            Arc::new(mock_workspace_repo),
            None,
        ),
        bal_repo,
        req_repo,
        ros_repo,
        log_repo,
    )
}

fn dummy_employee() -> Employee {
    Employee::new(
        Uuid::new_v4(),
        Uuid::new_v4(),
        None,
        "Test".into(),
        "EMP001".into(),
        vec![],
        Utc::now(),
        Utc::now(),
        None,
    )
}

#[tokio::test]
async fn test_get_workspace_balances() {
    let (svc, _, _, _, _) = build_service(None, true);
    let res = svc
        .get_workspace_balances(
            &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
            2024,
            crate::models::PaginationParams {
                page: Some(1),
                limit: Some(10),
                q: None,
                year: None,
                sort_by: None,
                sort_dir: None,
                department_id: None,
                permission: None,
                custom_field_filters: None,
                status: None,
            },
        )
        .await
        .unwrap();
    assert_eq!(res.total, 0);
}

#[tokio::test]
async fn test_get_employee_leave_history_empty() {
    let (svc, _, _, _, _) = build_service(Some(dummy_employee()), true);
    let history = svc
        .get_employee_leave_history(
            &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
            Uuid::new_v4(),
            2024,
            None,
        )
        .await
        .unwrap();
    assert!(history.is_empty());
}

#[tokio::test]
async fn test_adjust_balance_success() {
    let (svc, bal_repo, _, _, _) = build_service(Some(dummy_employee()), true);
    let req = crate::models::AdjustBalanceRequest {
        leave_type_id: Uuid::new_v4(),
        amount: 10.5,
        transaction_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        reason: "Test".into(),
    };
    svc.adjust_balance(
        &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
        Uuid::new_v4(),
        req,
    )
    .await
    .unwrap();
    assert!(*bal_repo.adjust_called.lock().unwrap());
}

#[tokio::test]
async fn test_get_employee_balances_not_found() {
    let (svc, _, _, _, _) = build_service(None, true); // No employee mock
    let err = svc
        .get_employee_balances(
            &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
            Uuid::new_v4(),
            2024,
        )
        .await
        .unwrap_err();
    assert!(matches!(err, AppError::NotFound(_)));
}

#[tokio::test]
async fn test_get_employee_balances_success() {
    let (svc, _, _, _, _) = build_service(Some(dummy_employee()), true);
    let res = svc
        .get_employee_balances(
            &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
            Uuid::new_v4(),
            2024,
        )
        .await
        .unwrap();
    assert!(res.is_empty());
}

#[tokio::test]
async fn test_submit_leave_request_employee_not_found() {
    let (svc, _, _, _, _) = build_service(None, true);
    let req = crate::models::SubmitLeaveRequest {
        employment_id: Uuid::new_v4(),
        leave_type_id: Uuid::new_v4(),
        start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        end_date: NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
        duration_type: LeaveDurationType::FullDay,
        start_time: None,
        end_time: None,
        reason: None,
        attachment_file_ids: None,
    };
    let err = svc
        .submit_leave_request(
            &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
            Uuid::new_v4(),
            req,
            false,
        )
        .await
        .unwrap_err();
    assert!(matches!(err, AppError::NotFound(_)));
}

#[tokio::test]
async fn test_submit_leave_request_invalid_leave_type() {
    let (svc, _, _, _, _) = build_service(Some(dummy_employee()), false); // leave_type exists = false
    let req = crate::models::SubmitLeaveRequest {
        employment_id: Uuid::new_v4(),
        leave_type_id: Uuid::new_v4(),
        start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        end_date: NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
        duration_type: LeaveDurationType::FullDay,
        start_time: None,
        end_time: None,
        reason: None,
        attachment_file_ids: None,
    };
    let err = svc
        .submit_leave_request(
            &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
            Uuid::new_v4(),
            req,
            false,
        )
        .await
        .unwrap_err();
    assert!(matches!(err, AppError::Validation(_)));
}

#[tokio::test]
async fn test_submit_leave_request_success() {
    let (svc, _, _, _, _) = build_service(Some(dummy_employee()), true);
    let req = crate::models::SubmitLeaveRequest {
        employment_id: Uuid::new_v4(),
        leave_type_id: Uuid::new_v4(),
        start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        end_date: NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
        duration_type: LeaveDurationType::FullDay,
        start_time: None,
        end_time: None,
        reason: None,
        attachment_file_ids: None,
    };
    let res = svc
        .submit_leave_request(
            &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
            Uuid::new_v4(),
            req,
            true,
        )
        .await
        .unwrap();
    assert_eq!(res.status(), LeaveStatus::PreApproved);
}
#[tokio::test]
async fn test_submit_leave_request_half_day_validation() {
    let (svc, _, _, _, _) = build_service(Some(dummy_employee()), true);
    let req = crate::models::SubmitLeaveRequest {
        employment_id: Uuid::new_v4(),
        leave_type_id: Uuid::new_v4(),
        start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        end_date: NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
        duration_type: LeaveDurationType::FirstHalf,
        start_time: None,
        end_time: None,
        reason: None,
        attachment_file_ids: None,
    };
    let err = svc
        .submit_leave_request(
            &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
            Uuid::new_v4(),
            req,
            false,
        )
        .await
        .unwrap_err();
    assert!(matches!(err, AppError::Validation(_)));
}

#[tokio::test]
async fn test_submit_leave_request_custom_hours_validation() {
    let (svc, _, _, _, _) = build_service(Some(dummy_employee()), true);

    // Test missing time
    let mut req = crate::models::SubmitLeaveRequest {
        employment_id: Uuid::new_v4(),
        leave_type_id: Uuid::new_v4(),
        start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        end_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        duration_type: LeaveDurationType::CustomHours,
        start_time: None,
        end_time: None,
        reason: None,
        attachment_file_ids: None,
    };
    let err = svc
        .submit_leave_request(
            &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
            Uuid::new_v4(),
            req.clone(),
            false,
        )
        .await
        .unwrap_err();
    assert!(
        matches!(err, AppError::Validation(ref msg) if msg.to_string() == "error.leave_custom_hours_missing_time")
    );

    // Test cross-day
    req.start_time = NaiveTime::from_hms_opt(9, 0, 0);
    req.end_time = NaiveTime::from_hms_opt(10, 0, 0);
    req.end_date = NaiveDate::from_ymd_opt(2024, 1, 2).unwrap();
    let err = svc
        .submit_leave_request(
            &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
            Uuid::new_v4(),
            req.clone(),
            false,
        )
        .await
        .unwrap_err();
    assert!(
        matches!(err, AppError::Validation(ref msg) if msg.to_string() == "error.leave_custom_hours_single_day")
    );

    // Test end before start
    req.end_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    req.start_time = NaiveTime::from_hms_opt(10, 0, 0);
    req.end_time = NaiveTime::from_hms_opt(9, 0, 0);
    let err = svc
        .submit_leave_request(
            &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
            Uuid::new_v4(),
            req,
            false,
        )
        .await
        .unwrap_err();
    assert!(
        matches!(err, AppError::Validation(ref msg) if msg.to_string() == "error.leave_start_time_before_end_time")
    );
}

#[tokio::test]
async fn test_approve_leave_request_not_found() {
    let (svc, _, req_repo, _, _) = build_service(Some(dummy_employee()), true);
    *req_repo.leave_request.lock().unwrap() = None;
    let req = crate::models::ApproveLeaveRequest {
        status: LeaveStatus::Approved,
    };
    let err = svc
        .approve_leave_request(
            &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
            Uuid::new_v4(),
            Uuid::new_v4(),
            req,
        )
        .await
        .unwrap_err();
    assert!(matches!(err, AppError::NotFound(_)));
}

#[tokio::test]
async fn test_approve_leave_request_forbidden() {
    // Employee is NOT found in the workspace employee repo
    let (svc, _, req_repo, _, _) = build_service(None, true);

    let mock_req = LeaveRequest::new(NewLeaveRequestParams {
            id: Uuid::new_v4(),
            employment_id: Uuid::new_v4(),
            leave_type_id: Uuid::new_v4(),
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            reason: None,
            status: LeaveStatus::Pending,
            duration_type: LeaveDurationType::FullDay,
            start_time: None,
            end_time: None,
            requested_by_user_id: None,
            approved_by_user_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            attachment_file_ids: None,
        });
    *req_repo.leave_request.lock().unwrap() = Some(mock_req);

    let req = crate::models::ApproveLeaveRequest {
        status: LeaveStatus::Approved,
    };
    let err = svc
        .approve_leave_request(
            &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
            Uuid::new_v4(),
            Uuid::new_v4(),
            req,
        )
        .await
        .unwrap_err();
    assert!(matches!(err, AppError::Forbidden(_)));
}

#[tokio::test]
async fn test_approve_leave_request_success() {
    let (svc, _, req_repo, _, _) = build_service(Some(dummy_employee()), true);
    let mock_req = LeaveRequest::new(NewLeaveRequestParams {
            id: Uuid::new_v4(),
            employment_id: Uuid::new_v4(),
            leave_type_id: Uuid::new_v4(),
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            reason: None,
            status: LeaveStatus::Pending,
            duration_type: LeaveDurationType::FullDay,
            start_time: None,
            end_time: None,
            requested_by_user_id: None,
            approved_by_user_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            attachment_file_ids: None,
        });
    *req_repo.leave_request.lock().unwrap() = Some(mock_req);
    let req = crate::models::ApproveLeaveRequest {
        status: LeaveStatus::Approved,
    };
    svc.approve_leave_request(
        &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
        Uuid::new_v4(),
        Uuid::new_v4(),
        req,
    )
    .await
    .unwrap();
    assert!(*req_repo.update_called.lock().unwrap());
}

#[tokio::test]
async fn test_get_leave_history() {
    let (svc, _, _, _, _) = build_service(None, true);
    let res = svc
        .get_employee_leave_history(
            &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
            Uuid::new_v4(),
            2024,
            None,
        )
        .await
        .unwrap();
    assert_eq!(res.len(), 0);
}

#[tokio::test]
async fn test_get_employee_leave_requests() {
    let (svc, _, _, _, _) = build_service(Some(dummy_employee()), true);
    let result = svc
        .get_employee_leave_requests(
            &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
            Uuid::new_v4(),
        )
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_get_workspace_leave_requests() {
    let (svc, _, _, _, _) = build_service(None, true);
    let result = svc
        .get_workspace_leave_requests(
            &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
            None,
        )
        .await;
    assert!(result.is_ok());
}
#[tokio::test]
async fn test_get_employee_leave_history_with_data() {
    let (svc, _, req_repo, _, log_repo) = build_service(None, true);

    let req_item = LeaveHistoryRequestItem {
        id: Uuid::new_v4(),
        start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), // Monday
        end_date: NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),   // Wednesday (3 days)
        reason: Some("Vacation".into()),
        created_at: Some(Utc::now()),
        leave_type_name: "Vacation".into(),
    };
    req_repo.history_requests.lock().unwrap().push(req_item);

    let log_item = LeaveHistoryLogItem {
        id: Uuid::new_v4(),
        accrual_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        amount: Decimal::new(10, 0), // 10
        notes: Some("Accrual".into()),
        policy_assignment_id: Some(Uuid::new_v4()),
        created_at: Some(Utc::now()),
        leave_type_name: "Vacation".into(),
    };
    log_repo.history_logs.lock().unwrap().push(log_item);

    let log_item_adj = LeaveHistoryLogItem {
        id: Uuid::new_v4(),
        accrual_date: NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
        amount: Decimal::new(-2, 0), // -2
        notes: Some("Adjustment".into()),
        policy_assignment_id: None,
        created_at: Some(Utc::now()),
        leave_type_name: "Vacation".into(),
    };
    log_repo.history_logs.lock().unwrap().push(log_item_adj);

    let res = svc
        .get_employee_leave_history(
            &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
            Uuid::new_v4(),
            2024,
            None,
        )
        .await
        .unwrap();

    assert_eq!(res.len(), 3);

    let usage = res
        .iter()
        .find(|r| r.transaction_type == crate::models::LeaveTransactionType::Usage)
        .unwrap();
    assert_eq!(usage.amount, -3.0);

    let accrual = res
        .iter()
        .find(|r| r.transaction_type == crate::models::LeaveTransactionType::Accrual)
        .unwrap();
    assert_eq!(accrual.amount, 10.0);

    let adjust = res
        .iter()
        .find(|r| r.transaction_type == crate::models::LeaveTransactionType::Adjustment)
        .unwrap();
    assert_eq!(adjust.amount, -2.0);
}

#[tokio::test]
async fn test_submit_leave_request_auto_approve_multi_day() {
    let (svc, _, _, ros_repo, _) = build_service(Some(dummy_employee()), true);

    let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let end_date = NaiveDate::from_ymd_opt(2024, 1, 3).unwrap();

    let req = crate::models::SubmitLeaveRequest {
        employment_id: Uuid::new_v4(),
        leave_type_id: Uuid::new_v4(),
        start_date,
        end_date,
        duration_type: LeaveDurationType::FullDay,
        start_time: None,
        end_time: None,
        reason: None,
        attachment_file_ids: None,
    };

    let res = svc
        .submit_leave_request(
            &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
            Uuid::new_v4(),
            req,
            true,
        )
        .await
        .unwrap();

    assert_eq!(res.status(), LeaveStatus::PreApproved);

    let upserts = ros_repo.upserts.lock().unwrap();
    assert!(
        upserts.is_empty(),
        "Option B should not write roster overrides"
    );
    drop(upserts);
}

#[tokio::test]
async fn test_submit_leave_request_auto_approve_custom_hours() {
    let (svc, _, _, ros_repo, _) = build_service(Some(dummy_employee()), true);

    let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let end_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

    let req = crate::models::SubmitLeaveRequest {
        employment_id: Uuid::new_v4(),
        leave_type_id: Uuid::new_v4(),
        start_date,
        end_date,
        duration_type: LeaveDurationType::CustomHours,
        start_time: Some(NaiveTime::from_hms_opt(10, 0, 0).unwrap()),
        end_time: Some(NaiveTime::from_hms_opt(14, 0, 0).unwrap()),
        reason: None,
        attachment_file_ids: None,
    };

    let res = svc
        .submit_leave_request(
            &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
            Uuid::new_v4(),
            req,
            true,
        )
        .await
        .unwrap();

    assert_eq!(res.status(), LeaveStatus::PreApproved);

    let upserts = ros_repo.upserts.lock().unwrap();
    assert!(
        upserts.is_empty(),
        "Option B should not write roster overrides"
    );
    drop(upserts);
}

#[tokio::test]
async fn test_approve_leave_request_first_half() {
    let (svc, _, req_repo, ros_repo, _) = build_service(Some(dummy_employee()), true);

    let mock_req = LeaveRequest::new(NewLeaveRequestParams {
            id: Uuid::new_v4(),
            employment_id: Uuid::new_v4(),
            leave_type_id: Uuid::new_v4(),
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            reason: None,
            status: LeaveStatus::Pending,
            duration_type: LeaveDurationType::FirstHalf,
            start_time: None,
            end_time: None,
            requested_by_user_id: None,
            approved_by_user_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            attachment_file_ids: None,
        });
    *req_repo.leave_request.lock().unwrap() = Some(mock_req);

    let req = crate::models::ApproveLeaveRequest {
        status: LeaveStatus::Approved,
    };
    svc.approve_leave_request(
        &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
        Uuid::new_v4(),
        Uuid::new_v4(),
        req,
    )
    .await
    .unwrap();

    assert!(*req_repo.update_called.lock().unwrap());

    let upserts = ros_repo.upserts.lock().unwrap();
    assert!(
        upserts.is_empty(),
        "Option B should not write roster overrides"
    );
    drop(upserts);
}

#[tokio::test]
async fn test_approve_leave_request_second_half() {
    let (svc, _, req_repo, ros_repo, _) = build_service(Some(dummy_employee()), true);

    let mock_req = LeaveRequest::new(NewLeaveRequestParams {
            id: Uuid::new_v4(),
            employment_id: Uuid::new_v4(),
            leave_type_id: Uuid::new_v4(),
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            reason: None,
            status: LeaveStatus::Pending,
            duration_type: LeaveDurationType::SecondHalf,
            start_time: None,
            end_time: None,
            requested_by_user_id: None,
            approved_by_user_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            attachment_file_ids: None,
        });
    *req_repo.leave_request.lock().unwrap() = Some(mock_req);

    let req = crate::models::ApproveLeaveRequest {
        status: LeaveStatus::Approved,
    };
    svc.approve_leave_request(
        &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
        Uuid::new_v4(),
        Uuid::new_v4(),
        req,
    )
    .await
    .unwrap();

    assert!(*req_repo.update_called.lock().unwrap());

    let upserts = ros_repo.upserts.lock().unwrap();
    assert!(
        upserts.is_empty(),
        "Option B should not write roster overrides"
    );
    drop(upserts);
}
#[tokio::test]
async fn test_approve_leave_request_custom_hours() {
    let (svc, _, req_repo, ros_repo, _) = build_service(Some(dummy_employee()), true);

    let mock_req = LeaveRequest::new(NewLeaveRequestParams {
            id: Uuid::new_v4(),
            employment_id: Uuid::new_v4(),
            leave_type_id: Uuid::new_v4(),
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            reason: None,
            status: LeaveStatus::Pending,
            duration_type: LeaveDurationType::CustomHours,
            start_time: Some(NaiveTime::from_hms_opt(9, 0, 0).unwrap()),
            end_time: Some(NaiveTime::from_hms_opt(11, 0, 0).unwrap()),
            requested_by_user_id: None,
            approved_by_user_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            attachment_file_ids: None,
        });
    *req_repo.leave_request.lock().unwrap() = Some(mock_req);

    let req = crate::models::ApproveLeaveRequest {
        status: LeaveStatus::Approved,
    };
    svc.approve_leave_request(
        &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
        Uuid::new_v4(),
        Uuid::new_v4(),
        req,
    )
    .await
    .unwrap();

    assert!(*req_repo.update_called.lock().unwrap());

    let upserts = ros_repo.upserts.lock().unwrap();
    assert!(
        upserts.is_empty(),
        "Option B should not write roster overrides"
    );
    drop(upserts);
}

#[tokio::test]
async fn test_on_workflow_completed() {
    use crate::services::approval_workflow::WorkflowCallbackHandler;

    let (svc, _, req_repo, _, _) = build_service(Some(dummy_employee()), true);

    let mock_req = LeaveRequest::new(NewLeaveRequestParams {
            id: Uuid::new_v4(),
            employment_id: Uuid::new_v4(),
            leave_type_id: Uuid::new_v4(),
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            reason: None,
            status: LeaveStatus::Pending,
            duration_type: LeaveDurationType::FullDay,
            start_time: None,
            end_time: None,
            requested_by_user_id: None,
            approved_by_user_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            attachment_file_ids: None,
        });
    let request_id = mock_req.id();
    *req_repo.leave_request.lock().unwrap() = Some(mock_req);

    let ws_id = Uuid::new_v4();
    let auth_token = crate::diesel_pool::TenantAuthToken::system_override(ws_id);

    svc.on_workflow_completed(&auth_token, request_id, "approved", Uuid::new_v4())
        .await
        .unwrap();

    assert!(*req_repo.update_called.lock().unwrap());
}

#[tokio::test]
async fn test_leave_conflict_auto_cancel() -> Result<(), Box<dyn std::error::Error>> {
    let employee_id = Uuid::new_v4();
    let employment_id = Uuid::new_v4();
    let workspace_id = Uuid::new_v4();
    let leave_type_id = Uuid::new_v4();

    // 1. Mock employment repo
    let emp_repo = crate::repositories::employee::MockEmployeeRepository::new();
    let mut employment_repo =
        crate::repositories::employment_repository::MockEmploymentRepository::new();
    let dummy_employment = crate::models::employment::Employment::new(
        employment_id,
        workspace_id,
        employee_id,
        Uuid::new_v4(),
        None,
        None,
        "active".to_string(),
        NaiveDate::from_ymd_opt(2024, 1, 1).ok_or("date")?,
        None,
        None,
        "full_time".to_string(),
        None,
        Utc::now(),
        Utc::now(),
        None,
        None,
    );
    employment_repo
        .expect_get_employment_by_id()
        .returning(move |_, _| Ok(Some(dummy_employment.clone())));

    // 2. Mock leave request repo
    let mut req_repo =
        crate::repositories::leave_request_repository::MockLeaveRequestRepository::new();

    // Incoming request
    let incoming_req = LeaveRequest::new(NewLeaveRequestParams {
            id: Uuid::new_v4(),
            employment_id: employment_id,
            leave_type_id: leave_type_id,
            start_date: NaiveDate::from_ymd_opt(2024, 6, 10).ok_or("date")?,
            end_date: NaiveDate::from_ymd_opt(2024, 6, 12).ok_or("date")?,
            reason: None,
            status: LeaveStatus::Approved,
            duration_type: LeaveDurationType::FullDay,
            start_time: None,
            end_time: None,
            requested_by_user_id: Some(Uuid::new_v4()),
            approved_by_user_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            attachment_file_ids: None,
        });

    // Conflicting request
    let conflicting_req = LeaveRequest::new(NewLeaveRequestParams {
            id: Uuid::new_v4(),
            employment_id: employment_id,
            leave_type_id: leave_type_id,
            start_date: NaiveDate::from_ymd_opt(2024, 6, 11).ok_or("date")?,
            end_date: NaiveDate::from_ymd_opt(2024, 6, 11).ok_or("date")?,
            reason: None,
            status: LeaveStatus::Approved,
            duration_type: LeaveDurationType::FullDay,
            start_time: None,
            end_time: None,
            requested_by_user_id: None,
            approved_by_user_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            attachment_file_ids: None,
        });
    let conflicting_req_id = conflicting_req.id();

    // Mock requests query: return the conflicting request (and self, which is ignored)
    let other_requests = vec![incoming_req.clone(), conflicting_req.clone()];
    let other_requests_clone = other_requests.clone();
    req_repo
        .expect_get_employee_leave_requests()
        .returning(move |_, _| {
            let val = other_requests_clone.clone();
            Box::pin(async move { Ok(val) })
        });

    // Expect bulk cancel call with the conflicting request ID
    req_repo
        .expect_bulk_cancel_leave_requests()
        .withf(move |_, ids| ids.len() == 1 && ids[0] == conflicting_req_id)
        .times(1)
        .returning(|_, _| Box::pin(async move { Ok(1) }));

    // 3. Mock roster repo: returns no roster conflicts
    let mut ros_repo = crate::repositories::roster::MockRosterRepository::new();
    ros_repo
        .expect_get_employment_rosters()
        .returning(|_, _, _, _, _, _| Box::pin(async move { Ok((vec![], 0)) }));

    // 4. Mock config repo: returns "auto_cancel" policy
    let mut config_repo =
        crate::repositories::workspace_config_repository::MockWorkspaceConfigRepository::new();
    let config_entry = crate::models::WorkspaceConfigEntry::new(NewWorkspaceConfigEntryParams {
        id: Uuid::new_v4(),
        workspace_id: workspace_id,
        scope_type: "workspace".to_string(),
        scope_id: None,
        config_key: "leave.conflict_policy".to_string(),
        value_int: None,
        value_decimal: None,
        value_text: Some("auto_cancel".to_string()),
        value_bool: None,
        value_json: None,
        effective_from: NaiveDate::from_ymd_opt(1970, 1, 1).ok_or("date")?,
        effective_to: None,
        created_by: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    });
    let config_entry_clone = config_entry.clone();
    config_repo
        .expect_get_effective_config()
        .returning(move |_, _, _, _, _, _| {
            let val = config_entry_clone.clone();
            Box::pin(async move { Ok(Some(val)) })
        });

    // 5. Unused mocks
    let bal_repo = crate::repositories::leave_balance_repository::MockLeaveBalanceRepository::new();
    let log_repo =
        crate::repositories::leave_accrual_log_repository::MockLeaveAccrualLogRepository::new();
    let user_repo = crate::repositories::user_repository::MockUserRepository::new();
    let workspace_repo = crate::repositories::workspace_repository::MockWorkspaceRepository::new();

    let svc = LeaveService::new(
        Arc::new(bal_repo),
        Arc::new(req_repo),
        Arc::new(emp_repo),
        Arc::new(employment_repo),
        Arc::new(ros_repo),
        Arc::new(log_repo),
        Arc::new(config_repo),
        Arc::new(user_repo),
        Arc::new(workspace_repo),
        None,
    );

    svc.check_and_handle_conflicts(
        &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
        &incoming_req,
    )
    .await?;

    Ok(())
}

#[tokio::test]
async fn test_leave_conflict_notify() -> Result<(), Box<dyn std::error::Error>> {
    let employee_id = Uuid::new_v4();
    let employment_id = Uuid::new_v4();
    let workspace_id = Uuid::new_v4();
    let leave_type_id = Uuid::new_v4();
    let requester_user_id = Uuid::new_v4();
    let admin_user_id = Uuid::new_v4();

    // 1. Mock employment repo
    let emp_repo = crate::repositories::employee::MockEmployeeRepository::new();
    let mut employment_repo =
        crate::repositories::employment_repository::MockEmploymentRepository::new();
    let dummy_employment = crate::models::employment::Employment::new(
        employment_id,
        workspace_id,
        employee_id,
        Uuid::new_v4(),
        None,
        None,
        "active".to_string(),
        NaiveDate::from_ymd_opt(2024, 1, 1).ok_or("date")?,
        None,
        None,
        "full_time".to_string(),
        None,
        Utc::now(),
        Utc::now(),
        None,
        None,
    );
    employment_repo
        .expect_get_employment_by_id()
        .returning(move |_, _| Ok(Some(dummy_employment.clone())));

    // 2. Mock leave request repo
    let mut req_repo =
        crate::repositories::leave_request_repository::MockLeaveRequestRepository::new();

    // Incoming request (Pending)
    let incoming_req = LeaveRequest::new(NewLeaveRequestParams {
            id: Uuid::new_v4(),
            employment_id: employment_id,
            leave_type_id: leave_type_id,
            start_date: NaiveDate::from_ymd_opt(2024, 6, 10).ok_or("date")?,
            end_date: NaiveDate::from_ymd_opt(2024, 6, 12).ok_or("date")?,
            reason: None,
            status: LeaveStatus::Pending,
            duration_type: LeaveDurationType::FullDay,
            start_time: None,
            end_time: None,
            requested_by_user_id: Some(requester_user_id),
            approved_by_user_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            attachment_file_ids: None,
        });

    // Conflicting request
    let conflicting_req = LeaveRequest::new(NewLeaveRequestParams {
            id: Uuid::new_v4(),
            employment_id: employment_id,
            leave_type_id: leave_type_id,
            start_date: NaiveDate::from_ymd_opt(2024, 6, 11).ok_or("date")?,
            end_date: NaiveDate::from_ymd_opt(2024, 6, 11).ok_or("date")?,
            reason: None,
            status: LeaveStatus::Approved,
            duration_type: LeaveDurationType::FullDay,
            start_time: None,
            end_time: None,
            requested_by_user_id: None,
            approved_by_user_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            attachment_file_ids: None,
        });

    // Mock requests query
    let other_requests = vec![incoming_req.clone(), conflicting_req.clone()];
    let other_requests_clone = other_requests.clone();
    req_repo
        .expect_get_employee_leave_requests()
        .returning(move |_, _| {
            let val = other_requests_clone.clone();
            Box::pin(async move { Ok(val) })
        });

    // Expect NO cancel calls because status is Pending and policy is notify
    req_repo.expect_cancel_leave_request().never();
    req_repo.expect_bulk_cancel_leave_requests().never();

    // 3. Mock roster repo: returns no roster conflicts
    let mut ros_repo = crate::repositories::roster::MockRosterRepository::new();
    ros_repo
        .expect_get_employment_rosters()
        .returning(|_, _, _, _, _, _| Box::pin(async move { Ok((vec![], 0)) }));

    // 4. Mock config repo: returns "notify" policy
    let mut config_repo =
        crate::repositories::workspace_config_repository::MockWorkspaceConfigRepository::new();
    let config_entry = crate::models::WorkspaceConfigEntry::new(NewWorkspaceConfigEntryParams {
        id: Uuid::new_v4(),
        workspace_id: workspace_id,
        scope_type: "workspace".to_string(),
        scope_id: None,
        config_key: "leave.conflict_policy".to_string(),
        value_int: None,
        value_decimal: None,
        value_text: Some("notify".to_string()),
        value_bool: None,
        value_json: None,
        effective_from: NaiveDate::from_ymd_opt(1970, 1, 1).ok_or("date")?,
        effective_to: None,
        created_by: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    });
    let config_entry_clone = config_entry.clone();
    config_repo
        .expect_get_effective_config()
        .returning(move |_, _, _, _, _, _| {
            let val = config_entry_clone.clone();
            Box::pin(async move { Ok(Some(val)) })
        });

    // 5. Mock workspace repo: return list of admins
    let mut workspace_repo =
        crate::repositories::workspace_repository::MockWorkspaceRepository::new();
    workspace_repo
        .expect_get_admins_with_permissions()
        .returning(move |_, _| Box::pin(async move { Ok(vec![admin_user_id]) }));

    // 6. Mock user repo: expects create_notifications_batch (now batched instead of N+1 per-user calls)
    let mut user_repo = crate::repositories::user_repository::MockUserRepository::new();
    user_repo
        .expect_create_notifications_batch()
        .times(1)
        .returning(|_| Ok(()));

    let bal_repo = crate::repositories::leave_balance_repository::MockLeaveBalanceRepository::new();
    let log_repo =
        crate::repositories::leave_accrual_log_repository::MockLeaveAccrualLogRepository::new();

    let svc = LeaveService::new(
        Arc::new(bal_repo),
        Arc::new(req_repo),
        Arc::new(emp_repo),
        Arc::new(employment_repo),
        Arc::new(ros_repo),
        Arc::new(log_repo),
        Arc::new(config_repo),
        Arc::new(user_repo),
        Arc::new(workspace_repo),
        None,
    );

    svc.check_and_handle_conflicts(
        &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
        &incoming_req,
    )
    .await?;

    Ok(())
}
