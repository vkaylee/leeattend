use tracing::{error, info};

use crate::queue::Job;

pub struct ExpiringContractsHandler;

impl ExpiringContractsHandler {
    #[allow(clippy::too_many_lines)]
    pub async fn handle(deps: &crate::worker::WorkerDeps, job: &Job) -> Result<(), String> {
        let _payload = job
            .expiring_contracts_payload()
            .ok_or("Missing expiring contracts payload")?;

        info!(
            job_id = %job.job_id,
            "Starting expiring contracts check job"
        );

        // 1. Get all workspaces
        let workspaces = deps
            .workspace_repo
            .get_all_workspaces()
            .await
            .map_err(|e| format!("Failed to fetch workspaces: {e}"))?;

        for ws in workspaces {
            let ws_id = ws.id();

            // 2. Get expiring contracts in next 30 days
            // Note: We use purely PostgreSQL to find contracts that expire in <= 30 days
            // We check only active contracts.
            let contracts = deps
                .contract_repo
                .get_expiring_contracts(
                    &crate::diesel_pool::TenantAuthToken::system_override(ws_id),
                    30,
                )
                .await
                .unwrap_or_default();

            if contracts.is_empty() {
                continue;
            }

            // 3. Find admins/HR to notify
            // Target users with global roles or specific permissions (manage_employees, manage_contracts)
            let admins = deps
                .workspace_repo
                .get_admins_with_permissions(ws_id, &["manage_employees", "manage_contracts"])
                .await
                .unwrap_or_default();

            if admins.is_empty() {
                continue;
            }

            // 4. Batch-fetch employments and employees to avoid N+1 queries
            let employment_ids: Vec<uuid::Uuid> = contracts
                .iter()
                .map(crate::models::contract::EmployeeContract::employment_id)
                .collect();

            let employments = deps
                .employment_repo
                .get_employments_by_ids(
                    &crate::diesel_pool::TenantAuthToken::system_override(ws_id),
                    &employment_ids,
                )
                .await
                .unwrap_or_default();

            let employments_map: std::collections::HashMap<
                uuid::Uuid,
                crate::models::employment::Employment,
            > = employments.into_iter().map(|e| (e.id(), e)).collect();

            let employee_ids: Vec<uuid::Uuid> = employments_map
                .values()
                .map(crate::models::employment::Employment::employee_id)
                .collect();

            let employees = deps
                .employee_repo
                .get_employees_by_ids(
                    &employee_ids,
                    &crate::diesel_pool::TenantAuthToken::system_override(ws_id),
                )
                .await
                .unwrap_or_default();

            let employees_map: std::collections::HashMap<uuid::Uuid, crate::models::Employee> =
                employees.into_iter().map(|emp| (emp.id(), emp)).collect();

            // 5. Create notifications
            for contract in contracts {
                // Ensure end_date exists since we filtered by IS NOT NULL
                let Some(end_date) = contract.end_date() else {
                    continue;
                };

                let Some(employment) = employments_map.get(&contract.employment_id()) else {
                    continue;
                };
                let employee_id = employment.employee_id();

                let Some(emp) = employees_map.get(&employee_id) else {
                    continue;
                };

                let emp_code = emp.employee_id_number().to_string();

                for admin_id in &admins {
                    let notif_id = crate::utils::id::new_id();
                    let title = "Hợp đồng sắp hết hạn".to_string();
                    let content = format!(
                        "Hợp đồng của nhân viên {} ({}) sẽ hết hạn vào ngày {}. Vui lòng kiểm tra và gia hạn.",
                        emp.full_name(),
                        emp_code,
                        end_date.format("%d/%m/%Y")
                    );
                    let link = format!("/workspace/employees/{employee_id}");

                    // Check if a notification for this specific contract has already been created recently
                    let existing = deps
                        .user_repo
                        .check_recent_notification_exists(*admin_id, "contract_expiring", &link, 30)
                        .await
                        .unwrap_or(false);

                    if existing {
                        continue;
                    }

                    let res = deps
                        .user_repo
                        .create_notification(&crate::models::Notification {
                            id: notif_id,
                            user_id: *admin_id,
                            r#type: "contract_expiring".to_string(),
                            title,
                            content,
                            is_read: false,
                            link: Some(link),
                            created_at: chrono::Utc::now(),
                        })
                        .await;

                    if let Err(e) = res {
                        error!(error = %e, "Failed to create notification");
                    }
                }
            }
        }

        info!("Expiring contracts check completed");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::queue::{ExpiringContractsPayload, JobPayload, TaskType};
    use chrono::NaiveDate;
    use uuid::Uuid;

    #[test]
    fn test_payload_extraction() {
        let job = Job::new(
            Uuid::new_v4(),
            0,
            TaskType::ExpiringContractsCheck,
            Uuid::new_v4(),
            JobPayload::ExpiringContractsCheck(ExpiringContractsPayload {
                reference_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            }),
        );

        let payload = job.expiring_contracts_payload().expect("should extract");
        assert_eq!(
            payload.reference_date,
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()
        );
    }

    #[test]
    fn test_wrong_payload_returns_none() {
        let job = Job::new(
            Uuid::new_v4(),
            0,
            TaskType::NightlyHousekeeping,
            Uuid::new_v4(),
            JobPayload::NightlyHousekeeping(crate::queue::HousekeepingPayload {
                reference_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            }),
        );

        assert!(job.expiring_contracts_payload().is_none());
    }

    #[tokio::test]
    async fn test_expiring_contracts_handler_success() {
        use crate::models::{Employee, NewEmploymentParams, NewWorkspaceParams, Workspace, contract::EmployeeContract, contract::NewEmployeeContractParams};
        use crate::repositories::employee::MockEmployeeRepository;
        use crate::repositories::employee_contract_repository::MockEmployeeContractRepository;
        use crate::repositories::user_repository::MockUserRepository;
        use crate::repositories::workspace_repository::MockWorkspaceRepository;
        use crate::worker::test_utils::MockWorkerDepsBuilder;

        let ws_id = Uuid::new_v4();
        let emp_id = Uuid::new_v4();
        let admin_id = Uuid::new_v4();

        let mut workspace_repo = MockWorkspaceRepository::new();
        workspace_repo
            .expect_get_all_workspaces()
            .returning(move || {
                Box::pin(async move {
                    Ok(vec![Workspace::new(NewWorkspaceParams {
            id: ws_id,
            name: "Test Workspace".to_string(),
            date_format: "YYYY-MM-DD".to_string(),
            time_format: "HH:mm".to_string(),
            timezone: "UTC".to_string(),
            theme_primary_color: "#000000".to_string(),
            default_language: "en".to_string(),
            logo_url: None,
            tagline: None,
            holiday_overnight_strategy: "split".to_string(),
            default_holiday_ot_multiplier: rust_decimal::Decimal::new(15, 1),
            created_at: chrono::Utc::now(),
            custom_db_url_encrypted: None,
        })])
                })
            });

        workspace_repo
            .expect_get_admins_with_permissions()
            .returning(move |_, _| Box::pin(async move { Ok(vec![admin_id]) }));

        let employment_id = Uuid::new_v4();
        let mut contract_repo = MockEmployeeContractRepository::new();
        contract_repo
            .expect_get_expiring_contracts()
            .returning(move |_, _| {
                Ok(vec![EmployeeContract::new(NewEmployeeContractParams {
                    id: Uuid::new_v4(),
                    workspace_id: ws_id,
                    employment_id: employment_id,
                    contract_number: None,
                    contract_type: crate::models::enums::ContractType::FixedTerm,
                    status: crate::models::enums::ContractStatus::Active,
                    start_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
                    end_date: Some(NaiveDate::from_ymd_opt(2026, 1, 15).unwrap()),
                    document_id: None,
                    notes: None,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                })])
            });

        let mut emp_repo = MockEmployeeRepository::new();
        emp_repo
            .expect_get_employees_by_ids()
            .returning(move |ids, _| {
                if ids.contains(&emp_id) {
                    Ok(vec![Employee::new(
                        emp_id,
                        ws_id,
                        None,
                        "Test Employee".to_string(),
                        "EMP001".to_string(),
                        vec![],
                        chrono::Utc::now(),
                        chrono::Utc::now(),
                        None,
                    )])
                } else {
                    Ok(vec![])
                }
            });

        let mut user_repo = MockUserRepository::new();
        user_repo
            .expect_check_recent_notification_exists()
            .returning(|_, _, _, _| Ok(false));
        user_repo.expect_create_notification().returning(|_| Ok(()));

        let mut employment_repo =
            crate::repositories::employment_repository::MockEmploymentRepository::new();
        employment_repo
            .expect_get_employments_by_ids()
            .returning(move |_, ids| {
                if ids.contains(&employment_id) {
                    Ok(vec![crate::models::employment::Employment::new(NewEmploymentParams {
                        id: employment_id,
                        workspace_id: ws_id,
                        employee_id: emp_id,
                        employer_id: Uuid::new_v4(),
                        primary_location_id: None,
                        department_id: None,
                        status: "active".to_string(),
                        start_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
                        end_date: None,
                        job_title: None,
                        employment_type: "full_time".to_string(),
                        manager_employment_id: None,
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                        termination_reason: None,
                        notes: None,
                    })])
                } else {
                    Ok(vec![])
                }
            });

        let mut builder = MockWorkerDepsBuilder::new();
        builder.workspace_repo = workspace_repo;
        builder.contract_repo = contract_repo;
        builder.employee_repo = emp_repo;
        builder.employment_repo = employment_repo;
        builder.user_repo = user_repo;

        let deps = builder.build();

        let job = Job::new(
            Uuid::new_v4(),
            0,
            TaskType::ExpiringContractsCheck,
            Uuid::new_v4(),
            JobPayload::ExpiringContractsCheck(ExpiringContractsPayload {
                reference_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            }),
        );

        let result = ExpiringContractsHandler::handle(&deps, &job).await;

        assert!(result.is_ok(), "Handler failed: {:?}", result.err());
    }
}
