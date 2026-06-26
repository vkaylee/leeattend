use crate::errors::AppError;
use crate::utils::i18n::I18nKey;
use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::diesel_pool::DieselPools;
use crate::models::diesel::roster::ShiftRosterEntity;
use crate::models::{EffectiveSchedule, EmployeeSchedule, RosterItem, ScheduleTrace, ShiftRoster};
use crate::repositories::roster::RosterRepository;
use crate::schema::shift_rosters;
use std::collections::HashMap;

pub struct DieselRosterRepository {
    diesel_pools: DieselPools,
}

impl DieselRosterRepository {
    pub fn new(diesel_pools: DieselPools) -> Self {
        Self { diesel_pools }
    }
}

#[allow(clippy::too_many_lines)]
#[async_trait]
impl RosterRepository for DieselRosterRepository {
    async fn get_workspace_schedules(
        &self,
        roster_cache: Option<&crate::cache::RosterCache>,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        start_date: NaiveDate,
        end_date: NaiveDate,
        employment_ids: Option<&Vec<Uuid>>,
        page: Option<i64>,
        limit: Option<i64>,
    ) -> Result<(Vec<EmployeeSchedule>, i64), AppError> {
        let workspace_id = auth_token.workspace_id();
        // 1. Fetch Roster Data (Cache or DB)
        let roster_data = match roster_cache {
            Some(cache) => cache
                .get_roster_data(workspace_id, &self.diesel_pools)
                .await
                .map_err(|e| AppError::Internal(format!("cache: {e}")))?,
            None => crate::cache::build_roster_data(&self.diesel_pools, workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db build: {e}")))?,
        };

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        // 2. Fetch Employees and Employments
        let mut query = crate::schema::employees::table
            .inner_join(crate::schema::employments::table)
            .filter(crate::schema::employees::workspace_id.eq(workspace_id))
            .filter(crate::schema::employments::deleted_at.is_null())
            .into_boxed();

        if let Some(ids) = employment_ids {
            query = query.filter(crate::schema::employments::id.eq_any(ids));
        } else {
            query = query
                .filter(crate::schema::employments::start_date.le(end_date))
                .filter(
                    crate::schema::employments::end_date
                        .is_null()
                        .or(crate::schema::employments::end_date.ge(start_date)),
                );
        }

        let total = if page.is_some() || limit.is_some() {
            let mut count_query = crate::schema::employees::table
                .inner_join(crate::schema::employments::table)
                .filter(crate::schema::employees::workspace_id.eq(workspace_id))
                .filter(crate::schema::employments::deleted_at.is_null())
                .into_boxed();

            if let Some(ids) = employment_ids {
                count_query = count_query.filter(crate::schema::employments::id.eq_any(ids));
            } else {
                count_query = count_query
                    .filter(crate::schema::employments::start_date.le(end_date))
                    .filter(
                        crate::schema::employments::end_date
                            .is_null()
                            .or(crate::schema::employments::end_date.ge(start_date)),
                    );
            }
            count_query
                .count()
                .get_result::<i64>(&mut *conn)
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
            ))
            .load::<(
                Uuid,
                String,
                String,
                Option<Vec<Option<String>>>,
                Option<Uuid>,
                Uuid,
            )>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let employees_entities: Vec<(
            Uuid,
            String,
            String,
            Option<Vec<String>>,
            Option<Uuid>,
            Uuid,
        )> = employees_entities_raw
            .into_iter()
            .map(|(emp_id, name, num, tags, primary_loc, employee_id)| {
                (
                    emp_id,
                    name,
                    num,
                    tags.map(|v| v.into_iter().flatten().collect()),
                    primary_loc,
                    employee_id,
                )
            })
            .collect();

        // 2a. Batch load employee contracts active in the range
        let emp_ids: Vec<Uuid> = employees_entities
            .iter()
            .map(|(id, _, _, _, _, _)| *id)
            .collect();

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
            )>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db load batch contracts: {e}")))?;

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

