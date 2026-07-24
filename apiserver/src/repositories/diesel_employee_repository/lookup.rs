use super::*;
use crate::errors::AppError;
use crate::models::diesel::employee::EmployeeEntity;
use crate::models::Employee;
use crate::schema::employees;
use diesel::prelude::*;
use diesel_async::AsyncConnection;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

impl DieselEmployeeRepository {
    #[allow(clippy::too_many_lines)]
    pub(super) async fn bulk_create_employees_skip_duplicates_impl(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        new_employees: &[Employee],
    ) -> Result<(), AppError> {
        if new_employees.is_empty() {
            return Ok(());
        }
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        // 1. Get unique input numbers (case-insensitive deduplication)
        let mut unique_emps = std::collections::HashMap::new();
        for emp in new_employees {
            let key = emp.employee_id_number().to_lowercase();
            unique_emps.entry(key).or_insert(emp);
        }

        let numbers: Vec<String> = unique_emps.keys().cloned().collect();

        // 2. Find which ones already exist in the DB
        let existing = self
            .get_employee_ids_by_numbers_impl(auth_token, &numbers)
            .await?;

        // 3. Filter out those that already exist
        let mut to_insert = Vec::new();
        for (key, emp) in &unique_emps {
            if !existing.contains_key(key) {
                let entity: EmployeeEntity = (*emp).clone().into();
                to_insert.push(entity);
            }
        }

        tracing::debug!("bulk_create_employees_skip_duplicates: workspace_id={}, new_employees_len={}, unique_emps_keys={:?}, existing={:?}, to_insert_len={}", workspace_id, new_employees.len(), unique_emps.keys().collect::<Vec<_>>(), existing, to_insert.len());

        // 4. Perform chunked insertion inside a transaction
        conn.transaction(|conn| {
            Box::pin(async move {
                if !to_insert.is_empty() {
                    for chunk in to_insert.chunks(1000) {
                        tracing::debug!("inserting chunk of size {}", chunk.len());
                        let res = diesel::insert_into(employees::table)
                            .values(chunk)
                            .execute(&mut *conn)
                            .await;
                        tracing::debug!("insert res={:?}", res);
                        res.map_err(AppError::from)?;
                    }
                }

                use crate::schema::employers;
                use crate::schema::work_locations;

                let default_employer_id: Option<Uuid> = diesel_async::RunQueryDsl::first(
                    employers::table
                        .filter(employers::workspace_id.eq(workspace_id))
                        .select(employers::id),
                    &mut *conn,
                )
                .await
                .optional()
                .map_err(|e| AppError::Internal(format!("db select first employer: {e}")))?;
                let default_employer_id = default_employer_id.unwrap_or_else(Uuid::new_v4);

                let hq_id: Option<Uuid> = diesel_async::RunQueryDsl::first(
                    work_locations::table
                        .filter(work_locations::workspace_id.eq(workspace_id))
                        .filter(work_locations::is_headquarters.eq(true))
                        .select(work_locations::id),
                    &mut *conn,
                )
                .await
                .optional()
                .unwrap_or(None);

                // Insert employments & contracts for each inserted employee
                let mut employments_to_insert = Vec::new();
                let mut contracts_to_insert = Vec::new();
                for emp in unique_emps.values() {
                    if !existing.contains_key(&emp.employee_id_number().to_lowercase()) {
                        let employment_id = Uuid::new_v4();
                        employments_to_insert.push(
                            crate::models::diesel::employment::EmploymentEntity {
                                id: employment_id,
                                workspace_id,
                                employee_id: emp.id(),
                                employer_id: default_employer_id,
                                primary_location_id: hq_id,
                                department_id: None,
                                status: "active".to_string(),
                                start_date: emp.created_at().date_naive(),
                                end_date: None,
                                created_at: chrono::Utc::now(),
                                updated_at: chrono::Utc::now(),
                                deleted_at: None,
                                job_title: None,
                                employment_type: "full_time".to_string(),
                                manager_employment_id: None,
                                notes: None,
                                termination_reason: None,
                            },
                        );
                        contracts_to_insert.push(
                            crate::models::diesel::employee_contract::EmployeeContractEntity {
                                id: Uuid::new_v4(),
                                workspace_id,
                                contract_number: None,
                                contract_type: crate::models::enums::ContractType::Indefinite,
                                status: crate::models::enums::ContractStatus::Active,
                                start_date: emp.created_at().date_naive(),
                                end_date: None,
                                document_id: None,
                                notes: Some(
                                    "Hệ thống tự động khởi tạo thỏa thuận làm việc khi import"
                                        .to_string(),
                                ),
                                created_at: chrono::Utc::now(),
                                updated_at: chrono::Utc::now(),
                                deleted_at: None,
                                employment_id,
                            },
                        );
                    }
                }

                if !employments_to_insert.is_empty() {
                    use crate::schema::employments;
                    for chunk in employments_to_insert.chunks(1000) {
                        diesel::insert_into(employments::table)
                            .values(chunk)
                            .execute(&mut *conn)
                            .await
                            .map_err(|e| {
                                AppError::Internal(format!("db employments insert: {e}"))
                            })?;
                    }
                }

                if !contracts_to_insert.is_empty() {
                    for chunk in contracts_to_insert.chunks(1000) {
                        diesel::insert_into(crate::schema::employee_contracts::table)
                            .values(chunk)
                            .execute(&mut *conn)
                            .await
                            .map_err(|e| AppError::Internal(format!("db contracts insert: {e}")))?;
                    }
                }

                Ok::<(), AppError>(())
            })
        })
        .await?;

        Ok(())
    }

