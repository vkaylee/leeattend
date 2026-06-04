use crate::diesel_pool::{DieselPools, TenantAuthToken};
use crate::errors::AppError;
use crate::models::diesel::employer::EmployerEntity;
use crate::models::{CreateEmployerRequest, Employer, UpdateEmployerRequest};
use crate::repositories::employer_repository::EmployerRepository;
use crate::schema::{employees, employers};
use async_trait::async_trait;
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

#[derive(Clone)]
pub struct DieselEmployerRepository {
    diesel_pools: DieselPools,
}

impl DieselEmployerRepository {
    pub fn new(diesel_pools: DieselPools) -> Self {
        Self { diesel_pools }
    }
}

#[async_trait]
impl EmployerRepository for DieselEmployerRepository {
    async fn create_employer(
        &self,
        auth_token: &TenantAuthToken,
        data: CreateEmployerRequest,
    ) -> Result<Employer, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        let new_employer = EmployerEntity {
            id: Uuid::new_v4(),
            workspace_id,
            name: data.name,
            tax_code: data.tax_code,
            legal_address: data.legal_address,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let result = diesel::insert_into(employers::table)
            .values(&new_employer)
            .get_result::<EmployerEntity>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result.into())
    }

    async fn get_employer_by_id(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
    ) -> Result<Option<Employer>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        let result = employers::table
            .filter(employers::id.eq(id))
            .filter(employers::workspace_id.eq(workspace_id))
            .first::<EmployerEntity>(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result.map(Into::into))
    }

    async fn get_employers(
        &self,
        auth_token: &TenantAuthToken,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<Employer>, i64), AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        let total: i64 = employers::table
            .filter(employers::workspace_id.eq(workspace_id))
            .count()
            .get_result(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db count: {e}")))?;

        let results = employers::table
            .filter(employers::workspace_id.eq(workspace_id))
            .order(employers::name.asc())
            .limit(limit)
            .offset((page - 1) * limit)
            .load::<EmployerEntity>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok((results.into_iter().map(Into::into).collect(), total))
    }

    async fn update_employer(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
        data: UpdateEmployerRequest,
    ) -> Result<Employer, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        let mut existing = employers::table
            .filter(employers::id.eq(id))
            .filter(employers::workspace_id.eq(workspace_id))
            .first::<EmployerEntity>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        if let Some(name) = data.name {
            existing.name = name;
        }
        if data.tax_code.is_some() || data.tax_code.is_none() {
            existing.tax_code = data.tax_code;
        }
        if data.legal_address.is_some() || data.legal_address.is_none() {
            existing.legal_address = data.legal_address;
        }
        existing.updated_at = Utc::now();

        let result = diesel::update(
            employers::table
                .filter(employers::id.eq(id))
                .filter(employers::workspace_id.eq(workspace_id)),
        )
        .set(&existing)
        .get_result::<EmployerEntity>(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result.into())
    }

    async fn delete_employer(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
    ) -> Result<(), AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        diesel::delete(
            employers::table
                .filter(employers::id.eq(id))
                .filter(employers::workspace_id.eq(workspace_id)),
        )
        .execute(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }

    async fn count_employees_by_employer(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
    ) -> Result<i64, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        let count = employees::table
            .filter(employees::workspace_id.eq(workspace_id))
            .filter(employees::employer_id.eq(id))
            .count()
            .get_result::<i64>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_repo_initialization() {
        let test_db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://user:pass@localhost:5432/leeattend_test_db".to_string()
        });

        let pools = DieselPools::for_test(&test_db_url);
        let _repo = DieselEmployerRepository::new(pools);
    }
}
