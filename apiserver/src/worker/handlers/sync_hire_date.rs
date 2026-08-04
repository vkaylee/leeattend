use tracing::{error, info, warn};
use uuid::Uuid;

use crate::queue::Job;

pub struct SyncHireDateHandler;

impl SyncHireDateHandler {
    #[allow(clippy::too_many_lines)]
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

        let _emp = match deps
            .employee_repo
            .get_employee_by_id(
                payload.employee_id,
                &crate::diesel_pool::TenantAuthToken::system_override(job.workspace_id),
            )
            .await
        {
            Ok(Some(emp)) => emp,
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

        let employments = match deps
            .employment_repo
            .list_employee_employments(
                &crate::diesel_pool::TenantAuthToken::system_override(job.workspace_id),
                payload.employee_id,
            )
            .await
        {
            Ok(list) => list,
            Err(e) => {
                error!("Database error fetching employments: {:?}", e);
                return Err(format!("DB error: {e}"));
            }
        };

        let employment_ids: Vec<Uuid> = employments
            .iter()
            .map(crate::models::employment::Employment::id)
            .collect();

        let mut contracts_map = if employment_ids.is_empty() {
            std::collections::HashMap::new()
        } else {
            match deps
                .contract_repo
                .list_contracts_batch(
                    &crate::diesel_pool::TenantAuthToken::system_override(job.workspace_id),
                    &employment_ids,
                )
                .await
            {
                Ok(map) => map,
                Err(e) => {
                    error!("Database error batch fetching contracts: {:?}", e);
                    return Err(format!("DB error: {e}"));
                }
            }
        };

        for emp in employments {
            let employment_id = emp.id();
            let current_start_date = emp.start_date();

            let contracts = contracts_map.remove(&employment_id).unwrap_or_default();
            let mut oldest_date = None;
            for c in contracts {
                if c.status() != crate::models::enums::ContractStatus::Draft {
                    let c_date = c.start_date();
                    match oldest_date {
                        None => oldest_date = Some(c_date),
                        Some(d) if c_date < d => oldest_date = Some(c_date),
                        _ => {}
                    }
                }
            }

            if let Some(earliest_date) = oldest_date {
                if current_start_date == earliest_date {
                    info!(
                        employment_id = %employment_id,
                        "Employment start_date is already correctly synced"
                    );
                } else {
                    info!(
                        employment_id = %employment_id,
                        old_start_date = ?current_start_date,
                        new_start_date = %earliest_date,
                        "Updating employment start_date based on earliest contract"
                    );

                    if let Err(e) = deps
                        .employment_repo
                        .update_employment(
                            &crate::diesel_pool::TenantAuthToken::system_override(job.workspace_id),
                            employment_id,
                            &crate::models::employment::UpdateEmploymentRequest {
                                employer_id: None,
                                primary_location_id: None,
                                department_id: None,
                                status: None,
                                start_date: Some(earliest_date),
                                end_date: None,
                                job_title: None,
                                employment_type: None,
                                manager_employment_id: None,
                                notes: None,
                                termination_reason: None,
                            },
                        )
                        .await
                    {
                        error!("Failed to update employment start_date: {:?}", e);
                        return Err(format!("Update error: {e}"));
                    }
                }
            } else {
                warn!(
                    employment_id = %employment_id,
                    current_start_date = ?current_start_date,
                    "No valid contracts found for employment. Leaving start_date unchanged."
                );
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::enums::{ContractStatus, ContractType};
    use crate::models::{Employee, NewEmploymentParams, contract::EmployeeContract, contract::NewEmployeeContractParams};
    use crate::queue::{JobPayload, SyncHireDatePayload, TaskType};
    use crate::repositories::employee::MockEmployeeRepository;
    use crate::repositories::employee_contract_repository::MockEmployeeContractRepository;
    use crate::repositories::employment_repository::MockEmploymentRepository;
    use crate::worker::test_utils::MockWorkerDepsBuilder;
    use chrono::{NaiveDate, Utc};
    use uuid::Uuid;

    #[tokio::test]
    async fn test_sync_hire_date_handler_success() {
        let ws_id = Uuid::new_v4();
        let emp_id = Uuid::new_v4();

        let mut emp_repo = MockEmployeeRepository::new();
        emp_repo.expect_get_employee_by_id().returning(move |_, _| {
            Ok(Some(Employee::new(
                emp_id,
                ws_id,
                None,
                "Test Employee".to_string(),
                "EMP001".to_string(),
                vec![],
                Utc::now(),
                Utc::now(),
                None,
            )))
        });

        let employment_id = Uuid::new_v4();
        let employer_id = Uuid::new_v4();
        let mut employment_repo = MockEmploymentRepository::new();
        employment_repo
            .expect_list_employee_employments()
            .returning(move |_, _| {
                Ok(vec![crate::models::employment::Employment::new(NewEmploymentParams {
                    id: employment_id,
                    workspace_id: ws_id,
                    employee_id: emp_id,
                    employer_id: employer_id,
                    primary_location_id: None,
                    department_id: None,
                    status: "active".to_string(),
                    start_date: NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
                    end_date: None,
                    job_title: None,
                    employment_type: "full_time".to_string(),
                    manager_employment_id: None,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    termination_reason: None,
                    notes: None,
                })])
            });

        employment_repo
            .expect_update_employment()
            .returning(move |_, _, _| {
                Ok(crate::models::employment::Employment::new(NewEmploymentParams {
                    id: employment_id,
                    workspace_id: ws_id,
                    employee_id: emp_id,
                    employer_id: employer_id,
                    primary_location_id: None,
                    department_id: None,
                    status: "active".to_string(),
                    start_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                    end_date: None,
                    job_title: None,
                    employment_type: "full_time".to_string(),
                    manager_employment_id: None,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    termination_reason: None,
                    notes: None,
                }))
            });

        let mut contract_repo = MockEmployeeContractRepository::new();
        contract_repo
            .expect_list_contracts_batch()
            .returning(move |_, employment_ids| {
                let mut map = std::collections::HashMap::new();
                if employment_ids.contains(&employment_id) {
                    map.insert(
                        employment_id,
                        vec![EmployeeContract::new(NewEmployeeContractParams {
                            id: Uuid::new_v4(),
                            workspace_id: ws_id,
                            employment_id: employment_id,
                            contract_number: None,
                            contract_type: ContractType::Indefinite,
                            status: ContractStatus::Active,
                            start_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                            end_date: None,
                            document_id: None,
                            notes: None,
                            created_at: Utc::now(),
                            updated_at: Utc::now(),
                        })],
                    );
                }
                Ok(map)
            });

        let mut builder = MockWorkerDepsBuilder::new();
        builder.employee_repo = emp_repo;
        builder.employment_repo = employment_repo;
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
            trace_context: None,
        };

        let result = SyncHireDateHandler::process(&job, &deps).await;

        assert!(result.is_ok(), "Handler failed: {:?}", result.err());
    }
}