    pub(super) async fn get_managed_departments_impl(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        user_id: Uuid,
    ) -> Result<Vec<Uuid>, AppError> {
        let workspace_id = auth_token.workspace_id();
        use crate::schema::employments;

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let result: Vec<Option<Uuid>> = employees::table
            .inner_join(employments::table)
            .select(employments::department_id)
            .filter(employees::workspace_id.eq(workspace_id))
            .filter(employees::deleted_at.is_null())
            .filter(employments::deleted_at.is_null())
            .filter(employees::user_id.eq(user_id))
            .distinct()
            .load(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result.into_iter().flatten().collect())
    }

    pub(super) async fn get_employee_id_by_user_id_impl(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        user_id: Uuid,
    ) -> Result<Option<Uuid>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        use crate::schema::employments;
        let result: Option<Uuid> = employees::table
            .select(employees::id)
            .filter(employees::workspace_id.eq(workspace_id))
            .filter(employees::deleted_at.is_null())
            .filter(employees::user_id.eq(user_id))
            .filter(diesel::dsl::exists(
                employments::table
                    .filter(employments::employee_id.eq(employees::id))
                    .filter(employments::status.eq("active"))
                    .filter(employments::deleted_at.is_null()),
            ))
            .order(employees::updated_at.desc())
            .first(&mut *conn)
            .await
            .optional()
            .map_err(AppError::from)?;

