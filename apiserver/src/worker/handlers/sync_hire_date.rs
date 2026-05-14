#![allow(clippy::all)]
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
            .get_employee_by_id(payload.employee_id, job.workspace_id)
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
            .list_employee_contracts(job.workspace_id, payload.employee_id)
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
                        job.workspace_id,
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
