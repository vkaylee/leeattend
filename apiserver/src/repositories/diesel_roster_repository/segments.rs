use super::*;
use crate::diesel_pool::DieselPoolsTenantTxExt;
use crate::errors::AppError;
use chrono::NaiveDate;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use std::collections::HashMap;
use uuid::Uuid;

#[allow(clippy::too_many_lines)]
impl DieselRosterRepository {
    /// Persist resolved segments atomically, replacing stale interval-backed rows.
    ///
    /// The read path invokes this operation in a detached task, while exposing the
    /// transaction boundary here keeps the cleanup/insert sequence directly testable.
    pub async fn persist_workspace_segments(
        pools: crate::diesel_pool::DieselPools,
        workspace_id: Uuid,
        all_new_entities: Vec<crate::models::diesel::work_segment::NewWorkSegmentEntity>,
    ) -> Result<(), AppError> {
        let start = std::time::Instant::now();
        let tenant_token = crate::diesel_pool::TenantAuthToken::system_override(workspace_id);

        pools
            .with_tenant_tx(&tenant_token, |conn| {
                Box::pin(async move {
                    let affected_pairs: Vec<(Uuid, chrono::NaiveDate)> = {
                        let mut pairs: Vec<_> = all_new_entities
                            .iter()
                            .map(|entity| (entity.employment_id, entity.date))
                            .collect();
                        pairs.sort();
                        pairs.dedup();
                        pairs
                    };

                    let new_interval_ids: Vec<Uuid> = all_new_entities
                        .iter()
                        .filter_map(|entity| entity.interval_id)
                        .collect();

                    let mut deleted = 0u64;
                    if !new_interval_ids.is_empty() {
                        for (employment_id, date) in &affected_pairs {
                            let deleted_count = diesel::delete(
                                crate::schema::work_segments::table
                                    .filter(
                                        crate::schema::work_segments::workspace_id.eq(workspace_id),
                                    )
                                    .filter(
                                        crate::schema::work_segments::employment_id
                                            .eq(employment_id),
                                    )
                                    .filter(crate::schema::work_segments::date.eq(date))
                                    .filter(crate::schema::work_segments::source.ne("manual"))
                                    .filter(crate::schema::work_segments::interval_id.is_not_null())
                                    .filter(
                                        crate::schema::work_segments::interval_id
                                            .ne_all(&new_interval_ids),
                                    ),
                            )
                            .execute(conn)
                            .await
                            .map_err(AppError::from)?;
                            deleted += deleted_count as u64;
                        }
                    }

                    let inserted = diesel::insert_into(crate::schema::work_segments::table)
                        .values(&all_new_entities)
                        .on_conflict_do_nothing()
                        .execute(conn)
                        .await
                        .map_err(AppError::from)?;

                    tracing::info!(
                        workspace_id = %workspace_id,
                        segments_attempted = all_new_entities.len(),
                        segments_inserted = inserted,
                        segments_deleted = deleted,
                        elapsed_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
                        "work_segments.write_through.completed"
                    );
                    Ok(())
                })
            })
            .await
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) async fn get_workspace_segments(
        &self,
        roster_cache: Option<&crate::cache::RosterCache>,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        start_date: NaiveDate,
        end_date: NaiveDate,
        employment_ids: Option<&Vec<Uuid>>,
        page: Option<i64>,
        limit: Option<i64>,
    ) -> Result<(Vec<crate::models::schedule::EmployeeSegments>, i64), AppError> {
        use crate::models::schedule::EmployeeSegments;
        let workspace_id = auth_token.workspace_id();

        // 1. Roster data (shared cache / loader)
        let roster_data = if let Some(cache) = roster_cache {
            cache
                .get_roster_data(workspace_id, &self.diesel_pools)
                .await
                .map_err(|e| {
                    tracing::error!("RosterCache error: {e}");
                    AppError::Internal(format!("roster_cache: {e}"))
                })?
        } else {
            crate::cache::build_roster_data(&self.diesel_pools, workspace_id)
                .await
                .map_err(|e| {
                    tracing::error!("build_roster_data error: {e}");
                    AppError::Internal(format!("roster: {e}"))
                })?
        };

        let (result, total) = self
            .diesel_pools
            .with_tenant_tx(auth_token, |conn| {
                Box::pin(async move {
                    // 2. Paginated employees (identical query structure as get_workspace_schedules)
                    let mut query = crate::schema::employments::table
                        .inner_join(crate::schema::employees::table)
                        .filter(crate::schema::employees::workspace_id.eq(workspace_id))
                        .filter(crate::schema::employments::deleted_at.is_null())
                        .filter(crate::schema::employments::start_date.le(end_date))
                        .filter(
                            crate::schema::employments::end_date
                                .is_null()
                                .or(crate::schema::employments::end_date.ge(start_date)),
                        )
                        .into_boxed();

                    if let Some(eids) = employment_ids {
                        query = query
                            .filter(crate::schema::employments::id.eq_any(eids.iter().copied()));
                    }

                    // Separate count query (build independently to avoid consuming `query`)
                    let mut count_query = crate::schema::employments::table
                        .inner_join(crate::schema::employees::table)
                        .filter(crate::schema::employees::workspace_id.eq(workspace_id))
                        .filter(crate::schema::employments::deleted_at.is_null())
                        .into_boxed();
                    if let Some(eids) = employment_ids {
                        count_query = count_query
                            .filter(crate::schema::employments::id.eq_any(eids.iter().copied()));
                    }
                    let total: i64 = if page.is_some() || limit.is_some() {
                        count_query
                            .count()
                            .get_result::<i64>(conn)
                            .await
                            .map_err(|e| AppError::Internal(format!("db count: {e}")))?
                    } else {
                        0
                    };

                    if let (Some(p), Some(l)) = (page, limit) {
                        let limit_val = l.clamp(1, 3000);
                        let offset_val = (p.max(1) - 1) * limit_val;
                        query = query.limit(limit_val).offset(offset_val);
                    }

                    let employees_entities_raw = query
                        .select((
                            crate::schema::employments::id,
                            crate::schema::employees::full_name,
                            crate::schema::employees::employee_id_number,
                            crate::schema::employees::tags,
                            crate::schema::employments::primary_location_id,
                            crate::schema::employees::id,
                            crate::schema::employments::start_date,
                            crate::schema::employments::end_date,
                        ))
                        .load::<(
                            Uuid,
                            String,
                            String,
                            Option<Vec<Option<String>>>,
                            Option<Uuid>,
                            Uuid,
                            NaiveDate,
                            Option<NaiveDate>,
                        )>(conn)
                        .await
                        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

                    let employees_entities: Vec<EmployeeEntityRow> = employees_entities_raw
                        .into_iter()
                        .map(
                            |(
                                emp_id,
                                name,
                                num,
                                tags,
                                primary_loc,
                                employee_id,
                                employment_start,
                                employment_end,
                            )| {
                                (
                                    emp_id,
                                    name,
                                    num,
                                    tags.map(|v| v.into_iter().flatten().collect()),
                                    primary_loc,
                                    employee_id,
                                    employment_start,
                                    employment_end,
                                )
                            },
                        )
                        .collect();

                    if employees_entities.is_empty() {
                        return Ok((Vec::new(), total));
                    }

                    let emp_ids: Vec<Uuid> =
                        employees_entities.iter().map(|(id, ..)| *id).collect();

                    // 3. Batch-load contracts
                    let batch_contracts = crate::schema::employee_contracts::table
                        .inner_join(crate::schema::employments::table)
                        .filter(crate::schema::employee_contracts::workspace_id.eq(workspace_id))
                        .filter(crate::schema::employments::id.eq_any(&emp_ids))
                        .filter(
                            crate::schema::employee_contracts::status
                                .eq(crate::models::enums::ContractStatus::Active),
                        )
                        .filter(crate::schema::employee_contracts::deleted_at.is_null())
                        .filter(crate::schema::employee_contracts::start_date.le(end_date))
                        .filter(
                            crate::schema::employee_contracts::end_date
                                .is_null()
                                .or(crate::schema::employee_contracts::end_date.ge(start_date)),
                        )
                        .select((
                            crate::schema::employee_contracts::all_columns,
                            crate::schema::employments::id,
                            crate::schema::employments::employer_id,
                        ))
                        .load::<(
                            crate::models::diesel::employee_contract::EmployeeContractEntity,
                            Uuid,
                            Uuid,
                        )>(conn)
                        .await
                        .map_err(|e| AppError::Internal(format!("db batch contracts: {e}")))?;

                    let mut contracts_by_employment: HashMap<
                        Uuid,
                        Vec<(
                            crate::models::diesel::employee_contract::EmployeeContractEntity,
                            Uuid,
                        )>,
                    > = HashMap::new();
                    for (contract, employment_id, employer_id) in batch_contracts {
                        contracts_by_employment
                            .entry(employment_id)
                            .or_default()
                            .push((contract, employer_id));
                    }

                    // 4. Batch-load assignments
                    let batch_assignments = crate::schema::employee_assignments::table
                        .inner_join(crate::schema::employments::table)
                        .filter(crate::schema::employee_assignments::workspace_id.eq(workspace_id))
                        .filter(crate::schema::employments::id.eq_any(&emp_ids))
                        .filter(
                            crate::schema::employee_assignments::status.eq(
                                crate::models::employee_assignment::AssignmentStatusEnum::Active,
                            ),
                        )
                        .filter(crate::schema::employee_assignments::start_date.le(end_date))
                        .filter(
                            crate::schema::employee_assignments::end_date
                                .is_null()
                                .or(crate::schema::employee_assignments::end_date.ge(start_date)),
                        )
                        .select((
                            crate::schema::employee_assignments::all_columns,
                            crate::schema::employments::id,
                        ))
                        .load::<(crate::models::employee_assignment::EmployeeAssignment, Uuid)>(
                            conn,
                        )
                        .await
                        .map_err(|e| AppError::Internal(format!("db batch assignments: {e}")))?;

                    let mut assignments_by_employment: HashMap<
                        Uuid,
                        Vec<crate::models::employee_assignment::EmployeeAssignment>,
                    > = HashMap::new();
                    for (assignment, employment_id) in batch_assignments {
                        assignments_by_employment
                            .entry(employment_id)
                            .or_default()
                            .push(assignment);
                    }

                    // 5. Holidays
                    let holiday_entities = crate::schema::holidays::table
                        .filter(crate::schema::holidays::workspace_id.eq(workspace_id))
                        .filter(crate::schema::holidays::date.ge(start_date))
                        .filter(crate::schema::holidays::date.le(end_date))
                        .load::<crate::models::diesel::holiday::HolidayEntity>(conn)
                        .await
                        .map_err(AppError::from)?;

                    let holidays_map: HashMap<NaiveDate, crate::models::Holiday> = holiday_entities
                        .into_iter()
                        .map(|e| (e.date, e.into()))
                        .collect();

                    // 6. Rosters
                    let rosters_entities = crate::schema::shift_rosters::table
                        .inner_join(crate::schema::employments::table)
                        .filter(crate::schema::employments::workspace_id.eq(workspace_id))
                        .filter(crate::schema::shift_rosters::date.ge(start_date))
                        .filter(crate::schema::shift_rosters::date.le(end_date))
                        .select(crate::schema::shift_rosters::all_columns)
                        .load::<crate::models::diesel::roster::ShiftRosterEntity>(conn)
                        .await
                        .map_err(AppError::from)?;

                    let mut roster_map: HashMap<(Uuid, NaiveDate), Option<Uuid>> = HashMap::new();
                    for r in rosters_entities {
                        roster_map.insert((r.employment_id, r.date), r.template_id);
                    }

                    // 7. Build EmployeeSegments via resolve_work_segments_from_data
                    let mut result = Vec::new();

                    for (
                        emp_id,
                        emp_full_name,
                        emp_id_number,
                        emp_tags,
                        primary_loc,
                        employee_id,
                        employment_start,
                        employment_end,
                    ) in employees_entities
                    {
                        // Active employer
                        let active_employer_id = contracts_by_employment
                            .get(&emp_id)
                            .and_then(|contracts| {
                                contracts
                                    .iter()
                                    .filter(|(c, _)| {
                                        c.start_date <= end_date
                                            && c.end_date.is_none_or(|end| end >= start_date)
                                    })
                                    .max_by_key(|(c, _)| c.start_date)
                            })
                            .map(|(_, employer_id)| *employer_id);

                        // Active assignment location
                        let active_location_id = assignments_by_employment
                            .get(&emp_id)
                            .and_then(|assignments| {
                                assignments
                                    .iter()
                                    .filter(|a| {
                                        a.start_date <= end_date
                                            && a.end_date.is_none_or(|end| end >= start_date)
                                    })
                                    .max_by_key(|a| a.start_date)
                            })
                            .map(|a| a.host_location_id)
                            .or(primary_loc);

                        // Resolve segments only while this temporal employment is effective.
                        let mut all_segments = Vec::new();
                        let Some((mut curr_date, effective_end)) = effective_employment_range(
                            start_date,
                            end_date,
                            employment_start,
                            employment_end,
                        ) else {
                            continue;
                        };
                        while curr_date <= effective_end {
                            let segments =
                    crate::repositories::roster::shift_resolution::resolve_work_segments_from_data(
                        &roster_data,
                        &roster_map,
                        emp_id,
                        employee_id,
                        &emp_id_number,
                        emp_tags.as_ref(),
                        curr_date,
                        active_location_id,
                        active_employer_id,
                    );

                            let holiday = holidays_map.get(&curr_date);
                            for mut seg in segments {
                                seg.is_holiday = holiday.is_some();
                                seg.is_holiday_paid =
                                    holiday.is_some_and(crate::models::Holiday::is_paid);
                                seg.holiday_name = holiday.map(|h| h.name().to_string());
                                all_segments.push(seg);
                            }

                            if let Some(next_date) = curr_date.succ_opt() {
                                curr_date = next_date;
                            } else {
                                break;
                            }
                        }

                        result.push(EmployeeSegments::new(
                            emp_id,
                            employee_id,
                            emp_id_number.clone(),
                            emp_full_name.clone(),
                            all_segments,
                        ));
                    }

                    Ok((result, total))
                })
            })
            .await?;

        // ── Write-through: persist resolved segments to work_segments ──
        // Fire-and-forget: does not block the read path.
        // Strategy: DELETE stale interval-based segments, then INSERT new ones.
        // This handles shift template reconfigurations (intervals added/removed).
        // Manual segments (source='manual') are preserved.
        {
            use crate::models::diesel::work_segment::resolved_segment_to_new_entity;

            let all_new_entities: Vec<_> = result
                .iter()
                .flat_map(|emp_segs| {
                    emp_segs.segments().iter().map(|seg| {
                        resolved_segment_to_new_entity(seg, workspace_id, emp_segs.employment_id())
                    })
                })
                .collect();

            if !all_new_entities.is_empty() {
                let pools = self.diesel_pools.clone();
                let ws_id = workspace_id;
                tokio::spawn(async move {
                    if let Err(e) =
                        Self::persist_workspace_segments(pools, ws_id, all_new_entities).await
                    {
                        tracing::warn!(
                            workspace_id = %ws_id,
                            error = %e,
                            "work_segments.write_through.failed"
                        );
                    }
                });
            }
        }

        let total = if page.is_some() || limit.is_some() {
            total
        } else {
            i64::try_from(result.len()).unwrap_or(0)
        };

        Ok((result, total))
    }
}
