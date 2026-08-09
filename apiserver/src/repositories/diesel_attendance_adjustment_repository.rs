use async_trait::async_trait;
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::diesel_pool::{DieselPoolsTenantTxExt, TenantAuthToken};
use crate::errors::AppError;
use crate::models::diesel::attendance_adjustment::{
    AttendanceAdjustmentEntity, NewAttendanceAdjustmentEntity, UpdateAttendanceAdjustmentEntity,
};
use crate::models::{AdjustmentStatus, AttendanceAdjustmentRequest};
use crate::repositories::attendance_adjustment_repository::AttendanceAdjustmentRepository;
use crate::schema::{attendance_adjustment_requests, employees, employments};

pub struct DieselAttendanceAdjustmentRepository {
    diesel_pools: crate::diesel_pool::DieselPools,
}

impl DieselAttendanceAdjustmentRepository {
    pub fn new(diesel_pools: crate::diesel_pool::DieselPools) -> Self {
        Self { diesel_pools }
    }
}

#[async_trait]
impl AttendanceAdjustmentRepository for DieselAttendanceAdjustmentRepository {
    async fn create_request(
        &self,
        workspace_id: Uuid,
        request: &AttendanceAdjustmentRequest,
    ) -> Result<(), AppError> {
        let token = TenantAuthToken::system_override(workspace_id);
        let new_entity = NewAttendanceAdjustmentEntity {
            id: request.id(),
            employment_id: request.employment_id(),
            time_entry_id: request.time_entry_id(),
            requested_punch_time: request.requested_punch_time(),
            reason: request.reason().map(ToString::to_string),
            status: request.status(),
            requested_by_user_id: request.requested_by_user_id(),
            created_at: Some(request.created_at()),
            updated_at: Some(request.updated_at()),
            attachment_file_ids: request.attachment_file_ids().map(<[Option<Uuid>]>::to_vec),
        };

        self.diesel_pools
            .with_tenant_tx(&token, |conn| {
                Box::pin(async move {
                    diesel::insert_into(attendance_adjustment_requests::table)
                        .values(&new_entity)
                        .execute(conn)
                        .await
                        .map_err(AppError::from)?;

                    Ok(())
                })
            })
            .await
    }

