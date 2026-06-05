use std::sync::Arc;

use crate::errors::AppError;
use chrono::{Datelike, Utc};

use uuid::Uuid;

use crate::models::{
    AdjustBalanceRequest, ApproveLeaveRequest, EmployeeLeaveBalance, LeaveDurationType,
    LeaveHistoryItem, LeaveRequest, LeaveStatus, LeaveTransactionType, SubmitLeaveRequest,
    WorkspaceLeaveBalance,
};
use crate::repositories::leave_balance_repository::LeaveBalanceRepository;
use crate::repositories::leave_request_repository::LeaveRequestRepository;
use crate::repositories::roster::RosterRepository;
use crate::repositories::EmployeeRepository;
use rust_decimal::prelude::ToPrimitive;

pub struct LeaveService {
    leave_balance_repo: Arc<dyn LeaveBalanceRepository>,
    leave_request_repo: Arc<dyn LeaveRequestRepository>,
    employee_repo: Arc<dyn EmployeeRepository>,
    #[allow(dead_code)]
    roster_repo: Arc<dyn RosterRepository>,
    leave_accrual_log_repo:
        Arc<dyn crate::repositories::leave_accrual_log_repository::LeaveAccrualLogRepository>,
}

impl LeaveService {
    pub fn new(
        leave_balance_repo: Arc<dyn LeaveBalanceRepository>,
        leave_request_repo: Arc<dyn LeaveRequestRepository>,
        employee_repo: Arc<dyn EmployeeRepository>,
        roster_repo: Arc<dyn RosterRepository>,
        leave_accrual_log_repo: Arc<
            dyn crate::repositories::leave_accrual_log_repository::LeaveAccrualLogRepository,
        >,
    ) -> Self {
        Self {
            leave_balance_repo,
            leave_request_repo,
            employee_repo,
            roster_repo,
            leave_accrual_log_repo,
        }
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn get_workspace_balances(
        &self,
        workspace_id: Uuid,
        year: i32,
        params: crate::models::PaginationParams,
    ) -> Result<crate::models::PaginatedResponse<WorkspaceLeaveBalance>, AppError> {
        let limit = params.limit.unwrap_or(20).max(1);
        let page = params.page.unwrap_or(1).max(1);
        let offset = (page - 1) * limit;

        let (data, total) = self
            .leave_balance_repo
            .get_workspace_balances(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                year,
                limit,
                offset,
                params.q,
            )
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "SQLx Error in get_workspace_balances");
                AppError::Internal("Internal error".into())
            })?;

        Ok(crate::models::PaginatedResponse {
            data,
            total: total as usize,
            page,
            limit,
        })
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn get_employee_leave_history(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        year: i32,
        leave_type_id: Option<Uuid>,
    ) -> Result<Vec<LeaveHistoryItem>, AppError> {
        let logs = self
            .leave_accrual_log_repo
            .get_employee_leave_history_logs(workspace_id, employee_id, year, leave_type_id)
            .await?;

        let requests = self
            .leave_request_repo
            .get_employee_leave_history_requests(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                employee_id,
                year,
                leave_type_id,
            )
            .await?;

        let mut history: Vec<LeaveHistoryItem> = Vec::new();

        for log in logs {
            let transaction_type = if log.policy_assignment_id.is_some() {
                LeaveTransactionType::Accrual
            } else {
                LeaveTransactionType::Adjustment
            };
            history.push(LeaveHistoryItem {
                id: log.id,
                date: log.accrual_date,
                transaction_type,
                amount: ToPrimitive::to_f64(&log.amount).unwrap_or(0.0),
                reason: log.notes,
                reference_id: None,
                created_at: log.created_at.unwrap_or_else(chrono::Utc::now),
            });
        }

        for req in requests {
            // Count only weekdays (Mon-Fri) between start_date and end_date
            let mut weekday_count = 0i64;
            let mut d = req.start_date;
            while d <= req.end_date {
                let dow = d.weekday();
                if dow != chrono::Weekday::Sat && dow != chrono::Weekday::Sun {
                    weekday_count += 1;
                }
                if let Some(next) = d.succ_opt() {
                    d = next;
                } else {
                    break;
                }
            }

            // Adjust for partial day leaves
            let amount = if weekday_count > 0 {
                // For future: join duration_type into the history query
                // to support partial-day leave calculation here.
                -(weekday_count as f64)
            } else {
                0.0
            };

            history.push(LeaveHistoryItem {
                id: Uuid::new_v4(),
                date: req.start_date,
                transaction_type: LeaveTransactionType::Usage,
                amount,
                reason: req.reason,
                reference_id: Some(req.id),
                created_at: req.created_at.unwrap_or_else(chrono::Utc::now),
            });
        }

        // Sort by date desc
        history.sort_by(|a, b| b.date.cmp(&a.date));

        Ok(history)
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn adjust_balance(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        req: AdjustBalanceRequest,
    ) -> Result<(), AppError> {
        let amount_dec = rust_decimal::Decimal::from_f64_retain(req.amount)
            .ok_or(AppError::Validation("Invalid request".into()))?;

        self.leave_balance_repo
            .adjust_balance(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                employee_id,
                req.leave_type_id,
                amount_dec,
                req.transaction_date,
                &req.reason,
            )
            .await
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn get_employee_balances(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        year: i32,
    ) -> Result<Vec<EmployeeLeaveBalance>, AppError> {
        // Verify employee belongs to workspace
        let _ = self
            .employee_repo
            .get_employee_by_id(
                employee_id,
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
            )
            .await?
            .ok_or(AppError::NotFound("Not found".into()))?;

        self.leave_balance_repo
            .get_employee_balances(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                employee_id,
                year,
            )
            .await
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn submit_leave_request(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
        req: SubmitLeaveRequest,
        auto_approve: bool,
    ) -> Result<LeaveRequest, AppError> {
        // Verify employee belongs to workspace
        let _ = self
            .employee_repo
            .get_employee_by_id(
                req.employee_id,
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
            )
            .await?
            .ok_or(AppError::NotFound("Not found".into()))?;

        // Validate leave_type_id exists for this workspace
        if !self
            .leave_request_repo
            .check_leave_type_exists(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                req.leave_type_id,
            )
            .await?
        {
            return Err(AppError::Validation("Invalid request".into()));
        }

        // Validate partial day constraints
        match req.duration_type {
            LeaveDurationType::FirstHalf | LeaveDurationType::SecondHalf => {
                if req.start_date != req.end_date {
                    return Err(AppError::Validation(
                        "Half-day leave must be a single day (start_date == end_date)".into(),
                    ));
                }
            }
            LeaveDurationType::CustomHours => {
                if req.start_time.is_none() || req.end_time.is_none() {
                    return Err(AppError::Validation(
                        "Custom hours leave requires both start_time and end_time".into(),
                    ));
                }
                if req.start_date != req.end_date {
                    return Err(AppError::Validation(
                        "Custom hours leave must be a single day (start_date == end_date)".into(),
                    ));
                }
                if let (Some(st), Some(et)) = (req.start_time, req.end_time) {
                    if st >= et {
                        return Err(AppError::Validation(
                            "start_time must be before end_time".into(),
                        ));
                    }
                }
            }
            LeaveDurationType::FullDay => {}
        }

        let (status, approved_by) = if auto_approve {
            (LeaveStatus::PreApproved, Some(user_id))
        } else {
            (LeaveStatus::Pending, None)
        };

        let request = LeaveRequest {
            id: Uuid::new_v4(),
            employee_id: req.employee_id,
            leave_type_id: req.leave_type_id,
            start_date: req.start_date,
            end_date: req.end_date,
            reason: req.reason,
            status,
            duration_type: req.duration_type,
            start_time: req.start_time,
            end_time: req.end_time,
            requested_by_user_id: Some(user_id),
            approved_by_user_id: approved_by,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.leave_request_repo
            .create_leave_request(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                &request,
            )
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "SQLx Error in submit_leave_request");
                AppError::Internal("Internal error".into())
            })?;

        // If auto-approved, apply roster side effects immediately
        if auto_approve {
            self.apply_leave_to_roster(workspace_id, &request).await?;
        }

        Ok(request)
    }

    /// Apply leave-to-roster side effects (create roster entries with "on_leave" status).
    /// Shared between auto-approve (submit) and manual approve flows.
    #[allow(clippy::unused_async)]
    async fn apply_leave_to_roster(
        &self,
        _workspace_id: Uuid,
        _request: &LeaveRequest,
    ) -> Result<(), AppError> {
        // Option B: Giữ nguyên ca làm việc gốc và đối chiếu đơn phép khi chấm công.
        // Không tạo bản ghi đè shift_id = NULL trong bảng shift_rosters.
        Ok(())
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn approve_leave_request(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
        request_id: Uuid,
        req: ApproveLeaveRequest,
    ) -> Result<(), AppError> {
        let request = self
            .leave_request_repo
            .get_leave_request_by_id(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                request_id,
            )
            .await?
            .ok_or(AppError::NotFound("Not found".into()))?;

        // Verify employee of this request belongs to the workspace
        let _ = self
            .employee_repo
            .get_employee_by_id(
                request.employee_id,
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
            )
            .await?
            .ok_or(AppError::Forbidden("Forbidden".into()))?;

        self.leave_request_repo
            .update_leave_request_status(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                request_id,
                req.status,
                Some(user_id),
            )
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "SQLx Error in approve_leave_request");
                AppError::Internal("Internal error".into())
            })?;

        // If approved, update roster
        if req.status == LeaveStatus::Approved {
            self.apply_leave_to_roster(workspace_id, &request).await?;
        }

        Ok(())
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn get_employee_leave_requests(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
    ) -> Result<Vec<LeaveRequest>, AppError> {
        // Verify employee belongs to workspace
        let _ = self
            .employee_repo
            .get_employee_by_id(
                employee_id,
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
            )
            .await?
            .ok_or(AppError::NotFound("Not found".into()))?;

        self.leave_request_repo
            .get_employee_leave_requests(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                employee_id,
            )
            .await
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn get_workspace_leave_requests(
        &self,
        workspace_id: Uuid,
        status: Option<LeaveStatus>,
    ) -> Result<Vec<LeaveRequest>, AppError> {
        self.leave_request_repo
            .get_workspace_leave_requests(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                status,
            )
            .await
    }
}

