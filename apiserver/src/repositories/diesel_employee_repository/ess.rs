use crate::errors::AppError;
use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::AsyncConnection;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::models::diesel::employee::{
    EmployeeDocumentEntity, EmployeeEmergencyContactEntity, EmployeeProfileEntity,
};
use crate::models::{
    CreateEmergencyContactRequest, EmployeeDocument, EmployeeEmergencyContact, EmployeeProfile,
    UpdateEmployeeProfileRequest,
};
use crate::repositories::employee::*;
use crate::schema::{
    employee_documents, employee_emergency_contacts, employee_profiles, employees,
};

use super::DieselEmployeeRepository;

#[allow(clippy::too_many_lines)]
#[async_trait]
impl EmployeeSelfServiceRepository for DieselEmployeeRepository {
    async fn get_employee_profile(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
    ) -> Result<Option<EmployeeProfile>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let result: Option<EmployeeProfileEntity> = employee_profiles::table
            .inner_join(employees::table)
            .select(EmployeeProfileEntity::as_select())
            .filter(employee_profiles::employee_id.eq(employee_id))
            .filter(employees::workspace_id.eq(workspace_id))
            .first(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result.map(Into::into))
    }

    async fn upsert_employee_profile(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        req: &UpdateEmployeeProfileRequest,
    ) -> Result<EmployeeProfile, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let result: EmployeeProfileEntity = diesel::insert_into(employee_profiles::table)
            .values((
                employee_profiles::employee_id.eq(employee_id),
                employee_profiles::phone.eq(&req.phone),
                employee_profiles::address.eq(&req.address),
                employee_profiles::date_of_birth.eq(req.date_of_birth),
                employee_profiles::gender.eq(&req.gender),
                employee_profiles::national_id.eq(&req.national_id),
                employee_profiles::bank_name.eq(&req.bank_name),
                employee_profiles::bank_account_number.eq(&req.bank_account_number),
                employee_profiles::bank_branch.eq(&req.bank_branch),
            ))
            .on_conflict(employee_profiles::employee_id)
            .do_update()
            .set((
                employee_profiles::phone.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Text>,
                >(
                    "COALESCE(EXCLUDED.phone, employee_profiles.phone)"
                )),
                employee_profiles::address.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Text>,
                >(
                    "COALESCE(EXCLUDED.address, employee_profiles.address)",
                )),
                employee_profiles::date_of_birth.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Date>,
                >(
                    "COALESCE(EXCLUDED.date_of_birth, employee_profiles.date_of_birth)",
                )),
                employee_profiles::gender.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Text>,
                >(
                    "COALESCE(EXCLUDED.gender, employee_profiles.gender)",
                )),
                employee_profiles::national_id.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Text>,
                >(
                    "COALESCE(EXCLUDED.national_id, employee_profiles.national_id)",
                )),
                employee_profiles::bank_name.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Text>,
                >(
                    "COALESCE(EXCLUDED.bank_name, employee_profiles.bank_name)",
                )),
                employee_profiles::bank_account_number.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Text>,
                >(
                    "COALESCE(EXCLUDED.bank_account_number, employee_profiles.bank_account_number)",
                )),
                employee_profiles::bank_branch.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Text>,
                >(
                    "COALESCE(EXCLUDED.bank_branch, employee_profiles.bank_branch)",
                )),
                employee_profiles::updated_at.eq(chrono::Utc::now()),
            ))
            .get_result(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result.into())
    }

    async fn get_emergency_contacts(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
    ) -> Result<Vec<EmployeeEmergencyContact>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let results: Vec<EmployeeEmergencyContactEntity> = employee_emergency_contacts::table
            .inner_join(employees::table)
            .select(EmployeeEmergencyContactEntity::as_select())
            .filter(employee_emergency_contacts::employee_id.eq(employee_id))
            .filter(employees::workspace_id.eq(workspace_id))
            .order((
                employee_emergency_contacts::is_primary.desc(),
                employee_emergency_contacts::created_at.asc(),
            ))
            .load(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn upsert_emergency_contact(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        req: &CreateEmergencyContactRequest,
    ) -> Result<EmployeeEmergencyContact, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let is_primary = req.is_primary.unwrap_or(false);

        conn.transaction::<_, AppError, _>(|conn| {
            Box::pin(async move {
                if is_primary {
                    diesel::update(employee_emergency_contacts::table)
                        .filter(employee_emergency_contacts::employee_id.eq(employee_id))
                        .set(employee_emergency_contacts::is_primary.eq(false))
                        .execute(&mut *conn)
                        .await
                        .map_err(|e| AppError::Internal(format!("db: {e}")))?;
                }

                let result: EmployeeEmergencyContactEntity =
                    diesel::insert_into(employee_emergency_contacts::table)
                        .values((
                            employee_emergency_contacts::id.eq(Uuid::new_v4()),
                            employee_emergency_contacts::employee_id.eq(employee_id),
                            employee_emergency_contacts::contact_name.eq(&req.contact_name),
                            employee_emergency_contacts::relationship.eq(&req.relationship),
                            employee_emergency_contacts::phone.eq(&req.phone),
                            employee_emergency_contacts::is_primary.eq(is_primary),
                        ))
                        .get_result(&mut *conn)
                        .await
                        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

                Ok(result.into())
            })
        })
        .await
    }

    async fn delete_emergency_contact(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        contact_id: Uuid,
    ) -> Result<u64, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let affected = diesel::delete(employee_emergency_contacts::table)
            .filter(employee_emergency_contacts::id.eq(contact_id))
            .filter(employee_emergency_contacts::employee_id.eq(employee_id))
            .filter(diesel::dsl::exists(
                employees::table
                    .filter(employees::id.eq(employee_id))
                    .filter(employees::workspace_id.eq(workspace_id)),
            ))
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(affected as u64)
    }

    async fn get_employee_documents(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
    ) -> Result<Vec<EmployeeDocument>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let results: Vec<EmployeeDocumentEntity> = employee_documents::table
            .select(EmployeeDocumentEntity::as_select())
            .filter(employee_documents::employee_id.eq(employee_id))
            .filter(employee_documents::workspace_id.eq(workspace_id))
            .order(employee_documents::created_at.desc())
            .load(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn create_employee_document(&self, doc: &EmployeeDocument) -> Result<(), AppError> {
        let workspace_id = doc.workspace_id;
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let entity: EmployeeDocumentEntity = doc.clone().into();

        diesel::insert_into(employee_documents::table)
            .values(&entity)
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }
}
