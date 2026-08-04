use crate::diesel_pool::DieselPools;
use crate::errors::AppError;
use crate::models::{AttendanceDailySummary, OtStatus};
use crate::repositories::attendance_daily_summary_repository::{
    AttendanceDailySummaryRepository, GetSummariesPaginatedArgs, UpsertCalculatedSummaryArgs,
    UpsertCalculatedSummaryTxArgs, UpsertPaperOtArgs,
};
use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use uuid::Uuid;

pub struct DieselAttendanceDailySummaryRepository {
    diesel_pools: DieselPools,
}

impl DieselAttendanceDailySummaryRepository {
    pub fn new(diesel_pools: DieselPools) -> Self {
        Self { diesel_pools }
    }
}

/// Raw SQL CASE for OT status transition on upsert:
/// - If current status is 'none' AND new system_detected_ot > 0 → 'pending_employee'
/// - Otherwise keep existing status
const OT_STATUS_UPSERT_CASE_SQL: &str = "\
    CASE \
        WHEN attendance_daily_summaries.ot_status = 'none'::ot_status_type \
             AND EXCLUDED.system_detected_ot_minutes > 0 \
        THEN 'pending_employee'::ot_status_type \
        ELSE attendance_daily_summaries.ot_status \
    END";

#[async_trait]
impl AttendanceDailySummaryRepository for DieselAttendanceDailySummaryRepository {
    async fn get_summary(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<AttendanceDailySummary>, AppError> {
        let workspace_id = auth_token.workspace_id();
        use crate::schema::attendance_daily_summaries::dsl as ads;

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await?,
        );

        use crate::models::diesel::attendance_daily_summary::AttendanceDailySummaryEntity;

        ads::attendance_daily_summaries
            .filter(ads::workspace_id.eq(workspace_id))
            .filter(ads::employment_id.eq(employment_id))
            .filter(ads::date.eq(date))
            .first::<AttendanceDailySummaryEntity>(&mut *conn)
            .await
            .optional()
            .map(|opt| opt.map(Into::into))
            .map_err(|e| AppError::Internal(format!("db: {e}")))
    }

    async fn get_summary_by_id(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
    ) -> Result<Option<AttendanceDailySummary>, AppError> {
        let workspace_id = auth_token.workspace_id();
        use crate::schema::attendance_daily_summaries::dsl as ads;

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await?,
        );

        use crate::models::diesel::attendance_daily_summary::AttendanceDailySummaryEntity;

