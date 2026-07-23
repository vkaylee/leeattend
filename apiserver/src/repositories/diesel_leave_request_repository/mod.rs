use crate::diesel_pool::DieselPools;
use crate::errors::AppError;
use crate::models::{LeaveRequest, LeaveStatus};
use crate::repositories::leave_request_repository::{
    LeaveBreakdown, LeaveHistoryRequestItem, LeaveRequestRepository,
};
use async_trait::async_trait;
use chrono::NaiveDate;
use uuid::Uuid;

mod approval;
mod crud;
mod queries;

#[cfg(test)]
mod tests_mod;

pub struct DieselLeaveRequestRepository {
    pub(super) diesel_pools: DieselPools,
}

impl DieselLeaveRequestRepository {
    pub fn new(diesel_pools: DieselPools) -> Self {
        Self { diesel_pools }
    }
}

// ── LeaveRequestRepository delegation ──────────────────────────────────
//
// Methods are implemented in sub-modules (crud, queries, approval) with
// an `_impl` suffix. This trait impl delegates to them.

#[async_trait]
impl LeaveRequestRepository for DieselLeaveRequestRepository {
    async fn create_leave_request(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        request: &LeaveRequest,
    ) -> Result<(), AppError> {
        self.create_leave_request_impl(auth_token, request).await
    }

    async fn create_leave_request_tx(
        &self,
        conn: &mut diesel_async::AsyncPgConnection,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        request: &LeaveRequest,
    ) -> Result<(), AppError> {
        self.create_leave_request_tx_impl(conn, auth_token, request).await
    }

    async fn get_leave_request_by_id(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
    ) -> Result<Option<LeaveRequest>, AppError> {
        self.get_leave_request_by_id_impl(auth_token, id).await
    }

    async fn update_leave_request_status(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
        status: LeaveStatus,
        approved_by: Option<Uuid>,
    ) -> Result<(), AppError> {
        self.update_leave_request_status_impl(auth_token, id, status, approved_by).await
    }

    async fn get_employee_leave_requests(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
    ) -> Result<Vec<LeaveRequest>, AppError> {
        self.get_employee_leave_requests_impl(auth_token, employee_id).await
    }

    async fn get_workspace_leave_requests(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        status: Option<LeaveStatus>,
    ) -> Result<Vec<LeaveRequest>, AppError> {
        self.get_workspace_leave_requests_impl(auth_token, status).await
    }

    async fn get_paginated_workspace_leave_requests(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        status: Option<LeaveStatus>,
        employee_id: Option<Uuid>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<LeaveRequest>, i64), AppError> {
        self.get_paginated_workspace_leave_requests_impl(auth_token, status, employee_id, limit, offset).await
    }

    async fn get_approved_leave_breakdown(
        &self,
        employee_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<LeaveBreakdown, AppError> {
        self.get_approved_leave_breakdown_impl(employee_id, start_date, end_date, auth_token).await
    }

    async fn get_employee_department_for_request(
        &self,
        request_id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Option<Uuid>, AppError> {
        self.get_employee_department_for_request_impl(request_id, auth_token).await
    }

    async fn get_leave_types(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<(String, String)>, AppError> {
        self.get_leave_types_impl(auth_token).await
    }

    async fn get_system_generated_holiday_leave(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<(Uuid, Uuid, String)>, AppError> {
        self.get_system_generated_holiday_leave_impl(auth_token, employee_id, date).await
    }

    async fn cancel_leave_request(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
    ) -> Result<(), AppError> {
        self.cancel_leave_request_impl(auth_token, id).await
    }

    async fn bulk_cancel_leave_requests(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        ids: &[Uuid],
    ) -> Result<u64, AppError> {
        self.bulk_cancel_leave_requests_impl(auth_token, ids).await
    }

    async fn bulk_cancel_leave_requests_tx(
        &self,
        conn: &mut diesel_async::AsyncPgConnection,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        ids: &[Uuid],
    ) -> Result<u64, AppError> {
        self.bulk_cancel_leave_requests_tx_impl(conn, auth_token, ids).await
    }

    async fn get_employee_leave_history_requests(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        year: i32,
        leave_type_id: Option<Uuid>,
    ) -> Result<Vec<LeaveHistoryRequestItem>, AppError> {
        self.get_employee_leave_history_requests_impl(auth_token, employee_id, year, leave_type_id).await
    }

    async fn check_leave_type_exists(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        leave_type_id: Uuid,
    ) -> Result<bool, AppError> {
        self.check_leave_type_exists_impl(auth_token, leave_type_id).await
    }

    async fn get_leave_type_name(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        leave_type_id: Uuid,
    ) -> Result<Option<String>, AppError> {
        self.get_leave_type_name_impl(auth_token, leave_type_id).await
    }

    async fn get_unpaid_holiday_days_without_leave(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<i64, AppError> {
        self.get_unpaid_holiday_days_without_leave_impl(auth_token, employee_id, start_date, end_date).await
    }

    async fn get_batch_leave_days(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_ids: &[Uuid],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<std::collections::HashMap<Uuid, i64>, AppError> {
        self.get_batch_leave_days_impl(auth_token, employee_ids, start_date, end_date).await
    }

    async fn get_approved_leave_dates(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_ids: &[Uuid],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<std::collections::HashSet<(Uuid, NaiveDate)>, AppError> {
        self.get_approved_leave_dates_impl(auth_token, employee_ids, start_date, end_date).await
    }

    async fn get_approved_leave_details(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_ids: &[Uuid],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<std::collections::HashMap<(Uuid, NaiveDate), bool>, AppError> {
        self.get_approved_leave_details_impl(auth_token, employee_ids, start_date, end_date).await
    }

    async fn get_approved_leave_periods(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<(NaiveDate, NaiveDate)>, AppError> {
        self.get_approved_leave_periods_impl(auth_token, employee_id, start_date, end_date).await
    }

    async fn get_approved_leave_breakdowns_batch(
        &self,
        employee_ids: &[Uuid],
        start_date: NaiveDate,
        end_date: NaiveDate,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<std::collections::HashMap<Uuid, LeaveBreakdown>, AppError> {
        self.get_approved_leave_breakdowns_batch_impl(employee_ids, start_date, end_date, auth_token).await
    }

    async fn get_unpaid_holiday_days_without_leave_batch(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_ids: &[Uuid],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<std::collections::HashMap<Uuid, i64>, AppError> {
        self.get_unpaid_holiday_days_without_leave_batch_impl(auth_token, employee_ids, start_date, end_date).await
    }

    async fn get_leave_requests_by_ids(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        ids: &[Uuid],
    ) -> Result<Vec<LeaveRequest>, AppError> {
        self.get_leave_requests_by_ids_impl(auth_token, ids).await
    }
}
