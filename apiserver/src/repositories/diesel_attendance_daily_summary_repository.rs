use crate::diesel_pool::DieselPools;
use crate::errors::AppError;
use crate::models::{AttendanceDailySummary, OtStatus};
use crate::repositories::attendance_daily_summary_repository::AttendanceDailySummaryRepository;
use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

pub struct DieselAttendanceDailySummaryRepository {
    diesel_pools: DieselPools,
}

impl DieselAttendanceDailySummaryRepository {
    pub fn new(diesel_pools: DieselPools) -> Self {
        Self { diesel_pools }
    }
}

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

    #[allow(clippy::too_many_arguments)]
    async fn upsert_calculated_summary(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        date: NaiveDate,
        actual_work_minutes: i32,
        system_detected_ot_minutes: i32,
        new_status: OtStatus,
        is_holiday: bool,
        holiday_name: Option<String>,
    ) -> Result<(), AppError> {
        let workspace_id = auth_token.workspace_id();
        use crate::schema::attendance_daily_summaries::dsl as ads;
        use diesel::upsert::excluded;

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await?,
        );

        diesel::insert_into(ads::attendance_daily_summaries)
            .values((
                ads::workspace_id.eq(workspace_id),
                ads::employment_id.eq(employment_id),
                ads::date.eq(date),
                ads::actual_work_minutes.eq(actual_work_minutes),
                ads::system_detected_ot_minutes.eq(system_detected_ot_minutes),
                ads::ot_status.eq(new_status),
                ads::is_holiday.eq(is_holiday),
                ads::holiday_name.eq(holiday_name),
            ))
            .on_conflict((ads::employment_id, ads::date))
            .do_update()
            .set((
                ads::actual_work_minutes.eq(excluded(ads::actual_work_minutes)),
                ads::system_detected_ot_minutes.eq(excluded(ads::system_detected_ot_minutes)),
                ads::is_holiday.eq(excluded(ads::is_holiday)),
                ads::holiday_name.eq(excluded(ads::holiday_name)),
                ads::ot_status.eq(diesel::dsl::sql::<crate::schema::sql_types::OtStatusType>(
                    "CASE
                        WHEN attendance_daily_summaries.ot_status = 'none'::ot_status_type AND EXCLUDED.system_detected_ot_minutes > 0 THEN 'pending_employee'::ot_status_type
                        ELSE attendance_daily_summaries.ot_status
                    END"
                )),
                ads::updated_at.eq(diesel::dsl::now),
            ))
            .execute(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(())
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

    #[allow(clippy::too_many_arguments)]
    async fn upsert_paper_ot(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        date: NaiveDate,
        claimed_minutes: i32,
        reason: String,
        entered_by_user_id: Uuid,
        photo_url: Option<String>,
    ) -> Result<AttendanceDailySummary, AppError> {
        let workspace_id = auth_token.workspace_id();
        use crate::schema::attendance_daily_summaries::dsl as ads;
        use diesel::upsert::excluded;

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await?,
        );

        use crate::models::diesel::attendance_daily_summary::AttendanceDailySummaryEntity;

        diesel::insert_into(ads::attendance_daily_summaries)
            .values((
                ads::id.eq(Uuid::new_v4()),
                ads::workspace_id.eq(workspace_id),
                ads::employment_id.eq(employment_id),
                ads::date.eq(date),
                ads::claimed_ot_minutes.eq(claimed_minutes),
                ads::ot_reason.eq(reason),
                ads::ot_status.eq(OtStatus::PendingManager),
                ads::entered_via.eq(Some("paper_form".to_string())),
                ads::claim_entered_by_user_id.eq(Some(entered_by_user_id)),
                ads::paper_form_photo_url.eq(photo_url),
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

    async fn get_summaries_paginated(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Option<Uuid>,
        start_date: NaiveDate,
        end_date: NaiveDate,
        ot_status: Option<OtStatus>,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<AttendanceDailySummary>, usize), AppError> {
        let workspace_id = auth_token.workspace_id();
        use crate::schema::attendance_daily_summaries::dsl as ads;

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await?,
        );

        // Count total matching records
        let mut count_query = ads::attendance_daily_summaries
            .filter(ads::workspace_id.eq(workspace_id))
            .filter(ads::date.ge(start_date))
            .filter(ads::date.le(end_date))
            .into_boxed();

        if let Some(eid) = employment_id {
            count_query = count_query.filter(ads::employment_id.eq(eid));
        }

        if let Some(ref status) = ot_status {
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
            .filter(ads::date.ge(start_date))
            .filter(ads::date.le(end_date))
            .into_boxed();

        if let Some(eid) = employment_id {
            data_query = data_query.filter(ads::employment_id.eq(eid));
        }

        if let Some(status) = ot_status {
            data_query = data_query.filter(ads::ot_status.eq(status));
        }

        use crate::models::diesel::attendance_daily_summary::AttendanceDailySummaryEntity;

        let results = data_query
            .order_by((ads::date.asc(), ads::employment_id.asc()))
            .limit(limit)
            .offset((page - 1) * limit)
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
}
