use super::*;
use crate::diesel_pool::DieselPoolsTenantTxExt;
use crate::errors::AppError;
use crate::models::diesel::employee::{EmployeeEntity, EmployeeRow, UpdateDieselEmployee};
use crate::models::{Employee, UpdateEmployeeRequest};
use crate::schema::employees;
use diesel::prelude::*;
use diesel_async::AsyncConnection;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

impl DieselEmployeeRepository {
    pub(super) async fn update_employee_impl(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        req: &UpdateEmployeeRequest,
    ) -> Result<Option<Employee>, AppError> {
        self.update_employee_transactional_impl(id, auth_token, req)
            .await
    }

    pub(super) async fn update_employee_transactional_impl(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        req: &UpdateEmployeeRequest,
    ) -> Result<Option<Employee>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        conn.transaction(|conn| {
            Box::pin(async move {
                if let Some(Some(new_user_id)) = req.user_id {
                    if req.force_transfer == Some(true) {
                        diesel::update(employees::table)
                            .filter(employees::user_id.eq(new_user_id))
                            .filter(employees::workspace_id.eq(workspace_id))
                            .filter(employees::id.ne(id))
                            .set(employees::user_id.eq(None::<Uuid>))
                            .execute(&mut *conn)
                            .await
                            .map_err(AppError::from)?;
                    }
                }

                let normalized_id = req
                    .employee_id_number
                    .as_ref()
                    .map(|id| id.as_str().to_string());

                let tags_mapped = req.tags.as_ref().map(|tags| {
                    tags.iter()
                        .map(|t| Some(t.clone()))
                        .collect::<Vec<Option<String>>>()
                });

                let update_model = UpdateDieselEmployee {
                    user_id: req.user_id,
                    full_name: req.full_name.clone(),
                    employee_id_number: normalized_id,
                    tags: tags_mapped,
                    updated_at: Some(chrono::Utc::now()),
                };

                let result: Option<EmployeeEntity> = diesel::update(employees::table)
                    .filter(employees::id.eq(id))
                    .filter(employees::workspace_id.eq(workspace_id))
                    .set(&update_model)
                    .get_result(&mut *conn)
                    .await
                    .optional()
                    .map_err(AppError::from)?;

                if req.recruitment_source_id.is_some() {
                    diesel::update(employees::table)
                        .filter(employees::id.eq(id))
                        .filter(employees::workspace_id.eq(workspace_id))
                        .set((
                            employees::recruitment_source_id.eq(req.recruitment_source_id),
                            employees::updated_at.eq(Some(chrono::Utc::now())),
                        ))
                        .execute(&mut *conn)
                        .await
                        .map_err(AppError::from)?;
                }

                Ok(result.map(Into::into))
            })
        })
        .await
    }

    pub(super) async fn create_employee_impl(&self, employee: &Employee) -> Result<(), AppError> {
        let auth_token =
            crate::diesel_pool::TenantAuthToken::system_override(employee.workspace_id());
        let workspace_id = employee.workspace_id();
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(&auth_token)
            .await?;
        use crate::schema::employers;
        let emp_id_opt: Option<Uuid> = diesel_async::RunQueryDsl::first(
            employers::table
                .filter(employers::workspace_id.eq(workspace_id))
                .select(employers::id),
            &mut conn,
        )
        .await
        .optional()
        .map_err(|e| AppError::Internal(format!("db select first employer: {e}")))?;
        let employer_id = emp_id_opt.unwrap_or_else(Uuid::new_v4);

        self.create_employee_transactional_impl(
            &auth_token,
            employee,
            employee.created_at().date_naive(),
            None,
            employer_id,
            None,
            None,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) async fn create_employee_transactional_impl(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee: &Employee,
        hire_date: chrono::NaiveDate,
        recruitment_source_id: Option<Uuid>,
        employer_id: Uuid,
        primary_location_id: Option<Uuid>,
        department_id: Option<Uuid>,
    ) -> Result<(), AppError> {
        let workspace_id = auth_token.workspace_id();
        if employee.workspace_id() != workspace_id {
            return Err(AppError::forbidden(crate::utils::i18n::I18nKey::Forbidden));
        }

        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let mut new_entity: EmployeeEntity = employee.clone().into();
        new_entity.recruitment_source_id = recruitment_source_id;

        conn.transaction(|conn| {
            Box::pin(async move {
                diesel::insert_into(crate::schema::employees::table)
                    .values(&new_entity)
                    .execute(&mut *conn)
                    .await
                    .map_err(|e| AppError::Internal(format!("db employee insert: {e}")))?;

                use crate::schema::employee_contracts;
                use crate::schema::employments;

                let employment_id = Uuid::new_v4();
                let new_emp = crate::models::diesel::employment::EmploymentEntity {
                    id: employment_id,
                    workspace_id,
                    employee_id: employee.id(),
                    employer_id,
                    primary_location_id,
                    department_id,
                    status: "active".to_string(),
                    start_date: hire_date,
                    end_date: None,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    deleted_at: None,
                    job_title: None,
                    employment_type: "full_time".to_string(),
                    manager_employment_id: None,
                    notes: None,
                    termination_reason: None,
                };

                diesel::insert_into(employments::table)
                    .values(&new_emp)
                    .execute(&mut *conn)
                    .await
                    .map_err(|e| AppError::Internal(format!("db insert employment: {e}")))?;

                let default_contract =
                    crate::models::diesel::employee_contract::EmployeeContractEntity {
                        id: Uuid::new_v4(),
                        workspace_id,
                        contract_number: None,
                        contract_type: crate::models::enums::ContractType::Indefinite,
                        status: crate::models::enums::ContractStatus::Active,
                        start_date: hire_date,
                        end_date: None,
                        document_id: None,
                        notes: Some(
                            "Hệ thống tự động khởi tạo thỏa thuận làm việc khi onboard".to_string(),
                        ),
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                        deleted_at: None,
                        employment_id,
                    };

                diesel::insert_into(employee_contracts::table)
                    .values(&default_contract)
                    .execute(&mut *conn)
                    .await
                    .map_err(|e| AppError::Internal(format!("db contract insert: {e}")))?;

                Ok::<(), AppError>(())
            })
        })
        .await
    }

    pub(super) async fn create_employee_skip_duplicates_impl(
        &self,
        employee: &Employee,
    ) -> Result<bool, AppError> {
        let workspace_id = employee.workspace_id();
        use crate::schema::employees;

        let auth_token = crate::diesel_pool::TenantAuthToken::system_override(workspace_id);
        if self
            .get_employee_by_id_number_impl(employee.employee_id_number(), &auth_token)
            .await?
            .is_some()
        {
            return Ok(false);
        }

        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(&auth_token)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let new_entity: EmployeeEntity = employee.clone().into();

        conn.transaction(|conn| {
            Box::pin(async move {
                diesel::insert_into(employees::table)
                    .values(&new_entity)
                    .execute(&mut *conn)
                    .await
                    .map_err(|e| AppError::Internal(format!("db employee insert: {e}")))?;

                use crate::schema::employers;
                let emp_id_opt: Option<Uuid> = diesel_async::RunQueryDsl::first(
                    employers::table
                        .filter(employers::workspace_id.eq(workspace_id))
                        .select(employers::id),
                    &mut *conn,
                )
                .await
                .optional()
                .map_err(|e| AppError::Internal(format!("db select first employer: {e}")))?;
                let emp_id = emp_id_opt.unwrap_or_else(Uuid::new_v4);

                use crate::schema::employee_contracts;
                use crate::schema::employments;

                let employment_id = Uuid::new_v4();
                let new_emp = crate::models::diesel::employment::EmploymentEntity {
                    id: employment_id,
                    workspace_id,
                    employee_id: employee.id(),
                    employer_id: emp_id,
                    primary_location_id: None,
                    department_id: None,
                    status: "active".to_string(),
                    start_date: employee.created_at().date_naive(),
                    end_date: None,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    deleted_at: None,
                    job_title: None,
                    employment_type: "full_time".to_string(),
                    manager_employment_id: None,
                    notes: None,
                    termination_reason: None,
                };

                diesel::insert_into(employments::table)
                    .values(&new_emp)
                    .execute(&mut *conn)
                    .await
                    .map_err(|e| AppError::Internal(format!("db insert employment: {e}")))?;

                let default_contract =
                    crate::models::diesel::employee_contract::EmployeeContractEntity {
                        id: Uuid::new_v4(),
                        workspace_id,
                        contract_number: None,
                        contract_type: crate::models::enums::ContractType::Indefinite,
                        status: crate::models::enums::ContractStatus::Active,
                        start_date: employee.created_at().date_naive(),
                        end_date: None,
                        document_id: None,
                        notes: Some(
                            "Hệ thống tự động khởi tạo thỏa thuận làm việc khi onboard".to_string(),
                        ),
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                        deleted_at: None,
                        employment_id,
                    };

                diesel::insert_into(employee_contracts::table)
                    .values(&default_contract)
                    .execute(&mut *conn)
                    .await
                    .map_err(|e| AppError::Internal(format!("db contract insert: {e}")))?;

                Ok::<(), AppError>(())
            })
        })
        .await?;

        Ok(true)
    }

    pub(super) async fn rehire_employee_impl(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        hire_date: chrono::NaiveDate,
        employer_id: Uuid,
        primary_location_id: Option<Uuid>,
        department_id: Option<Uuid>,
    ) -> Result<Option<Employee>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        conn.transaction(|conn| {
            Box::pin(async move {
                let employee_exists: bool = employees::table
                    .filter(employees::id.eq(employee_id))
                    .filter(employees::workspace_id.eq(workspace_id))
                    .select(diesel::dsl::sql::<diesel::sql_types::Bool>("1"))
                    .first::<bool>(&mut *conn)
                    .await
                    .optional()
                    .map_err(|e| AppError::Internal(format!("db select employee: {e}")))?
                    .is_some();

                if !employee_exists {
                    return Ok::<Option<Employee>, AppError>(None);
                }

                use crate::schema::employments;

                let previous_employments = diesel_async::RunQueryDsl::load::<crate::models::diesel::employment::EmploymentEntity>(
                    employments::table
                        .filter(employments::employee_id.eq(employee_id))
                        .filter(employments::deleted_at.is_null()),
                    &mut *conn,
                )
                .await
                .map_err(|e| AppError::Internal(format!("db select previous employments: {e}")))?;

                for emp in &previous_employments {
                    if emp.status == "active" {
                        return Err(AppError::validation(
                            crate::utils::i18n::I18nKey::EmployeeAlreadyActive,
                        ));
                    }
                    if let Some(end_date) = emp.end_date {
                        if hire_date <= end_date {
                            return Err(AppError::validation(
                                crate::utils::i18n::I18nKey::RehireDateInvalid,
                            ));
                        }
                    }
                }

                let employment_id = Uuid::new_v4();
                let new_emp = crate::models::diesel::employment::EmploymentEntity {
                    id: employment_id,
                    workspace_id,
                    employee_id,
                    employer_id,
                    primary_location_id,
                    department_id,
                    status: "active".to_string(),
                    start_date: hire_date,
                    end_date: None,
                    job_title: None,
                    employment_type: "full_time".to_string(),
                    manager_employment_id: None,
                    notes: None,
                    termination_reason: None,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    deleted_at: None,
                };

                diesel::insert_into(employments::table)
                    .values(&new_emp)
                    .execute(&mut *conn)
                    .await
                    .map_err(|e| AppError::Internal(format!("db insert employment: {e}")))?;

                let query = format!("{EMPLOYEE_SELECT_QUERY} WHERE e.id = $1 AND e.workspace_id = $2 AND e.deleted_at IS NULL");
                let row: Option<EmployeeRow> = diesel::sql_query(query)
                    .bind::<diesel::sql_types::Uuid, _>(employee_id)
                    .bind::<diesel::sql_types::Uuid, _>(workspace_id)
                    .get_result(&mut *conn)
                    .await
                    .optional()
                    .map_err(|e| AppError::Internal(format!("db select employee row: {e}")))?;

                Ok(row.map(Into::into))
            })
        })
        .await
    }

    pub(super) async fn delete_employee_impl(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<u64, AppError> {
        let workspace_id = auth_token.workspace_id();
        self.diesel_pools
            .with_tenant_tx(auth_token, |conn| {
                Box::pin(async move {
                    let affected = diesel::update(employees::table)
                        .filter(employees::id.eq(id))
                        .filter(employees::workspace_id.eq(workspace_id))
                        .set(employees::deleted_at.eq(Some(chrono::Utc::now())))
                        .execute(conn)
                        .await
                        .map_err(AppError::from)?;

                    Ok(affected as u64)
                })
            })
            .await
    }
}
