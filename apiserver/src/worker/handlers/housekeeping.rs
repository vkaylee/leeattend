use chrono::{Datelike, NaiveDate};
use tracing::{error, info};

use crate::queue::Job;

pub struct NightlyHousekeepingHandler;

impl NightlyHousekeepingHandler {
    #[allow(
        clippy::too_many_lines,
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::cast_sign_loss
    )]
    pub async fn handle(deps: &crate::worker::WorkerDeps, job: &Job) -> Result<(), String> {
        let payload = job
            .housekeeping_payload()
            .ok_or("Missing housekeeping payload")?;
        let ref_date = payload.reference_date;

        info!(
            job_id = %job.job_id,
            ref_date = %ref_date,
            "Starting nightly housekeeping job"
        );

        let workspaces = deps
            .workspace_repo
            .get_all_workspaces()
            .await
            .map_err(|e| e.to_string())?;

        let mut errors = Vec::new();
        use futures_util::stream::StreamExt;

        let results = futures_util::stream::iter(workspaces)
            .map(|workspace| async move {
                    let tenant_id = workspace.id();
                    let mut queries = Vec::new();

                    // Ensure partitions for current month + next 3 months
                    let Some(mut target_date) =
                        NaiveDate::from_ymd_opt(ref_date.year(), ref_date.month(), 1)
                    else {
                        return Err(format!("Tenant {tenant_id}: Invalid reference date"));
                    };

                    let mut target_date_err = false;
                    for _ in 0..4 {
                        let year = target_date.year();
                        let month = target_date.month();

                        let next_month = if month == 12 { 1 } else { month + 1 };
                        let next_year = if month == 12 { year + 1 } else { year };

                        let from_date_str = format!("{year:04}-{month:02}-01");
                        let to_date_str = format!("{next_year:04}-{next_month:02}-01");

                        let audit_table = format!("audit_logs_y{year:04}m{month:02}");
                        let staging_table = format!("attendance_staging_logs_y{year:04}m{month:02}");

                        queries.push(format!("CREATE TABLE IF NOT EXISTS {audit_table} PARTITION OF audit_logs FOR VALUES FROM ('{from_date_str}') TO ('{to_date_str}')"));
                        queries.push(format!("CREATE TABLE IF NOT EXISTS {staging_table} PARTITION OF attendance_staging_logs FOR VALUES FROM ('{from_date_str}') TO ('{to_date_str}')"));

                        target_date = if let Some(date) = NaiveDate::from_ymd_opt(next_year, next_month, 1)
                        {
                            date
                        } else {
                            target_date_err = true;
                            break;
                        };
                    }
                    if target_date_err {
                        return Err(format!("Tenant {tenant_id}: Invalid next target date"));
                    }

                    // Retention Policy: Drop partitions older than 3 years (36 months) for audit_logs
                    // We drop partitions for the 37th, 38th, and 39th months ago to provide a safety buffer.
                    for i in 37..=39 {
                        let mut total_months = ref_date.year() * 12 + (ref_date.month() as i32) - 1;
                        total_months -= i;
                        let drop_year = total_months / 12;
                        let drop_month = (total_months % 12) as u32 + 1;

                        let audit_table = format!("audit_logs_y{drop_year:04}m{drop_month:02}");
                        queries.push(format!("DROP TABLE IF EXISTS {audit_table}"));
                    }

                    // Drop partitions older than 2 years (24 months) for attendance_staging_logs
                    for i in 25..=27 {
                        let mut total_months = ref_date.year() * 12 + (ref_date.month() as i32) - 1;
                        total_months -= i;
                        let drop_year = total_months / 12;
                        let drop_month = (total_months % 12) as u32 + 1;

                        let staging_table =
                            format!("attendance_staging_logs_y{drop_year:04}m{drop_month:02}");
                        queries.push(format!("DROP TABLE IF EXISTS {staging_table}"));
                    }

                    // Clean up expired session tokens (T1 data / Redis/DB sync)
                    queries.push("DELETE FROM sessions WHERE expires_at < NOW()".to_string());

                    // Clean up temporary import jobs older than 7 days
                    queries.push(format!(
                        "DELETE FROM attendance_import_jobs WHERE workspace_id = '{tenant_id}' AND created_at < NOW() - INTERVAL '7 days'"
                    ));

                    // Clean up attendance and leave records older than 5 years for employees terminated >= 5 years ago
                    queries.push(format!(
                        "DELETE FROM time_entries WHERE workspace_id = '{tenant_id}' AND employment_id IN (SELECT id FROM employments WHERE workspace_id = '{tenant_id}' AND status = 'terminated' AND end_date <= CURRENT_DATE - INTERVAL '5 years') AND punch_in_time <= CURRENT_TIMESTAMP - INTERVAL '5 years'"
                    ));
                    queries.push(format!(
                        "DELETE FROM attendance_daily_summaries WHERE workspace_id = '{tenant_id}' AND employment_id IN (SELECT id FROM employments WHERE workspace_id = '{tenant_id}' AND status = 'terminated' AND end_date <= CURRENT_DATE - INTERVAL '5 years') AND date <= CURRENT_DATE - INTERVAL '5 years'"
                    ));
                    queries.push(format!(
                        "DELETE FROM leave_requests WHERE employment_id IN (SELECT id FROM employments WHERE workspace_id = '{tenant_id}' AND status = 'terminated' AND end_date <= CURRENT_DATE - INTERVAL '5 years') AND end_date <= CURRENT_DATE - INTERVAL '5 years'"
                    ));
                    queries.push(format!(
                        "DELETE FROM leave_accrual_logs WHERE workspace_id = '{tenant_id}' AND employment_id IN (SELECT id FROM employments WHERE workspace_id = '{tenant_id}' AND status = 'terminated' AND end_date <= CURRENT_DATE - INTERVAL '5 years') AND accrual_date <= CURRENT_DATE - INTERVAL '5 years'"
                    ));

                    // Anonymize Employee PII for employees terminated >= 1 year ago (GDPR right to erasure)
                    queries.push(format!(
                        "UPDATE employees SET full_name = 'Anonymized Employee', employee_id_number = 'ANON_' || md5(id::text) WHERE workspace_id = '{tenant_id}' AND full_name <> 'Anonymized Employee' AND EXISTS (SELECT 1 FROM employments WHERE employee_id = employees.id AND workspace_id = '{tenant_id}' AND status = 'terminated' AND end_date <= CURRENT_DATE - INTERVAL '1 year') AND NOT EXISTS (SELECT 1 FROM employments WHERE employee_id = employees.id AND workspace_id = '{tenant_id}' AND (status = 'active' OR end_date > CURRENT_DATE - INTERVAL '1 year' OR end_date IS NULL))"
                    ));
                    queries.push(format!(
                        "UPDATE employee_profiles SET phone = NULL, address = NULL, national_id = NULL, bank_name = NULL, bank_account_number = NULL, bank_branch = NULL WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = '{tenant_id}' AND EXISTS (SELECT 1 FROM employments WHERE employee_id = employees.id AND workspace_id = '{tenant_id}' AND status = 'terminated' AND end_date <= CURRENT_DATE - INTERVAL '1 year') AND NOT EXISTS (SELECT 1 FROM employments WHERE employee_id = employees.id AND workspace_id = '{tenant_id}' AND (status = 'active' OR end_date > CURRENT_DATE - INTERVAL '1 year' OR end_date IS NULL)))"
                    ));

                    match deps
                        .system_repo
                        .execute_tenant_partition_queries_batch(tenant_id, &queries)
                        .await
                    {
                        Ok(()) => {
                            // Throttling: short pause to release connections and allow other operations to execute
                            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                            Ok(())
                        }
                        Err(e) => {
                            error!(error = %e, tenant_id = %tenant_id, "Failed to execute batch partition queries for workspace");
                            Err(format!("Tenant {tenant_id}: {e}"))
                        }
                    }
            })
            .buffer_unordered(5)
            .collect::<Vec<Result<(), String>>>()
            .await;

        for res in results {
            if let Err(e) = res {
                errors.push(e);
            }
        }

        if !errors.is_empty() {
            return Err(format!(
                "Nightly housekeeping job failed with errors: {}",
                errors.join("; ")
            ));
        }

        info!(
            job_id = %job.job_id,
            "Nightly housekeeping job completed successfully"
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::queue::{HousekeepingPayload, JobPayload, JobPriority, TaskType};
    use crate::worker::test_utils::MockWorkerDepsBuilder;
    use uuid::Uuid;

    #[test]
    fn test_payload_extraction() {
        let job = Job::with_priority(
            Uuid::new_v4(),
            0,
            TaskType::NightlyHousekeeping,
            Uuid::new_v4(),
            JobPayload::NightlyHousekeeping(HousekeepingPayload {
                reference_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            }),
            JobPriority::Low,
        );

        let payload = job.housekeeping_payload().expect("should extract");
        assert_eq!(
            payload.reference_date,
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()
        );
        assert_eq!(job.priority, JobPriority::Low);
    }

    #[tokio::test]
    async fn test_housekeeping_handle_success() {
        let mut builder = MockWorkerDepsBuilder::new();
        builder
            .workspace_repo
            .expect_get_all_workspaces()
            .returning(|| {
                Box::pin(async move {
                    Ok(vec![crate::models::workspace::Workspace::new(NewWorkspaceParams {
            id: Uuid::new_v4(),
            name: "Test Workspace".to_string(),
            date_format: "YYYY-MM-DD".to_string(),
            time_format: "24h".to_string(),
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

        builder
            .system_repo
            .expect_execute_tenant_partition_queries_batch()
            .returning(|_, _| Box::pin(async { Ok(()) }));
        let deps = builder.build();

        let job = Job::with_priority(
            Uuid::new_v4(),
            0,
            TaskType::NightlyHousekeeping,
            Uuid::new_v4(),
            JobPayload::NightlyHousekeeping(HousekeepingPayload {
                reference_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            }),
            JobPriority::Low,
        );

        let result = NightlyHousekeepingHandler::handle(&deps, &job).await;
        assert!(result.is_ok());
    }
}
