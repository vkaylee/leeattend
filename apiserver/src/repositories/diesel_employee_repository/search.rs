use super::*;
use crate::diesel_pool::DieselPoolsTenantTxExt;
use crate::errors::AppError;
use crate::models::diesel::employee::EmployeeRow;
use crate::models::Employee;
use crate::schema::employees;
use diesel::prelude::*;
use diesel::sql_types::Uuid as UuidType;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

impl DieselEmployeeRepository {
    pub(super) async fn find_by_number_impl(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id_number: &str,
    ) -> Result<Option<Employee>, AppError> {
        let workspace_id = auth_token.workspace_id();
        self.diesel_pools
            .with_tenant_tx(auth_token, |conn| {
                Box::pin(async move {
        let normalized_id = crate::utils::normalize_employee_id(employee_id_number);
        let query = format!(
            "{EMPLOYEE_SELECT_QUERY} WHERE e.workspace_id = $1 AND e.employee_id_number = $2 AND e.deleted_at IS NULL"
        );
        let emp: Option<EmployeeRow> = diesel::sql_query(&query)
            .bind::<UuidType, _>(workspace_id)
            .bind::<diesel::sql_types::Text, _>(normalized_id)
            .get_result(conn)
            .await
            .optional()
            .map_err(AppError::from)?;

        Ok(emp.map(Into::into))

                })
            })
            .await
    }

    pub(super) async fn get_employee_by_id_impl(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Option<Employee>, AppError> {
        let workspace_id = auth_token.workspace_id();
        self.diesel_pools
            .with_tenant_tx(auth_token, |conn| {
                Box::pin(async move {
        let query = format!("{EMPLOYEE_SELECT_QUERY} WHERE e.id = $1 AND e.workspace_id = $2 AND e.deleted_at IS NULL");
        let emp: Option<EmployeeRow> = diesel::sql_query(&query)
            .bind::<UuidType, _>(id)
            .bind::<UuidType, _>(workspace_id)
            .get_result(conn)
            .await
            .optional()
            .map_err(AppError::from)?;

        Ok(emp.map(Into::into))

                })
            })
            .await
    }

    pub(super) async fn get_employee_by_id_number_impl(
        &self,
        id_number: &str,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Option<Employee>, AppError> {
        self.find_by_number_impl(auth_token, id_number).await
    }

    pub(super) async fn get_employees_by_ids_impl(
        &self,
        ids: &[Uuid],
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<Employee>, AppError> {
        let workspace_id = auth_token.workspace_id();
        self.diesel_pools
            .with_tenant_tx(auth_token, |conn| {
                Box::pin(async move {
        let query = format!("{EMPLOYEE_SELECT_QUERY} WHERE e.workspace_id = $1 AND e.id = ANY($2) AND e.deleted_at IS NULL");
        let emps: Vec<EmployeeRow> = diesel::sql_query(&query)
            .bind::<UuidType, _>(workspace_id)
            .bind::<diesel::sql_types::Array<UuidType>, _>(ids)
            .load(conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(emps.into_iter().map(Into::into).collect())

                })
            })
            .await
    }

    pub(super) async fn get_employees_by_id_numbers_impl(
        &self,
        id_numbers: &[String],
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<Employee>, AppError> {
        let workspace_id = auth_token.workspace_id();
        self.diesel_pools
            .with_tenant_tx(auth_token, |conn| {
                Box::pin(async move {
        let query = format!("{EMPLOYEE_SELECT_QUERY} WHERE e.workspace_id = $1 AND e.employee_id_number = ANY($2) AND e.deleted_at IS NULL");
        let emps: Vec<EmployeeRow> = diesel::sql_query(&query)
            .bind::<UuidType, _>(workspace_id)
            .bind::<diesel::sql_types::Array<diesel::sql_types::Text>, _>(id_numbers)
            .load(conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(emps.into_iter().map(Into::into).collect())

                })
            })
            .await
    }

    pub(super) async fn get_active_employees_impl(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<Employee>, AppError> {
        let workspace_id = auth_token.workspace_id();
        self.diesel_pools
            .with_tenant_tx(auth_token, |conn| {
                Box::pin(async move {
        let query = format!(
            "{EMPLOYEE_SELECT_QUERY} WHERE e.workspace_id = $1 AND EXISTS (SELECT 1 FROM employments em WHERE em.employee_id = e.id AND em.status = 'active' AND em.deleted_at IS NULL) AND e.deleted_at IS NULL ORDER BY e.full_name ASC"
        );
        let emps: Vec<EmployeeRow> = diesel::sql_query(&query)
            .bind::<UuidType, _>(workspace_id)
            .load(conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(emps.into_iter().map(Into::into).collect())

                })
            })
            .await
    }

    pub(super) async fn get_active_employee_ids_impl(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        filter_ids: Option<&Vec<Uuid>>,
    ) -> Result<Vec<Uuid>, AppError> {
        let workspace_id = auth_token.workspace_id();
        self.diesel_pools
            .with_tenant_tx(auth_token, |conn| {
                Box::pin(async move {
                    use crate::schema::employments;
                    let mut query = employees::table
                        .select(employees::id)
                        .filter(employees::workspace_id.eq(workspace_id))
                        .filter(employees::deleted_at.is_null())
                        .filter(diesel::dsl::exists(
                            employments::table
                                .filter(employments::employee_id.eq(employees::id))
                                .filter(employments::status.eq("active"))
                                .filter(employments::deleted_at.is_null()),
                        ))
                        .into_boxed();

                    if let Some(ids) = filter_ids {
                        query = query.filter(employees::id.eq_any(ids.clone()));
                    }

                    let result: Vec<Uuid> = query
                        .order(employees::created_at.asc())
                        .load(conn)
                        .await
                        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

                    Ok(result)
                })
            })
            .await
    }

    pub(super) async fn get_existing_employee_ids_by_numbers_impl(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id_numbers: &[String],
    ) -> Result<Vec<String>, AppError> {
        let workspace_id = auth_token.workspace_id();
        if id_numbers.is_empty() {
            return Ok(Vec::new());
        }

        self.diesel_pools
            .with_tenant_tx(auth_token, |conn| {
                Box::pin(async move {
                    let result: Vec<String> = employees::table
                        .select(employees::employee_id_number)
                        .filter(employees::workspace_id.eq(workspace_id))
                        .filter(employees::deleted_at.is_null())
                        .filter(employees::employee_id_number.eq_any(id_numbers))
                        .load(conn)
                        .await
                        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

                    Ok(result)
                })
            })
            .await
    }

    pub(super) async fn get_employee_ids_by_numbers_impl(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id_numbers: &[String],
    ) -> Result<std::collections::HashMap<String, Uuid>, AppError> {
        let workspace_id = auth_token.workspace_id();
        if id_numbers.is_empty() {
            return Ok(std::collections::HashMap::new());
        }

        self.diesel_pools
            .with_tenant_tx(auth_token, |conn| {
                Box::pin(async move {
                    let results: Vec<(String, Uuid)> = employees::table
                        .select((employees::employee_id_number, employees::id))
                        .filter(employees::workspace_id.eq(workspace_id))
                        .filter(employees::deleted_at.is_null())
                        .filter(employees::employee_id_number.eq_any(id_numbers))
                        .load(conn)
                        .await
                        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

                    let mut map = std::collections::HashMap::new();
                    for (num, id) in results {
                        map.insert(num, id);
                    }

                    Ok(map)
                })
            })
            .await
    }
}
