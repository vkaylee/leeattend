use tracing::{error, info, warn};

use crate::queue::Job;

pub struct SyncHireDateHandler;

impl SyncHireDateHandler {
    pub async fn process(job: &Job, deps: &crate::worker::WorkerDeps) -> Result<(), String> {
        let Some(payload) = job.sync_hire_date_payload() else {
            error!("Missing or invalid payload for SyncEmployeeHireDate job");
            return Err("Invalid payload".to_string());
        };

        info!(
            employee_id = %payload.employee_id,
            triggered_by_contract_id = ?payload.triggered_by_contract_id,
            "Starting sync_hire_date job"
        );

        let current_hire_date = match deps
            .employee_repo
            .get_employee_by_id(
                payload.employee_id,
                &crate::diesel_pool::TenantAuthToken::system_override(job.workspace_id),
            )
            .await
        {
            Ok(Some(emp)) => Some(emp.hire_date),
            Ok(None) => {
                warn!(
                    employee_id = %payload.employee_id,
                    "Employee not found, skipping sync"
                );
                return Ok(());
            }
            Err(e) => {
                error!("Database error fetching employee: {:?}", e);
                return Err(format!("DB error: {e}"));
            }
        };

        let oldest_contract = match deps
            .contract_repo
            .list_employee_contracts(
                &crate::diesel_pool::TenantAuthToken::system_override(job.workspace_id),
                payload.employee_id,
            )
            .await
        {
            Ok(contracts) => contracts
                .into_iter()
                .filter(|c| c.status != crate::models::enums::ContractStatus::Draft)
                .last()
                .map(|c| c.start_date),
            Err(e) => {
                error!("Database error fetching oldest contract: {:?}", e);
                return Err(format!("DB error: {e}"));
            }
        };

        if let Some(earliest_date) = oldest_contract {
            if current_hire_date == Some(earliest_date) {
                info!("Employee hire_date is already correctly synced");
                Ok(())
            } else {
                info!(
                    old_hire_date = ?current_hire_date,
                    new_hire_date = %earliest_date,
                    "Updating employee hire_date based on earliest contract"
                );

                match deps
                    .employee_repo
                    .update_employee(
                        payload.employee_id,
                        &crate::diesel_pool::TenantAuthToken::system_override(job.workspace_id),
                        &crate::models::UpdateEmployeeRequest {
                            user_id: None,
                            full_name: None,
                            employee_id_number: None,
                            hire_date: Some(earliest_date),
                            termination_date: None,
                            tags: None,
                            is_active: None,
                            max_concurrent_workers: None,
                            recruitment_source_id: None,
                            primary_location_id: None,
                        },
                    )
                    .await
                {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        error!("Failed to update employee hire_date: {:?}", e);
                        Err(format!("Update error: {e}"))
                    }
                }
            }
        } else {
            warn!(
                employee_id = %payload.employee_id,
                current_hire_date = ?current_hire_date,
                "No valid contracts found for employee. Leaving hire_date unchanged."
            );
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::enums::{ContractStatus, ContractType};
    use crate::models::{contract::EmployeeContract, Employee};
    use crate::queue::{JobPayload, SyncHireDatePayload, TaskType};
    use crate::repositories::employee::MockEmployeeRepository;
    use crate::repositories::employee_contract_repository::MockEmployeeContractRepository;
    use crate::worker::test_utils::MockWorkerDepsBuilder;
    use chrono::{NaiveDate, Utc};
    use uuid::Uuid;

    #[tokio::test]
    async fn test_sync_hire_date_handler_success() {
        let ws_id = Uuid::new_v4();
        let emp_id = Uuid::new_v4();

        let mut emp_repo = MockEmployeeRepository::new();
        emp_repo.expect_get_employee_by_id().returning(move |_, _| {
            Box::pin(async move {
                Ok(Some(Employee {
                    id: emp_id,
                    workspace_id: ws_id,
                    user_id: None,
                    full_name: "Test Employee".to_string(),
                    employee_id_number: "EMP001".to_string(),
                    original_hire_date: NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
                    hire_date: NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
                    termination_date: None,
                    tags: vec![],
                    is_active: true,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    department_name: None,
                    primary_location_id: uuid::Uuid::new_v4(),
                }))
            })
        });

        emp_repo
            .expect_update_employee()
            .returning(|_, _, _| Box::pin(async { Ok(None) }));

        let mut contract_repo = MockEmployeeContractRepository::new();
        contract_repo
            .expect_list_employee_contracts()
            .returning(move |_, _| {
                Ok(vec![EmployeeContract {
                    id: Uuid::new_v4(),
                    workspace_id: ws_id,
                    employee_id: emp_id,
                    contract_number: None,
                    contract_type: ContractType::Indefinite,
                    status: ContractStatus::Active,
                    start_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                    end_date: None,
                    document_id: None,
                    notes: None,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                }])
            });

        let mut builder = MockWorkerDepsBuilder::new();
        builder.employee_repo = emp_repo;
        builder.contract_repo = contract_repo;

        let deps = builder.build();

        let job = Job {
            job_id: Uuid::new_v4(),
            parent_job_id: Uuid::new_v4(),
            batch_index: 0,
            task_type: TaskType::SyncEmployeeHireDate,
            workspace_id: ws_id,
            payload: JobPayload::SyncEmployeeHireDate(SyncHireDatePayload {
                employee_id: emp_id,
                triggered_by_contract_id: None,
            }),
            priority: crate::queue::JobPriority::Normal,
            created_at: Utc::now(),
            retry_count: 0,
            max_retries: 3,
        };

        let result = SyncHireDateHandler::process(&job, &deps).await;

        assert!(result.is_ok(), "Handler failed: {:?}", result.err());
    }
}