        Ok(result)
    }

    pub(super) async fn get_employee_timezone_fallback_impl(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
    ) -> Result<chrono_tz::Tz, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        #[derive(Debug, diesel::QueryableByName)]
        struct TzRow {
            #[diesel(sql_type = diesel::sql_types::Text)]
            timezone: String,
        }

        let query = r"
            SELECT wl.timezone as timezone
            FROM employments emp
            JOIN work_locations wl ON wl.id = emp.primary_location_id
            WHERE (emp.employee_id = $1 OR emp.id = $1) AND emp.workspace_id = $2
            ORDER BY CASE WHEN emp.status = 'active' THEN 0 ELSE 1 END, emp.start_date DESC
            LIMIT 1
        ";

        let row_result = diesel::sql_query(query)
            .bind::<diesel::sql_types::Uuid, _>(employee_id)
            .bind::<diesel::sql_types::Uuid, _>(workspace_id)
            .get_result::<TzRow>(&mut *conn)
            .await;

        let tz_str = match row_result {
            Ok(row) => row.timezone,
            Err(diesel::result::Error::NotFound) => {
                // Fallback to workspace timezone from system DB
                let mut system_conn = self
                    .diesel_pools
                    .api
                    .get()
                    .await
                    .map_err(|e| AppError::Internal(format!("db api pool: {e}")))?;

                #[derive(diesel::QueryableByName)]
                struct WorkspaceTz {
                    #[diesel(sql_type = diesel::sql_types::Text)]
                    timezone: String,
                }

                let ws_tz: Result<WorkspaceTz, _> =
                    diesel::sql_query("SELECT timezone::text FROM workspaces WHERE id = $1")
                        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
                        .get_result(&mut *system_conn)
                        .await;

                match ws_tz {
                    Ok(w) => w.timezone,
                    Err(e) => {
                        tracing::warn!("Failed to query workspace timezone fallback for workspace_id={}: {:?}. Defaulting to UTC.", workspace_id, e);
                        "UTC".to_string()
                    }
                }
            }
            Err(e) => return Err(AppError::Internal(format!("db: {e}"))),
        };

        Ok(crate::utils::tz::parse_workspace_tz(&tz_str))
    }

    pub(super) async fn get_employee_timezone_at_date_impl(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        tentative_date: chrono::NaiveDate,
    ) -> Result<chrono_tz::Tz, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        #[derive(Debug, diesel::QueryableByName)]
        struct TzRow {
            #[diesel(sql_type = diesel::sql_types::Text)]
            timezone: String,
        }

        let query = r"
            SELECT COALESCE(
                (SELECT host_wl.timezone
                 FROM employee_assignments ea
                 JOIN work_locations host_wl ON host_wl.id = ea.host_location_id
                 WHERE ea.employment_id = emp.id AND ea.status = 'active'
                   AND ea.start_date <= $3
                   AND (ea.end_date IS NULL OR ea.end_date >= $3)
                 ORDER BY ea.created_at DESC LIMIT 1),
                primary_wl.timezone
            ) as timezone
            FROM employments emp
            JOIN work_locations primary_wl ON primary_wl.id = emp.primary_location_id
            WHERE (emp.employee_id = $1 OR emp.id = $1) AND emp.workspace_id = $2
            ORDER BY CASE WHEN emp.status = 'active' THEN 0 ELSE 1 END, emp.start_date DESC
            LIMIT 1
        ";

        let row_result = diesel::sql_query(query)
            .bind::<diesel::sql_types::Uuid, _>(employee_id)
            .bind::<diesel::sql_types::Uuid, _>(workspace_id)
            .bind::<diesel::sql_types::Date, _>(tentative_date)
            .get_result::<TzRow>(&mut *conn)
            .await;

        let tz_str = match row_result {
            Ok(row) => row.timezone,
            Err(diesel::result::Error::NotFound) => {
                // Fallback to workspace timezone from system DB
                let mut system_conn = self
                    .diesel_pools
                    .api
                    .get()
                    .await
                    .map_err(|e| AppError::Internal(format!("db api pool: {e}")))?;

                #[derive(diesel::QueryableByName)]
                struct WorkspaceTz {
                    #[diesel(sql_type = diesel::sql_types::Text)]
                    timezone: String,
                }

                let ws_tz: Result<WorkspaceTz, _> =
                    diesel::sql_query("SELECT timezone::text FROM workspaces WHERE id = $1")
                        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
                        .get_result(&mut *system_conn)
                        .await;

                match ws_tz {
                    Ok(w) => w.timezone,
                    Err(e) => {
                        tracing::warn!("Failed to query workspace timezone fallback for workspace_id={}: {:?}. Defaulting to UTC.", workspace_id, e);
                        "UTC".to_string()
                    }
                }
            }
            Err(e) => return Err(AppError::Internal(format!("db: {e}"))),
        };

        Ok(crate::utils::tz::parse_workspace_tz(&tz_str))
    }

    pub(super) async fn get_headquarters_location_id_impl(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Option<Uuid>, AppError> {
        let workspace_id = auth_token.workspace_id();
        use crate::schema::work_locations;
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let hq_id: Option<Uuid> = work_locations::table
            .filter(work_locations::workspace_id.eq(workspace_id))
            .filter(work_locations::is_headquarters.eq(true))
            .select(work_locations::id)
            .first(&mut *conn)
            .await
            .optional()
            .map_err(AppError::from)?;

        Ok(hq_id)
    }
}
