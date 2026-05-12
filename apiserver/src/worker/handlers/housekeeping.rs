use chrono::{Datelike, NaiveDate};
use sqlx::{Executor, PgPool};
use tracing::{error, info};

use crate::queue::Job;

pub struct NightlyHousekeepingHandler;

impl NightlyHousekeepingHandler {
    pub async fn handle(
        db: &PgPool,
        _diesel: &crate::diesel_pool::DieselPools,
        job: &Job,
    ) -> Result<(), String> {
        let payload = job
            .housekeeping_payload()
            .ok_or("Missing housekeeping payload")?;
        let ref_date = payload.reference_date;

        info!(
            job_id = %job.job_id,
            ref_date = %ref_date,
            "Starting nightly housekeeping job"
        );

        // Ensure partitions for current month + next 3 months
        let mut target_date = NaiveDate::from_ymd_opt(ref_date.year(), ref_date.month(), 1)
            .ok_or("Invalid reference date")?;

        for _ in 0..4 {
            let year = target_date.year();
            let month = target_date.month();

            let next_month = if month == 12 { 1 } else { month + 1 };
            let next_year = if month == 12 { year + 1 } else { year };

            let from_date_str = format!("{year:04}-{month:02}-01");
            let to_date_str = format!("{next_year:04}-{next_month:02}-01");

            let audit_table = format!("audit_logs_y{year:04}m{month:02}");
            let staging_table = format!("attendance_staging_logs_y{year:04}m{month:02}");

            let query = format!(
                r#"
                CREATE TABLE IF NOT EXISTS {audit_table} PARTITION OF audit_logs FOR VALUES FROM ('{from_date_str}') TO ('{to_date_str}');
                CREATE TABLE IF NOT EXISTS {staging_table} PARTITION OF attendance_staging_logs FOR VALUES FROM ('{from_date_str}') TO ('{to_date_str}');
                "#
            );

            if let Err(e) = db.execute(query.as_str()).await {
                error!(error = %e, year, month, "Failed to create partition");
                return Err(format!(
                    "Failed to create partition for {year}-{month}: {e}"
                ));
            }
            info!(year, month, "Ensured partitions exist");

            target_date = NaiveDate::from_ymd_opt(next_year, next_month, 1)
                .ok_or("Invalid next target date")?;
        }

        // Retention Policy: Drop partitions older than 2 years (24 months).
        // We drop partitions for the 25th, 26th, and 27th months ago to provide a safety buffer.
        for i in 25..=27 {
            let mut total_months = ref_date.year() * 12 + (ref_date.month() as i32) - 1;
            total_months -= i;
            let drop_year = total_months / 12;
            let drop_month = (total_months % 12) as u32 + 1;

            let audit_table = format!("audit_logs_y{drop_year:04}m{drop_month:02}");
            let staging_table = format!("attendance_staging_logs_y{drop_year:04}m{drop_month:02}");

            let query = format!(
                r#"
                DROP TABLE IF EXISTS {audit_table};
                DROP TABLE IF EXISTS {staging_table};
                "#
            );

            if let Err(e) = db.execute(query.as_str()).await {
                error!(error = %e, drop_year, drop_month, "Failed to drop old partition");
                // Don't fail the job, just log it so other maintenance can continue
            } else {
                info!(drop_year, drop_month, "Ensured old partitions are dropped");
            }
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
}
