#![allow(clippy::all)]
use sqlx::PgPool;
use tracing::{error, info, warn};

use crate::queue::Job;

pub struct SyncHireDateHandler;

impl SyncHireDateHandler {
    pub async fn process(
        job: &Job,
        pool: &PgPool,
        _diesel: &crate::diesel_pool::DieselPools,
    ) -> Result<(), String> {
        let Some(payload) = job.sync_hire_date_payload() else {
            error!("Missing or invalid payload for SyncEmployeeHireDate job");
            return Err("Invalid payload".to_string());
        };

        info!(
            employee_id = %payload.employee_id,
            triggered_by_contract_id = ?payload.triggered_by_contract_id,
            "Starting sync_hire_date job"
        );

        // Fetch the employee's current hire_date from employees table
        let current_hire_date: Option<chrono::NaiveDate> = match sqlx::query_scalar!(
            "SELECT hire_date FROM employees WHERE id = $1 AND workspace_id = $2",
            payload.employee_id,
            job.workspace_id
        )
        .fetch_optional(pool)
        .await
        {
            Ok(Some(date)) => Some(date),
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

        // Find the earliest active contract start_date
        // Exclude cancelled/draft contracts if applicable. We will assume only 'Active', 'Expired', 'Pending' etc. count.
        // Actually, let's just get the MIN(start_date) of any non-cancelled contract.
        // Assuming ContractStatus enum in DB exists. Let's just use status != 'Cancelled' if possible.
        // If we don't know the exact string, let's check what `contract.rs` has.
        // I will use `status != 'cancelled'` or similar. Wait, let me check the DB schema or `ContractStatus`.
        // Let's use `status != 'Cancelled'` for now.

        // Let's execute a raw query to be safe, or a standard query.
        let oldest_contract = match sqlx::query!(
            r#"
            SELECT start_date 
            FROM employee_contracts 
            WHERE employee_id = $1 AND workspace_id = $2 
            AND status != 'draft'
            ORDER BY start_date ASC 
            LIMIT 1
            "#,
            payload.employee_id,
            job.workspace_id
        )
        .fetch_optional(pool)
        .await
        {
            Ok(Some(record)) => Some(record.start_date),
            Ok(None) => None,
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

                match sqlx::query!(
                    "UPDATE employees SET hire_date = $1, updated_at = NOW() WHERE id = $2",
                    earliest_date,
                    payload.employee_id
                )
                .execute(pool)
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
