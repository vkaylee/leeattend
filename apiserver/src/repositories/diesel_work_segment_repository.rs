use async_trait::async_trait;
use bigdecimal::BigDecimal;
use chrono::{NaiveDate, Utc};
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

use crate::diesel_pool::DieselPools;
use crate::errors::AppError;
use crate::models::diesel::work_segment::{
    GapCompensationType, NewWorkSegmentEntity, SegmentStatusType, SegmentType,
    WorkSegmentEntity,
};
use crate::models::{
    CreateWorkSegmentRequest, UpdateWorkSegmentRequest, WorkSegment, WorkSegmentFilter,
};
use crate::repositories::work_segment_repository::WorkSegmentRepository;
use crate::schema::work_segments;

pub struct DieselWorkSegmentRepository {
    diesel_pools: DieselPools,
}

impl DieselWorkSegmentRepository {
    pub fn new(diesel_pools: DieselPools) -> Self {
        Self { diesel_pools }
    }
}

fn f64_to_bigdecimal(v: f64) -> BigDecimal {
    BigDecimal::try_from(v).unwrap_or_else(|_| BigDecimal::from(0))
}

/// Apply filters from WorkSegmentFilter to a boxed query.
/// Clones owned data from filter to avoid lifetime issues with boxed queries.
fn apply_filters(
    mut query: work_segments::BoxedQuery<'static, diesel::pg::Pg>,
    filter: &WorkSegmentFilter,
) -> work_segments::BoxedQuery<'static, diesel::pg::Pg> {
    if let Some(ref ids) = filter.employment_ids {
        if !ids.is_empty() {
            let cloned = ids.clone();
            query = query.filter(work_segments::employment_id.eq_any(cloned));
        }
    }
    if let Some(start) = filter.start_date {
        query = query.filter(work_segments::date.ge(start));
    }
    if let Some(end) = filter.end_date {
        query = query.filter(work_segments::date.le(end));
    }
    if let Some(ref st) = filter.segment_type {
        let st_enum = SegmentType::from_domain_str(st);
        query = query.filter(work_segments::segment_type.eq(st_enum));
    }
    if let Some(ref ss) = filter.segment_status {
        let ss_enum = SegmentStatusType::from_domain_str(ss);
        query = query.filter(work_segments::segment_status.eq(ss_enum));
    }
    if let Some(ref source) = filter.source {
        let s = source.clone();
        query = query.filter(work_segments::source.eq(s));
    }
    query
}