        // 2b. Batch load employee assignments active in the range
        let batch_assignments = crate::schema::employee_assignments::table
            .inner_join(crate::schema::employments::table)
            .filter(crate::schema::employee_assignments::workspace_id.eq(workspace_id))
            .filter(crate::schema::employments::id.eq_any(&emp_ids))
            .filter(
                crate::schema::employee_assignments::status
                    .eq(crate::models::employee_assignment::AssignmentStatusEnum::Active),
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
            .load::<(crate::models::employee_assignment::EmployeeAssignment, Uuid)>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db load batch assignments: {e}")))?;

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

        // 3. Fetch Holidays
        let holiday_entities = crate::schema::holidays::table
            .filter(crate::schema::holidays::workspace_id.eq(workspace_id))
            .filter(crate::schema::holidays::date.ge(start_date))
            .filter(crate::schema::holidays::date.le(end_date))
            .load::<crate::models::diesel::holiday::HolidayEntity>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let holidays_map: HashMap<NaiveDate, crate::models::Holiday> = holiday_entities
            .into_iter()
            .map(|e| (e.date, e.into()))
            .collect();

        // 4. Fetch Rosters
        let rosters_entities = crate::schema::shift_rosters::table
            .inner_join(crate::schema::employments::table)
            .filter(crate::schema::employments::workspace_id.eq(workspace_id))
            .filter(crate::schema::shift_rosters::date.ge(start_date))
            .filter(crate::schema::shift_rosters::date.le(end_date))
            .select(crate::schema::shift_rosters::all_columns)
            .load::<crate::models::diesel::roster::ShiftRosterEntity>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let mut roster_map: HashMap<(Uuid, NaiveDate), Option<Uuid>> = HashMap::new();
        for r in rosters_entities {
            roster_map.insert((r.employment_id, r.date), r.shift_id);
        }

        // 5. Build Result
        let mut result = Vec::new();

        for (emp_id, emp_full_name, emp_id_number, emp_tags, primary_loc, employee_id) in
            employees_entities
        {
            let mut daily_schedules = Vec::new();
            let mut curr_date = start_date;

            while curr_date <= end_date {
                // Find active contract on curr_date
                let active_contract = contracts_by_employment.get(&emp_id).and_then(|contracts| {
                    contracts
                        .iter()
                        .filter(|(c, _)| {
                            c.start_date <= curr_date
                                && c.end_date.is_none_or(|end| end >= curr_date)
                        })
                        .max_by_key(|(c, _)| c.start_date)
                });
                let active_employer_id = active_contract.map(|(_, employer_id)| *employer_id);

                // Find active assignment on curr_date
                let active_assignment =
                    assignments_by_employment
                        .get(&emp_id)
                        .and_then(|assignments| {
                            assignments
                                .iter()
                                .filter(|a| {
                                    a.start_date <= curr_date
                                        && a.end_date.is_none_or(|end| end >= curr_date)
                                })
                                .max_by_key(|a| a.start_date)
                        });
                let active_location_id = active_assignment
                    .map(|a| a.host_location_id)
                    .or(primary_loc);

                let (effective_shift_ids, source, traces) =
            crate::repositories::roster::shift_resolution::resolve_effective_shift_from_data(
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

                for effective_shift_id in &effective_shift_ids {
                    // Resolve shift metadata (name/symbol) from RosterShift and
                    // time/break/policy fields from the first RosterInterval.
                    let (
                        shift_name,
                        shift_symbol,
                        start_time,
                        end_time,
                        break_dur,
                        brk_start,
                        brk_end,
                        is_bad,
                        early_grace,
                        allow_eot,
                        bio,
                        max_eot,
                    ) = if let Some(sid) = effective_shift_id {
                        let name = roster_data.shifts.get(sid).map(|s| s.name().to_string());
                        let sym = roster_data
                            .shifts
                            .get(sid)
                            .and_then(|s| s.report_symbol().map(String::from));
                        #[allow(clippy::get_first)]
                        if let Some(first) = roster_data
                            .shift_intervals
                            .get(sid)
                            .and_then(|intervals| intervals.get(0))
                        {
                            (
                                name,
                                sym,
                                Some(first.start_time),
                                Some(first.end_time),
                                Some(first.break_duration_minutes),
                                first.break_start,
                                first.break_end,
                                Some(first.is_break_auto_deduct),
                                Some(first.early_punch_in_grace_period_minutes),
                                Some(first.allow_early_ot),
                                Some(first.break_is_overtime),
                                first.max_early_ot_minutes,
                            )
                        } else {
                            (
                                name, sym, None, None, None, None, None, None, None, None, None,
                                None,
                            )
                        }
                    } else {
                        (
                            None, None, None, None, None, None, None, None, None, None, None, None,
                        )
                    };

                    let sched = EffectiveSchedule::new(
                        curr_date,
                        *effective_shift_id,
                        source.clone(),
                        shift_name,
                        shift_symbol,
                        start_time,
                        end_time,
                        break_dur,
                        brk_start,
                        brk_end,
                        is_bad,
                        early_grace,
                        allow_eot,
                        bio,
                        max_eot,
                        traces.clone(),
                        holiday.is_some(),
                        holiday.is_some_and(crate::models::Holiday::is_paid),
                        holiday.map(|h| h.name().to_string()),
                    );

                    daily_schedules.push(sched);
                }

                if let Some(next_date) = curr_date.succ_opt() {
                    curr_date = next_date;
                } else {
                    break;
                }
            }

            result.push(EmployeeSchedule::new(
                emp_id,
                employee_id,
                emp_id_number.clone(),
                emp_full_name.clone(),
                daily_schedules,
            ));
        }

        let total = if page.is_some() || limit.is_some() {
            total
        } else {
            i64::try_from(result.len()).unwrap_or(0)
        };

        Ok((result, total))
    }

    /// Segment-centric variant — see [`RosterRepository::get_workspace_segments`].
    ///
    /// Shares the exact same data-loading pipeline as
    /// [`get_workspace_schedules`] but resolves work segments (with optional
    /// interval expansion) instead of building flat [`EffectiveSchedule`]
    /// objects.
    #[allow(clippy::too_many_lines)]
    async fn get_workspace_segments(
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

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        // 2. Paginated employees (identical query structure as get_workspace_schedules)
        let mut query = crate::schema::employments::table
            .inner_join(crate::schema::employees::table)
            .filter(crate::schema::employees::workspace_id.eq(workspace_id))
            .filter(crate::schema::employments::status.eq("active"))
            .filter(crate::schema::employments::deleted_at.is_null())
            .filter(crate::schema::employments::start_date.le(end_date))
            .filter(
                crate::schema::employments::end_date
                    .is_null()
                    .or(crate::schema::employments::end_date.ge(start_date)),
            )
            .into_boxed();

        if let Some(eids) = employment_ids {
            query = query.filter(crate::schema::employments::id.eq_any(eids.iter().copied()));
        }

        // Separate count query (build independently to avoid consuming `query`)
        let mut count_query = crate::schema::employments::table
            .inner_join(crate::schema::employees::table)
            .filter(crate::schema::employees::workspace_id.eq(workspace_id))
            .filter(crate::schema::employments::status.eq("active"))
            .filter(crate::schema::employments::deleted_at.is_null())
            .into_boxed();
        if let Some(eids) = employment_ids {
            count_query =
                count_query.filter(crate::schema::employments::id.eq_any(eids.iter().copied()));
        }
        let total: i64 = if page.is_some() || limit.is_some() {
            count_query
                .count()
                .get_result::<i64>(&mut *conn)
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
            ))
            .load::<(
                Uuid,
                String,
                String,
                Option<Vec<Option<String>>>,
                Option<Uuid>,
                Uuid,
            )>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let employees_entities: Vec<(
            Uuid,
            String,
            String,
            Option<Vec<String>>,
            Option<Uuid>,
            Uuid,
        )> = employees_entities_raw
            .into_iter()
            .map(|(emp_id, name, num, tags, primary_loc, employee_id)| {
                (
                    emp_id,
                    name,
                    num,
                    tags.map(|v| v.into_iter().flatten().collect()),
                    primary_loc,
                    employee_id,
                )
            })
            .collect();

        if employees_entities.is_empty() {
            return Ok((Vec::new(), total));
        }

        let emp_ids: Vec<Uuid> = employees_entities
            .iter()
            .map(|(id, _, _, _, _, _)| *id)
            .collect();

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
            )>(&mut *conn)
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
                crate::schema::employee_assignments::status
                    .eq(crate::models::employee_assignment::AssignmentStatusEnum::Active),
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
            .load::<(crate::models::employee_assignment::EmployeeAssignment, Uuid)>(&mut *conn)
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
            .load::<crate::models::diesel::holiday::HolidayEntity>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

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
            .load::<crate::models::diesel::roster::ShiftRosterEntity>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let mut roster_map: HashMap<(Uuid, NaiveDate), Option<Uuid>> = HashMap::new();
        for r in rosters_entities {
            roster_map.insert((r.employment_id, r.date), r.shift_id);
        }

