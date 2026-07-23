use super::*;
use crate::errors::AppError;
use crate::models::ScheduleTrace;
use crate::utils::i18n::I18nKey;
use chrono::NaiveDate;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

#[allow(clippy::too_many_lines, clippy::too_many_arguments)]
impl DieselRosterRepository {
    pub(super) async fn get_effective_shift(
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
            .select(crate::schema::shift_rosters::template_id)
            .first::<Option<Uuid>>(&mut *conn)
            .await
            .optional()
            .map_err(AppError::from)?;

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
                    crate::schema::working_pattern_details::template_id,
                    crate::schema::employee_working_patterns::pattern_id,
                ))
                .first::<(Option<Uuid>, Uuid)>(&mut *conn)
                .await
                .optional()
                .map_err(AppError::from)?;

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
                            .select(crate::schema::working_pattern_details::template_id)
                            .first::<Option<Uuid>>(&mut *conn)
                            .await
                            .optional()
                            .map_err(AppError::from)?;

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
            .select(crate::schema::employee_shifts::template_id)
            .first::<Uuid>(&mut *conn)
            .await
            .optional()
            .map_err(AppError::from)?;

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

    pub(super) async fn get_raw_roster_data(
        &self,
        workspace_id: Uuid,
    ) -> Result<std::sync::Arc<crate::cache::RosterData>, AppError> {
        crate::cache::build_roster_data(&self.diesel_pools, workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db build: {e}")))
    }
}
