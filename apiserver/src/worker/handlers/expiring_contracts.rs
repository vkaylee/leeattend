use sqlx::PgPool;
use tracing::{error, info};

use crate::queue::Job;

pub struct ExpiringContractsHandler;

impl ExpiringContractsHandler {
    #[allow(clippy::too_many_lines)]
    pub async fn handle(db: &PgPool, job: &Job) -> Result<(), String> {
        let _payload = job
            .expiring_contracts_payload()
            .ok_or("Missing expiring contracts payload")?;

        info!(
            job_id = %job.job_id,
            "Starting expiring contracts check job"
        );

        // 1. Get all workspaces
        let workspaces = sqlx::query!("SELECT id FROM workspaces")
            .fetch_all(db)
            .await
            .map_err(|e| format!("Failed to fetch workspaces: {e}"))?;

        for ws in workspaces {
            let ws_id = ws.id;

            // 2. Get expiring contracts in next 30 days
            // Note: We use purely PostgreSQL to find contracts that expire in <= 30 days
            // We check only active contracts.
            let contracts = sqlx::query!(
                r#"
                SELECT 
                    ec.id as contract_id,
                    ec.employee_id,
                    e.full_name,
                    e.employee_id_number,
                    ec.end_date
                FROM employee_contracts ec
                JOIN employees e ON ec.employee_id = e.id
                WHERE ec.workspace_id = $1
                  AND ec.status = 'active'
                  AND ec.end_date IS NOT NULL
                  AND ec.end_date <= CURRENT_DATE + INTERVAL '30 days'
                  AND ec.end_date >= CURRENT_DATE
                "#,
                ws_id
            )
            .fetch_all(db)
            .await
            .unwrap_or_default();

            if contracts.is_empty() {
                continue;
            }

            // 3. Find admins/HR to notify
            // Target users with global roles or specific permissions (manage_employees, manage_contracts)
            let admins = sqlx::query!(
                r#"
                SELECT DISTINCT wm.user_id 
                FROM workspace_members wm
                JOIN workspace_roles wr ON wm.role_id = wr.id
                LEFT JOIN workspace_role_permissions wrp ON wr.id = wrp.role_id
                WHERE wm.workspace_id = $1 
                  AND (wr.scope_type = 'global' OR wrp.permission_name IN ('manage_employees', 'manage_contracts'))
                "#,
                ws_id
            )
            .fetch_all(db)
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

                let emp_code = contract.employee_id_number;

                for admin in &admins {
                    let notif_id = crate::utils::id::new_id();
                    let title = "Hợp đồng sắp hết hạn".to_string();
                    let content = format!(
                        "Hợp đồng của nhân viên {} ({}) sẽ hết hạn vào ngày {}. Vui lòng kiểm tra và gia hạn.",
                        contract.full_name,
                        emp_code,
                        end_date.format("%d/%m/%Y")
                    );
                    let link = format!("/workspace/employees/{}", contract.employee_id);

                    // Check if a notification for this specific contract has already been created recently
                    // to prevent spamming notifications every day for the same contract
                    let existing = sqlx::query_scalar!(
                        r#"
                        SELECT EXISTS(
                            SELECT 1 FROM notifications 
                            WHERE user_id = $1 
                              AND type = 'contract_expiring'
                              AND link = $2
                              AND created_at >= NOW() - INTERVAL '30 days'
                        )
                        "#,
                        admin.user_id,
                        link
                    )
                    .fetch_one(db)
                    .await
                    .unwrap_or(Some(false))
                    .unwrap_or(false);

                    if existing {
                        continue;
                    }

                    let res = sqlx::query!(
                        r#"
                        INSERT INTO notifications (id, user_id, type, title, content, is_read, link, created_at)
                        VALUES ($1, $2, 'contract_expiring', $3, $4, false, $5, NOW())
                        "#,
                        notif_id,
                        admin.user_id,
                        title,
                        content,
                        link
                    )
                    .execute(db)
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
}
