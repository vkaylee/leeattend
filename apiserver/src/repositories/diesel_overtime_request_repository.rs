use crate::errors::AppError;
use crate::models::{OvertimeRequest, OvertimeRequestEmployee};
use crate::repositories::OvertimeRequestRepository;
use crate::schema::{overtime_request_employees, overtime_requests};
use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

pub struct DieselOvertimeRequestRepository {
    diesel_pools: crate::diesel_pool::DieselPools,
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
        if request.workspace_id() != auth_token.workspace_id() {
            return Err(AppError::Forbidden("error.tenant_mismatch".into()));
        }

        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        conn.transaction::<_, AppError, _>(|conn| {
            Box::pin(async move {
                let req: OvertimeRequest = diesel::insert_into(overtime_requests::table)
                    .values(&request)
                    .get_result(&mut *conn)
                    .await
                    .map_err(|e| {
                        AppError::Internal(format!("Failed to create overtime request: {e}"))
                    })?;

                if !employees.is_empty() {
                    diesel::insert_into(overtime_request_employees::table)
                        .values(&employees)
                        .execute(&mut *conn)
                        .await
                        .map_err(|e| {
                            AppError::Internal(format!(
                                "Failed to create overtime request employees: {e}"
                            ))
                        })?;
                }

                Ok(req)
            })
        })
        .await
    }

    async fn update_request(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        request: OvertimeRequest,
    ) -> Result<OvertimeRequest, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let updated = diesel::update(
            overtime_requests::table
                .filter(overtime_requests::id.eq(request.id()))
                .filter(overtime_requests::workspace_id.eq(request.workspace_id())),
        )
        .set((
            overtime_requests::status.eq(request.status()),
            overtime_requests::current_step.eq(request.current_step()),
            overtime_requests::reconciled_at.eq(request.reconciled_at()),
            overtime_requests::reconciled_by.eq(request.reconciled_by()),
            overtime_requests::updated_at.eq(chrono::Utc::now()),
        ))
        .get_result(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to update overtime request: {e}")))?;

        Ok(updated)
    }

    async fn get_request(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        workspace_id: Uuid,
        request_id: Uuid,
    ) -> Result<Option<OvertimeRequest>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let req = overtime_requests::table
            .filter(overtime_requests::id.eq(request_id))
            .filter(overtime_requests::workspace_id.eq(workspace_id))
            .first::<OvertimeRequest>(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("Failed to get overtime request: {e}")))?;

        Ok(req)
    }

    async fn get_request_employees(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        request_id: Uuid,
    ) -> Result<Vec<OvertimeRequestEmployee>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let emps = overtime_request_employees::table
            .filter(overtime_request_employees::overtime_request_id.eq(request_id))
            .filter(
                overtime_request_employees::overtime_request_id.eq_any(
                    overtime_requests::table
                        .select(overtime_requests::id)
                        .filter(overtime_requests::workspace_id.eq(auth_token.workspace_id())),
                ),
            )
            .load::<OvertimeRequestEmployee>(&mut *conn)
            .await
            .map_err(|e| {
                AppError::Internal(format!("Failed to get overtime request employees: {e}"))
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
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let updated = diesel::update(
            overtime_request_employees::table
                .filter(overtime_request_employees::overtime_request_id.eq(request_id))
                .filter(
                    overtime_request_employees::employment_id.eq_any(
                        crate::schema::employments::table
                            .select(crate::schema::employments::id)
                            .filter(crate::schema::employments::employee_id.eq(employee_id))
                            .filter(
                                crate::schema::employments::workspace_id
                                    .eq(auth_token.workspace_id()),
                            ),
                    ),
                )
                .filter(
                    overtime_request_employees::overtime_request_id.eq_any(
                        overtime_requests::table
                            .select(overtime_requests::id)
                            .filter(overtime_requests::workspace_id.eq(auth_token.workspace_id())),
                    ),
                ),
        )
        .set((
            overtime_request_employees::consent_status.eq(consent_status),
            overtime_request_employees::consent_at.eq(chrono::Utc::now()),
            overtime_request_employees::decline_reason.eq(reason),
        ))
        .get_result(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to update employee consent: {e}")))?;

        Ok(updated)
    }

    async fn list_requests(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<OvertimeRequest>, i64), AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let workspace_id = auth_token.workspace_id();

        let total: i64 = overtime_requests::table
            .filter(overtime_requests::workspace_id.eq(workspace_id))
            .count()
            .get_result(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to count overtime requests: {e}")))?;

        let results = overtime_requests::table
            .filter(overtime_requests::workspace_id.eq(workspace_id))
            .order(overtime_requests::created_at.desc())
            .limit(limit)
            .offset((page - 1) * limit)
            .load::<OvertimeRequest>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to list overtime requests: {e}")))?;

        Ok((results, total))
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
        let _repo = DieselOvertimeRequestRepository::new(pools);
    }

    #[tokio::test]
    async fn test_overtime_request_repository_methods() {
        let test_db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://user:pass@localhost:5432/leeattend_test_db".to_string()
        });

        let pools = DieselPools::for_test(&test_db_url);
        let repo = DieselOvertimeRequestRepository::new(pools.clone());
        let ws_id = Uuid::new_v4();
        let auth_token = crate::diesel_pool::TenantAuthToken::system_override(ws_id);
        let emp_id = Uuid::new_v4();

        // Check if database is available by attempting to get a connection.
        // If it is not available (e.g. during offline unit tests), we skip the rest of the test.
        let mut conn = if let Ok(c) = pools.tenant_transaction_conn(&auth_token).await {
            c
        } else {
            println!("Database not available, skipping integration assertions");
            return;
        };

        // 1. Arrange: Insert workspace, location, and employee to satisfy foreign key constraints
        {
            // Insert Workspace
            let workspace_entity = crate::models::diesel::core::WorkspaceEntity {
                id: ws_id,
                name: "Test Workspace".to_string(),
                created_at: Some(chrono::Utc::now()),
                date_format: "YYYY-MM-DD".to_string(),
                time_format: "HH:mm".to_string(),
                timezone: "UTC".to_string(),
                theme_primary_color: None,
                default_language: "en".to_string(),
                logo_url: None,
                tagline: None,
                custom_db_url_encrypted: None,
            };
            diesel::insert_into(crate::schema::workspaces::table)
                .values(&workspace_entity)
                .execute(&mut *conn)
                .await
                .expect("insert workspace");

            // Insert Work Location
            let loc_id = Uuid::new_v4();
            let work_location_entity = crate::models::diesel::work_location::WorkLocationEntity {
                id: loc_id,
                workspace_id: ws_id,
                name: "Test Location".to_string(),
                address: None,
                timezone: "UTC".to_string(),
                is_headquarters: true,
                status: "active".to_string(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };
            diesel::insert_into(crate::schema::work_locations::table)
                .values(&work_location_entity)
                .execute(&mut *conn)
                .await
                .expect("insert location");

            // Insert Employee
            let employee_entity = crate::models::diesel::employee::EmployeeEntity {
                id: emp_id,
                workspace_id: ws_id,
                user_id: None,
                employee_id_number: "EMP001".to_string(),
                is_active: true,
                created_at: Some(chrono::Utc::now()),
                updated_at: Some(chrono::Utc::now()),
                tags: None,
                full_name: "Test Employee".to_string(),
                recruitment_source_id: None,
                deleted_at: None,
                hire_date: chrono::Utc::now().date_naive(),
                termination_date: None,
                original_hire_date: chrono::Utc::now().date_naive(),
            };
            diesel::insert_into(crate::schema::employees::table)
                .values(&employee_entity)
                .execute(&mut *conn)
                .await
                .expect("insert employee");

            // Insert Employer
            let employer_id = Uuid::new_v4();
            let employer_entity = crate::models::diesel::employer::EmployerEntity {
                id: employer_id,
                workspace_id: ws_id,
                name: "Test Employer".to_string(),
                tax_code: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                legal_address: None,
                deleted_at: None,
                parent_id: None,
                relationship_type: None,
                business_type: None,
                registration_number: None,
                phone: None,
                email: None,
                website: None,
                incorporation_date: None,
                country_code: None,
                currency_code: None,
            };
            diesel::insert_into(crate::schema::employers::table)
                .values(&employer_entity)
                .execute(&mut *conn)
                .await
                .expect("insert employer");

            // Insert Employment
            let employment_entity = crate::models::diesel::employment::EmploymentEntity {
                id: emp_id,
                workspace_id: ws_id,
                employee_id: emp_id,
                employer_id,
                primary_location_id: Some(loc_id),
                department_id: None,
                status: "active".to_string(),
                start_date: chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
                end_date: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                deleted_at: None,
                job_title: None,
                employment_type: "full_time".to_string(),
                manager_employment_id: None,
                notes: None,
            };
            diesel::insert_into(crate::schema::employments::table)
                .values(&employment_entity)
                .execute(&mut *conn)
                .await
                .expect("insert employment");
        }

        // Drop the connection to release it back to the pool before running repository methods
        drop(conn);

        let req_id = Uuid::new_v4();
        let request = OvertimeRequest::new(
            req_id,
            ws_id,
            Uuid::new_v4(), // requested_by_user_id (no constraint check)
            None,
            chrono::NaiveDate::from_ymd_opt(2023, 10, 10).unwrap(),
            None,
            None,
            120,
            "test reason".to_string(),
            crate::models::OtRequestStatus::PendingApproval,
            None,
            1,
            None,
            None,
            None,
            chrono::Utc::now(),
            chrono::Utc::now(),
            None,
        );

        let req_employee = OvertimeRequestEmployee::new(
            Uuid::new_v4(),
            req_id,
            emp_id,
            "pending".to_string(),
            None,
            None,
            None,
            None,
            chrono::Utc::now(),
        );

        // 2. Act & Assert: Create request with an employee link
        let created = repo
            .create_request(&auth_token, request.clone(), vec![req_employee.clone()])
            .await
            .expect("Failed to create request");
        assert_eq!(created.id(), request.id());
        assert_eq!(created.reason(), "test reason");
        assert_eq!(created.planned_minutes(), 120);

        // Act & Assert: Get Request
        let retrieved = repo
            .get_request(&auth_token, ws_id, request.id())
            .await
            .expect("Failed to get request");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().reason(), "test reason");

        // Act & Assert: Get Request Employees
        let emps = repo
            .get_request_employees(&auth_token, request.id())
            .await
            .expect("Failed to get request employees");
        assert_eq!(emps.len(), 1);
        assert_eq!(emps[0].employment_id(), emp_id);
        assert_eq!(emps[0].consent_status(), "pending");

        // Act & Assert: Update Employee Consent
        let updated_emp = repo
            .update_employee_consent(
                &auth_token,
                request.id(),
                emp_id,
                "accepted",
                Some("Approved by worker".to_string()),
            )
            .await
            .expect("Failed to update employee consent");
        assert_eq!(updated_emp.consent_status(), "accepted");
        assert_eq!(updated_emp.decline_reason(), Some("Approved by worker"));

        // Act & Assert: Update Request (change status/step)
        let mut request_to_update = request.clone();
        request_to_update.set_status(crate::models::OtRequestStatus::Approved);
        request_to_update.set_current_step(2);
        let updated_req = repo
            .update_request(&auth_token, request_to_update)
            .await
            .expect("Failed to update request");
        assert_eq!(
            updated_req.status(),
            crate::models::OtRequestStatus::Approved
        );
        assert_eq!(updated_req.current_step(), 2);

        // Act & Assert: List Requests
        let (list, total) = repo
            .list_requests(&auth_token, 1, 10)
            .await
            .expect("Failed to list requests");
        assert_eq!(total, 1);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id(), request.id());
    }

    #[tokio::test]
    async fn test_overtime_request_repository_errors() {
        let test_db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://user:pass@localhost:5432/leeattend_test_db".to_string()
        });

        let pools = DieselPools::for_test(&test_db_url);
        let repo = DieselOvertimeRequestRepository::new(pools.clone());
        let ws_id = Uuid::new_v4();
        let auth_token = crate::diesel_pool::TenantAuthToken::system_override(ws_id);
        let emp_id = Uuid::new_v4();

        let req_id = Uuid::new_v4();
        let request = OvertimeRequest::new(
            req_id,
            ws_id,
            Uuid::new_v4(),
            None,
            chrono::NaiveDate::from_ymd_opt(2023, 10, 10).unwrap(),
            None,
            None,
            120,
            "test reason".to_string(),
            crate::models::OtRequestStatus::PendingApproval,
            None,
            1,
            None,
            None,
            None,
            chrono::Utc::now(),
            chrono::Utc::now(),
            None,
        );

        // 1. Assert workspace mismatch error (runnable even without DB availability)
        let mismatched_ws_id = Uuid::new_v4();
        let mismatched_request = OvertimeRequest::new(
            req_id,
            mismatched_ws_id,
            Uuid::new_v4(),
            None,
            chrono::NaiveDate::from_ymd_opt(2023, 10, 10).unwrap(),
            None,
            None,
            120,
            "mismatched reason".to_string(),
            crate::models::OtRequestStatus::PendingApproval,
            None,
            1,
            None,
            None,
            None,
            chrono::Utc::now(),
            chrono::Utc::now(),
            None,
        );
        let create_err = repo
            .create_request(&auth_token, mismatched_request, vec![])
            .await;
        assert!(create_err.is_err());
        assert!(matches!(create_err.unwrap_err(), AppError::Forbidden(_)));

        // Check DB availability to perform database-dependent assertions
        let db_available = pools.tenant_transaction_conn(&auth_token).await.is_ok();
        if !db_available {
            println!("Database not available, skipping database error assertions");
            return;
        }

        // 2. Assert get request returns None for non-existent requests
        let non_existent_id = Uuid::new_v4();
        let get_res = repo.get_request(&auth_token, ws_id, non_existent_id).await;
        assert!(get_res.is_ok());
        assert!(get_res.unwrap().is_none());

        // 3. Assert get request employees returns empty vector for non-existent requests
        let get_emps_res = repo
            .get_request_employees(&auth_token, non_existent_id)
            .await;
        assert!(get_emps_res.is_ok());
        assert!(get_emps_res.unwrap().is_empty());

        // 4. Assert update request of non-existent request fails
        let update_res = repo.update_request(&auth_token, request.clone()).await;
        assert!(update_res.is_err());

        // 5. Assert update employee consent of non-existent employee/request fails
        let update_consent_res = repo
            .update_employee_consent(&auth_token, req_id, emp_id, "accepted", None)
            .await;
        assert!(update_consent_res.is_err());
    }
}