        // 7. Build EmployeeSegments via resolve_work_segments_from_data
        let mut result = Vec::new();

        for (emp_id, emp_full_name, emp_id_number, emp_tags, primary_loc, employee_id) in
            employees_entities
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

            // Resolve segments per-date
            let mut all_segments = Vec::new();
            let mut curr_date = start_date;
            while curr_date <= end_date {
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
                    seg.is_holiday_paid = holiday.is_some_and(crate::models::Holiday::is_paid);
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

        let total = if page.is_some() || limit.is_some() {
            total
        } else {
            i64::try_from(result.len()).unwrap_or(0)
        };

        Ok((result, total))
    }

    async fn get_assignment_history(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        page: Option<i64>,
        limit: Option<i64>,
    ) -> Result<(Vec<crate::models::AssignmentHistory>, i64), AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        #[derive(diesel::QueryableByName)]
        struct CountResult {
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            count: i64,
        }

        let count_res = diesel::sql_query(
            r"
            SELECT COUNT(*)::BIGINT as count FROM (
                -- Fixed Assignments
                SELECT
                    es.id,
                    emp.employee_id,
                    es.created_at
                FROM employee_shifts es
                JOIN employments emp ON es.employment_id = emp.id
                WHERE emp.workspace_id = $1

                UNION ALL

                -- Pattern Assignments
                SELECT
                    ewp.id,
                    emp.employee_id,
                    ewp.created_at
                FROM employee_working_patterns ewp
                JOIN employments emp ON ewp.employment_id = emp.id
                WHERE emp.workspace_id = $1

                UNION ALL

                -- Roster Assignments
                SELECT
                    sr.id,
                    emp.employee_id,
                    sr.created_at
                FROM shift_rosters sr
                JOIN employments emp ON sr.employment_id = emp.id
                WHERE emp.workspace_id = $1
            ) as history
            ",
        )
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .get_result::<CountResult>(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(format!("db history count: {e}")))?;