    async fn get_request_by_id(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<Option<AttendanceAdjustmentRequest>, AppError> {
        let token = TenantAuthToken::system_override(workspace_id);
        self.diesel_pools
            .with_tenant_tx(&token, |conn| {
                Box::pin(async move {
                    let entry: Option<AttendanceAdjustmentEntity> =
                        attendance_adjustment_requests::table
                            .inner_join(employments::table.on(
                                attendance_adjustment_requests::employment_id.eq(employments::id),
                            ))
                            .inner_join(
                                employees::table.on(employments::employee_id.eq(employees::id)),
                            )
                            .filter(employees::workspace_id.eq(workspace_id))
                            .filter(attendance_adjustment_requests::id.eq(id))
                            .select(attendance_adjustment_requests::all_columns)
                            .first(conn)
                            .await
                            .optional()
                            .map_err(AppError::from)?;

                    Ok(entry.map(Into::into))
                })
            })
            .await
    }

    async fn update_adjustment_status(
        &self,
        workspace_id: Uuid,
        id: Uuid,
        status: AdjustmentStatus,
        approved_by: Option<Uuid>,
    ) -> Result<(), AppError> {
        let token = TenantAuthToken::system_override(workspace_id);
        let update_model = UpdateAttendanceAdjustmentEntity {
            status: Some(status),
            approved_by_user_id: Some(approved_by),
            updated_at: Some(Utc::now()),
        };

        self.diesel_pools
            .with_tenant_tx(&token, |conn| {
                Box::pin(async move {
                    diesel::update(attendance_adjustment_requests::table)
                        .filter(attendance_adjustment_requests::id.eq(id))
                        .filter(
                            attendance_adjustment_requests::employment_id.eq_any(
                                employments::table
                                    .filter(employments::workspace_id.eq(workspace_id))
                                    .select(employments::id),
                            ),
                        )
                        .set(&update_model)
                        .execute(conn)
                        .await
                        .map_err(AppError::from)?;

                    Ok(())
                })
            })
            .await
    }

    async fn get_workspace_adjustment_requests(
        &self,
        workspace_id: Uuid,
        status: Option<AdjustmentStatus>,
    ) -> Result<Vec<AttendanceAdjustmentRequest>, AppError> {
        let token = TenantAuthToken::system_override(workspace_id);
        self.diesel_pools
            .with_tenant_tx(&token, |conn| {
                Box::pin(async move {
                    let mut query = attendance_adjustment_requests::table
                        .inner_join(
                            employments::table
                                .on(attendance_adjustment_requests::employment_id
                                    .eq(employments::id)),
                        )
                        .inner_join(employees::table.on(employments::employee_id.eq(employees::id)))
                        .filter(employees::workspace_id.eq(workspace_id))
                        .into_boxed();

                    if let Some(s) = status {
                        query = query.filter(attendance_adjustment_requests::status.eq(s));
                    }

                    let results: Vec<AttendanceAdjustmentEntity> = query
                        .select(attendance_adjustment_requests::all_columns)
                        .order(attendance_adjustment_requests::created_at.desc())
                        .load(conn)
                        .await
                        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

                    Ok(results.into_iter().map(Into::into).collect())
                })
            })
            .await
    }

    async fn get_paginated_workspace_adjustment_requests(
        &self,
        workspace_id: Uuid,
        status: Option<AdjustmentStatus>,
        employee_id: Option<Uuid>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<AttendanceAdjustmentRequest>, i64), AppError> {
        let token = TenantAuthToken::system_override(workspace_id);
        self.diesel_pools
            .with_tenant_tx(&token, |conn| {
                Box::pin(async move {
                    let mut query = attendance_adjustment_requests::table
                        .inner_join(
                            employments::table
                                .on(attendance_adjustment_requests::employment_id
                                    .eq(employments::id)),
                        )
                        .inner_join(employees::table.on(employments::employee_id.eq(employees::id)))
                        .filter(employees::workspace_id.eq(workspace_id))
                        .into_boxed();

                    if let Some(s) = status {
                        query = query.filter(attendance_adjustment_requests::status.eq(s));
                    }

                    if let Some(emp_id) = employee_id {
                        query = query.filter(employees::id.eq(emp_id));
                    }

                    let total = query
                        .select(diesel::dsl::count(attendance_adjustment_requests::id))
                        .get_result::<i64>(conn)
                        .await
                        .map_err(|e| AppError::Internal(format!("db count: {e}")))?;

                    let mut query = attendance_adjustment_requests::table
                        .inner_join(
                            employments::table
                                .on(attendance_adjustment_requests::employment_id
                                    .eq(employments::id)),
                        )
                        .inner_join(employees::table.on(employments::employee_id.eq(employees::id)))
                        .filter(employees::workspace_id.eq(workspace_id))
                        .into_boxed();

                    if let Some(s) = status {
                        query = query.filter(attendance_adjustment_requests::status.eq(s));
                    }

                    if let Some(emp_id) = employee_id {
                        query = query.filter(employees::id.eq(emp_id));
                    }

                    let results: Vec<AttendanceAdjustmentEntity> = query
                        .select(attendance_adjustment_requests::all_columns)
                        .order(attendance_adjustment_requests::created_at.desc())
                        .limit(limit)
                        .offset(offset)
                        .load(conn)
                        .await
                        .map_err(|e| AppError::Internal(format!("db load: {e}")))?;

                    Ok((results.into_iter().map(Into::into).collect(), total))
                })
            })
            .await
    }

    async fn get_employment_adjustment_requests(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
    ) -> Result<Vec<AttendanceAdjustmentRequest>, AppError> {
        let token = TenantAuthToken::system_override(workspace_id);
        self.diesel_pools
            .with_tenant_tx(&token, |conn| {
                Box::pin(async move {
                    let results: Vec<AttendanceAdjustmentEntity> =
                        attendance_adjustment_requests::table
                            .inner_join(employments::table.on(
                                attendance_adjustment_requests::employment_id.eq(employments::id),
                            ))
                            .inner_join(
                                employees::table.on(employments::employee_id.eq(employees::id)),
                            )
                            .filter(employees::workspace_id.eq(workspace_id))
                            .filter(attendance_adjustment_requests::employment_id.eq(employment_id))
                            .select(attendance_adjustment_requests::all_columns)
                            .order(attendance_adjustment_requests::created_at.desc())
                            .load(conn)
                            .await
                            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

                    Ok(results.into_iter().map(Into::into).collect())
                })
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diesel_pool::DieselPools;

    #[tokio::test]
    async fn test_repo_initialization() {
        let test_db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://user:pass@localhost:5432/leeattend_test_db".to_string()
        });

        let pools = DieselPools::for_test(&test_db_url);
        let _repo = DieselAttendanceAdjustmentRepository::new(pools);
        // DI connection test passes if it compiles
    }
}
