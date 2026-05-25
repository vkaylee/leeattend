use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::diesel_pool::DieselPools;
use crate::errors::AppError;
use crate::models::diesel::roster::ShiftSwapRequestEntity;
use crate::models::shift_swap::{ShiftSwapRequest, ShiftSwapRequestWithNames, ShiftSwapStatus};
use crate::repositories::shift_swap_repository::{ShiftSwapRepository, SwapStats};
use crate::schema::shift_swap_requests;

#[derive(Debug, diesel::QueryableByName)]
pub struct ShiftSwapRequestWithNamesEntity {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub workspace_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub requester_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub target_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Date)]
    pub swap_date: NaiveDate,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
    pub requester_shift_id: Option<Uuid>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
    pub target_shift_id: Option<Uuid>,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub status: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub reason: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
    pub requested_by_user_id: Option<Uuid>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
    pub approved_by_user_id: Option<Uuid>,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    pub created_at: chrono::DateTime<Utc>,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    pub updated_at: chrono::DateTime<Utc>,

    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub requester_name: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub target_name: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub requester_shift_name: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub target_shift_name: Option<String>,
}

impl From<ShiftSwapRequestWithNamesEntity> for ShiftSwapRequestWithNames {
    fn from(entity: ShiftSwapRequestWithNamesEntity) -> Self {
        let status = match entity.status.as_str() {
            "pending_admin" => ShiftSwapStatus::PendingAdmin,
            "approved" => ShiftSwapStatus::Approved,
            "rejected" => ShiftSwapStatus::Rejected,
            "cancelled" => ShiftSwapStatus::Cancelled,
            _ => ShiftSwapStatus::PendingTarget,
        };
        Self {
            id: entity.id,
            workspace_id: entity.workspace_id,
            requester_id: entity.requester_id,
            target_id: entity.target_id,
            swap_date: entity.swap_date,
            requester_shift_id: entity.requester_shift_id,
            target_shift_id: entity.target_shift_id,
            status,
            reason: entity.reason,
            requested_by_user_id: entity.requested_by_user_id,
            approved_by_user_id: entity.approved_by_user_id,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
            requester_name: entity.requester_name,
            target_name: entity.target_name,
            requester_shift_name: entity.requester_shift_name,
            target_shift_name: entity.target_shift_name,
        }
    }
}

#[derive(Debug, diesel::QueryableByName)]
pub struct SwapStatsEntity {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub pending_target: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub pending_admin: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub approved: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub rejected: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub cancelled: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub approved_this_month: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub rejected_this_month: i64,
}

impl From<SwapStatsEntity> for SwapStats {
    fn from(entity: SwapStatsEntity) -> Self {
        Self {
            pending_target: entity.pending_target,
            pending_admin: entity.pending_admin,
            approved: entity.approved,
            rejected: entity.rejected,
            cancelled: entity.cancelled,
            approved_this_month: entity.approved_this_month,
            rejected_this_month: entity.rejected_this_month,
        }
    }
}
pub struct DieselShiftSwapRepository {
    pub diesel_pools: DieselPools,
}

impl DieselShiftSwapRepository {
    pub fn new(diesel_pools: DieselPools) -> Self {
        Self { diesel_pools }
    }
}

#[async_trait]
impl ShiftSwapRepository for DieselShiftSwapRepository {
    async fn create_swap_request(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        request: &ShiftSwapRequest,
    ) -> Result<ShiftSwapRequest, AppError> {
        let _workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let entity = ShiftSwapRequestEntity {
            id: request.id,
            workspace_id: request.workspace_id,
            requester_id: request.requester_id,
            target_id: request.target_id,
            swap_date: request.swap_date,
            requester_shift_id: request.requester_shift_id,
            target_shift_id: request.target_shift_id,
            status: match request.status {
                ShiftSwapStatus::PendingTarget => "pending_target".to_string(),
                ShiftSwapStatus::PendingAdmin => "pending_admin".to_string(),
                ShiftSwapStatus::Approved => "approved".to_string(),
                ShiftSwapStatus::Rejected => "rejected".to_string(),
                ShiftSwapStatus::Cancelled => "cancelled".to_string(),
            },
            reason: request.reason.clone(),
            requested_by_user_id: request.requested_by_user_id,
            approved_by_user_id: request.approved_by_user_id,
            created_at: request.created_at,
            updated_at: request.updated_at,
        };

        let result: ShiftSwapRequestEntity = diesel::insert_into(shift_swap_requests::table)
            .values(&entity)
            .get_result(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result.into())
    }

    async fn get_swap_request_by_id(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Option<ShiftSwapRequest>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let entity: Option<ShiftSwapRequestEntity> = shift_swap_requests::table
            .filter(shift_swap_requests::id.eq(id))
            .filter(shift_swap_requests::workspace_id.eq(workspace_id))
            .first(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entity.map(Into::into))
    }

