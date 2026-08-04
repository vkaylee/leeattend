use super::*;
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
        .load::<crate::models::AssignmentHistory>(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(format!("db history: {e}")))?;

        Ok((history, total))
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
            .map_err(AppError::from)?;

        let data: Vec<ShiftRoster> = rosters.into_iter().map(Into::into).collect();
        let total = if page.is_some() || limit.is_some() {
            total
        } else {
            i64::try_from(data.len()).unwrap_or(0)
        };

        Ok((data, total))
    }

    pub(super) async fn get_employment_rosters_batch(
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
            .map_err(AppError::from)?;

        let mut map: std::collections::HashMap<Uuid, Vec<ShiftRoster>> =
            std::collections::HashMap::new();
        for roster in rosters {
            map.entry(roster.employment_id)
                .or_default()
                .push(roster.into());
        }
        Ok(map)
    }

    pub(super) async fn upsert_roster(
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
                crate::schema::shift_rosters::template_id.eq(entity.template_id),
                crate::schema::shift_rosters::status.eq(entity.status.clone()),
                crate::schema::shift_rosters::notes.eq(entity.notes.clone()),
                crate::schema::shift_rosters::updated_at.eq(Utc::now()),
            ))
            .get_result::<crate::models::diesel::roster::ShiftRosterEntity>(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(result.into())
    }

    pub(super) async fn bulk_upsert_roster(
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
            .execute(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(())
    }

    pub(super) async fn delete_roster(
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
        .map_err(AppError::from)?;

        Ok(())
    }
}
