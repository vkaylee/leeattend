use super::*;
use crate::diesel_pool::DieselPoolsTenantTxExt;
use crate::errors::AppError;
use crate::models::diesel::roster::*;
use crate::models::{RosterItem, ShiftRoster};
use crate::schema::*;
use crate::utils::i18n::I18nKey;
use chrono::{NaiveDate, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

#[allow(clippy::too_many_lines)]
impl DieselRosterRepository {
    pub(super) async fn get_assignment_history(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        page: Option<i64>,
        limit: Option<i64>,
    ) -> Result<(Vec<crate::models::AssignmentHistory>, i64), AppError> {
        let workspace_id = auth_token.workspace_id();
        self.diesel_pools
            .with_tenant_tx(auth_token, |conn| {
                Box::pin(async move {

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
        .get_result::<CountResult>(conn)
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
                    st.name as shift_name,
                    TO_CHAR(es.start_date, 'YYYY-MM-DD') ||
                    CASE WHEN es.end_date IS NULL THEN ' -> ∞' ELSE ' -> ' || TO_CHAR(es.end_date, 'YYYY-MM-DD') END as date_range,
                    es.created_at
                FROM employee_shifts es
                JOIN shift_templates st ON es.template_id = st.id
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
                    CASE WHEN sr.template_id IS NULL THEN 'OFF' ELSE st.name END as shift_name,
                    TO_CHAR(sr.date, 'YYYY-MM-DD') as date_range,
                    sr.created_at
                FROM shift_rosters sr
                LEFT JOIN shift_templates st ON sr.template_id = st.id
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
        .load::<crate::models::AssignmentHistory>(conn)
        .await
        .map_err(|e| AppError::Internal(format!("db history: {e}")))?;

        Ok((history, total))
                    })
            })
            .await
    }

    pub(super) async fn get_employment_rosters(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
        page: Option<i64>,
        limit: Option<i64>,
    ) -> Result<(Vec<ShiftRoster>, i64), AppError> {
        let workspace_id = auth_token.workspace_id();
        self.diesel_pools
            .with_tenant_tx(auth_token, |conn| {
                Box::pin(async move {
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

                    let rosters = query
                        .select(crate::schema::shift_rosters::all_columns)
                        .load::<crate::models::diesel::roster::ShiftRosterEntity>(conn)
                        .await
                        .map_err(AppError::from)?;

                    let data: Vec<ShiftRoster> = rosters.into_iter().map(Into::into).collect();
                    let total = if page.is_some() || limit.is_some() {
                        total
                    } else {
                        i64::try_from(data.len()).unwrap_or(0)
                    };

                    Ok((data, total))
                })
            })
            .await
    }

    pub(super) async fn get_employment_rosters_batch(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_ids: &[Uuid],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<std::collections::HashMap<Uuid, Vec<ShiftRoster>>, AppError> {
        let workspace_id = auth_token.workspace_id();
        self.diesel_pools
            .with_tenant_tx(auth_token, |conn| {
                Box::pin(async move {
                    let rosters = crate::schema::shift_rosters::table
                        .inner_join(crate::schema::employments::table)
                        .filter(crate::schema::employments::workspace_id.eq(workspace_id))
                        .filter(crate::schema::shift_rosters::employment_id.eq_any(employment_ids))
                        .filter(crate::schema::shift_rosters::date.ge(start_date))
                        .filter(crate::schema::shift_rosters::date.le(end_date))
                        .select(crate::schema::shift_rosters::all_columns)
                        .load::<crate::models::diesel::roster::ShiftRosterEntity>(conn)
                        .await
                        .map_err(AppError::from)?;

                    let mut map: std::collections::HashMap<Uuid, Vec<ShiftRoster>> =
                        std::collections::HashMap::new();
                    for roster in rosters {
                        map.entry(roster.employment_id)
                            .or_default()
                            .push(roster.into());
                    }
                    Ok(map)
                })
            })
            .await
    }

    pub(super) async fn get_employment_rosters_batch_by_windows(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        windows: &[(Uuid, NaiveDate, NaiveDate)],
    ) -> Result<std::collections::HashMap<Uuid, Vec<ShiftRoster>>, AppError> {
        if windows.is_empty() {
            return Ok(std::collections::HashMap::new());
        }

        let workspace_id = auth_token.workspace_id();
        self.diesel_pools
            .with_tenant_tx(auth_token, |conn| {
                Box::pin(async move {
                    // UUIDs and dates are strongly typed before interpolation here;
                    // the predicate contains no caller-controlled raw text.
                    use std::fmt::Write as _;
                    let mut predicate = String::from("FALSE");
                    for (employment_id, start_date, end_date) in windows {
                        let _ = write!(
                            predicate,
                            " OR (shift_rosters.employment_id = '{employment_id}' AND shift_rosters.date >= '{start_date}' AND shift_rosters.date <= '{end_date}')"
                        );
                    }

                    let rosters = shift_rosters::table
                        .inner_join(employments::table)
                        .filter(employments::workspace_id.eq(workspace_id))
                        .filter(diesel::dsl::sql::<diesel::sql_types::Bool>(&predicate))
                        .select(shift_rosters::all_columns)
                        .load::<ShiftRosterEntity>(conn)
                        .await
                        .map_err(|e| AppError::Internal(format!("db roster windows: {e}")))?;

                    let mut map = std::collections::HashMap::new();
                    for roster in rosters {
                        map.entry(roster.employment_id)
                            .or_insert_with(Vec::new)
                            .push(roster.into());
                    }
                    Ok(map)
                })
            })
            .await
    }

    pub(super) async fn upsert_roster(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        roster: &ShiftRoster,
    ) -> Result<ShiftRoster, AppError> {
        let workspace_id = auth_token.workspace_id();
        self.diesel_pools
            .with_tenant_tx(auth_token, |conn| {
                Box::pin(async move {
                    // Check if employment exists and belongs to the workspace of this tenant
                    let employment_exists = crate::schema::employments::table
                        .filter(crate::schema::employments::id.eq(roster.employment_id))
                        .filter(crate::schema::employments::workspace_id.eq(workspace_id))
                        .filter(crate::schema::employments::deleted_at.is_null())
                        .select(crate::schema::employments::id)
                        .first::<Uuid>(conn)
                        .await
                        .optional()
                        .map_err(|e| AppError::Internal(format!("db check employment: {e}")))?;

                    if employment_exists.is_none() {
                        return Err(AppError::not_found(I18nKey::EmploymentNotFound));
                    }

                    if let Some(template_id) = roster.shift_id {
                        let template_exists = shift_templates::table
                            .filter(shift_templates::id.eq(template_id))
                            .filter(shift_templates::workspace_id.eq(workspace_id))
                            .select(shift_templates::id)
                            .first::<Uuid>(conn)
                            .await
                            .optional()
                            .map_err(|e| {
                                AppError::Internal(format!("db check shift template: {e}"))
                            })?;

                        if template_exists.is_none() {
                            return Err(AppError::not_found(I18nKey::ShiftNotFound));
                        }
                    }

                    let entity: crate::models::diesel::roster::ShiftRosterEntity =
                        roster.clone().into();

                    let result = diesel::insert_into(crate::schema::shift_rosters::table)
                        .values(&entity)
                        .on_conflict((
                            crate::schema::shift_rosters::employment_id,
                            crate::schema::shift_rosters::date,
                        ))
                        .do_update()
                        .set((
                            crate::schema::shift_rosters::template_id.eq(entity.template_id),
                            crate::schema::shift_rosters::status.eq(entity.status.clone()),
                            crate::schema::shift_rosters::notes.eq(entity.notes.clone()),
                            crate::schema::shift_rosters::updated_at.eq(Utc::now()),
                        ))
                        .get_result::<crate::models::diesel::roster::ShiftRosterEntity>(conn)
                        .await
                        .map_err(AppError::from)?;

                    Ok(result.into())
                })
            })
            .await
    }

    pub(super) async fn bulk_upsert_roster(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        items: Vec<RosterItem>,
    ) -> Result<(), AppError> {
        let workspace_id = auth_token.workspace_id();
        self.diesel_pools
            .with_tenant_tx(auth_token, |conn| {
                Box::pin(async move {
                    // Check if employment exists and belongs to the workspace of this tenant
                    let employment_exists = crate::schema::employments::table
                        .filter(crate::schema::employments::id.eq(employment_id))
                        .filter(crate::schema::employments::workspace_id.eq(workspace_id))
                        .filter(crate::schema::employments::deleted_at.is_null())
                        .select(crate::schema::employments::id)
                        .first::<Uuid>(conn)
                        .await
                        .optional()
                        .map_err(|e| AppError::Internal(format!("db check employment: {e}")))?;

                    if employment_exists.is_none() {
                        return Err(AppError::not_found(I18nKey::EmploymentNotFound));
                    }

                    let mut template_ids: Vec<Uuid> =
                        items.iter().filter_map(|item| item.shift_id).collect();
                    template_ids.sort_unstable();
                    template_ids.dedup();
                    if !template_ids.is_empty() {
                        let valid_template_ids = shift_templates::table
                            .filter(shift_templates::workspace_id.eq(workspace_id))
                            .filter(shift_templates::id.eq_any(&template_ids))
                            .select(shift_templates::id)
                            .load::<Uuid>(conn)
                            .await
                            .map_err(|e| {
                                AppError::Internal(format!("db check shift templates: {e}"))
                            })?;

                        if valid_template_ids.len() != template_ids.len() {
                            return Err(AppError::not_found(I18nKey::ShiftNotFound));
                        }
                    }

                    let entities: Vec<ShiftRosterEntity> = items
                        .into_iter()
                        .map(|item| ShiftRosterEntity {
                            id: crate::utils::id::new_id(),
                            employment_id,
                            template_id: item.shift_id,
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
                            shift_rosters::template_id.eq(diesel::dsl::sql("EXCLUDED.template_id")),
                            shift_rosters::status.eq(diesel::dsl::sql("EXCLUDED.status")),
                            shift_rosters::notes.eq(diesel::dsl::sql("EXCLUDED.notes")),
                            shift_rosters::updated_at.eq(Utc::now()),
                        ))
                        .execute(conn)
                        .await
                        .map_err(AppError::from)?;

                    Ok(())
                })
            })
            .await
    }

    pub(super) async fn delete_roster(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        date: NaiveDate,
    ) -> Result<(), AppError> {
        let workspace_id = auth_token.workspace_id();
        self.diesel_pools
            .with_tenant_tx(auth_token, |conn| {
                Box::pin(async move {
                    // Check if employment exists and belongs to the workspace of this tenant
                    let employment_exists = crate::schema::employments::table
                        .filter(crate::schema::employments::id.eq(employment_id))
                        .filter(crate::schema::employments::workspace_id.eq(workspace_id))
                        .filter(crate::schema::employments::deleted_at.is_null())
                        .select(crate::schema::employments::id)
                        .first::<Uuid>(conn)
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
                    .execute(conn)
                    .await
                    .map_err(AppError::from)?;

                    Ok(())
                })
            })
            .await
    }
}