    async fn get_workspace_swaps(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        status_filter: Option<ShiftSwapStatus>,
    ) -> Result<Vec<ShiftSwapRequestWithNames>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let status_str = status_filter.map(|s| match s {
            ShiftSwapStatus::PendingTarget => "pending_target",
            ShiftSwapStatus::PendingAdmin => "pending_admin",
            ShiftSwapStatus::Approved => "approved",
            ShiftSwapStatus::Rejected => "rejected",
            ShiftSwapStatus::Cancelled => "cancelled",
        });

        let mut query = shift_swap_requests::table
            .filter(shift_swap_requests::workspace_id.eq(workspace_id))
            .into_boxed();

        if let Some(s) = status_str {
            query = query.filter(shift_swap_requests::status.eq(s));
        }

        let swaps: Vec<ShiftSwapRequestEntity> = query
            .order(shift_swap_requests::created_at.desc())
            .limit(100)
            .load(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let mut employee_ids = std::collections::HashSet::new();
        let mut shift_ids = std::collections::HashSet::new();

        for s in &swaps {
            employee_ids.insert(s.requester_id);
            employee_ids.insert(s.target_id);
            if let Some(id) = s.requester_shift_id {
                shift_ids.insert(id);
            }
            if let Some(id) = s.target_shift_id {
                shift_ids.insert(id);
            }
        }

        let employee_ids: Vec<Uuid> = employee_ids.into_iter().collect();
        use crate::schema::employees;
        let emps: Vec<(Uuid, String)> = employees::table
            .filter(employees::id.eq_any(employee_ids))
            .select((employees::id, employees::full_name))
            .load(&mut *conn)
            .await
            .unwrap_or_default();
        let emp_map: std::collections::HashMap<Uuid, String> = emps.into_iter().collect();

        let shift_ids: Vec<Uuid> = shift_ids.into_iter().collect();
        use crate::schema::shifts;
        let shifts_data: Vec<(Uuid, String)> = shifts::table
            .filter(shifts::id.eq_any(shift_ids))
            .select((shifts::id, shifts::name))
            .load(&mut *conn)
            .await
            .unwrap_or_default();
        let shift_map: std::collections::HashMap<Uuid, String> = shifts_data.into_iter().collect();

        Ok(swaps
            .into_iter()
            .map(|s| ShiftSwapRequestWithNamesEntity {
                id: s.id,
                workspace_id: s.workspace_id,
                requester_id: s.requester_id,
                target_id: s.target_id,
                swap_date: s.swap_date,
                requester_shift_id: s.requester_shift_id,
                target_shift_id: s.target_shift_id,
                status: s.status,
                reason: s.reason,
                requested_by_user_id: s.requested_by_user_id,
                approved_by_user_id: s.approved_by_user_id,
                created_at: s.created_at,
                updated_at: s.updated_at,
                requester_name: emp_map.get(&s.requester_id).cloned(),
                target_name: emp_map.get(&s.target_id).cloned(),
                requester_shift_name: s
                    .requester_shift_id
                    .and_then(|id| shift_map.get(&id).cloned()),
                target_shift_name: s.target_shift_id.and_then(|id| shift_map.get(&id).cloned()),
            })
            .map(Into::into)
            .collect())
    }

    async fn update_swap_status(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        status: ShiftSwapStatus,
        approver_id: Option<Uuid>,
    ) -> Result<(), AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let status_str = match status {
            ShiftSwapStatus::PendingTarget => "pending_target",
            ShiftSwapStatus::PendingAdmin => "pending_admin",
            ShiftSwapStatus::Approved => "approved",
            ShiftSwapStatus::Rejected => "rejected",
            ShiftSwapStatus::Cancelled => "cancelled",
        };

