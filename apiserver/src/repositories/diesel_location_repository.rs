use crate::diesel_pool::DieselPools;
use crate::diesel_pool::tenant_manager::TenantAuthToken;
use crate::models::diesel::work_location::WorkLocationEntity;
use crate::models::{CreateWorkLocationRequest, UpdateWorkLocationRequest, WorkLocation};
use crate::repositories::location_repository::LocationRepository;
use crate::schema::{employees, work_locations};
use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

#[derive(Clone)]
pub struct DieselLocationRepository {
    pools: DieselPools,
}

impl DieselLocationRepository {
    pub fn new(pools: DieselPools) -> Self {
        Self { pools }
    }
}

#[async_trait]
impl LocationRepository for DieselLocationRepository {
    async fn find_all_by_workspace(&self, workspace_id: Uuid) -> Result<Vec<WorkLocation>, String> {
        let auth_token = TenantAuthToken::system_override(workspace_id);
        let mut conn = self
            .pools
            .tenant_transaction_conn(&auth_token)
            .await
            .map_err(|e| e.to_string())?;

        let results = work_locations::table
            .filter(work_locations::workspace_id.eq(workspace_id))
            .filter(work_locations::status.eq("active"))
            .order_by((
                work_locations::is_headquarters.desc(),
                work_locations::name.asc(),
            ))
            .load::<WorkLocationEntity>(&mut *conn)
            .await
            .map_err(|e| e.to_string())?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn find_by_id(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<Option<WorkLocation>, String> {
        let auth_token = TenantAuthToken::system_override(workspace_id);
        let mut conn = self
            .pools
            .tenant_transaction_conn(&auth_token)
            .await
            .map_err(|e| e.to_string())?;

        let result = work_locations::table
            .filter(work_locations::workspace_id.eq(workspace_id))
            .filter(work_locations::id.eq(id))
            .filter(work_locations::status.eq("active"))
            .first::<WorkLocationEntity>(&mut *conn)
            .await
            .optional()
            .map_err(|e| e.to_string())?;

        Ok(result.map(Into::into))
    }

    async fn create(
        &self,
        workspace_id: Uuid,
        data: CreateWorkLocationRequest,
    ) -> Result<WorkLocation, String> {
        let auth_token = TenantAuthToken::system_override(workspace_id);
        let mut conn = self
            .pools
            .tenant_transaction_conn(&auth_token)
            .await
            .map_err(|e| e.to_string())?;

        let new_location = WorkLocationEntity {
            id: Uuid::new_v4(),
            workspace_id,
            name: data.name,
            address: data.address,
            timezone: data.timezone.unwrap_or_else(|| "UTC".to_string()),
            is_headquarters: data.is_headquarters.unwrap_or(false),
            status: data.status.unwrap_or_else(|| "active".to_string()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let result = diesel::insert_into(work_locations::table)
            .values(&new_location)
            .get_result::<WorkLocationEntity>(&mut *conn)
            .await
            .map_err(|e| e.to_string())?;

        Ok(result.into())
    }

    async fn update(
        &self,
        workspace_id: Uuid,
        id: Uuid,
        data: UpdateWorkLocationRequest,
    ) -> Result<WorkLocation, String> {
        let auth_token = TenantAuthToken::system_override(workspace_id);
        let mut conn = self
            .pools
            .tenant_transaction_conn(&auth_token)
            .await
            .map_err(|e| e.to_string())?;

        let mut existing = work_locations::table
            .filter(work_locations::workspace_id.eq(workspace_id))
            .filter(work_locations::id.eq(id))
            .first::<WorkLocationEntity>(&mut *conn)
            .await
            .map_err(|e| e.to_string())?;

        if let Some(name) = data.name {
            existing.name = name;
        }
        if let Some(address) = data.address {
            existing.address = Some(address);
        }
        if let Some(timezone) = data.timezone {
            existing.timezone = timezone;
        }
        if let Some(is_headquarters) = data.is_headquarters {
            existing.is_headquarters = is_headquarters;
        }
        if let Some(status) = data.status {
            existing.status = status;
        }
        existing.updated_at = chrono::Utc::now();

        let result = diesel::update(
            work_locations::table
                .filter(work_locations::workspace_id.eq(workspace_id))
                .filter(work_locations::id.eq(id)),
        )
        .set(&existing)
        .get_result::<WorkLocationEntity>(&mut *conn)
        .await
        .map_err(|e| e.to_string())?;

        Ok(result.into())
    }

    async fn delete(&self, workspace_id: Uuid, id: Uuid) -> Result<(), String> {
        let auth_token = TenantAuthToken::system_override(workspace_id);
        let mut conn = self
            .pools
            .tenant_transaction_conn(&auth_token)
            .await
            .map_err(|e| e.to_string())?;

        diesel::update(
            work_locations::table
                .filter(work_locations::workspace_id.eq(workspace_id))
                .filter(work_locations::id.eq(id)),
        )
        .set(work_locations::status.eq("deleted"))
        .execute(&mut *conn)
        .await
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    async fn count_employees_in_location(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<i64, String> {
        let auth_token = TenantAuthToken::system_override(workspace_id);
        let mut conn = self
            .pools
            .tenant_transaction_conn(&auth_token)
            .await
            .map_err(|e| e.to_string())?;

        let count = employees::table
            .filter(employees::workspace_id.eq(workspace_id))
            .filter(employees::primary_location_id.eq(id))
            .filter(employees::is_active.eq(true))
            .count()
            .get_result::<i64>(&mut *conn)
            .await
            .map_err(|e| e.to_string())?;

        Ok(count)
    }
}
