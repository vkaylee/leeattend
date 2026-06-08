//! Service for automatically deducting leave balances when holidays are unpaid.
//!
//! When a holiday has `is_paid = false`, this service checks each active employee's
//! leave balance and either:
//! - Creates an auto-approved leave request (if balance >= 1 day), or
//! - Marks the employee as unpaid for that day (if balance < 1 day).

use chrono::{Datelike, Utc};

use std::sync::Arc;
use uuid::Uuid;

use crate::models::{Holiday, LeaveDurationType, LeaveRequest, LeaveStatus, ShiftRoster};
use crate::repositories::leave_balance_repository::LeaveBalanceRepository;
use crate::repositories::leave_request_repository::LeaveRequestRepository;
use crate::repositories::roster::RosterRepository;

use crate::repositories::holiday_repository::HolidayLeaveResult;
use crate::repositories::{EmployeeRepository, EmploymentRepository};

pub struct HolidayLeaveService {
    employee_repo: Arc<dyn EmployeeRepository>,
    employment_repo: Arc<dyn EmploymentRepository>,
    leave_balance_repo: Arc<dyn LeaveBalanceRepository>,
    leave_request_repo: Arc<dyn LeaveRequestRepository>,
    #[allow(dead_code)]
    roster_repo: Arc<dyn RosterRepository>,
    holiday_repo: Arc<dyn crate::repositories::holiday_repository::HolidayRepository>,
    leave_policy_repo: Arc<dyn crate::repositories::leave_policy_repository::LeavePolicyRepository>,
}

impl HolidayLeaveService {
    pub fn new(
        employee_repo: Arc<dyn EmployeeRepository>,
        employment_repo: Arc<dyn EmploymentRepository>,
        leave_balance_repo: Arc<dyn LeaveBalanceRepository>,
        leave_request_repo: Arc<dyn LeaveRequestRepository>,
        roster_repo: Arc<dyn RosterRepository>,
        holiday_repo: Arc<dyn crate::repositories::holiday_repository::HolidayRepository>,
        leave_policy_repo: Arc<
            dyn crate::repositories::leave_policy_repository::LeavePolicyRepository,
        >,
    ) -> Self {
        Self {
            employee_repo,
            employment_repo,
            leave_balance_repo,
            leave_request_repo,
            roster_repo,
            holiday_repo,
            leave_policy_repo,
        }
    }