        diesel::update(shift_swap_requests::table)
            .filter(shift_swap_requests::id.eq(id))
            .filter(shift_swap_requests::workspace_id.eq(workspace_id))
            .set((
                shift_swap_requests::status.eq(status_str),
                shift_swap_requests::approved_by_user_id.eq(approver_id),
                shift_swap_requests::updated_at.eq(Utc::now()),
            ))
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }

    async fn get_employee_swaps(
        &self,
        employee_id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<ShiftSwapRequestWithNames>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let swaps: Vec<ShiftSwapRequestEntity> = shift_swap_requests::table
            .filter(shift_swap_requests::workspace_id.eq(workspace_id))
            .filter(
                shift_swap_requests::requester_id
                    .eq(employee_id)
                    .or(shift_swap_requests::target_id.eq(employee_id)),
            )
            .order(shift_swap_requests::created_at.desc())
            .limit(50)
            .load(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let mut employee_ids = std::collections::HashSet::new();
        let mut shift_ids = std::collections::HashSet::new();

        for s in &swaps {
            employee_ids.insert(s.requester_id);
            employee_ids.insert(s.target_id);
            if let Some(id) = s.requester_shift_id {
                shift_ids.insert(id);
            }
            if let Some(id) = s.target_shift_id {
                shift_ids.insert(id);
            }
        }

        let employee_ids: Vec<Uuid> = employee_ids.into_iter().collect();
        use crate::schema::employees;
        let emps: Vec<(Uuid, String)> = employees::table
            .filter(employees::id.eq_any(employee_ids))
            .select((employees::id, employees::full_name))
            .load(&mut *conn)
            .await
            .unwrap_or_default();
        let emp_map: std::collections::HashMap<Uuid, String> = emps.into_iter().collect();

        let shift_ids: Vec<Uuid> = shift_ids.into_iter().collect();
        use crate::schema::shifts;
        let shifts_data: Vec<(Uuid, String)> = shifts::table
            .filter(shifts::id.eq_any(shift_ids))
            .select((shifts::id, shifts::name))
            .load(&mut *conn)
            .await
            .unwrap_or_default();
        let shift_map: std::collections::HashMap<Uuid, String> = shifts_data.into_iter().collect();

        Ok(swaps
            .into_iter()
            .map(|s| ShiftSwapRequestWithNamesEntity {
                id: s.id,
                workspace_id: s.workspace_id,
                requester_id: s.requester_id,
                target_id: s.target_id,
                swap_date: s.swap_date,
                requester_shift_id: s.requester_shift_id,
                target_shift_id: s.target_shift_id,
                status: s.status,
                reason: s.reason,
                requested_by_user_id: s.requested_by_user_id,
                approved_by_user_id: s.approved_by_user_id,
                created_at: s.created_at,
                updated_at: s.updated_at,
                requester_name: emp_map.get(&s.requester_id).cloned(),
                target_name: emp_map.get(&s.target_id).cloned(),
                requester_shift_name: s
                    .requester_shift_id
                    .and_then(|id| shift_map.get(&id).cloned()),
                target_shift_name: s.target_shift_id.and_then(|id| shift_map.get(&id).cloned()),
            })
            .map(Into::into)
            .collect())
    }

    async fn has_active_swap(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        requester_id: Uuid,
        target_id: Uuid,
        swap_date: NaiveDate,
    ) -> Result<bool, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let exists: bool = diesel::select(diesel::dsl::exists(
            shift_swap_requests::table
                .filter(shift_swap_requests::workspace_id.eq(workspace_id))
                .filter(shift_swap_requests::requester_id.eq(requester_id))
                .filter(shift_swap_requests::target_id.eq(target_id))
                .filter(shift_swap_requests::swap_date.eq(swap_date))
                .filter(shift_swap_requests::status.ne_all(["cancelled", "rejected"])),
        ))
        .get_result(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(exists)
    }

    async fn has_approved_swap_on_date(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        swap_date: NaiveDate,
    ) -> Result<bool, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let exists: bool = diesel::select(diesel::dsl::exists(
            shift_swap_requests::table
                .filter(shift_swap_requests::workspace_id.eq(workspace_id))
                .filter(
                    shift_swap_requests::requester_id
                        .eq(employee_id)
                        .or(shift_swap_requests::target_id.eq(employee_id)),
                )
                .filter(shift_swap_requests::swap_date.eq(swap_date))
                .filter(shift_swap_requests::status.eq("approved")),
        ))
        .get_result(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(exists)
    }

    async fn get_workspace_swap_stats(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<SwapStats, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let result = shift_swap_requests::table
        .filter(shift_swap_requests::workspace_id.eq(workspace_id))
        .select((
            diesel::dsl::sql::<diesel::sql_types::BigInt>("COALESCE(SUM(CASE WHEN status = 'pending_target' THEN 1 ELSE 0 END), 0)"),
            diesel::dsl::sql::<diesel::sql_types::BigInt>("COALESCE(SUM(CASE WHEN status = 'pending_admin' THEN 1 ELSE 0 END), 0)"),
            diesel::dsl::sql::<diesel::sql_types::BigInt>("COALESCE(SUM(CASE WHEN status = 'approved' THEN 1 ELSE 0 END), 0)"),
            diesel::dsl::sql::<diesel::sql_types::BigInt>("COALESCE(SUM(CASE WHEN status = 'rejected' THEN 1 ELSE 0 END), 0)"),
            diesel::dsl::sql::<diesel::sql_types::BigInt>("COALESCE(SUM(CASE WHEN status = 'cancelled' THEN 1 ELSE 0 END), 0)"),
            diesel::dsl::sql::<diesel::sql_types::BigInt>("COALESCE(SUM(CASE WHEN status = 'approved' AND created_at >= date_trunc('month', CURRENT_DATE) THEN 1 ELSE 0 END), 0)"),
            diesel::dsl::sql::<diesel::sql_types::BigInt>("COALESCE(SUM(CASE WHEN status = 'rejected' AND created_at >= date_trunc('month', CURRENT_DATE) THEN 1 ELSE 0 END), 0)"),
        ))
        .first::<(i64, i64, i64, i64, i64, i64, i64)>(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entity = SwapStatsEntity {
            pending_target: result.0,
            pending_admin: result.1,
            approved: result.2,
            rejected: result.3,
            cancelled: result.4,
            approved_this_month: result.5,
            rejected_this_month: result.6,
        };

        Ok(entity.into())
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
        let _repo = DieselShiftSwapRepository::new(pools);
        // DI connection test passes if it compiles
    }
}