        ads::attendance_daily_summaries
            .filter(ads::workspace_id.eq(workspace_id))
            .filter(ads::id.eq(id))
            .first::<AttendanceDailySummaryEntity>(&mut *conn)
            .await
            .optional()
            .map(|opt| opt.map(Into::into))
            .map_err(|e| AppError::Internal(format!("db: {e}")))
    }

    async fn upsert_calculated_summary(
        &self,
        args: UpsertCalculatedSummaryArgs,
    ) -> Result<(), AppError> {
        let workspace_id = args.auth_token.workspace_id();

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&args.auth_token.workspace_id())
                .await?,
        );

        execute_upsert_calculated_summary(
            &mut conn,
            workspace_id,
            args.employment_id,
            args.date,
            args.actual_work_minutes,
            args.system_detected_ot_minutes,
            args.standard_minutes,
            args.early_ot_minutes,
            args.late_ot_minutes,
            args.break_ot_minutes,
            args.is_weekend,
            args.new_status,
            args.is_holiday,
            args.holiday_name,
        )
        .await
    }

    /// Batch-upserts calculated attendance summaries in a single SQL statement.
    ///
    /// Uses multi-row `INSERT ... ON CONFLICT ... DO UPDATE` to persist all
    /// summaries in one DB round-trip, regardless of range size.
    /// The `ot_status` CASE expression preserves employee-approved OT status:
    /// only transitions `none` → `pending_employee` when new OT is detected.
    async fn batch_upsert_calculated_summaries(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        summaries: &[crate::repositories::attendance_daily_summary_repository::BatchSummaryInput],
    ) -> Result<(), AppError> {
        if summaries.is_empty() {
            return Ok(());
        }

        let workspace_id = auth_token.workspace_id();

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await?,
        );

        execute_batch_upsert_summaries(&mut conn, workspace_id, employment_id, summaries).await
    }

    async fn claim_ot(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        date: NaiveDate,
        claimed_minutes: i32,
        reason: Option<String>,
    ) -> Result<usize, AppError> {
        let workspace_id = auth_token.workspace_id();
        use crate::schema::attendance_daily_summaries::dsl as ads;

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await?,
        );

        let rows = diesel::update(ads::attendance_daily_summaries)
            .filter(ads::workspace_id.eq(workspace_id))
            .filter(ads::employment_id.eq(employment_id))
            .filter(ads::date.eq(date))
            .filter(
                ads::ot_status
                    .eq(OtStatus::PendingEmployee)
                    .or(ads::ot_status.eq(OtStatus::Rejected)),
            )
            .set((
                ads::claimed_ot_minutes.eq(claimed_minutes),
                ads::ot_reason.eq(reason),
                ads::ot_status.eq(OtStatus::PendingManager),
                ads::updated_at.eq(diesel::dsl::now),
            ))
            .execute(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(rows)
    }

    async fn approve_ot(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        date: NaiveDate,
        approved_minutes: i32,
        manager_note: Option<String>,
        status: OtStatus,
    ) -> Result<usize, AppError> {
        let workspace_id = auth_token.workspace_id();
        use crate::schema::attendance_daily_summaries::dsl as ads;

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await?,
        );

        let rows = diesel::update(ads::attendance_daily_summaries)
            .filter(ads::workspace_id.eq(workspace_id))
            .filter(ads::employment_id.eq(employment_id))
            .filter(ads::date.eq(date))
            .filter(ads::ot_status.eq(OtStatus::PendingManager))
            .set((
                ads::approved_ot_minutes.eq(approved_minutes),
                ads::manager_note.eq(manager_note),
                ads::ot_status.eq(status),
                ads::updated_at.eq(diesel::dsl::now),
            ))
            .execute(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(rows)
    }

    async fn upsert_paper_ot(
        &self,
        args: UpsertPaperOtArgs,
    ) -> Result<AttendanceDailySummary, AppError> {
        let workspace_id = args.auth_token.workspace_id();
        use crate::schema::attendance_daily_summaries::dsl as ads;
        use diesel::upsert::excluded;

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&args.auth_token.workspace_id())
                .await?,
        );

        use crate::models::diesel::attendance_daily_summary::AttendanceDailySummaryEntity;

        diesel::insert_into(ads::attendance_daily_summaries)
            .values((
                ads::id.eq(Uuid::new_v4()),
                ads::workspace_id.eq(workspace_id),
                ads::employment_id.eq(args.employment_id),
                ads::date.eq(args.date),
                ads::claimed_ot_minutes.eq(args.claimed_minutes),
                ads::ot_reason.eq(args.reason),
                ads::ot_status.eq(OtStatus::PendingManager),
                ads::entered_via.eq(Some("paper_form".to_string())),
                ads::claim_entered_by_user_id.eq(Some(args.entered_by_user_id)),
                ads::paper_form_photo_url.eq(args.photo_url),
                ads::claim_entered_at.eq(Some(Utc::now())),
                ads::actual_work_minutes.eq(0),
                ads::system_detected_ot_minutes.eq(0),
            ))
            .on_conflict((ads::employment_id, ads::date))
            .do_update()
            .set((
                ads::claimed_ot_minutes.eq(excluded(ads::claimed_ot_minutes)),
                ads::ot_reason.eq(excluded(ads::ot_reason)),
                ads::ot_status.eq(OtStatus::PendingManager),
                ads::entered_via.eq(Some("paper_form".to_string())),
                ads::claim_entered_by_user_id.eq(excluded(ads::claim_entered_by_user_id)),
                ads::paper_form_photo_url.eq(excluded(ads::paper_form_photo_url)),
                ads::claim_entered_at.eq(Some(Utc::now())),
                ads::updated_at.eq(Utc::now()),
            ))
            .get_result::<AttendanceDailySummaryEntity>(&mut *conn)
            .await
            .map(Into::into)
            .map_err(AppError::from)
    }

    async fn get_batch_ot_statuses(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_ids: &[Uuid],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<std::collections::HashMap<(Uuid, NaiveDate), String>, AppError> {
        let workspace_id = auth_token.workspace_id();
        use crate::schema::attendance_daily_summaries::dsl as ads;

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await?,
        );

        let rows = ads::attendance_daily_summaries
            .filter(ads::workspace_id.eq(workspace_id))
            .filter(ads::employment_id.eq_any(employment_ids))
            .filter(ads::date.ge(start_date))
            .filter(ads::date.le(end_date))
            .select((ads::employment_id, ads::date, ads::ot_status))
            .load::<(Uuid, NaiveDate, OtStatus)>(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(rows
            .into_iter()
            .map(|(eid, d, status)| {
                let status_str = match status {
                    OtStatus::None => "none",
                    OtStatus::PendingEmployee => "pending_employee",
                    OtStatus::PendingManager => "pending_manager",
                    OtStatus::Approved => "approved",
                    OtStatus::Rejected => "rejected",
                };
                ((eid, d), status_str.to_string())
            })
            .collect())
    }

    async fn get_summaries(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Option<Uuid>,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<AttendanceDailySummary>, AppError> {
        let workspace_id = auth_token.workspace_id();
        use crate::schema::attendance_daily_summaries::dsl as ads;

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await?,
        );

        let mut query = ads::attendance_daily_summaries
            .filter(ads::workspace_id.eq(workspace_id))
            .filter(ads::date.ge(start_date))
            .filter(ads::date.le(end_date))
            .into_boxed();

        if let Some(eid) = employment_id {
            query = query.filter(ads::employment_id.eq(eid));
        }

        use crate::models::diesel::attendance_daily_summary::AttendanceDailySummaryEntity;

        query
            .order_by((ads::date.asc(), ads::employment_id.asc()))
            .load::<AttendanceDailySummaryEntity>(&mut *conn)
            .await
            .map(|list| list.into_iter().map(Into::into).collect())
            .map_err(AppError::from)
    }

    async fn get_summaries_for_period(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<AttendanceDailySummary>, AppError> {
        let workspace_id = auth_token.workspace_id();
        use crate::schema::attendance_daily_summaries::dsl as ads;

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await?,
        );

        use crate::models::diesel::attendance_daily_summary::AttendanceDailySummaryEntity;

        ads::attendance_daily_summaries
            .filter(ads::workspace_id.eq(workspace_id))
            .filter(ads::employment_id.eq(employment_id))
            .filter(ads::date.ge(start_date))
            .filter(ads::date.le(end_date))
            .order_by(ads::date.asc())
            .load::<AttendanceDailySummaryEntity>(&mut *conn)
            .await
            .map(|list| list.into_iter().map(Into::into).collect())
            .map_err(AppError::from)
    }

    async fn get_summaries_paginated(
        &self,
        args: GetSummariesPaginatedArgs,
    ) -> Result<(Vec<AttendanceDailySummary>, usize), AppError> {
        let workspace_id = args.auth_token.workspace_id();
        use crate::schema::attendance_daily_summaries::dsl as ads;

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&args.auth_token.workspace_id())
                .await?,
        );

        // Count total matching records
        let mut count_query = ads::attendance_daily_summaries
            .filter(ads::workspace_id.eq(workspace_id))
            .filter(ads::date.ge(args.start_date))
            .filter(ads::date.le(args.end_date))
            .into_boxed();

        if let Some(eid) = args.employment_id {
            count_query = count_query.filter(ads::employment_id.eq(eid));
        }

        if let Some(ref status) = args.ot_status {
            count_query = count_query.filter(ads::ot_status.eq(status));
        }

        let total: i64 = count_query
            .select(diesel::dsl::count_star())
            .first(&mut *conn)
            .await
            .map_err(AppError::from)?;

        // Fetch page
        let mut data_query = ads::attendance_daily_summaries
            .filter(ads::workspace_id.eq(workspace_id))
            .filter(ads::date.ge(args.start_date))
            .filter(ads::date.le(args.end_date))
            .into_boxed();

        if let Some(eid) = args.employment_id {
            data_query = data_query.filter(ads::employment_id.eq(eid));
        }

        if let Some(status) = args.ot_status {
            data_query = data_query.filter(ads::ot_status.eq(status));
        }

        use crate::models::diesel::attendance_daily_summary::AttendanceDailySummaryEntity;

        let results = data_query
            .order_by((ads::date.asc(), ads::employment_id.asc()))
            .limit(args.limit)
            .offset((args.page - 1) * args.limit)
            .load::<AttendanceDailySummaryEntity>(&mut *conn)
            .await
            .map(|list| list.into_iter().map(Into::into).collect::<Vec<_>>())
            .map_err(AppError::from)?;

        let total_usize = usize::try_from(total)
            .map_err(|e| AppError::Internal(format!("Invalid total count: {e}")))?;

        Ok((results, total_usize))
    }

    async fn get_summaries_by_employments_batch(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_ids: &[Uuid],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<std::collections::HashMap<(Uuid, NaiveDate), AttendanceDailySummary>, AppError>
    {
        if employment_ids.is_empty() {
            return Ok(std::collections::HashMap::new());
        }

        let workspace_id = auth_token.workspace_id();
        use crate::schema::attendance_daily_summaries::dsl as ads;

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await?,
        );

        use crate::models::diesel::attendance_daily_summary::AttendanceDailySummaryEntity;

        let rows = ads::attendance_daily_summaries
            .filter(ads::workspace_id.eq(workspace_id))
            .filter(ads::employment_id.eq_any(employment_ids))
            .filter(ads::date.ge(start_date))
            .filter(ads::date.le(end_date))
            .load::<AttendanceDailySummaryEntity>(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(rows
            .into_iter()
            .map(|entity| {
                let summary: AttendanceDailySummary = entity.into();
                ((summary.employment_id, summary.date), summary)
            })
            .collect())
    }

    async fn get_distinct_work_days_from_summary(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<NaiveDate>, AppError> {
        let workspace_id = auth_token.workspace_id();
        use crate::schema::attendance_daily_summaries::dsl as ads;

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await?,
        );

        let rows = ads::attendance_daily_summaries
            .filter(ads::workspace_id.eq(workspace_id))
            .filter(ads::employment_id.eq(employment_id))
            .filter(ads::date.ge(start_date))
            .filter(ads::date.le(end_date))
            .filter(ads::actual_work_minutes.gt(0))
            .select(ads::date)
            .distinct()
            .load::<NaiveDate>(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(rows)
    }

    async fn get_summaries_by_ids(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        ids: &[Uuid],
    ) -> Result<Vec<AttendanceDailySummary>, AppError> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let workspace_id = auth_token.workspace_id();
        use crate::schema::attendance_daily_summaries::dsl as ads;

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await?,
        );

        use crate::models::diesel::attendance_daily_summary::AttendanceDailySummaryEntity;

        let rows = ads::attendance_daily_summaries
            .filter(ads::workspace_id.eq(workspace_id))
            .filter(ads::id.eq_any(ids))
            .select(AttendanceDailySummaryEntity::as_select())
            .load::<AttendanceDailySummaryEntity>(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn upsert_calculated_summary_tx(
        &self,
        conn: &mut AsyncPgConnection,
        args: UpsertCalculatedSummaryTxArgs,
    ) -> Result<(), AppError> {
        execute_upsert_calculated_summary(
            conn,
            args.workspace_id,
            args.employment_id,
            args.date,
            args.actual_work_minutes,
            args.system_detected_ot_minutes,
            args.standard_minutes,
            args.early_ot_minutes,
            args.late_ot_minutes,
            args.break_ot_minutes,
            args.is_weekend,
            args.new_status,
            args.is_holiday,
            args.holiday_name,
        )
        .await
    }

    async fn batch_upsert_calculated_summaries_tx(
        &self,
        conn: &mut AsyncPgConnection,
        workspace_id: Uuid,
        employment_id: Uuid,
        summaries: &[crate::repositories::attendance_daily_summary_repository::BatchSummaryInput],
    ) -> Result<(), AppError> {
        if summaries.is_empty() {
            return Ok(());
        }
        execute_batch_upsert_summaries(conn, workspace_id, employment_id, summaries).await
    }
}

/// Core logic for upserting a single calculated daily summary.
#[allow(clippy::too_many_arguments)]
async fn execute_upsert_calculated_summary(
    conn: &mut AsyncPgConnection,
    workspace_id: Uuid,
    employment_id: Uuid,
    date: NaiveDate,
    actual_work_minutes: i32,
    system_detected_ot_minutes: i32,
    standard_minutes: i32,
    early_ot_minutes: i32,
    late_ot_minutes: i32,
    break_ot_minutes: i32,
    is_weekend: bool,
    new_status: OtStatus,
    is_holiday: bool,
    holiday_name: Option<String>,
) -> Result<(), AppError> {
    use crate::schema::attendance_daily_summaries::dsl as ads;

    diesel::insert_into(ads::attendance_daily_summaries)
        .values((
            ads::workspace_id.eq(workspace_id),
            ads::employment_id.eq(employment_id),
            ads::date.eq(date),
            ads::actual_work_minutes.eq(actual_work_minutes),
            ads::system_detected_ot_minutes.eq(system_detected_ot_minutes),
            ads::standard_minutes.eq(standard_minutes),
            ads::early_ot_minutes.eq(early_ot_minutes),
            ads::late_ot_minutes.eq(late_ot_minutes),
            ads::break_ot_minutes.eq(break_ot_minutes),
            ads::is_weekend.eq(is_weekend),
            ads::ot_status.eq(new_status),
            ads::is_holiday.eq(is_holiday),
            ads::holiday_name.eq(holiday_name),
        ))
        .on_conflict((ads::employment_id, ads::date))
        .do_update()
        .set((
            ads::actual_work_minutes.eq(diesel::dsl::sql("EXCLUDED.actual_work_minutes")),
            ads::system_detected_ot_minutes
                .eq(diesel::dsl::sql("EXCLUDED.system_detected_ot_minutes")),
            ads::standard_minutes.eq(diesel::dsl::sql("EXCLUDED.standard_minutes")),
            ads::early_ot_minutes.eq(diesel::dsl::sql("EXCLUDED.early_ot_minutes")),
            ads::late_ot_minutes.eq(diesel::dsl::sql("EXCLUDED.late_ot_minutes")),
            ads::break_ot_minutes.eq(diesel::dsl::sql("EXCLUDED.break_ot_minutes")),
            ads::is_weekend.eq(diesel::dsl::sql("EXCLUDED.is_weekend")),
            ads::is_holiday.eq(diesel::dsl::sql("EXCLUDED.is_holiday")),
            ads::holiday_name.eq(diesel::dsl::sql("EXCLUDED.holiday_name")),
            ads::ot_status.eq(diesel::dsl::sql::<crate::schema::sql_types::OtStatusType>(
                OT_STATUS_UPSERT_CASE_SQL,
            )),
            ads::updated_at.eq(diesel::dsl::now),
        ))
        .execute(conn)
        .await
        .map_err(AppError::from)?;

    Ok(())
}

/// Core logic for batch-upserting calculated daily summaries.
async fn execute_batch_upsert_summaries(
    conn: &mut AsyncPgConnection,
    workspace_id: Uuid,
    employment_id: Uuid,
    summaries: &[crate::repositories::attendance_daily_summary_repository::BatchSummaryInput],
) -> Result<(), AppError> {
    use crate::schema::attendance_daily_summaries::dsl as ads;

    let values: Vec<_> = summaries
        .iter()
        .map(|s| {
            (
                ads::workspace_id.eq(workspace_id),
                ads::employment_id.eq(employment_id),
                ads::date.eq(s.date),
                ads::actual_work_minutes.eq(s.actual_work_minutes),
                ads::system_detected_ot_minutes.eq(s.system_detected_ot_minutes),
                ads::standard_minutes.eq(s.standard_minutes),
                ads::early_ot_minutes.eq(s.early_ot_minutes),
                ads::late_ot_minutes.eq(s.late_ot_minutes),
                ads::break_ot_minutes.eq(s.break_ot_minutes),
                ads::is_weekend.eq(s.is_weekend),
                ads::ot_status.eq(s.ot_status),
                ads::is_holiday.eq(s.is_holiday),
                ads::holiday_name.eq(&s.holiday_name),
            )
        })
        .collect();

    diesel::insert_into(ads::attendance_daily_summaries)
        .values(&values)
        .on_conflict((ads::employment_id, ads::date))
        .do_update()
        .set((
            ads::actual_work_minutes.eq(diesel::dsl::sql("EXCLUDED.actual_work_minutes")),
            ads::system_detected_ot_minutes
                .eq(diesel::dsl::sql("EXCLUDED.system_detected_ot_minutes")),
            ads::standard_minutes.eq(diesel::dsl::sql("EXCLUDED.standard_minutes")),
            ads::early_ot_minutes.eq(diesel::dsl::sql("EXCLUDED.early_ot_minutes")),
            ads::late_ot_minutes.eq(diesel::dsl::sql("EXCLUDED.late_ot_minutes")),
            ads::break_ot_minutes.eq(diesel::dsl::sql("EXCLUDED.break_ot_minutes")),
            ads::is_weekend.eq(diesel::dsl::sql("EXCLUDED.is_weekend")),
            ads::is_holiday.eq(diesel::dsl::sql("EXCLUDED.is_holiday")),
            ads::holiday_name.eq(diesel::dsl::sql("EXCLUDED.holiday_name")),
            ads::ot_status.eq(diesel::dsl::sql::<crate::schema::sql_types::OtStatusType>(
                OT_STATUS_UPSERT_CASE_SQL,
            )),
            ads::updated_at.eq(diesel::dsl::now),
        ))
        .execute(conn)
        .await
        .map_err(AppError::from)?;

    Ok(())
}
