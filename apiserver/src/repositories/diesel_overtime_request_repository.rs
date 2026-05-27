use crate::errors::AppError;
use crate::models::{OvertimeRequest, OvertimeRequestEmployee};
use crate::repositories::OvertimeRequestRepository;
use crate::schema::{overtime_request_employees, overtime_requests};
use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

pub struct DieselOvertimeRequestRepository {
    pub diesel_pools: crate::diesel_pool::DieselPools,
}

impl DieselOvertimeRequestRepository {
    pub fn new(diesel_pools: crate::diesel_pool::DieselPools) -> Self {
        Self { diesel_pools }
    }
}

#[async_trait]
impl OvertimeRequestRepository for DieselOvertimeRequestRepository {
    async fn create_request(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        request: OvertimeRequest,
        employees: Vec<OvertimeRequestEmployee>,
    ) -> Result<OvertimeRequest, AppError> {
        let mut conn = self.diesel_pools.tenant_transaction_conn(auth_token).await?;

        let req: OvertimeRequest = diesel::insert_into(overtime_requests::table)
            .values(&request)
            .get_result(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to create overtime request: {}", e)))?;

        if !employees.is_empty() {
            diesel::insert_into(overtime_request_employees::table)
                .values(&employees)
                .execute(&mut *conn)
                .await
                .map_err(|e| AppError::Internal(format!("Failed to create overtime request employees: {}", e)))?;
        }

        Ok(req)
    }

    async fn update_request(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        request: OvertimeRequest,
    ) -> Result<OvertimeRequest, AppError> {
        let mut conn = self.diesel_pools.tenant_transaction_conn(auth_token).await?;

        let updated = diesel::update(
            overtime_requests::table
                .filter(overtime_requests::id.eq(request.id))
                .filter(overtime_requests::workspace_id.eq(request.workspace_id)),
        )
        .set((
            overtime_requests::status.eq(&request.status),
            overtime_requests::current_step.eq(request.current_step),
            overtime_requests::reconciled_at.eq(request.reconciled_at),
            overtime_requests::reconciled_by.eq(request.reconciled_by),
            overtime_requests::updated_at.eq(chrono::Utc::now()),
        ))
        .get_result(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to update overtime request: {}", e)))?;

        Ok(updated)
    }

    async fn get_request(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        workspace_id: Uuid,
        request_id: Uuid,
    ) -> Result<Option<OvertimeRequest>, AppError> {
        let mut conn = self.diesel_pools.tenant_transaction_conn(auth_token).await?;

        let req = overtime_requests::table
            .filter(overtime_requests::id.eq(request_id))
            .filter(overtime_requests::workspace_id.eq(workspace_id))
            .first::<OvertimeRequest>(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("Failed to get overtime request: {}", e)))?;

        Ok(req)
    }

    async fn get_request_employees(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        request_id: Uuid,
    ) -> Result<Vec<OvertimeRequestEmployee>, AppError> {
        let mut conn = self.diesel_pools.tenant_transaction_conn(auth_token).await?;

        let emps = overtime_request_employees::table
            .filter(overtime_request_employees::overtime_request_id.eq(request_id))
            .load::<OvertimeRequestEmployee>(&mut *conn)
            .await
            .map_err(|e| {
                AppError::Internal(format!("Failed to get overtime request employees: {}", e))
            })?;

        Ok(emps)
    }

    async fn update_employee_consent(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        request_id: Uuid,
        employee_id: Uuid,
        consent_status: &str,
        reason: Option<String>,
    ) -> Result<OvertimeRequestEmployee, AppError> {
        let mut conn = self.diesel_pools.tenant_transaction_conn(auth_token).await?;

        let updated = diesel::update(
            overtime_request_employees::table
                .filter(overtime_request_employees::overtime_request_id.eq(request_id))
                .filter(overtime_request_employees::employee_id.eq(employee_id)),
        )
        .set((
            overtime_request_employees::consent_status.eq(consent_status),
            overtime_request_employees::consent_at.eq(chrono::Utc::now()),
            overtime_request_employees::decline_reason.eq(reason),
        ))
        .get_result(&mut *conn)
        .await
        .map_err(|e| {
            AppError::Internal(format!("Failed to update employee consent: {}", e))
        })?;

        Ok(updated)
    }
}
