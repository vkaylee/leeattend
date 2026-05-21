use crate::errors::AppError;
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
    pub diesel_pools: DieselPools,
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
        workspace_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
        employee_ids: Option<&Vec<Uuid>>,
    ) -> Result<Vec<EmployeeSchedule>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenants
            .get_pool(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db pool error: {e}")))?
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        // 1. Fetch Employees
        let mut query = crate::schema::employees::table
            .filter(crate::schema::employees::workspace_id.eq(workspace_id))
            .filter(crate::schema::employees::is_active.eq(true))
            .into_boxed();

        if let Some(ids) = employee_ids {
            query = query.filter(crate::schema::employees::id.eq_any(ids));
        }

        let employees_entities_raw = query
            .select((
                crate::schema::employees::id,
                crate::schema::employees::full_name,
                crate::schema::employees::employee_id_number,
                crate::schema::employees::tags,
            ))
            .load::<(Uuid, String, String, Option<Vec<Option<String>>>)>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let employees_entities: Vec<(Uuid, String, String, Option<Vec<String>>)> =
            employees_entities_raw
                .into_iter()
                .map(|(id, name, num, tags)| {
                    (
                        id,
                        name,
                        num,
                        tags.map(|v| v.into_iter().flatten().collect()),
                    )
                })
                .collect();

        // 2. Fetch Roster Data (Cache or DB)
        let roster_data = match roster_cache {
            Some(cache) => cache
                .get_roster_data(workspace_id, &self.diesel_pools)
                .await
                .map_err(|e| AppError::Internal(format!("cache: {e}")))?,
            None => crate::cache::build_roster_data(&self.diesel_pools, workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db build: {e}")))?,
        };

        // 3. Fetch Holidays
        let holiday_entities = crate::schema::holidays::table
            .filter(crate::schema::holidays::workspace_id.eq(workspace_id))
            .filter(crate::schema::holidays::date.ge(start_date))
            .filter(crate::schema::holidays::date.le(end_date))
            .load::<crate::models::diesel::holiday::HolidayEntity>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let holidays_map: HashMap<NaiveDate, crate::models::Holiday> = holiday_entities
            .into_iter()
            .map(|e| (e.date, e.into()))
            .collect();

        // 4. Fetch Rosters
        let rosters_entities = crate::schema::shift_rosters::table
            .inner_join(crate::schema::employees::table)
            .filter(crate::schema::employees::workspace_id.eq(workspace_id))
            .filter(crate::schema::shift_rosters::date.ge(start_date))
            .filter(crate::schema::shift_rosters::date.le(end_date))
            .select(crate::schema::shift_rosters::all_columns)
            .load::<crate::models::diesel::roster::ShiftRosterEntity>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let mut roster_map: HashMap<(Uuid, NaiveDate), Option<Uuid>> = HashMap::new();
        for r in rosters_entities {
            roster_map.insert((r.employee_id, r.date), r.shift_id);
        }

        // 5. Build Result
        let mut result = Vec::new();

        for (emp_id, emp_full_name, emp_id_number, emp_tags) in employees_entities {
            let mut daily_schedules = Vec::new();
            let mut curr_date = start_date;

            while curr_date <= end_date {
                let (effective_shift_ids, source, traces) =
                    crate::repositories::roster::shift_resolution::resolve_effective_shift_from_data(
                        &roster_data,
                        &roster_map,
                        emp_id,
                        &emp_id_number,
                        emp_tags.as_ref(),
                        curr_date,
                    );

                let holiday = holidays_map.get(&curr_date);

                for effective_shift_id in &effective_shift_ids {
                    let mut sched = EffectiveSchedule {
                        date: curr_date,
                        shift_id: *effective_shift_id,
                        source: source.clone(),
                        shift_name: None,
                        shift_symbol: None,
                        start_time: None,
                        end_time: None,
                        break_duration_minutes: None,
                        break_start: None,
                        break_end: None,
                        is_break_auto_deduct: None,
                        early_punch_in_grace_period_minutes: None,
                        allow_early_ot: None,
                        traces: traces.clone(),
                        is_holiday: holiday.is_some(),
                        holiday_name: holiday.map(|h| h.name.clone()),
                    };

                    if let Some(sid) = effective_shift_id {
                        if let Some(shift) = roster_data.shifts.get(sid) {
                            sched.shift_name = Some(shift.name.clone());
                            sched.shift_symbol = shift.report_symbol.clone();
                            sched.start_time = Some(shift.start_time);
                            sched.end_time = Some(shift.end_time);
                            sched.break_duration_minutes = Some(shift.break_duration_minutes);
                            sched.break_start = shift.break_start;
                            sched.break_end = shift.break_end;
                            sched.is_break_auto_deduct = Some(shift.is_break_auto_deduct);
                            sched.early_punch_in_grace_period_minutes =
                                Some(shift.early_punch_in_grace_period_minutes);
                            sched.allow_early_ot = Some(shift.allow_early_ot);
                        }
                    }

                    daily_schedules.push(sched);
                }

                if let Some(next_date) = curr_date.succ_opt() {
                    curr_date = next_date;
                } else {
                    break;
                }
            }

            result.push(EmployeeSchedule {
                employee_id: emp_id,
                employee_id_number: emp_id_number,
                full_name: emp_full_name,
                schedules: daily_schedules,
            });
        }

        Ok(result)
    }

    async fn get_assignment_history(
        &self,
        workspace_id: Uuid,
        limit: i64,
    ) -> Result<Vec<crate::models::AssignmentHistory>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenants
            .get_pool(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db pool error: {e}")))?
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let history = diesel::sql_query(
            r#"
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
                    es.employee_id,
                    'fixed'::assignment_history_type as assignment_type,
                    s.name as shift_name,
                    TO_CHAR(es.start_date, 'YYYY-MM-DD') || 
                    CASE WHEN es.end_date IS NULL THEN ' -> ∞' ELSE ' -> ' || TO_CHAR(es.end_date, 'YYYY-MM-DD') END as date_range,
                    es.created_at
                FROM employee_shifts es
                JOIN shifts s ON es.shift_id = s.id
                JOIN employees e ON es.employee_id = e.id
                WHERE e.workspace_id = $1

                UNION ALL

                -- Pattern Assignments
                SELECT 
                    ewp.id,
                    ewp.employee_id,
                    'pattern'::assignment_history_type as assignment_type,
                    wp.name as shift_name,
                    TO_CHAR(ewp.start_date, 'YYYY-MM-DD') || 
                    CASE WHEN ewp.end_date IS NULL THEN ' -> ∞' ELSE ' -> ' || TO_CHAR(ewp.end_date, 'YYYY-MM-DD') END as date_range,
                    ewp.created_at
                FROM employee_working_patterns ewp
                JOIN working_patterns wp ON ewp.pattern_id = wp.id
                JOIN employees e ON ewp.employee_id = e.id
                WHERE ewp.workspace_id = $1

                UNION ALL

                -- Roster Assignments
                SELECT 
                    sr.id,
                    sr.employee_id,
                    'roster'::assignment_history_type as assignment_type,
                    CASE WHEN sr.shift_id IS NULL THEN 'OFF' ELSE s.name END as shift_name,
                    TO_CHAR(sr.date, 'YYYY-MM-DD') as date_range,
                    sr.created_at
                FROM shift_rosters sr
                LEFT JOIN shifts s ON sr.shift_id = s.id
                JOIN employees e ON sr.employee_id = e.id
                WHERE e.workspace_id = $1

                UNION ALL

                -- Department Patterns
                SELECT 
                    edh.id,
                    edh.employee_id,
                    'department_pattern'::assignment_history_type as assignment_type,
                    wp.name as shift_name,
                    TO_CHAR(edh.effective_date, 'YYYY-MM-DD') || ' -> ∞' as date_range,
                    edh.created_at
                FROM employee_department_history edh
                JOIN departments d ON edh.department_id = d.id
                JOIN working_patterns wp ON d.default_working_pattern_id = wp.id
                JOIN employees e ON edh.employee_id = e.id
                WHERE e.workspace_id = $1

                UNION ALL

                -- Department Fixed
                SELECT 
                    edh.id,
                    edh.employee_id,
                    'department_fixed'::assignment_history_type as assignment_type,
                    s.name as shift_name,
                    TO_CHAR(edh.effective_date, 'YYYY-MM-DD') || ' -> ∞' as date_range,
                    edh.created_at
                FROM employee_department_history edh
                JOIN department_default_shifts dds ON edh.department_id = dds.department_id
                JOIN shifts s ON dds.shift_id = s.id
                JOIN employees e ON edh.employee_id = e.id
                WHERE e.workspace_id = $1
            ) as history
            ORDER BY created_at DESC
            LIMIT $2
            "#
        )
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .bind::<diesel::sql_types::BigInt, _>(limit)
        .load::<crate::models::AssignmentHistory>(&mut conn)
        .await
        .map_err(|e| AppError::Internal(format!("db history: {e}")))?;

        Ok(history)
    }

    async fn get_employee_rosters(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<ShiftRoster>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenants
            .get_pool(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db pool error: {e}")))?
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let rosters = crate::schema::shift_rosters::table
            .filter(crate::schema::shift_rosters::employee_id.eq(employee_id))
            .filter(crate::schema::shift_rosters::date.ge(start_date))
            .filter(crate::schema::shift_rosters::date.le(end_date))
            .load::<crate::models::diesel::roster::ShiftRosterEntity>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(rosters.into_iter().map(Into::into).collect())
    }

    async fn upsert_roster(
        &self,
        workspace_id: Uuid,
        roster: &ShiftRoster,
    ) -> Result<ShiftRoster, AppError> {
        let mut conn = self
            .diesel_pools
            .tenants
            .get_pool(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db pool error: {e}")))?
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entity: crate::models::diesel::roster::ShiftRosterEntity = roster.clone().into();

        let result = diesel::insert_into(crate::schema::shift_rosters::table)
            .values(&entity)
            .on_conflict((
                crate::schema::shift_rosters::employee_id,
                crate::schema::shift_rosters::date,
            ))
            .do_update()
            .set((
                crate::schema::shift_rosters::shift_id.eq(entity.shift_id),
                crate::schema::shift_rosters::status.eq(entity.status.clone()),
                crate::schema::shift_rosters::notes.eq(entity.notes.clone()),
                crate::schema::shift_rosters::updated_at.eq(Utc::now()),
            ))
            .get_result::<crate::models::diesel::roster::ShiftRosterEntity>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result.into())
    }

    async fn bulk_upsert_roster(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        items: Vec<RosterItem>,
    ) -> Result<(), AppError> {
        let mut conn = self
            .diesel_pools
            .tenants
            .get_pool(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db pool error: {e}")))?
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entities: Vec<ShiftRosterEntity> = items
            .into_iter()
            .map(|item| ShiftRosterEntity {
                id: crate::utils::id::new_id(),
                employee_id,
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
            .on_conflict((shift_rosters::employee_id, shift_rosters::date))
            .do_update()
            .set((
                shift_rosters::shift_id.eq(diesel::dsl::sql("EXCLUDED.shift_id")),
                shift_rosters::status.eq(diesel::dsl::sql("EXCLUDED.status")),
                shift_rosters::notes.eq(diesel::dsl::sql("EXCLUDED.notes")),
                shift_rosters::updated_at.eq(Utc::now()),
            ))
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }

    async fn get_effective_shift(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        date: NaiveDate,
    ) -> Result<(Option<Uuid>, Vec<ScheduleTrace>), AppError> {
        let mut conn = self
            .diesel_pools
            .tenants
            .get_pool(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db pool error: {e}")))?
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let mut traces = Vec::new();

        // Priority 1: Shift Roster
        let roster_shift: Option<Option<Uuid>> = crate::schema::shift_rosters::table
            .inner_join(crate::schema::employees::table)
            .filter(crate::schema::shift_rosters::employee_id.eq(employee_id))
            .filter(crate::schema::shift_rosters::date.eq(date))
            .filter(crate::schema::employees::workspace_id.eq(workspace_id))
            .select(crate::schema::shift_rosters::shift_id)
            .first::<Option<Uuid>>(&mut conn)
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

        // Priority 2: Working Pattern
        let dow = chrono::Datelike::weekday(&date).num_days_from_sunday() as i32;
        let pattern_info: Option<(Option<Uuid>, Uuid)> =
            crate::schema::working_pattern_details::table
                .inner_join(
                    crate::schema::employee_working_patterns::table
                        .on(crate::schema::employee_working_patterns::pattern_id
                            .eq(crate::schema::working_pattern_details::pattern_id)),
                )
                .inner_join(
                    crate::schema::employees::table.on(crate::schema::employees::id
                        .eq(crate::schema::employee_working_patterns::employee_id)),
                )
                .filter(crate::schema::employee_working_patterns::employee_id.eq(employee_id))
                .filter(crate::schema::employee_working_patterns::start_date.le(date))
                .filter(
                    crate::schema::employee_working_patterns::end_date
                        .is_null()
                        .or(crate::schema::employee_working_patterns::end_date.ge(date)),
                )
                .filter(crate::schema::working_pattern_details::schedule_day.eq(dow))
                .filter(crate::schema::employees::workspace_id.eq(workspace_id))
                .order(crate::schema::employee_working_patterns::created_at.desc())
                .select((
                    crate::schema::working_pattern_details::shift_id,
                    crate::schema::employee_working_patterns::pattern_id,
                ))
                .first::<(Option<Uuid>, Uuid)>(&mut conn)
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
            .left_join(
                crate::schema::employee_department_history::table.on(
                    crate::schema::employee_department_history::employee_id
                        .eq(crate::schema::employees::id)
                        .and(crate::schema::employee_department_history::effective_date.le(date)),
                ),
            )
            .filter(crate::schema::employees::id.eq(employee_id))
            .filter(crate::schema::employees::workspace_id.eq(workspace_id))
            .order(crate::schema::employee_department_history::effective_date.desc())
            .select((
                crate::schema::employees::employee_id_number,
                crate::schema::employees::tags,
                crate::schema::employee_department_history::department_id.nullable(),
            ))
            .first::<(String, Option<Vec<Option<String>>>, Option<Uuid>)>(&mut conn)
            .await
            .optional()
            .map(|opt| {
                opt.map(|(num, tags, dept)| {
                    (num, tags.map(|v| v.into_iter().flatten().collect()), dept)
                })
            })
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        if let Some((emp_id_num, tags, dept_id)) = employee_full {
            let rules_entities: Vec<crate::models::diesel::assignment_rule::AssignmentRuleEntity> =
                crate::schema::assignment_rules::table
                    .filter(crate::schema::assignment_rules::workspace_id.eq(workspace_id))
                    .filter(crate::schema::assignment_rules::is_active.eq(true))
                    .order((
                        crate::schema::assignment_rules::priority.desc(),
                        crate::schema::assignment_rules::created_at.asc(),
                    ))
                    .load(&mut conn)
                    .await
                    .map_err(|e| AppError::Internal(format!("db: {e}")))?;

            let rules: Vec<crate::models::employee::AssignmentRule> =
                rules_entities.into_iter().map(Into::into).collect();

            if !rules.is_empty() {
                let match_ctx = crate::services::rule_evaluator::EmployeeMatchContext {
                    department_id: dept_id,
                    employee_id_number: emp_id_num,
                    tags: tags.unwrap_or_default(),
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
                                    .eq(rule.pattern_id),
                            )
                            .filter(crate::schema::working_pattern_details::schedule_day.eq(dow))
                            .select(crate::schema::working_pattern_details::shift_id)
                            .first::<Option<Uuid>>(&mut conn)
                            .await
                            .optional()
                            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

                    if let Some(s) = shift_id {
                        return Ok((s, traces));
                    }
                }
            }
        }

        // Priority 4: Department Working Pattern
        let dept_pattern_info: Option<(Option<Uuid>, Uuid)> =
            crate::schema::working_pattern_details::table
                .inner_join(
                    crate::schema::departments::table
                        .on(crate::schema::departments::default_working_pattern_id
                            .eq(crate::schema::working_pattern_details::pattern_id.nullable())),
                )
                .inner_join(
                    crate::schema::employee_department_history::table
                        .on(crate::schema::employee_department_history::department_id
                            .eq(crate::schema::departments::id)),
                )
                .inner_join(
                    crate::schema::employees::table.on(crate::schema::employees::id
                        .eq(crate::schema::employee_department_history::employee_id)),
                )
                .filter(crate::schema::employee_department_history::employee_id.eq(employee_id))
                .filter(crate::schema::employee_department_history::effective_date.le(date))
                .filter(crate::schema::working_pattern_details::schedule_day.eq(dow))
                .filter(crate::schema::employees::workspace_id.eq(workspace_id))
                .order(crate::schema::employee_department_history::effective_date.desc())
                .select((
                    crate::schema::working_pattern_details::shift_id,
                    crate::schema::departments::default_working_pattern_id.assume_not_null(),
                ))
                .first::<(Option<Uuid>, Uuid)>(&mut conn)
                .await
                .optional()
                .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        if let Some((shift_id, pattern_id)) = dept_pattern_info {
            traces.push(ScheduleTrace {
                rule_id: Some(pattern_id),
                rule_name: "Department Pattern".to_string(),
                priority: 40,
                status: "applied".to_string(),
                reason: Some("Inherited from department default pattern".to_string()),
            });
            return Ok((shift_id, traces));
        }

        // Priority 5: Employee Default Shift
        let employee_shift: Option<Uuid> =
            crate::schema::employee_shifts::table
                .inner_join(crate::schema::employees::table.on(
                    crate::schema::employees::id.eq(crate::schema::employee_shifts::employee_id),
                ))
                .filter(crate::schema::employee_shifts::employee_id.eq(employee_id))
                .filter(crate::schema::employee_shifts::start_date.le(date))
                .filter(
                    crate::schema::employee_shifts::end_date
                        .is_null()
                        .or(crate::schema::employee_shifts::end_date.ge(date)),
                )
                .filter(crate::schema::employees::workspace_id.eq(workspace_id))
                .order(crate::schema::employee_shifts::created_at.desc())
                .select(crate::schema::employee_shifts::shift_id)
                .first::<Uuid>(&mut conn)
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

        // Priority 6: Department Default Shift
        let dept_shift: Option<Uuid> = crate::schema::department_default_shifts::table
            .inner_join(
                crate::schema::employee_department_history::table
                    .on(crate::schema::employee_department_history::department_id
                        .eq(crate::schema::department_default_shifts::department_id)),
            )
            .inner_join(
                crate::schema::employees::table.on(crate::schema::employees::id
                    .eq(crate::schema::employee_department_history::employee_id)),
            )
            .filter(crate::schema::employee_department_history::employee_id.eq(employee_id))
            .filter(crate::schema::employee_department_history::effective_date.le(date))
            .filter(crate::schema::employees::workspace_id.eq(workspace_id))
            .order(crate::schema::employee_department_history::effective_date.desc())
            .select(crate::schema::department_default_shifts::shift_id)
            .first::<Uuid>(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        if let Some(shift) = dept_shift {
            traces.push(ScheduleTrace {
                rule_id: None,
                rule_name: "Department Fixed".to_string(),
                priority: 10,
                status: "applied".to_string(),
                reason: Some("Inherited from department default shift".to_string()),
            });
            return Ok((Some(shift), traces));
        }

        Ok((None, traces))
    }

    async fn delete_roster(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        date: NaiveDate,
    ) -> Result<(), AppError> {
        let mut conn = self
            .diesel_pools
            .tenants
            .get_pool(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db pool error: {e}")))?
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        diesel::delete(
            shift_rosters::table
                .filter(shift_rosters::employee_id.eq(employee_id))
                .filter(shift_rosters::date.eq(date)),
        )
        .execute(&mut conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }
}