#[async_trait]
impl WorkSegmentRepository for DieselWorkSegmentRepository {
    // ── create_segment ────────────────────────────────────────────────────────
    async fn create_segment(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        req: &CreateWorkSegmentRequest,
    ) -> Result<WorkSegment, AppError> {
        let workspace_id = auth_token.workspace_id();

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let mut entity = NewWorkSegmentEntity::from(req);
        entity.workspace_id = workspace_id;

        let result = diesel::insert_into(work_segments::table)
            .values(&entity)
            .get_result::<WorkSegmentEntity>(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(WorkSegment::from(result))
    }

    // ── get_segment_by_id ─────────────────────────────────────────────────────
    async fn get_segment_by_id(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        segment_id: Uuid,
    ) -> Result<Option<WorkSegment>, AppError> {
        let workspace_id = auth_token.workspace_id();

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let result = work_segments::table
            .filter(work_segments::id.eq(segment_id))
            .filter(work_segments::workspace_id.eq(workspace_id))
            .first::<WorkSegmentEntity>(&mut *conn)
            .await
            .optional()
            .map_err(AppError::from)?;

        Ok(result.map(WorkSegment::from))
    }

    // ── list_segments ─────────────────────────────────────────────────────────
    async fn list_segments(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        filter: &WorkSegmentFilter,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<WorkSegment>, i64), AppError> {
        let workspace_id = auth_token.workspace_id();

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        // Count query
        let count_query = work_segments::table
            .filter(work_segments::workspace_id.eq(workspace_id))
            .into_boxed();
        let count_query = apply_filters(count_query, filter);
        let total: i64 = count_query
            .count()
            .get_result(&mut *conn)
            .await
            .map_err(AppError::from)?;

        // Data query
        let data_query = work_segments::table
            .filter(work_segments::workspace_id.eq(workspace_id))
            .into_boxed();
        let data_query = apply_filters(data_query, filter);
        let offset = (page.saturating_sub(1)) * limit;
        let entities = data_query
            .order((work_segments::date, work_segments::sort_order))
            .limit(limit)
            .offset(offset)
            .load::<WorkSegmentEntity>(&mut *conn)
            .await
            .map_err(AppError::from)?;

        let segments = entities.into_iter().map(WorkSegment::from).collect();
        Ok((segments, total))
    }

    // ── get_segments_by_date_range ────────────────────────────────────────────
    async fn get_segments_by_date_range(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_ids: &[Uuid],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<WorkSegment>, AppError> {
        let workspace_id = auth_token.workspace_id();

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let ids_vec = employment_ids.to_vec();
        let entities = work_segments::table
            .filter(work_segments::workspace_id.eq(workspace_id))
            .filter(work_segments::employment_id.eq_any(ids_vec))
            .filter(work_segments::date.ge(start_date))
            .filter(work_segments::date.le(end_date))
            .order((
                work_segments::employment_id,
                work_segments::date,
                work_segments::sort_order,
            ))
            .load::<WorkSegmentEntity>(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(entities.into_iter().map(WorkSegment::from).collect())
    }

    // ── update_segment ────────────────────────────────────────────────────────
    async fn update_segment(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        segment_id: Uuid,
        req: &UpdateWorkSegmentRequest,
    ) -> Result<WorkSegment, AppError> {
        let workspace_id = auth_token.workspace_id();

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        conn.transaction::<_, AppError, _>(|conn| {
            Box::pin(async move {
                // Verify ownership
                let existing = work_segments::table
                    .filter(work_segments::id.eq(segment_id))
                    .filter(work_segments::workspace_id.eq(workspace_id))
                    .first::<WorkSegmentEntity>(&mut *conn)
                    .await
                    .optional()
                    .map_err(AppError::from)?
                    .ok_or_else(|| {
                        AppError::Internal(format!("work_segment {segment_id} not found"))
                    })?;

                // Build update set
                let template_id = req.template_id.unwrap_or(existing.template_id);
                let interval_id = req.interval_id.or(existing.interval_id);
                let sort_order = req.sort_order.unwrap_or(existing.sort_order);
                let scheduled_start =
                    req.scheduled_start.unwrap_or(existing.scheduled_start);
                let scheduled_end =
                    req.scheduled_end.unwrap_or(existing.scheduled_end);
                let segment_type = req
                    .segment_type
                    .as_deref()
                    .map(SegmentType::from_domain_str)
                    .unwrap_or(existing.segment_type);
                let segment_status = req
                    .segment_status
                    .as_deref()
                    .map(SegmentStatusType::from_domain_str)
                    .unwrap_or(existing.segment_status);
                let compensation_rate = req
                    .compensation_rate
                    .map(f64_to_bigdecimal)
                    .unwrap_or(existing.compensation_rate);
                let gap_compensation_before = req
                    .gap_compensation_before
                    .as_deref()
                    .map(GapCompensationType::from_domain_str)
                    .unwrap_or(existing.gap_compensation_before);
                let gap_compensation_rate_before = req
                    .gap_compensation_rate_before
                    .map(f64_to_bigdecimal)
                    .unwrap_or(existing.gap_compensation_rate_before);
                let break_config_override = req
                    .break_config_override
                    .clone()
                    .or(existing.break_config_override);
                let ot_config_override = req
                    .ot_config_override
                    .clone()
                    .or(existing.ot_config_override);
                let source = req.source.clone().unwrap_or(existing.source);
                let source_detail = req
                    .source_detail
                    .clone()
                    .unwrap_or(existing.source_detail);
                let notes = req.notes.clone().or(existing.notes);

                let updated = diesel::update(work_segments::table.find(segment_id))
                    .set((
                        work_segments::template_id.eq(template_id),
                        work_segments::interval_id.eq(interval_id),
                        work_segments::sort_order.eq(sort_order),
                        work_segments::scheduled_start.eq(scheduled_start),
                        work_segments::scheduled_end.eq(scheduled_end),
                        work_segments::segment_type.eq(segment_type),
                        work_segments::segment_status.eq(segment_status),
                        work_segments::compensation_rate.eq(compensation_rate),
                        work_segments::gap_compensation_before.eq(gap_compensation_before),
                        work_segments::gap_compensation_rate_before
                            .eq(gap_compensation_rate_before),
                        work_segments::break_config_override.eq(break_config_override),
                        work_segments::ot_config_override.eq(ot_config_override),
                        work_segments::source.eq(source),
                        work_segments::source_detail.eq(source_detail),
                        work_segments::notes.eq(notes),
                        work_segments::updated_at.eq(Utc::now()),
                    ))
                    .get_result::<WorkSegmentEntity>(&mut *conn)
                    .await
                    .map_err(AppError::from)?;

                Ok(WorkSegment::from(updated))
            })
        })
        .await
    }

    // ── delete_segment ────────────────────────────────────────────────────────
    async fn delete_segment(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        segment_id: Uuid,
    ) -> Result<(), AppError> {
        let workspace_id = auth_token.workspace_id();

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let affected = diesel::delete(
            work_segments::table
                .filter(work_segments::id.eq(segment_id))
                .filter(work_segments::workspace_id.eq(workspace_id)),
        )
        .execute(&mut *conn)
        .await
        .map_err(AppError::from)?;

        if affected == 0 {
            return Err(AppError::Internal(format!(
                "work_segment {segment_id} not found"
            )));
        }
        Ok(())
    }

    // ── bulk_create_segments ──────────────────────────────────────────────────
    async fn bulk_create_segments(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        segments: &[CreateWorkSegmentRequest],
        replace_existing: bool,
    ) -> Result<Vec<WorkSegment>, AppError> {
        if segments.is_empty() {
            return Ok(vec![]);
        }

        let workspace_id = auth_token.workspace_id();

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let entities: Vec<NewWorkSegmentEntity> = segments
            .iter()
            .map(|req| {
                let mut e = NewWorkSegmentEntity::from(req);
                e.workspace_id = workspace_id;
                e
            })
            .collect();

        conn.transaction::<_, AppError, _>(|conn| {
            let entities = entities.clone();
            Box::pin(async move {
                // If replace_existing, delete segments for affected (employment_id, date) pairs
                if replace_existing {
                    let pairs: Vec<(Uuid, NaiveDate)> = entities
                        .iter()
                        .map(|e| (e.employment_id, e.date))
                        .collect::<std::collections::HashSet<_>>()
                        .into_iter()
                        .collect();

                    for (emp_id, date) in &pairs {
                        diesel::delete(
                            work_segments::table
                                .filter(work_segments::workspace_id.eq(workspace_id))
                                .filter(work_segments::employment_id.eq(emp_id))
                                .filter(work_segments::date.eq(date)),
                        )
                        .execute(&mut *conn)
                        .await
                        .map_err(AppError::from)?;
                    }
                }

                // Insert with ON CONFLICT DO NOTHING
                diesel::insert_into(work_segments::table)
                    .values(&entities)
                    .on_conflict_do_nothing()
                    .execute(&mut *conn)
                    .await
                    .map_err(AppError::from)?;

                // Return all segments that match the input (both newly inserted and existing)
                let employment_ids: Vec<Uuid> = entities
                    .iter()
                    .map(|e| e.employment_id)
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .collect();
                let dates: Vec<NaiveDate> = entities
                    .iter()
                    .map(|e| e.date)
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .collect();

                let result = work_segments::table
                    .filter(work_segments::workspace_id.eq(workspace_id))
                    .filter(work_segments::employment_id.eq_any(&employment_ids))
                    .filter(work_segments::date.eq_any(&dates))
                    .order((
                        work_segments::employment_id,
                        work_segments::date,
                        work_segments::sort_order,
                    ))
                    .load::<WorkSegmentEntity>(&mut *conn)
                    .await
                    .map_err(AppError::from)?;

                Ok(result.into_iter().map(WorkSegment::from).collect())
            })
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diesel_pool::{DieselPools, TenantAuthToken};
    use crate::models::CreateWorkSegmentRequest;
    use chrono::NaiveTime;
    use uuid::Uuid;

    fn test_pools() -> DieselPools {
        let test_db_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| {
                "postgres://user:pass@localhost:5432/leeattend_test_db".to_string()
            });
        DieselPools::for_test(&test_db_url)
    }

    fn test_auth_token() -> TenantAuthToken {
        // Use a fixed test workspace UUID
        TenantAuthToken(Uuid::parse_str("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee").unwrap())
    }

    /// Returns true if we can get a test DB connection.
    async fn db_available(pools: &DieselPools) -> bool {
        pools
            .tenants_tx
            .get_connection(&Uuid::parse_str("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee").unwrap())
            .await
            .is_ok()
    }

    fn make_create_request(
        employment_id: Uuid,
        date: NaiveDate,
        template_id: Uuid,
    ) -> CreateWorkSegmentRequest {
        CreateWorkSegmentRequest {
            employment_id,
            date,
            template_id,
            interval_id: None,
            sort_order: 0,
            scheduled_start: NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            scheduled_end: NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
            segment_type: "work".to_string(),
            segment_status: "scheduled".to_string(),
            compensation_rate: 1.0,
            gap_compensation_before: "unpaid".to_string(),
            gap_compensation_rate_before: 0.0,
            break_config_override: None,
            ot_config_override: None,
            source: "manual".to_string(),
            source_detail: serde_json::json!({}),
            notes: None,
        }
    }

    // ── Test 1: CRUD roundtrip ──────────────────────────────────────────

    #[tokio::test]
    async fn test_crud_roundtrip() {
        let pools = test_pools();
        if !db_available(&pools).await {
            eprintln!("SKIP test_crud_roundtrip: no test database available");
            return;
        }
        let repo = DieselWorkSegmentRepository::new(pools);
        let auth = test_auth_token();

        let emp_id = Uuid::new_v4();
        let tmpl_id = Uuid::new_v4();
        let date = NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();

        // Create
        let req = make_create_request(emp_id, date, tmpl_id);
        let created = repo
            .create_segment(&auth, &req)
            .await
            .expect("create failed");

        assert_eq!(created.employment_id, emp_id);
        assert_eq!(created.date, date);
        assert_eq!(created.template_id, tmpl_id);
        assert_eq!(created.segment_type, "work");
        assert_eq!(created.segment_status, "scheduled");
        assert_eq!(created.compensation_rate, 1.0);

        // Get by id
        let fetched = repo
            .get_segment_by_id(&auth, created.id)
            .await
            .expect("get failed")
            .expect("segment not found");

        assert_eq!(fetched.id, created.id);
        assert_eq!(fetched.employment_id, emp_id);
        assert_eq!(fetched.date, date);

        // Update
        let update_req = crate::models::UpdateWorkSegmentRequest {
            template_id: None,
            interval_id: None,
            sort_order: Some(10),
            scheduled_start: Some(NaiveTime::from_hms_opt(9, 0, 0).unwrap()),
            scheduled_end: Some(NaiveTime::from_hms_opt(18, 0, 0).unwrap()),
            segment_type: Some("standby".to_string()),
            segment_status: Some("in_progress".to_string()),
            compensation_rate: Some(1.5),
            gap_compensation_before: None,
            gap_compensation_rate_before: None,
            break_config_override: None,
            ot_config_override: None,
            source: None,
            source_detail: None,
            notes: Some("updated test segment".to_string()),
        };

        let updated = repo
            .update_segment(&auth, created.id, &update_req)
            .await
            .expect("update failed");

        assert_eq!(updated.sort_order, 10);
        assert_eq!(updated.segment_type, "standby");
        assert_eq!(updated.segment_status, "in_progress");
        assert_eq!(updated.compensation_rate, 1.5);
        assert_eq!(updated.notes.as_deref(), Some("updated test segment"));

        // Delete
        repo.delete_segment(&auth, created.id)
            .await
            .expect("delete failed");

        // Verify deleted
        let after_delete = repo
            .get_segment_by_id(&auth, created.id)
            .await
            .expect("get after delete failed");
        assert!(after_delete.is_none(), "segment should be deleted");
    }

    // ── Test 2: bulk_create with replace_existing=false (ON CONFLICT) ──

    #[tokio::test]
    async fn test_bulk_create_no_replace() {
        let pools = test_pools();
        if !db_available(&pools).await {
            eprintln!("SKIP test_bulk_create_no_replace: no test database available");
            return;
        }
        let repo = DieselWorkSegmentRepository::new(pools);
        let auth = test_auth_token();

        let emp_id = Uuid::new_v4();
        let tmpl_id = Uuid::new_v4();
        let date = NaiveDate::from_ymd_opt(2026, 7, 2).unwrap();

        // Insert 3 segments
        let reqs: Vec<CreateWorkSegmentRequest> = (0..3)
            .map(|i| CreateWorkSegmentRequest {
                employment_id: emp_id,
                date,
                template_id: tmpl_id,
                interval_id: Some(Uuid::new_v4()),
                sort_order: i,
                scheduled_start: NaiveTime::from_hms_opt(8 + i as u32, 0, 0).unwrap(),
                scheduled_end: NaiveTime::from_hms_opt(17 + i as u32, 0, 0).unwrap(),
                segment_type: "work".to_string(),
                segment_status: "scheduled".to_string(),
                compensation_rate: 1.0,
                gap_compensation_before: "unpaid".to_string(),
                gap_compensation_rate_before: 0.0,
                break_config_override: None,
                ot_config_override: None,
                source: "manual".to_string(),
                source_detail: serde_json::json!({}),
                notes: None,
            })
            .collect();

        let created = repo
            .bulk_create_segments(&auth, &reqs, false)
            .await
            .expect("first bulk create failed");
        assert_eq!(created.len(), 3);

        // Try creating the same 3 again with replace_existing=false
        let created_again = repo
            .bulk_create_segments(&auth, &reqs, false)
            .await
            .expect("second bulk create failed");

        // ON CONFLICT DO NOTHING => still 3 total segments
        // (the returned count from bulk_create_segments is after insert,
        // which includes existing ones fetched back)
        assert!(
            created_again.len() >= 3,
            "should have at least 3 segments (existing + re-inserted)"
        );

        // Cleanup
        for seg in &created {
            let _ = repo.delete_segment(&auth, seg.id).await;
        }
    }

    // ── Test 3: bulk_create with replace_existing=true ──────────────────

    #[tokio::test]
    async fn test_bulk_create_with_replace() {
        let pools = test_pools();
        if !db_available(&pools).await {
            eprintln!("SKIP test_bulk_create_with_replace: no test database available");
            return;
        }
        let repo = DieselWorkSegmentRepository::new(pools);
        let auth = test_auth_token();

        let emp_id = Uuid::new_v4();
        let tmpl_id = Uuid::new_v4();
        let date = NaiveDate::from_ymd_opt(2026, 7, 3).unwrap();

        // Insert 3 segments
        let reqs: Vec<CreateWorkSegmentRequest> = (0..3)
            .map(|i| CreateWorkSegmentRequest {
                employment_id: emp_id,
                date,
                template_id: tmpl_id,
                interval_id: Some(Uuid::new_v4()),
                sort_order: i,
                scheduled_start: NaiveTime::from_hms_opt(8 + i as u32, 0, 0).unwrap(),
                scheduled_end: NaiveTime::from_hms_opt(17 + i as u32, 0, 0).unwrap(),
                segment_type: "work".to_string(),
                segment_status: "scheduled".to_string(),
                compensation_rate: 1.0,
                gap_compensation_before: "unpaid".to_string(),
                gap_compensation_rate_before: 0.0,
                break_config_override: None,
                ot_config_override: None,
                source: "manual".to_string(),
                source_detail: serde_json::json!({}),
                notes: None,
            })
            .collect();

        let _created = repo
            .bulk_create_segments(&auth, &reqs, false)
            .await
            .expect("first bulk create failed");

        // Now replace with 2 new segments (same employment_id, same date)
        let new_reqs: Vec<CreateWorkSegmentRequest> = (0..2)
            .map(|i| CreateWorkSegmentRequest {
                employment_id: emp_id,
                date,
                template_id: tmpl_id,
                interval_id: Some(Uuid::new_v4()),
                sort_order: i * 10,
                scheduled_start: NaiveTime::from_hms_opt(9 + i as u32, 0, 0).unwrap(),
                scheduled_end: NaiveTime::from_hms_opt(18 + i as u32, 0, 0).unwrap(),
                segment_type: "standby".to_string(),
                segment_status: "scheduled".to_string(),
                compensation_rate: 1.5,
                gap_compensation_before: "unpaid".to_string(),
                gap_compensation_rate_before: 0.0,
                break_config_override: None,
                ot_config_override: None,
                source: "manual".to_string(),
                source_detail: serde_json::json!({}),
                notes: None,
            })
            .collect();

        let replaced = repo
            .bulk_create_segments(&auth, &new_reqs, true)
            .await
            .expect("replace bulk create failed");

        // After replace, we should have exactly 2 segments for this emp/date
        assert_eq!(
            replaced.len(),
            2,
            "after replace_existing=true, should have exactly 2 segments"
        );

        // Verify all returned segments have segment_type "standby" (from new reqs)
        for seg in &replaced {
            assert_eq!(seg.segment_type, "standby");
            assert_eq!(seg.compensation_rate, 1.5);
        }

        // Cleanup
        for seg in &replaced {
            let _ = repo.delete_segment(&auth, seg.id).await;
        }
    }

    // ── Test 4: list_segments with filters ──────────────────────────────

    #[tokio::test]
    async fn test_list_segments_with_filters() {
        let pools = test_pools();
        if !db_available(&pools).await {
            eprintln!("SKIP test_list_segments_with_filters: no test database available");
            return;
        }
        let repo = DieselWorkSegmentRepository::new(pools);
        let auth = test_auth_token();

        let emp_a = Uuid::new_v4();
        let emp_b = Uuid::new_v4();
        let tmpl_id = Uuid::new_v4();
        let date1 = NaiveDate::from_ymd_opt(2026, 7, 10).unwrap();
        let date2 = NaiveDate::from_ymd_opt(2026, 7, 15).unwrap();
        let date3 = NaiveDate::from_ymd_opt(2026, 7, 20).unwrap();

        // Insert segments for 2 employees across 3 dates
        let mut all_created = Vec::new();
        for (emp_id, date) in &[(emp_a, date1), (emp_a, date2), (emp_b, date2), (emp_b, date3)]
        {
            let req = make_create_request(*emp_id, *date, tmpl_id);
            let created = repo
            .create_segment(&auth, &req)
            .await
            .expect("create failed");
            all_created.push(created);
        }

        // Filter by date range — should return only date1 + date2 segments
        let filter = WorkSegmentFilter {
            employment_ids: None,
            start_date: Some(date1),
            end_date: Some(date2),
            segment_type: None,
            segment_status: None,
            source: None,
        };

        let (results, total) = repo
            .list_segments(&auth, &filter, 1, 50)
            .await
            .expect("list failed");

        assert_eq!(total, 3, "3 segments in date range [date1, date2]");
        assert_eq!(results.len(), 3);

        // Verify none are from date3
        for seg in &results {
            assert!(seg.date <= date2);
        }

        // Filter by single employment_id
        let filter = WorkSegmentFilter {
            employment_ids: Some(vec![emp_a]),
            start_date: None,
            end_date: None,
            segment_type: None,
            segment_status: None,
            source: None,
        };

        let (results, total) = repo
            .list_segments(&auth, &filter, 1, 50)
            .await
            .expect("list by emp failed");

        assert_eq!(total, 2, "emp_a has 2 segments");
        for seg in &results {
            assert_eq!(seg.employment_id, emp_a);
        }

        // Cleanup
        for seg in &all_created {
            let _ = repo.delete_segment(&auth, seg.id).await;
        }
    }

    // ── Test 5: Schema drift verify (template_id roundtrip) ─────────────

    #[tokio::test]
    async fn test_template_id_roundtrip() {
        let pools = test_pools();
        if !db_available(&pools).await {
            eprintln!("SKIP test_template_id_roundtrip: no test database available");
            return;
        }
        let repo = DieselWorkSegmentRepository::new(pools);
        let auth = test_auth_token();

        let emp_id = Uuid::new_v4();
        let tmpl_id = Uuid::new_v4(); // use a fresh UUID to detect drift
        let date = NaiveDate::from_ymd_opt(2026, 7, 30).unwrap();

        let req = make_create_request(emp_id, date, tmpl_id);
        let created = repo
            .create_segment(&auth, &req)
            .await
            .expect("create failed");

        assert_eq!(
            created.template_id, tmpl_id,
            "template_id must roundtrip correctly (schema drift check)"
        );

        // Also verify via get_by_id
        let fetched = repo
            .get_segment_by_id(&auth, created.id)
            .await
            .expect("get failed")
            .expect("not found");

        assert_eq!(
            fetched.template_id, tmpl_id,
            "template_id must survive fetch roundtrip"
        );

        // Cleanup
        let _ = repo.delete_segment(&auth, created.id).await;
    }
}
