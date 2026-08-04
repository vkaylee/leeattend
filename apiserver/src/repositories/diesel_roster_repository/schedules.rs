use super::*;
use crate::errors::AppError;
use crate::models::{EffectiveSchedule, EmployeeSchedule, NewEffectiveScheduleParams};
use chrono::NaiveDate;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use std::collections::HashMap;
use uuid::Uuid;

#[allow(clippy::too_many_lines, clippy::too_many_arguments)]
impl DieselRosterRepository {
    pub(super) async fn get_workspace_schedules(
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
            )>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        #[allow(clippy::type_complexity)]
        let employees_entities: Vec<(
            Uuid,
            String,
            String,
            Option<Vec<String>>,
            Option<Uuid>,
            Uuid,
            NaiveDate,
            Option<NaiveDate>,
        )> = employees_entities_raw
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

        // 2a. Batch load employee contracts active in the range
        let emp_ids: Vec<Uuid> = employees_entities.iter().map(|(id, ..)| *id).collect();

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
            .map_err(AppError::from)?;

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
            .map_err(AppError::from)?;

        let mut roster_map: HashMap<(Uuid, NaiveDate), Option<Uuid>> = HashMap::new();
        for r in rosters_entities {
            roster_map.insert((r.employment_id, r.date), r.template_id);
        }

        // 5. Build Result
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
            let mut daily_schedules = Vec::new();
            let Some((mut curr_date, effective_end)) =
                effective_employment_range(start_date, end_date, employment_start, employment_end)
            else {
                continue;
            };

            while curr_date <= effective_end {
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

                    let sched = EffectiveSchedule::new(NewEffectiveScheduleParams {
                        date: curr_date,
                        shift_id: *effective_shift_id,
                        source: source.clone(),
                        shift_name: shift_name,
                        shift_symbol: shift_symbol,
                        start_time: start_time,
                        end_time: end_time,
                        break_duration_minutes: break_dur,
                        break_start: brk_start,
                        break_end: brk_end,
                        is_break_auto_deduct: is_bad,
                        early_punch_in_grace_period_minutes: early_grace,
                        allow_early_ot: allow_eot,
                        break_is_overtime: bio,
                        max_early_ot_minutes: max_eot,
                        traces: traces.clone(),
                        is_holiday: holiday.is_some(),
                        is_holiday_paid: holiday.is_some_and(crate::models::Holiday::is_paid),
                        holiday_name: holiday.map(|h| h.name().to_string()),
                    });

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
}
