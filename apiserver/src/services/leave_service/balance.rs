//! Leave balance operations for [`LeaveService`].
//!
//! Handles workspace-level balance queries, employee-specific balance
//! lookups, leave history aggregation, and manual balance adjustments.

use crate::errors::AppError;
use crate::utils::i18n::I18nKey;
use chrono::Datelike;
use rust_decimal::prelude::ToPrimitive;
use uuid::Uuid;

use crate::models::{
    AdjustBalanceRequest, EmployeeLeaveBalance, LeaveHistoryItem, LeaveTransactionType,
    WorkspaceLeaveBalance,
};

impl super::LeaveService {
    #[tracing::instrument(skip_all, err)]
    pub async fn get_workspace_balances(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        year: i32,
        params: crate::models::PaginationParams,
    ) -> Result<crate::models::PaginatedResponse<WorkspaceLeaveBalance>, AppError> {
        let limit = params.limit.unwrap_or(20).max(1);
        let page = params.page.unwrap_or(1).max(1);
        let offset = (page - 1) * limit;

        let (data, total) = self
            .leave_balance_repo
            .get_workspace_balances(
                auth_token,
                year,
                limit,
                offset,
                params.q,
            )
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Error in get_workspace_balances");
                AppError::Internal("Internal error".into())
            })?;

        Ok(crate::models::PaginatedResponse {
            data,
            total: usize::try_from(total).unwrap_or(0),
            page,
            limit,
        })
    }

    #[tracing::instrument(skip_all, err)]
    #[allow(clippy::cast_precision_loss)]
    pub async fn get_employee_leave_history(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        year: i32,
        leave_type_id: Option<Uuid>,
    ) -> Result<Vec<LeaveHistoryItem>, AppError> {
        let employments = self
            .employment_repo
            .list_employee_employments(
                auth_token,
                employee_id,
            )
            .await?;

        // Batch-fetch all leave history logs to avoid N+1
        let employment_ids: Vec<Uuid> = employments
            .iter()
            .map(crate::models::employment::Employment::id)
            .collect();
        let logs = self
            .leave_accrual_log_repo
            .get_employment_leave_history_logs_batch(
                auth_token.workspace_id(),
                &employment_ids,
                year,
                leave_type_id,
            )
            .await?;

        let requests = self
            .leave_request_repo
            .get_employee_leave_history_requests(
                auth_token,
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
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        req: AdjustBalanceRequest,
    ) -> Result<(), AppError> {
        let amount_dec = rust_decimal::Decimal::from_f64_retain(req.amount)
            .ok_or_else(|| AppError::validation(I18nKey::Validation))?;

        self.leave_balance_repo
            .adjust_balance(
                auth_token,
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
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        year: i32,
    ) -> Result<Vec<EmployeeLeaveBalance>, AppError> {
        // Verify employee belongs to workspace
        let _ = self
            .employee_repo
            .get_employee_by_id(
                employee_id,
                auth_token,
            )
            .await?
            .ok_or_else(|| AppError::not_found(I18nKey::NotFound))?;

        self.leave_balance_repo
            .get_employee_balances(
                auth_token,
                employee_id,
                year,
            )
            .await
    }
}