        let total = count_res.count;

        let limit_val = limit.unwrap_or(50).clamp(1, 3000);
        let offset_val = (page.unwrap_or(1).max(1) - 1) * limit_val;

        let history = diesel::sql_query(r"
            SELECT
                id,
                employee_id,
                (SELECT full_name FROM employees WHERE id = history.employee_id) as employee_name,
                assignment_type,
                shift_name,
                date_range,
                created_at
            FROM (
                -- Fixed Assignments
                SELECT
                    es.id,
                    emp.employee_id,
                    'fixed'::assignment_history_type as assignment_type,
                    s.name as shift_name,
                    TO_CHAR(es.start_date, 'YYYY-MM-DD') ||
                    CASE WHEN es.end_date IS NULL THEN ' -> ∞' ELSE ' -> ' || TO_CHAR(es.end_date, 'YYYY-MM-DD') END as date_range,
                    es.created_at
                FROM employee_shifts es
                JOIN shifts s ON es.shift_id = s.id
                JOIN employments emp ON es.employment_id = emp.id
                WHERE emp.workspace_id = $1

                UNION ALL

                -- Pattern Assignments
                SELECT
                    ewp.id,
                    emp.employee_id,
                    'pattern'::assignment_history_type as assignment_type,
                    wp.name as shift_name,
                    TO_CHAR(ewp.start_date, 'YYYY-MM-DD') ||
                    CASE WHEN ewp.end_date IS NULL THEN ' -> ∞' ELSE ' -> ' || TO_CHAR(ewp.end_date, 'YYYY-MM-DD') END as date_range,
                    ewp.created_at
                FROM employee_working_patterns ewp
                JOIN working_patterns wp ON ewp.pattern_id = wp.id
                JOIN employments emp ON ewp.employment_id = emp.id
                WHERE emp.workspace_id = $1

                UNION ALL

                -- Roster Assignments
                SELECT
                    sr.id,
                    emp.employee_id,
                    'roster'::assignment_history_type as assignment_type,
                    CASE WHEN sr.shift_id IS NULL THEN 'OFF' ELSE s.name END as shift_name,
                    TO_CHAR(sr.date, 'YYYY-MM-DD') as date_range,
                    sr.created_at
                FROM shift_rosters sr
                LEFT JOIN shifts s ON sr.shift_id = s.id
                JOIN employments emp ON sr.employment_id = emp.id
                WHERE emp.workspace_id = $1
            ) as history
            ORDER BY created_at DESC
            limit $2 offset $3
            "
        )
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .bind::<diesel::sql_types::BigInt, _>(limit_val)
        .bind::<diesel::sql_types::BigInt, _>(offset_val)
        .load::<crate::models::AssignmentHistory>(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(format!("db history: {e}")))?;