    /// Process leave deductions for all active employees on an unpaid holiday.
    ///
    /// If the holiday is paid (`is_paid = true`), returns a single `HolidayLeaveResult::Paid`.
    /// Otherwise, for each active employee:
    /// - Checks leave balance for the appropriate leave type
    /// - If balance >= 1.0: creates auto-approved leave request + audit log
    /// - If balance < 1.0: marks as unpaid
    #[tracing::instrument(skip_all, err)]
    #[allow(clippy::too_many_lines)]
    pub async fn process_holiday_leave(
        &self,
        workspace_id: Uuid,
        holiday: &Holiday,
    ) -> Result<Vec<HolidayLeaveResult>, crate::errors::AppError> {
        // Paid holidays need no deduction
        if holiday.is_paid() {
            return Ok(vec![HolidayLeaveResult::Paid]);
        }

        // Resolve which leave type to deduct from
        let leave_type_id = match holiday.leave_type_id() {
            Some(id) => id,
            None => {
                // Fallback: find first is_paid=true leave type in workspace
                self.leave_policy_repo
                    .find_default_paid_leave_type(workspace_id)
                    .await?
                    .ok_or_else(|| {
                        crate::errors::AppError::NotFound(
                            "No paid leave type found to deduct for unpaid holiday".to_string(),
                        )
                    })?
            }
        };

        // Get leave type name for audit trail
        let leave_type_name = self
            .leave_policy_repo
            .get_leave_type_name(leave_type_id, workspace_id)
            .await?;

        let auth_token = crate::diesel_pool::TenantAuthToken::system_override(workspace_id);

        // Get all active employees in workspace
        let employees = self
            .employee_repo
            .get_active_employees(&auth_token)
            .await
            .map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;

        if employees.is_empty() {
            return Ok(vec![]);
        }

        let employee_ids: Vec<Uuid> = employees.iter().map(crate::models::Employee::id).collect();
        let year = holiday.date().year();

        // Fetch active employments on holiday date
        let active_employments = self
            .employment_repo
            .get_active_employments_for_employees(&auth_token, &employee_ids, holiday.date())
            .await?;

        let mut emp_to_employment_map = std::collections::HashMap::new();
        for emp_record in active_employments {
            emp_to_employment_map.insert(emp_record.employee_id(), emp_record.id());
        }

        // 1. Batch fetch existing leaves for this date
        let existing_leaves = self
            .leave_request_repo
            .get_approved_leave_dates(&auth_token, &employee_ids, holiday.date(), holiday.date())
            .await?;

        // 2. Batch fetch balances
        let (workspace_balances, _) = self
            .leave_balance_repo
            .get_workspace_balances(&auth_token, year, 100_000, 0, None)
            .await?;

        let mut balance_map = std::collections::HashMap::new();
        for b in workspace_balances {
            balance_map.insert((b.employee_id, b.leave_type_id), b.remaining);
        }

        let mut results = Vec::new();
        let mut new_requests = Vec::new();
        let mut new_rosters = Vec::new();
        let mut new_logs = Vec::new();

        for emp in employees {
            // Check existing leave
            if existing_leaves.contains(&(emp.id(), holiday.date())) {
                continue;
            }

            // Get employment ID
            let Some(&employment_id) = emp_to_employment_map.get(&emp.id()) else {
                results.push(HolidayLeaveResult::Unpaid {
                    employee_id: emp.id(),
                });
                continue;
            };

            let remaining = balance_map
                .get(&(emp.id(), leave_type_id))
                .copied()
                .unwrap_or(0.0);

            if remaining >= 1.0 {
                let req_id = crate::utils::id::new_id();
                let reason = format!("Nghỉ lễ (trừ phép): {}", holiday.name());

                new_requests.push(LeaveRequest::new(
                    req_id,
                    employment_id,
                    leave_type_id,
                    holiday.date(),
                    holiday.date(),
                    Some(reason.clone()),
                    LeaveStatus::Approved,
                    LeaveDurationType::FullDay,
                    None,
                    None,
                    None,
                    None,
                    Utc::now(),
                    Utc::now(),
                ));

                new_rosters.push(ShiftRoster {
                    id: crate::utils::id::new_id(),
                    employment_id,
                    shift_id: None,
                    date: holiday.date(),
                    status: "on_leave".to_string(),
                    notes: Some(reason.clone()),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                });

                new_logs.push((
                    emp.id(),
                    leave_type_id,
                    rust_decimal::Decimal::ZERO,
                    holiday.date(),
                    format!("Auto-deducted: Holiday {}", holiday.name()),
                ));

                results.push(HolidayLeaveResult::LeaveDeducted {
                    employee_id: emp.id(),
                    leave_request_id: req_id,
                    leave_type_name: leave_type_name.clone(),
                });
            } else {
                results.push(HolidayLeaveResult::Unpaid {
                    employee_id: emp.id(),
                });
            }
        }

        // 3. Batch apply to DB in a single transaction
        if !new_requests.is_empty() || !new_rosters.is_empty() || !new_logs.is_empty() {
            self.holiday_repo
                .apply_holiday_leave_deductions(&auth_token, &new_requests, &new_rosters, &new_logs)
                .await?;
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_holiday_leave_result_serialization() {
        let paid = HolidayLeaveResult::Paid;
        let json = serde_json::to_string(&paid).unwrap();
        assert!(json.contains("Paid"));

        let deducted = HolidayLeaveResult::LeaveDeducted {
            employee_id: Uuid::new_v4(),
            leave_request_id: Uuid::new_v4(),
            leave_type_name: "Annual Leave".to_string(),
        };
        let json = serde_json::to_string(&deducted).unwrap();
        assert!(json.contains("LeaveDeducted"));
        assert!(json.contains("Annual Leave"));

        let unpaid = HolidayLeaveResult::Unpaid {
            employee_id: Uuid::new_v4(),
        };
        let json = serde_json::to_string(&unpaid).unwrap();
        assert!(json.contains("Unpaid"));
    }
}
