use super::*;
use crate::diesel_pool::DieselPoolsTenantTxExt;
use crate::errors::AppError;
use crate::models::ScheduleTrace;
use crate::utils::i18n::I18nKey;
use chrono::NaiveDate;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

#[allow(clippy::too_many_lines)]
impl DieselRosterRepository {
    pub(super) async fn get_effective_shift(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        date: NaiveDate,
    ) -> Result<
        (
            Option<Uuid>,
            Vec<ScheduleTrace>,
            std::sync::Arc<crate::cache::RosterData>,
        ),
        AppError,
    > {
        let workspace_id = auth_token.workspace_id();
        let (
            actual_employment_id,
            employee_id,
            emp_id_number,
            tags,
            active_location_id,
            active_employer,
            roster_map,
        ) = self
            .diesel_pools
            .with_tenant_tx(auth_token, |conn| {
                Box::pin(async move {
                    // ── Phase 1: Resolve actual employment_id ──────────────────────
                    let actual_employment_id = {
                        let direct_exists = crate::schema::employments::table
                            .filter(crate::schema::employments::id.eq(employment_id))
                            .filter(crate::schema::employments::workspace_id.eq(workspace_id))
                            .filter(crate::schema::employments::deleted_at.is_null())
                            .select(crate::schema::employments::id)
                            .first::<Uuid>(conn)
                            .await
                            .optional()
                            .map_err(|e| AppError::Internal(format!("db check employment: {e}")))?;

                        if let Some(id) = direct_exists {
                            id
                        } else {
                            let active_emp = crate::schema::employments::table
                                .filter(crate::schema::employments::employee_id.eq(employment_id))
                                .filter(crate::schema::employments::workspace_id.eq(workspace_id))
                                .filter(crate::schema::employments::status.eq("active"))
                                .filter(crate::schema::employments::deleted_at.is_null())
                                .select(crate::schema::employments::id)
                                .first::<Uuid>(conn)
                                .await
                                .optional()
                                .map_err(|e| {
                                    AppError::Internal(format!("db select active employment: {e}"))
                                })?;

                            active_emp
                                .ok_or_else(|| AppError::not_found(I18nKey::EmploymentNotFound))?
                        }
                    };

                    // ── Phase 2: Fetch employee context (employee_id + tags + locations) ──
                    let emp_info = crate::schema::employments::table
                        .filter(crate::schema::employments::id.eq(actual_employment_id))
                        .filter(crate::schema::employments::workspace_id.eq(workspace_id))
                        .select(crate::schema::employments::employee_id)
                        .first::<Uuid>(conn)
                        .await
                        .optional()
                        .map_err(|e| {
                            AppError::Internal(format!("db select employee from employment: {e}"))
                        })?;

                    let Some(employee_id) = emp_info else {
                        return Err(AppError::not_found(I18nKey::EmploymentNotFound));
                    };

                    // Fetch employee details (id_number, tags, primary_location_id)
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
                            crate::schema::employments::primary_location_id,
                        ))
                        .first::<(String, Option<Vec<Option<String>>>, Option<Uuid>)>(conn)
                        .await
                        .optional()
                        .map(|opt| {
                            opt.map(|(num, tags, primary_loc)| {
                                (
                                    num,
                                    tags.map(|v| v.into_iter().flatten().collect()),
                                    primary_loc,
                                )
                            })
                        })
                        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

                    let (emp_id_number, tags, primary_loc) =
                        employee_full.unwrap_or_else(|| (String::new(), None, None));

                    // Fetch active location from assignments (overrides primary location)
                    let active_assignment: Option<Uuid> =
                        crate::schema::employee_assignments::table
                            .inner_join(crate::schema::employments::table)
                            .filter(crate::schema::employments::id.eq(actual_employment_id))
                            .filter(
                                crate::schema::employee_assignments::workspace_id.eq(workspace_id),
                            )
                            .filter(crate::schema::employee_assignments::status.eq(
                                crate::models::employee_assignment::AssignmentStatusEnum::Active,
                            ))
                            .filter(crate::schema::employee_assignments::start_date.le(date))
                            .filter(
                                crate::schema::employee_assignments::end_date
                                    .is_null()
                                    .or(crate::schema::employee_assignments::end_date.ge(date)),
                            )
                            .select(crate::schema::employee_assignments::host_location_id)
                            .first::<Uuid>(conn)
                            .await
                            .optional()
                            .map_err(|e| {
                                AppError::Internal(format!("db select assignment: {e}"))
                            })?;

                    let active_location_id = active_assignment.or(primary_loc);

                    // Fetch active employer from contracts
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
                        .first::<Uuid>(conn)
                        .await
                        .optional()
                        .map_err(|e| AppError::Internal(format!("db select contract: {e}")))?;

                    // ── Phase 3: Roster override map (P1) ─────────────────────────
                    let roster_map: HashMap<(Uuid, NaiveDate), Option<Uuid>> = {
                        let row = crate::schema::shift_rosters::table
                            .filter(
                                crate::schema::shift_rosters::employment_id
                                    .eq(actual_employment_id),
                            )
                            .filter(crate::schema::shift_rosters::date.eq(date))
                            .select((
                                crate::schema::shift_rosters::employment_id,
                                crate::schema::shift_rosters::date,
                                crate::schema::shift_rosters::template_id,
                            ))
                            .first::<(Uuid, NaiveDate, Option<Uuid>)>(conn)
                            .await
                            .optional()
                            .map_err(AppError::from)?;
                        let mut map = HashMap::new();
                        if let Some((emp_id, d, shift_opt)) = row {
                            map.insert((emp_id, d), shift_opt);
                        }
                        map
                    };

                    Ok((
                        actual_employment_id,
                        employee_id,
                        emp_id_number,
                        tags,
                        active_location_id,
                        active_employer,
                        roster_map,
                    ))
                })
            })
            .await?;

        // ── Phase 4: RosterData (cache hit or cold build) ─────────────
        let roster_data = if let Some(ref cache) = self.roster_cache {
            cache
                .get_roster_data(workspace_id, &self.diesel_pools)
                .await
                .map_err(|e| AppError::Internal(format!("roster_cache: {e}")))?
        } else {
            crate::cache::build_roster_data(&self.diesel_pools, workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("roster: {e}")))?
        };

        // ── Phase 5: Delegate to pure resolver ────────────────────────
        let (shift_ids, _source, traces) =
            crate::repositories::roster::shift_resolution::resolve_effective_shift_from_data(
                &roster_data,
                &roster_map,
                actual_employment_id,
                employee_id,
                &emp_id_number,
                tags.as_ref(),
                date,
                active_location_id,
                active_employer,
            );

        // NOTE: `.iter().next()` is used instead of `.first()` because `.first()`
        // triggers Diesel trait inference issues on the `shift_ids` return type.
        #[allow(clippy::iter_next_slice)]
        let shift_id = shift_ids.iter().next().copied().flatten();
        Ok((shift_id, traces, roster_data))
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