#[async_trait::async_trait]
impl crate::services::approval_workflow::WorkflowCallbackHandler for LeaveService {
    async fn on_workflow_completed(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        entity_id: Uuid,
        status: &str,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        let workspace_id = auth_token.workspace_id();
        let leave_status = match status {
            "approved" => LeaveStatus::Approved,
            "rejected" => LeaveStatus::Rejected,
            _ => LeaveStatus::Pending,
        };
        self.approve_leave_request(
            workspace_id,
            user_id,
            entity_id,
            ApproveLeaveRequest {
                status: leave_status,
            },
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        Employee, EmployeeLeaveBalance, LeaveRequest, ShiftRoster, WorkspaceLeaveBalance,
    };
    use crate::repositories::leave_accrual_log_repository::{
        ExistingAccrualLog, LeaveHistoryLogItem,
    };
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
    }

    struct MockRosterRepo {
        upserts: Mutex<Vec<ShiftRoster>>,
    }
    #[async_trait]
    impl RosterRepository for MockRosterRepo {
        async fn get_employee_rosters_batch(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employee_ids: &[uuid::Uuid],
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
        ) -> Result<(Option<Uuid>, Vec<crate::models::ScheduleTrace>), AppError> {
            Ok((None, vec![]))
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
        ) -> Result<Vec<crate::models::EmployeeSchedule>, AppError> {
            unimplemented!()
        }
        async fn get_assignment_history(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _l: i64,
        ) -> Result<Vec<crate::models::AssignmentHistory>, AppError> {
            unimplemented!()
        }
        async fn get_employee_rosters(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _e: Uuid,
            _s: NaiveDate,
            _en: NaiveDate,
        ) -> Result<Vec<ShiftRoster>, AppError> {
            unimplemented!()
        }
    }

