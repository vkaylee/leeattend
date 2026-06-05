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

            // 4. Create notifications
            for contract in contracts {
                // Ensure end_date exists since we filtered by IS NOT NULL
                let Some(end_date) = contract.end_date else {
                    continue;
                };

                // We need employee code and name for the message. We could fetch employee info
                let emp = deps
                    .employee_repo
                    .get_employee_by_id(
                        contract.employee_id,
                        &crate::diesel_pool::TenantAuthToken::system_override(ws_id),
                    )
                    .await
                    .unwrap_or_default();

                let Some(emp) = emp else {
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
                    let link = format!("/workspace/employees/{}", contract.employee_id);

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
        use crate::models::{contract::EmployeeContract, Employee, Workspace};
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
                    Ok(vec![Workspace::new(
                        ws_id,
                        "Test Workspace".to_string(),
                        "YYYY-MM-DD".to_string(),
                        "HH:mm".to_string(),
                        "UTC".to_string(),
                        "#000000".to_string(),
                        "en".to_string(),
                        None,
                        None,
                        "split".to_string(),
                        rust_decimal::Decimal::new(15, 1),
                        chrono::Utc::now(),
                        None,
                    )])
                })
            });

        workspace_repo
            .expect_get_admins_with_permissions()
            .returning(move |_, _| Box::pin(async move { Ok(vec![admin_id]) }));

        let mut contract_repo = MockEmployeeContractRepository::new();
        contract_repo
            .expect_get_expiring_contracts()
            .returning(move |_, _| {
                Ok(vec![EmployeeContract {
                    id: Uuid::new_v4(),
                    workspace_id: ws_id,
                    employee_id: emp_id,
                    contract_number: None,
                    contract_type: crate::models::enums::ContractType::FixedTerm,
                    status: crate::models::enums::ContractStatus::Active,
                    start_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
                    end_date: Some(NaiveDate::from_ymd_opt(2026, 1, 15).unwrap()),
                    document_id: None,
                    notes: None,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                }])
            });

        let mut emp_repo = MockEmployeeRepository::new();
        emp_repo.expect_get_employee_by_id().returning(move |_, _| {
            Ok(Some(Employee::new(
                emp_id,
                ws_id,
                None,
                "Test Employee".to_string(),
                "EMP001".to_string(),
                NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
                None,
                vec![],
                true,
                chrono::Utc::now(),
                chrono::Utc::now(),
                None,
                uuid::Uuid::new_v4(),
                None,
            )))
        });

        let mut user_repo = MockUserRepository::new();
        user_repo
            .expect_check_recent_notification_exists()
            .returning(|_, _, _, _| Ok(false));
        user_repo.expect_create_notification().returning(|_| Ok(()));

        let mut builder = MockWorkerDepsBuilder::new();
        builder.workspace_repo = workspace_repo;
        builder.contract_repo = contract_repo;
        builder.employee_repo = emp_repo;
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
