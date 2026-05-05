//! Service for automatically deducting leave balances when holidays are unpaid.
//!
//! When a holiday has `is_paid = false`, this service checks each active employee's
//! leave balance and either:
//! - Creates an auto-approved leave request (if balance >= 1 day), or
//! - Marks the employee as unpaid for that day (if balance < 1 day).

use std::sync::Arc;

use chrono::{Datelike, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{Holiday, LeaveDurationType, LeaveRequest, LeaveStatus, ShiftRoster};
use crate::repositories::leave_balance_repository::LeaveBalanceRepository;
use crate::repositories::leave_request_repository::LeaveRequestRepository;
use crate::repositories::roster::RosterRepository;

use crate::repositories::holiday_repository::HolidayLeaveResult;
use crate::repositories::EmployeeRepository;

pub struct HolidayLeaveService {
    employee_repo: Arc<dyn EmployeeRepository>,
    leave_balance_repo: Arc<dyn LeaveBalanceRepository>,
    leave_request_repo: Arc<dyn LeaveRequestRepository>,
    roster_repo: Arc<dyn RosterRepository>,
    pool: PgPool,
}

impl HolidayLeaveService {
    pub fn new(
        employee_repo: Arc<dyn EmployeeRepository>,
        leave_balance_repo: Arc<dyn LeaveBalanceRepository>,
        leave_request_repo: Arc<dyn LeaveRequestRepository>,
        roster_repo: Arc<dyn RosterRepository>,
        pool: PgPool,
    ) -> Self {
        Self {
            employee_repo,
            leave_balance_repo,
            leave_request_repo,
            roster_repo,
            pool,
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
    pub async fn process_holiday_leave(
        &self,
        workspace_id: Uuid,
        holiday: &Holiday,
    ) -> Result<Vec<HolidayLeaveResult>, String> {
        let pool = &self.pool;
        // Paid holidays need no deduction
        if holiday.is_paid {
            return Ok(vec![HolidayLeaveResult::Paid]);
        }

        // Resolve which leave type to deduct from
        let leave_type_id = match holiday.leave_type_id {
            Some(id) => id,
            None => {
                // Fallback: find first is_paid=true leave type in workspace
                self.find_default_paid_leave_type(workspace_id)
                    .await?
                    .ok_or_else(|| {
                        "No paid leave type found in workspace for auto-deduction".to_string()
                    })?
            }
        };

        // Get leave type name for audit trail
        let leave_type_name = sqlx::query_scalar!(
            "SELECT name FROM leave_types WHERE id = $1 AND workspace_id = $2",
            leave_type_id,
            workspace_id
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?
        .unwrap_or_else(|| "Leave".to_string());

        // Get all active employees in workspace
        let employees = self
            .employee_repo
            .get_active_employees(workspace_id)
            .await
            .map_err(|e| e.to_string())?;

        let year = holiday.date.year();
        let mut results = Vec::new();

        for emp in employees {
            // Check if a leave request already exists for this employee+date (idempotency)
            let existing = self.has_existing_leave_for_date(emp.id, holiday.date).await;
            if existing {
                // Already processed — skip
                continue;
            }

            // Check leave balance
            let balances = self
                .leave_balance_repo
                .get_employee_balances(workspace_id, emp.id, year)
                .await
                .map_err(|e| e.to_string())?;

            let balance = balances.iter().find(|b| b.leave_type_id == leave_type_id);
            let remaining = balance.map(|b| b.remaining).unwrap_or(0.0);

            if remaining >= 1.0 {
                // Has sufficient leave → create auto-approved leave request
                let leave_request = LeaveRequest {
                    id: crate::utils::id::new_id(),
                    employee_id: emp.id,
                    leave_type_id,
                    start_date: holiday.date,
                    end_date: holiday.date,
                    reason: Some(format!("Nghỉ lễ (trừ phép): {}", holiday.name)),
                    status: LeaveStatus::Approved,
                    duration_type: LeaveDurationType::FullDay,
                    start_time: None,
                    end_time: None,
                    requested_by_user_id: None, // System-generated
                    approved_by_user_id: None,  // Auto-approved
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                };

                self.leave_request_repo
                    .create_leave_request(&leave_request)
                    .await
                    .map_err(|e| e.to_string())?;

                // Update roster to reflect leave
                let roster = ShiftRoster {
                    id: crate::utils::id::new_id(),
                    employee_id: emp.id,
                    shift_id: None,
                    date: holiday.date,
                    status: "on_leave".to_string(),
                    notes: Some(format!("Nghỉ lễ (trừ phép): {}", holiday.name)),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                };
                let _ = self.roster_repo.upsert_roster(&roster).await;

                // Audit log via leave_accrual_logs
                let _ = self
                    .leave_balance_repo
                    .adjust_balance(
                        workspace_id,
                        emp.id,
                        leave_type_id,
                        rust_decimal::Decimal::ZERO, // Balance change tracked via leave_requests, not here
                        holiday.date,
                        &format!("Auto-deducted: Holiday {}", holiday.name),
                    )
                    .await;

                results.push(HolidayLeaveResult::LeaveDeducted {
                    employee_id: emp.id,
                    leave_request_id: leave_request.id,
                    leave_type_name: leave_type_name.clone(),
                });
            } else {
                // No leave remaining → unpaid
                results.push(HolidayLeaveResult::Unpaid {
                    employee_id: emp.id,
                });
            }
        }

        Ok(results)
    }

    /// Find the first leave type with `is_paid = true` in the workspace.
    async fn find_default_paid_leave_type(
        &self,
        workspace_id: Uuid,
    ) -> Result<Option<Uuid>, String> {
        let pool = &self.pool;
        let result = sqlx::query_scalar!(
            "SELECT id FROM leave_types WHERE workspace_id = $1 AND is_paid = true ORDER BY name ASC LIMIT 1",
            workspace_id
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(result)
    }

    /// Check if employee already has an approved leave request covering this date.
    async fn has_existing_leave_for_date(
        &self,
        employee_id: Uuid,
        date: chrono::NaiveDate,
    ) -> bool {
        let pool = &self.pool;
        sqlx::query_scalar!(
            r#"SELECT EXISTS(
                SELECT 1 FROM leave_requests
                WHERE employee_id = $1
                  AND status IN ('approved', 'pre_approved')
                  AND start_date <= $2
                  AND end_date >= $2
            ) as "exists!""#,
            employee_id,
            date
        )
        .fetch_one(pool)
        .await
        .unwrap_or(false)
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