    struct MockLeaveAccrualLogRepo {
        history_logs: Mutex<Vec<LeaveHistoryLogItem>>,
    }
    #[async_trait]
    impl crate::repositories::leave_accrual_log_repository::LeaveAccrualLogRepository
        for MockLeaveAccrualLogRepo
    {
        async fn get_employee_leave_history_logs(
            &self,
            _w: Uuid,
            _e: Uuid,
            _y: i32,
            _t: Option<Uuid>,
        ) -> Result<Vec<LeaveHistoryLogItem>, AppError> {
            Ok(self.history_logs.lock().unwrap().clone())
        }

        async fn persist_accrual(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _workspace_id: Uuid,
            _employee_id: Uuid,
            _assignment_id: Uuid,
            _leave_type_id: Uuid,
            _accrual_date: NaiveDate,
            _amount: f64,
            _force: bool,
        ) -> Result<Option<crate::models::LeaveAccrualLog>, AppError> {
            Ok(None)
        }
        // Unused
        async fn find_by_employee_date_type(
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

        (
            LeaveService::new(
                bal_repo.clone(),
                req_repo.clone(),
                emp_repo,
                ros_repo.clone(),
                log_repo.clone(),
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
            NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            None,
            vec![],
            true,
            Utc::now(),
            Utc::now(),
            None,
            uuid::Uuid::new_v4(),
            None,
        )
    }

    #[tokio::test]
    async fn test_get_workspace_balances() {
        let (svc, _, _, _, _) = build_service(None, true);
        let res = svc
            .get_workspace_balances(
                Uuid::new_v4(),
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
            .get_employee_leave_history(Uuid::new_v4(), Uuid::new_v4(), 2024, None)
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
        svc.adjust_balance(Uuid::new_v4(), Uuid::new_v4(), req)
            .await
            .unwrap();
        assert!(*bal_repo.adjust_called.lock().unwrap());
    }

    #[tokio::test]
    async fn test_get_employee_balances_not_found() {
        let (svc, _, _, _, _) = build_service(None, true); // No employee mock
        let err = svc
            .get_employee_balances(Uuid::new_v4(), Uuid::new_v4(), 2024)
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn test_get_employee_balances_success() {
        let (svc, _, _, _, _) = build_service(Some(dummy_employee()), true);
        let res = svc
            .get_employee_balances(Uuid::new_v4(), Uuid::new_v4(), 2024)
            .await
            .unwrap();
        assert!(res.is_empty());
    }

    #[tokio::test]
    async fn test_submit_leave_request_employee_not_found() {
        let (svc, _, _, _, _) = build_service(None, true);
        let req = crate::models::SubmitLeaveRequest {
            employee_id: Uuid::new_v4(),
            leave_type_id: Uuid::new_v4(),
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
            duration_type: LeaveDurationType::FullDay,
            start_time: None,
            end_time: None,
            reason: None,
        };
        let err = svc
            .submit_leave_request(Uuid::new_v4(), Uuid::new_v4(), req, false)
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn test_submit_leave_request_invalid_leave_type() {
        let (svc, _, _, _, _) = build_service(Some(dummy_employee()), false); // leave_type exists = false
        let req = crate::models::SubmitLeaveRequest {
            employee_id: Uuid::new_v4(),
            leave_type_id: Uuid::new_v4(),
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
            duration_type: LeaveDurationType::FullDay,
            start_time: None,
            end_time: None,
            reason: None,
        };
        let err = svc
            .submit_leave_request(Uuid::new_v4(), Uuid::new_v4(), req, false)
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[tokio::test]
    async fn test_submit_leave_request_success() {
        let (svc, _, _, _, _) = build_service(Some(dummy_employee()), true);
        let req = crate::models::SubmitLeaveRequest {
            employee_id: Uuid::new_v4(),
            leave_type_id: Uuid::new_v4(),
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
            duration_type: LeaveDurationType::FullDay,
            start_time: None,
            end_time: None,
            reason: None,
        };
        let res = svc
            .submit_leave_request(Uuid::new_v4(), Uuid::new_v4(), req, true)
            .await
            .unwrap();
        assert_eq!(res.status, LeaveStatus::PreApproved);
    }
    #[tokio::test]
    async fn test_submit_leave_request_half_day_validation() {
        let (svc, _, _, _, _) = build_service(Some(dummy_employee()), true);
        let req = crate::models::SubmitLeaveRequest {
            employee_id: Uuid::new_v4(),
            leave_type_id: Uuid::new_v4(),
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
            duration_type: LeaveDurationType::FirstHalf,
            start_time: None,
            end_time: None,
            reason: None,
        };
        let err = svc
            .submit_leave_request(Uuid::new_v4(), Uuid::new_v4(), req, false)
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[tokio::test]
    async fn test_submit_leave_request_custom_hours_validation() {
        let (svc, _, _, _, _) = build_service(Some(dummy_employee()), true);

        // Test missing time
        let mut req = crate::models::SubmitLeaveRequest {
            employee_id: Uuid::new_v4(),
            leave_type_id: Uuid::new_v4(),
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            duration_type: LeaveDurationType::CustomHours,
            start_time: None,
            end_time: None,
            reason: None,
        };
        let err = svc
            .submit_leave_request(Uuid::new_v4(), Uuid::new_v4(), req.clone(), false)
            .await
            .unwrap_err();
        assert!(
            matches!(err, AppError::Validation(msg) if msg.contains("requires both start_time and end_time"))
        );

        // Test cross-day
        req.start_time = NaiveTime::from_hms_opt(9, 0, 0);
        req.end_time = NaiveTime::from_hms_opt(10, 0, 0);
        req.end_date = NaiveDate::from_ymd_opt(2024, 1, 2).unwrap();
        let err = svc
            .submit_leave_request(Uuid::new_v4(), Uuid::new_v4(), req.clone(), false)
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::Validation(msg) if msg.contains("must be a single day")));

        // Test end before start
        req.end_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        req.start_time = NaiveTime::from_hms_opt(10, 0, 0);
        req.end_time = NaiveTime::from_hms_opt(9, 0, 0);
        let err = svc
            .submit_leave_request(Uuid::new_v4(), Uuid::new_v4(), req, false)
            .await
            .unwrap_err();
        assert!(
            matches!(err, AppError::Validation(msg) if msg.contains("must be before end_time"))
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
            .approve_leave_request(Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4(), req)
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn test_approve_leave_request_forbidden() {
        // Employee is NOT found in the workspace employee repo
        let (svc, _, req_repo, _, _) = build_service(None, true);

        let mock_req = LeaveRequest {
            id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
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
        };
        *req_repo.leave_request.lock().unwrap() = Some(mock_req);

        let req = crate::models::ApproveLeaveRequest {
            status: LeaveStatus::Approved,
        };
        let err = svc
            .approve_leave_request(Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4(), req)
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::Forbidden(_)));
    }

    #[tokio::test]
    async fn test_approve_leave_request_success() {
        let (svc, _, req_repo, _, _) = build_service(Some(dummy_employee()), true);
        let mock_req = LeaveRequest {
            id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
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
        };
        *req_repo.leave_request.lock().unwrap() = Some(mock_req);
        let req = crate::models::ApproveLeaveRequest {
            status: LeaveStatus::Approved,
        };
        svc.approve_leave_request(Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4(), req)
            .await
            .unwrap();
        assert!(*req_repo.update_called.lock().unwrap());
    }

    #[tokio::test]
    async fn test_get_leave_history() {
        let (svc, _, _, _, _) = build_service(None, true);
        let res = svc
            .get_employee_leave_history(Uuid::new_v4(), Uuid::new_v4(), 2024, None)
            .await
            .unwrap();
        assert_eq!(res.len(), 0);
    }

    #[tokio::test]
    async fn test_get_employee_leave_requests() {
        let (svc, _, _, _, _) = build_service(Some(dummy_employee()), true);
        let result = svc
            .get_employee_leave_requests(Uuid::new_v4(), Uuid::new_v4())
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_workspace_leave_requests() {
        let (svc, _, _, _, _) = build_service(None, true);
        let result = svc.get_workspace_leave_requests(Uuid::new_v4(), None).await;
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
            .get_employee_leave_history(Uuid::new_v4(), Uuid::new_v4(), 2024, None)
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
            employee_id: Uuid::new_v4(),
            leave_type_id: Uuid::new_v4(),
            start_date,
            end_date,
            duration_type: LeaveDurationType::FullDay,
            start_time: None,
            end_time: None,
            reason: None,
        };

        let res = svc
            .submit_leave_request(Uuid::new_v4(), Uuid::new_v4(), req, true)
            .await
            .unwrap();

        assert_eq!(res.status, LeaveStatus::PreApproved);

        let upserts = ros_repo.upserts.lock().unwrap();
        assert!(
            upserts.is_empty(),
            "Option B should not write roster overrides"
        );
    }

    #[tokio::test]
    async fn test_submit_leave_request_auto_approve_custom_hours() {
        let (svc, _, _, ros_repo, _) = build_service(Some(dummy_employee()), true);

        let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

        let req = crate::models::SubmitLeaveRequest {
            employee_id: Uuid::new_v4(),
            leave_type_id: Uuid::new_v4(),
            start_date,
            end_date,
            duration_type: LeaveDurationType::CustomHours,
            start_time: Some(NaiveTime::from_hms_opt(10, 0, 0).unwrap()),
            end_time: Some(NaiveTime::from_hms_opt(14, 0, 0).unwrap()),
            reason: None,
        };

        let res = svc
            .submit_leave_request(Uuid::new_v4(), Uuid::new_v4(), req, true)
            .await
            .unwrap();

        assert_eq!(res.status, LeaveStatus::PreApproved);

        let upserts = ros_repo.upserts.lock().unwrap();
        assert!(
            upserts.is_empty(),
            "Option B should not write roster overrides"
        );
    }

    #[tokio::test]
    async fn test_approve_leave_request_first_half() {
        let (svc, _, req_repo, ros_repo, _) = build_service(Some(dummy_employee()), true);

        let mock_req = LeaveRequest {
            id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
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
        };
        *req_repo.leave_request.lock().unwrap() = Some(mock_req);

        let req = crate::models::ApproveLeaveRequest {
            status: LeaveStatus::Approved,
        };
        svc.approve_leave_request(Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4(), req)
            .await
            .unwrap();

        assert!(*req_repo.update_called.lock().unwrap());

        let upserts = ros_repo.upserts.lock().unwrap();
        assert!(
            upserts.is_empty(),
            "Option B should not write roster overrides"
        );
    }

    #[tokio::test]
    async fn test_approve_leave_request_second_half() {
        let (svc, _, req_repo, ros_repo, _) = build_service(Some(dummy_employee()), true);

        let mock_req = LeaveRequest {
            id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
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
        };
        *req_repo.leave_request.lock().unwrap() = Some(mock_req);

        let req = crate::models::ApproveLeaveRequest {
            status: LeaveStatus::Approved,
        };
        svc.approve_leave_request(Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4(), req)
            .await
            .unwrap();

        assert!(*req_repo.update_called.lock().unwrap());

        let upserts = ros_repo.upserts.lock().unwrap();
        assert!(
            upserts.is_empty(),
            "Option B should not write roster overrides"
        );
    }
    #[tokio::test]
    async fn test_approve_leave_request_custom_hours() {
        let (svc, _, req_repo, ros_repo, _) = build_service(Some(dummy_employee()), true);

        let mock_req = LeaveRequest {
            id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
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
        };
        *req_repo.leave_request.lock().unwrap() = Some(mock_req);

        let req = crate::models::ApproveLeaveRequest {
            status: LeaveStatus::Approved,
        };
        svc.approve_leave_request(Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4(), req)
            .await
            .unwrap();

        assert!(*req_repo.update_called.lock().unwrap());

        let upserts = ros_repo.upserts.lock().unwrap();
        assert!(
            upserts.is_empty(),
            "Option B should not write roster overrides"
        );
    }

    #[tokio::test]
    async fn test_on_workflow_completed() {
        use crate::services::approval_workflow::WorkflowCallbackHandler;

        let (svc, _, req_repo, _, _) = build_service(Some(dummy_employee()), true);

        let mock_req = LeaveRequest {
            id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
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
        };
        let request_id = mock_req.id;
        *req_repo.leave_request.lock().unwrap() = Some(mock_req);

        let ws_id = Uuid::new_v4();
        let auth_token = crate::diesel_pool::TenantAuthToken::system_override(ws_id);

        svc.on_workflow_completed(&auth_token, request_id, "approved", Uuid::new_v4())
            .await
            .unwrap();

        assert!(*req_repo.update_called.lock().unwrap());
    }
}