        Ok((history, total))
    }

    async fn get_employment_rosters(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
        page: Option<i64>,
        limit: Option<i64>,
    ) -> Result<(Vec<ShiftRoster>, i64), AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let mut query = crate::schema::shift_rosters::table
            .inner_join(crate::schema::employments::table)
            .filter(crate::schema::employments::workspace_id.eq(workspace_id))
            .filter(crate::schema::shift_rosters::employment_id.eq(employment_id))
            .filter(crate::schema::shift_rosters::date.ge(start_date))
            .filter(crate::schema::shift_rosters::date.le(end_date))
            .into_boxed();

        let total = if page.is_some() || limit.is_some() {
            let count_query = crate::schema::shift_rosters::table
                .inner_join(crate::schema::employments::table)
                .filter(crate::schema::employments::workspace_id.eq(workspace_id))
                .filter(crate::schema::shift_rosters::employment_id.eq(employment_id))
                .filter(crate::schema::shift_rosters::date.ge(start_date))
                .filter(crate::schema::shift_rosters::date.le(end_date))
                .into_boxed();
            count_query
                .count()
                .get_result::<i64>(&mut *conn)
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

        let rosters = query
            .select(crate::schema::shift_rosters::all_columns)
            .load::<crate::models::diesel::roster::ShiftRosterEntity>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let data: Vec<ShiftRoster> = rosters.into_iter().map(Into::into).collect();
        let total = if page.is_some() || limit.is_some() {
            total
        } else {
            i64::try_from(data.len()).unwrap_or(0)
        };

        Ok((data, total))
    }

    async fn get_employment_rosters_batch(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_ids: &[Uuid],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<std::collections::HashMap<Uuid, Vec<ShiftRoster>>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let rosters = crate::schema::shift_rosters::table
            .inner_join(crate::schema::employments::table)
            .filter(crate::schema::employments::workspace_id.eq(workspace_id))
            .filter(crate::schema::shift_rosters::employment_id.eq_any(employment_ids))
            .filter(crate::schema::shift_rosters::date.ge(start_date))
            .filter(crate::schema::shift_rosters::date.le(end_date))
            .select(crate::schema::shift_rosters::all_columns)
            .load::<crate::models::diesel::roster::ShiftRosterEntity>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let mut map: std::collections::HashMap<Uuid, Vec<ShiftRoster>> =
            std::collections::HashMap::new();
        for roster in rosters {
            map.entry(roster.employment_id)
                .or_default()
                .push(roster.into());
        }
        Ok(map)
    }

    async fn upsert_roster(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        roster: &ShiftRoster,
    ) -> Result<ShiftRoster, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        // Check if employment exists and belongs to the workspace of this tenant
        let employment_exists = crate::schema::employments::table
            .filter(crate::schema::employments::id.eq(roster.employment_id))
            .filter(crate::schema::employments::workspace_id.eq(workspace_id))
            .filter(crate::schema::employments::deleted_at.is_null())
            .select(crate::schema::employments::id)
            .first::<Uuid>(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db check employment: {e}")))?;

        if employment_exists.is_none() {
            return Err(AppError::not_found(I18nKey::EmploymentNotFound));
        }

        let entity: crate::models::diesel::roster::ShiftRosterEntity = roster.clone().into();

        let result = diesel::insert_into(crate::schema::shift_rosters::table)
            .values(&entity)
            .on_conflict((
                crate::schema::shift_rosters::employment_id,
                crate::schema::shift_rosters::date,
            ))
            .do_update()
            .set((
                crate::schema::shift_rosters::shift_id.eq(entity.shift_id),
                crate::schema::shift_rosters::status.eq(entity.status.clone()),
                crate::schema::shift_rosters::notes.eq(entity.notes.clone()),
                crate::schema::shift_rosters::updated_at.eq(Utc::now()),
            ))
            .get_result::<crate::models::diesel::roster::ShiftRosterEntity>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result.into())
    }

    async fn bulk_upsert_roster(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        items: Vec<RosterItem>,
    ) -> Result<(), AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        // Check if employment exists and belongs to the workspace of this tenant
        let employment_exists = crate::schema::employments::table
            .filter(crate::schema::employments::id.eq(employment_id))
            .filter(crate::schema::employments::workspace_id.eq(workspace_id))
            .filter(crate::schema::employments::deleted_at.is_null())
            .select(crate::schema::employments::id)
            .first::<Uuid>(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db check employment: {e}")))?;

        if employment_exists.is_none() {
            return Err(AppError::not_found(I18nKey::EmploymentNotFound));
        }

        let entities: Vec<ShiftRosterEntity> = items
            .into_iter()
            .map(|item| ShiftRosterEntity {
                id: crate::utils::id::new_id(),
                employment_id,
                shift_id: item.shift_id,
                date: item.date,
                status: Some("confirmed".to_string()),
                notes: item.notes,
                created_at: Some(Utc::now()),
                updated_at: Some(Utc::now()),
            })
            .collect();

        diesel::insert_into(shift_rosters::table)
            .values(&entities)
            .on_conflict((shift_rosters::employment_id, shift_rosters::date))
            .do_update()
            .set((
                shift_rosters::shift_id.eq(diesel::dsl::sql("EXCLUDED.shift_id")),
                shift_rosters::status.eq(diesel::dsl::sql("EXCLUDED.status")),
                shift_rosters::notes.eq(diesel::dsl::sql("EXCLUDED.notes")),
                shift_rosters::updated_at.eq(Utc::now()),
            ))
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }

    async fn get_effective_shift(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        date: NaiveDate,
    ) -> Result<(Option<Uuid>, Vec<ScheduleTrace>), AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let mut traces = Vec::new();

        // 1. Resolve actual_employment_id (in case employee_id was passed)
        let actual_employment_id = {
            let direct_exists = crate::schema::employments::table
                .filter(crate::schema::employments::id.eq(employment_id))
                .filter(crate::schema::employments::workspace_id.eq(workspace_id))
                .filter(crate::schema::employments::deleted_at.is_null())
                .select(crate::schema::employments::id)
                .first::<Uuid>(&mut *conn)
                .await
                .optional()
                .map_err(|e| AppError::Internal(format!("db check employment: {e}")))?;

            if let Some(id) = direct_exists {
                id
            } else {
                // Try to find the active employment for this employee ID
                let active_emp = crate::schema::employments::table
                    .filter(crate::schema::employments::employee_id.eq(employment_id))
                    .filter(crate::schema::employments::workspace_id.eq(workspace_id))
                    .filter(crate::schema::employments::status.eq("active"))
                    .filter(crate::schema::employments::deleted_at.is_null())
                    .select(crate::schema::employments::id)
                    .first::<Uuid>(&mut *conn)
                    .await
                    .optional()
                    .map_err(|e| AppError::Internal(format!("db select active employment: {e}")))?;

                active_emp.ok_or_else(|| AppError::not_found(I18nKey::EmploymentNotFound))?
            }
        };

        // Priority 1: Shift Roster
        let roster_shift: Option<Option<Uuid>> = crate::schema::shift_rosters::table
            .inner_join(crate::schema::employments::table)
            .filter(crate::schema::shift_rosters::employment_id.eq(actual_employment_id))
            .filter(crate::schema::shift_rosters::date.eq(date))
            .filter(crate::schema::employments::workspace_id.eq(workspace_id))
            .select(crate::schema::shift_rosters::shift_id)
            .first::<Option<Uuid>>(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        if let Some(shift) = roster_shift {
            traces.push(ScheduleTrace {
                rule_id: None,
                rule_name: "Manual Roster".to_string(),
                priority: 100,
                status: "applied".to_string(),
                reason: Some("Directly assigned via roster".to_string()),
            });
            return Ok((shift, traces));
        }

        // Fetch the employee_id associated with this employment to check patterns and default shifts
        let emp_info = crate::schema::employments::table
            .filter(crate::schema::employments::id.eq(actual_employment_id))
            .filter(crate::schema::employments::workspace_id.eq(workspace_id))
            .select(crate::schema::employments::employee_id)
            .first::<Uuid>(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db select employee from employment: {e}")))?;

        let Some(employee_id) = emp_info else {
            return Err(AppError::not_found(I18nKey::EmploymentNotFound));
        };

        // Priority 2: Working Pattern
        let dow =
            i32::try_from(chrono::Datelike::weekday(&date).num_days_from_sunday()).unwrap_or(0);
        let pattern_info: Option<(Option<Uuid>, Uuid)> =
            crate::schema::working_pattern_details::table
                .inner_join(
                    crate::schema::employee_working_patterns::table
                        .on(crate::schema::employee_working_patterns::pattern_id
                            .eq(crate::schema::working_pattern_details::pattern_id)),
                )
                .inner_join(
                    crate::schema::employments::table.on(crate::schema::employments::id
                        .eq(crate::schema::employee_working_patterns::employment_id)),
                )
                .filter(
                    crate::schema::employee_working_patterns::employment_id
                        .eq(actual_employment_id),
                )
                .filter(crate::schema::employee_working_patterns::start_date.le(date))
                .filter(
                    crate::schema::employee_working_patterns::end_date
                        .is_null()
                        .or(crate::schema::employee_working_patterns::end_date.ge(date)),
                )
                .filter(crate::schema::working_pattern_details::schedule_day.eq(dow))
                .filter(crate::schema::employments::workspace_id.eq(workspace_id))
                .order(crate::schema::employee_working_patterns::created_at.desc())
                .select((
                    crate::schema::working_pattern_details::shift_id,
                    crate::schema::employee_working_patterns::pattern_id,
                ))
                .first::<(Option<Uuid>, Uuid)>(&mut *conn)
                .await
                .optional()
                .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        if let Some((shift_id, pattern_id)) = pattern_info {
            traces.push(ScheduleTrace {
                rule_id: Some(pattern_id),
                rule_name: "Individual Pattern".to_string(),
                priority: 80,
                status: "applied".to_string(),
                reason: Some("Assigned via individual working pattern".to_string()),
            });
            return Ok((shift_id, traces));
        }

        // Priority 3: Dynamic Assignment Rules
        let employee_full = crate::schema::employees::table
            .inner_join(crate::schema::employments::table)
            .filter(crate::schema::employees::id.eq(employee_id))
            .filter(crate::schema::employees::workspace_id.eq(workspace_id))
            .filter(crate::schema::employments::deleted_at.is_null())
            .filter(crate::schema::employments::start_date.le(date))
            .filter(
                crate::schema::employments::end_date
                    .is_null()
                    .or(crate::schema::employments::end_date.ge(date)),
            )
            .select((
                crate::schema::employees::employee_id_number,
                crate::schema::employees::tags,
                crate::schema::employments::department_id,
                crate::schema::employments::primary_location_id,
            ))
            .first::<(
                String,
                Option<Vec<Option<String>>>,
                Option<Uuid>,
                Option<Uuid>,
            )>(&mut *conn)
            .await
            .optional()
            .map(|opt| {
                opt.map(|(num, tags, dept, primary_loc)| {
                    (
                        num,
                        tags.map(|v| v.into_iter().flatten().collect()),
                        dept,
                        primary_loc,
                    )
                })
            })
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        if let Some((emp_id_num, tags, dept_id, primary_loc)) = employee_full {
            let active_assignment: Option<Uuid> = crate::schema::employee_assignments::table
                .inner_join(crate::schema::employments::table)
                .filter(crate::schema::employments::id.eq(actual_employment_id))
                .filter(crate::schema::employee_assignments::workspace_id.eq(workspace_id))
                .filter(
                    crate::schema::employee_assignments::status
                        .eq(crate::models::employee_assignment::AssignmentStatusEnum::Active),
                )
                .filter(crate::schema::employee_assignments::start_date.le(date))
                .filter(
                    crate::schema::employee_assignments::end_date
                        .is_null()
                        .or(crate::schema::employee_assignments::end_date.ge(date)),
                )
                .select(crate::schema::employee_assignments::host_location_id)
                .first::<Uuid>(&mut *conn)
                .await
                .optional()
                .map_err(|e| AppError::Internal(format!("db select assignment: {e}")))?;

            let active_location_id = active_assignment.or(primary_loc);

            let active_employer: Option<Uuid> = crate::schema::employee_contracts::table
                .inner_join(crate::schema::employments::table)
                .filter(crate::schema::employments::id.eq(actual_employment_id))
                .filter(crate::schema::employee_contracts::workspace_id.eq(workspace_id))
                .filter(
                    crate::schema::employee_contracts::status
                        .eq(crate::models::enums::ContractStatus::Active),
                )
                .filter(crate::schema::employee_contracts::deleted_at.is_null())
                .filter(crate::schema::employee_contracts::start_date.le(date))
                .filter(
                    crate::schema::employee_contracts::end_date
                        .is_null()
                        .or(crate::schema::employee_contracts::end_date.ge(date)),
                )
                .select(crate::schema::employments::employer_id)
                .first::<Uuid>(&mut *conn)
                .await
                .optional()
                .map_err(|e| AppError::Internal(format!("db select contract: {e}")))?;

            let rules_entities: Vec<crate::models::diesel::assignment_rule::AssignmentRuleEntity> =
                crate::schema::assignment_rules::table
                    .filter(crate::schema::assignment_rules::workspace_id.eq(workspace_id))
                    .filter(crate::schema::assignment_rules::is_active.eq(true))
                    .order((
                        crate::schema::assignment_rules::priority.desc(),
                        crate::schema::assignment_rules::created_at.asc(),
                    ))
                    .load(&mut *conn)
                    .await
                    .map_err(|e| AppError::Internal(format!("db: {e}")))?;

            let rules: Vec<crate::models::employee::AssignmentRule> =
                rules_entities.into_iter().map(Into::into).collect();

            if !rules.is_empty() {
                let match_ctx = crate::services::rule_evaluator::EmployeeMatchContext {
                    date,
                    department_id: dept_id,
                    employee_id_number: emp_id_num,
                    tags: tags.unwrap_or_default(),
                    location_id: active_location_id,
                    employer_id: active_employer,
                };

                let (matched_rule, rule_traces) =
                    crate::services::rule_evaluator::RuleEvaluator::resolve_with_trace(
                        &match_ctx, &rules,
                    );
                traces.extend(rule_traces);

                if let Some(rule) = matched_rule {
                    let shift_id: Option<Option<Uuid>> =
                        crate::schema::working_pattern_details::table
                            .filter(
                                crate::schema::working_pattern_details::pattern_id
                                    .eq(rule.pattern_id()),
                            )
                            .filter(crate::schema::working_pattern_details::schedule_day.eq(dow))
                            .select(crate::schema::working_pattern_details::shift_id)
                            .first::<Option<Uuid>>(&mut *conn)
                            .await
                            .optional()
                            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

                    if let Some(s) = shift_id {
                        return Ok((s, traces));
                    }
                }
            }
        }

        // Priority 5: Employee Default Shift
        let employee_shift: Option<Uuid> = crate::schema::employee_shifts::table
            .inner_join(crate::schema::employments::table.on(
                crate::schema::employments::id.eq(crate::schema::employee_shifts::employment_id),
            ))
            .filter(crate::schema::employee_shifts::employment_id.eq(actual_employment_id))
            .filter(crate::schema::employee_shifts::start_date.le(date))
            .filter(
                crate::schema::employee_shifts::end_date
                    .is_null()
                    .or(crate::schema::employee_shifts::end_date.ge(date)),
            )
            .filter(crate::schema::employments::workspace_id.eq(workspace_id))
            .order(crate::schema::employee_shifts::created_at.desc())
            .select(crate::schema::employee_shifts::shift_id)
            .first::<Uuid>(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        if let Some(shift) = employee_shift {
            traces.push(ScheduleTrace {
                rule_id: None,
                rule_name: "Fixed Assignment".to_string(),
                priority: 20,
                status: "applied".to_string(),
                reason: Some("Directly assigned shift".to_string()),
            });
            return Ok((Some(shift), traces));
        }

        Ok((None, traces))
    }

    async fn delete_roster(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        date: NaiveDate,
    ) -> Result<(), AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        // Check if employment exists and belongs to the workspace of this tenant
        let employment_exists = crate::schema::employments::table
            .filter(crate::schema::employments::id.eq(employment_id))
            .filter(crate::schema::employments::workspace_id.eq(workspace_id))
            .filter(crate::schema::employments::deleted_at.is_null())
            .select(crate::schema::employments::id)
            .first::<Uuid>(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db check employment: {e}")))?;

        if employment_exists.is_none() {
            return Err(AppError::not_found(I18nKey::EmploymentNotFound));
        }

        diesel::delete(
            shift_rosters::table
                .filter(shift_rosters::employment_id.eq(employment_id))
                .filter(shift_rosters::date.eq(date)),
        )
        .execute(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }

    async fn get_raw_roster_data(
        &self,
        workspace_id: Uuid,
    ) -> Result<std::sync::Arc<crate::cache::RosterData>, AppError> {
        crate::cache::build_roster_data(&self.diesel_pools, workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db build: {e}")))
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
        let _repo = DieselRosterRepository::new(pools);
        // DI connection test passes if it compiles
    }

    #[tokio::test]
    async fn test_repo_methods_error_handling() {
        let bad_url = "postgres://invalid:invalid@localhost:12345/invalid_db";
        let pools = DieselPools::for_test(bad_url);
        let repo = DieselRosterRepository::new(pools);
        let ws_id = Uuid::new_v4();
        let auth_token = crate::diesel_pool::TenantAuthToken::system_override(ws_id);
        let date = chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

        let err = repo
            .get_workspace_schedules(None, &auth_token, date, date, None, None, None)
            .await;
        assert!(err.is_err());

        let err = repo
            .get_assignment_history(&auth_token, None, Some(10))
            .await;
        assert!(err.is_err());

        let err = repo
            .get_employment_rosters(&auth_token, Uuid::new_v4(), date, date, None, None)
            .await;
        assert!(err.is_err());

        let err = repo
            .get_employment_rosters_batch(&auth_token, &[Uuid::new_v4()], date, date)
            .await;
        assert!(err.is_err());

        let roster = crate::models::ShiftRoster {
            id: Uuid::new_v4(),
            employment_id: Uuid::new_v4(),
            shift_id: None,
            date,
            status: "confirmed".to_string(),
            notes: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let err = repo.upsert_roster(&auth_token, &roster).await;
        assert!(err.is_err());

        let err = repo
            .bulk_upsert_roster(&auth_token, Uuid::new_v4(), vec![])
            .await;
        assert!(err.is_err());

        let err = repo
            .get_effective_shift(&auth_token, Uuid::new_v4(), date)
            .await;
        assert!(err.is_err());

        let err = repo.delete_roster(&auth_token, Uuid::new_v4(), date).await;
        assert!(err.is_err());

        let err = repo.get_raw_roster_data(ws_id).await;
        assert!(err.is_err());
    }
}
