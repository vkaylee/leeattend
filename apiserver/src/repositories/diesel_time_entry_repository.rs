use async_trait::async_trait;
use chrono::{DateTime, Datelike, NaiveDate, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use std::collections::HashMap;
use uuid::Uuid;

/// Stable per-(employee, calendar-day) advisory-lock key.
///
/// Used with Postgres `pg_advisory_xact_lock` inside `replace_time_entries` so
/// that two concurrent reprocess runs for the same calendar day serialize their
/// DELETE+INSERT. Without this lock the two transactions can interleave
/// (both DELETE the old rows, both INSERT two new rows) and the daily report
/// ends up with 4 duplicated work sessions instead of 2.
///
/// Derivation: XOR-fold the two `u64` halves of the UUID with a bit-packed
/// (year:32, ordinal:32) view of the local date. Bijective per (year, day-of-
/// year), so distinct calendar days never collide.
pub(crate) fn reprocess_advisory_lock_key(employment_id: Uuid, date: NaiveDate) -> i64 {
    let (hi, lo) = employment_id.as_u64_pair();
    // Year offset so distinct calendar days never collide. Using `cast_signed`
    // makes the sign-preserving cast explicit to clippy's pedantic lint.
    let year = u64::try_from(date.year()).unwrap_or(0);
    let ordinal = date.ordinal(); // 1..366
    let date_key = (year << 32) | u64::from(ordinal);
    (hi ^ lo ^ date_key).cast_signed()
}

use crate::errors::AppError;
use crate::models::diesel::time_entry::{
    NewTimeEntryEntity, TimeEntryEntity, UpdateTimeEntryEntity,
};
use crate::models::{TimeEntry, TimeEntryLogFilter, TimeEntryWithEmployee};
use crate::repositories::time_entry_repository::TimeEntryRepository;
use crate::schema::{employees, employments, time_entries};

pub struct DieselTimeEntryRepository {
    diesel_pools: crate::diesel_pool::DieselPools,
}

impl DieselTimeEntryRepository {
    pub fn new(diesel_pools: crate::diesel_pool::DieselPools) -> Self {
        Self { diesel_pools }
    }
}

#[async_trait]
impl TimeEntryRepository for DieselTimeEntryRepository {
    async fn create_time_entry(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        entry: &TimeEntry,
    ) -> Result<(), AppError> {
        let _workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let new_entry = NewTimeEntryEntity {
            id: entry.id(),
            employment_id: entry.employment_id(),
            workspace_id: entry.workspace_id(),
            punch_in_time: entry.punch_in_time(),
            punch_out_time: entry.punch_out_time(),
            entry_type: entry.entry_type().to_string(),
            is_break_overtime: entry.is_break_overtime(),
            note: entry.note().map(String::from),
            template_id: entry.shift_id(),
        };

        diesel::insert_into(time_entries::table)
            .values(&new_entry)
            .execute(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(())
    }

    async fn get_time_entry_by_id(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
    ) -> Result<Option<TimeEntry>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let entry: Option<TimeEntryEntity> = time_entries::table
            .filter(time_entries::workspace_id.eq(workspace_id))
            .filter(time_entries::id.eq(id))
            .first(&mut *conn)
            .await
            .optional()
            .map_err(AppError::from)?;

        Ok(entry.map(Into::into))
    }

    async fn get_active_time_entry(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
    ) -> Result<Option<TimeEntry>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let entry: Option<TimeEntryEntity> = time_entries::table
            .filter(time_entries::workspace_id.eq(workspace_id))
            .filter(time_entries::employment_id.eq(employment_id))
            .filter(time_entries::punch_out_time.is_null())
            .order(time_entries::punch_in_time.desc())
            .first(&mut *conn)
            .await
            .optional()
            .map_err(AppError::from)?;

        Ok(entry.map(Into::into))
    }

    async fn get_latest_active_entry(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
    ) -> Result<Option<TimeEntry>, AppError> {
        let _workspace_id = auth_token.workspace_id();
        self.get_active_time_entry(auth_token, employment_id).await
    }

    async fn get_latest_entry_for_employment(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
    ) -> Result<Option<TimeEntry>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let entry: Option<TimeEntryEntity> = time_entries::table
            .filter(time_entries::workspace_id.eq(workspace_id))
            .filter(time_entries::employment_id.eq(employment_id))
            .order(time_entries::punch_in_time.desc())
            .first(&mut *conn)
            .await
            .optional()
            .map_err(AppError::from)?;

        Ok(entry.map(Into::into))
    }

    async fn update_time_entry(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        entry: &TimeEntry,
    ) -> Result<(), AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let update_model = UpdateTimeEntryEntity {
            punch_in_time: Some(entry.punch_in_time()),
            punch_out_time: Some(entry.punch_out_time()),
            entry_type: Some(entry.entry_type().to_string()),
            note: Some(entry.note().map(String::from)),
            updated_at: Some(Utc::now()),
        };

        diesel::update(time_entries::table)
            .filter(time_entries::id.eq(entry.id()))
            .filter(time_entries::workspace_id.eq(workspace_id))
            .set(&update_model)
            .execute(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(())
    }

    async fn delete_time_entry(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
    ) -> Result<(), AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        diesel::delete(time_entries::table)
            .filter(time_entries::id.eq(id))
            .filter(time_entries::workspace_id.eq(workspace_id))
            .execute(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(())
    }

    async fn get_employment_time_entries(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<TimeEntry>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let entries: Vec<TimeEntryEntity> = time_entries::table
            .filter(time_entries::workspace_id.eq(workspace_id))
            .filter(time_entries::employment_id.eq(employment_id))
            .filter(time_entries::punch_in_time.ge(from))
            .filter(time_entries::punch_in_time.le(to))
            .order(time_entries::punch_in_time.desc())
            .load(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entries.into_iter().map(Into::into).collect())
    }

    async fn get_paginated_logs(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        filter: TimeEntryLogFilter,
    ) -> Result<Vec<TimeEntryWithEmployee>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let mut query = time_entries::table
            .inner_join(employments::table.on(time_entries::employment_id.eq(employments::id)))
            .inner_join(employees::table.on(employments::employee_id.eq(employees::id)))
            .filter(time_entries::workspace_id.eq(workspace_id))
            .into_boxed();

        if let Some(emp_id) = filter.employment_id() {
            query = query.filter(time_entries::employment_id.eq(emp_id));
        }
        if let Some(from_date) = filter.from() {
            query = query.filter(time_entries::punch_in_time.ge(from_date));
        }
        if let Some(to_date) = filter.to() {
            query = query.filter(time_entries::punch_in_time.le(to_date));
        }

        let results = query
            .select((
                time_entries::all_columns,
                employees::full_name,
                employees::employee_id_number,
            ))
            .order(time_entries::punch_in_time.desc())
            .limit(filter.limit())
            .offset(filter.offset())
            .load::<(TimeEntryEntity, String, String)>(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(results
            .into_iter()
            .map(|(entry, full_name, id_number)| {
                let domain: TimeEntry = entry.into();
                TimeEntryWithEmployee::new(
                    domain.id(),
                    domain.employment_id(),
                    domain.workspace_id(),
                    domain.shift_id(),
                    domain.segment_id(),
                    domain.punch_in_time(),
                    domain.punch_out_time(),
                    domain.duration_minutes(),
                    domain.is_break_overtime(),
                    domain.entry_type(),
                    domain.note().map(String::from),
                    domain.generation_metadata().cloned(),
                    domain.created_at(),
                    domain.updated_at(),
                    full_name,
                    id_number,
                )
            })
            .collect())
    }

    async fn count_logs(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        p_employment_id: Option<Uuid>,
        p_from: Option<DateTime<Utc>>,
        p_to: Option<DateTime<Utc>>,
    ) -> Result<i64, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let mut query = time_entries::table
            .inner_join(employments::table.on(time_entries::employment_id.eq(employments::id)))
            .inner_join(employees::table.on(employments::employee_id.eq(employees::id)))
            .filter(time_entries::workspace_id.eq(workspace_id))
            .into_boxed();

        if let Some(emp_id) = p_employment_id {
            query = query.filter(time_entries::employment_id.eq(emp_id));
        }
        if let Some(from_date) = p_from {
            query = query.filter(time_entries::punch_in_time.ge(from_date));
        }
        if let Some(to_date) = p_to {
            query = query.filter(time_entries::punch_in_time.le(to_date));
        }

        let count: i64 = query
            .count()
            .get_result(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(count)
    }

    async fn get_distinct_work_days(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        tz: &str,
    ) -> Result<Vec<chrono::NaiveDate>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        #[derive(diesel::QueryableByName)]
        struct DateRow {
            #[diesel(sql_type = diesel::sql_types::Date)]
            date: chrono::NaiveDate,
        }

        let work_days: Vec<DateRow> = diesel::sql_query(
            r"
            SELECT DISTINCT(te.punch_in_time AT TIME ZONE $2)::date as date
            FROM time_entries te
            JOIN employments em ON te.employment_id = em.id
            JOIN employees e ON em.employee_id = e.id
            WHERE te.employment_id = $1
              AND e.workspace_id = $5
              AND te.punch_in_time >= $3
              AND te.punch_in_time <= $4
              AND te.entry_type = 'work'
            ",
        )
        .bind::<diesel::sql_types::Uuid, _>(employment_id)
        .bind::<diesel::sql_types::Text, _>(tz.to_string())
        .bind::<diesel::sql_types::Timestamptz, _>(start_time)
        .bind::<diesel::sql_types::Timestamptz, _>(end_time)
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .load(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(work_days.into_iter().map(|row| row.date).collect())
    }

    async fn get_workspace_time_entries_batch_by_employments(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_ids: &[Uuid],
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<HashMap<Uuid, Vec<TimeEntry>>, AppError> {
        let workspace_id = auth_token.workspace_id();
        if employment_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let results: Vec<TimeEntryEntity> = time_entries::table
            .filter(time_entries::workspace_id.eq(workspace_id))
            .filter(time_entries::employment_id.eq_any(employment_ids))
            .filter(time_entries::punch_in_time.ge(from))
            .filter(time_entries::punch_in_time.le(to))
            .order((
                time_entries::employment_id.asc(),
                time_entries::punch_in_time.asc(),
            ))
            .load(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let mut result_map: HashMap<Uuid, Vec<TimeEntry>> = HashMap::new();
        for entry in results {
            let domain: TimeEntry = entry.into();
            result_map
                .entry(domain.employment_id())
                .or_default()
                .push(domain);
        }

        Ok(result_map)
    }

    async fn replace_time_entries(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        entries: Vec<TimeEntry>,
    ) -> Result<(), AppError> {
        let workspace_id = auth_token.workspace_id();
        use diesel_async::AsyncConnection;

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        // Acquire a per-(employee, calendar-day) transaction advisory lock
        // before mutating the row set. Concurrent reprocess runs targeting the
        // same calendar day serialize here so a DELETE+INSERT cannot interleave
        // into a double-inserted set of sessions. Auto-released on COMMIT/ROLLBACK.
        // See `reprocess_advisory_lock_key` for derivation.
        let lock_key = reprocess_advisory_lock_key(employment_id, start_time.date_naive());

        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            Box::pin(async move {
                diesel::sql_query("SELECT pg_advisory_xact_lock($1)")
                    .bind::<diesel::sql_types::BigInt, _>(lock_key)
                    .execute(conn)
                    .await?;

                diesel::delete(
                    time_entries::table
                        .filter(time_entries::workspace_id.eq(workspace_id))
                        .filter(time_entries::employment_id.eq(employment_id))
                        .filter(time_entries::punch_in_time.ge(start_time))
                        .filter(time_entries::punch_in_time.lt(end_time)),
                )
                .execute(conn)
                .await?;

                if !entries.is_empty() {
                    let entities: Vec<TimeEntryEntity> = entries
                        .into_iter()
                        .map(|entry| TimeEntryEntity {
                            id: entry.id(),
                            workspace_id: entry.workspace_id(),
                            employment_id: entry.employment_id(),
                            template_id: entry.shift_id(),
                            punch_in_time: entry.punch_in_time(),
                            punch_out_time: entry.punch_out_time(),
                            duration_minutes: entry.duration_minutes(),
                            is_break_overtime: entry.is_break_overtime(),
                            entry_type: entry.entry_type().to_string(),
                            note: entry.note().map(String::from),
                            generation_metadata: entry.generation_metadata().cloned(),
                            created_at: Some(entry.created_at()),
                            updated_at: Some(entry.updated_at()),
                        })
                        .collect();

                    diesel::insert_into(time_entries::table)
                        .values(&entities)
                        .execute(conn)
                        .await?;
                }

                Ok(())
            })
        })
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
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
        let _repo = DieselTimeEntryRepository::new(pools);
        // DI connection test passes if it compiles
    }

    /// Verifies the lock key is a pure function of (employment_id, date):
    /// identical inputs yield the same key, distinct days yield distinct keys.
    #[test]
    fn test_advisory_lock_key_stable_and_unique() {
        use chrono::NaiveDate;

        let emp = Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap();
        let d1 = NaiveDate::from_ymd_opt(2026, 7, 3).unwrap();
        let d2 = NaiveDate::from_ymd_opt(2026, 7, 4).unwrap();

        // Deterministic
        assert_eq!(
            reprocess_advisory_lock_key(emp, d1),
            reprocess_advisory_lock_key(emp, d1)
        );

        // Keyed by both employment_id and date
        assert_ne!(
            reprocess_advisory_lock_key(emp, d1),
            reprocess_advisory_lock_key(emp, d2),
            "different days must yield different keys"
        );
        assert_ne!(
            reprocess_advisory_lock_key(emp, d1),
            reprocess_advisory_lock_key(Uuid::new_v4(), d1),
            "different employees must yield different keys"
        );
    }
}
