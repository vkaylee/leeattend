use crate::diesel_pool::{DieselPools, TenantAuthToken};
use crate::errors::AppError;
use crate::models::diesel::employer::EmployerEntity;
use crate::models::{CreateEmployerRequest, Employer, UpdateEmployerRequest};
use crate::repositories::employer_repository::EmployerRepository;
use crate::schema::employers;
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
            deleted_at: None,
            parent_id: data.parent_id,
            relationship_type: data.relationship_type,
            business_type: data.business_type,
            registration_number: data.registration_number,
            phone: data.phone,
            email: data.email,
            website: data.website,
            incorporation_date: data.incorporation_date,
            country_code: data.country_code,
            currency_code: data.currency_code,
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
            .filter(employers::deleted_at.is_null())
            .first::<EmployerEntity>(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result.map(Into::into))
    }

    async fn get_employers_by_ids(
        &self,
        auth_token: &TenantAuthToken,
        ids: &[Uuid],
    ) -> Result<Vec<Employer>, AppError> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        let results = employers::table
            .filter(employers::id.eq_any(ids))
            .filter(employers::workspace_id.eq(workspace_id))
            .filter(employers::deleted_at.is_null())
            .load::<EmployerEntity>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(results.into_iter().map(Into::into).collect())
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
            .filter(employers::deleted_at.is_null())
            .count()
            .get_result(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db count: {e}")))?;

        let results = employers::table
            .filter(employers::workspace_id.eq(workspace_id))
            .filter(employers::deleted_at.is_null())
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
            .filter(employers::deleted_at.is_null())
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
        if let Some(parent_opt) = data.parent_id {
            if parent_opt.is_none() {
                diesel::update(
                    employers::table
                        .filter(employers::id.eq(id))
                        .filter(employers::workspace_id.eq(workspace_id)),
                )
                .set(employers::parent_id.eq(None::<Uuid>))
                .execute(&mut *conn)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?;
            }
            existing.parent_id = parent_opt;
        }
        if let Some(rel_opt) = data.relationship_type {
            if rel_opt.is_none() {
                diesel::update(
                    employers::table
                        .filter(employers::id.eq(id))
                        .filter(employers::workspace_id.eq(workspace_id)),
                )
                .set(employers::relationship_type.eq(None::<String>))
                .execute(&mut *conn)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?;
            }
            existing.relationship_type = rel_opt;
        }
        if data.business_type.is_some() || data.business_type.is_none() {
            existing.business_type = data.business_type;
        }
        if data.registration_number.is_some() || data.registration_number.is_none() {
            existing.registration_number = data.registration_number;
        }
        if data.phone.is_some() || data.phone.is_none() {
            existing.phone = data.phone;
        }
        if data.email.is_some() || data.email.is_none() {
            existing.email = data.email;
        }
        if data.website.is_some() || data.website.is_none() {
            existing.website = data.website;
        }
        if data.incorporation_date.is_some() || data.incorporation_date.is_none() {
            existing.incorporation_date = data.incorporation_date;
        }
        if data.country_code.is_some() || data.country_code.is_none() {
            existing.country_code = data.country_code;
        }
        if data.currency_code.is_some() || data.currency_code.is_none() {
            existing.currency_code = data.currency_code;
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

        diesel::update(
            employers::table
                .filter(employers::id.eq(id))
                .filter(employers::workspace_id.eq(workspace_id)),
        )
        .set(employers::deleted_at.eq(Some(Utc::now())))
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

        use crate::schema::employments;

        let count = employments::table
            .filter(employments::workspace_id.eq(workspace_id))
            .filter(employments::employer_id.eq(id))
            .filter(employments::status.eq("active"))
            .filter(employments::deleted_at.is_null())
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

    /// Empty input must return empty output before acquiring a tenant connection.
    #[tokio::test]
    async fn test_get_employers_by_ids_empty_input() {
        let pools = DieselPools::for_test("postgres://invalid:invalid@localhost:12345/invalid_db");
        let repo = DieselEmployerRepository::new(pools);
        let auth_token = TenantAuthToken::system_override(Uuid::new_v4());

        let result = repo.get_employers_by_ids(&auth_token, &[]).await;

        assert!(
            result.is_ok(),
            "Empty input should return Ok([]), got: {result:?}"
        );
        assert!(result.unwrap().is_empty());
    }
}
