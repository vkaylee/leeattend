use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

use crate::diesel_pool::DieselPools;
use crate::errors::AppError;
use crate::models::diesel::roster::{
    ShiftDefaultIntervalEntity, ShiftTemplateEntity, ShiftVersionEntity,
};
use crate::models::shift::Shift;
use crate::repositories::shift_repository::ShiftRepository;
use crate::schema::{
    employee_shifts, shift_default_intervals, shift_templates, shift_versions,
};

pub struct DieselShiftRepository {
    diesel_pools: DieselPools,
}

impl DieselShiftRepository {
    pub fn new(diesel_pools: DieselPools) -> Self {
        Self { diesel_pools }
    }
}

// ── Build domain Shift from version + template ──────────────────────────────────

fn shift_from(current_version: &ShiftVersionEntity, template: &ShiftTemplateEntity) -> Shift {
    Shift::new(
        template.id,                          // id = template_id (stable identity)
        current_version.workspace_id,
        current_version.name.clone(),
        current_version.report_symbol.clone(),
        true,                                 // is_active = true (resolved from current)
        None,                                 // parent_shift_id (deprecated)
        current_version.location_id,
        current_version.employer_id,
        current_version.created_at,
        current_version.created_at,           // updated_at ≈ created_at (immutable)
    )
}

#[async_trait]
impl ShiftRepository for DieselShiftRepository {
    // ── create_shift ─────────────────────────────────────────────────────────────
    async fn create_shift(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        req: &crate::models::CreateShiftRequest,
    ) -> Result<Shift, AppError> {
        let workspace_id = auth_token.0;
        let now = Utc::now();
        let template_id = Uuid::new_v4();
        let version_id = Uuid::new_v4();

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        // 1. Create template (without current_version_id — version doesn't exist yet)
        let template = ShiftTemplateEntity {
            id: template_id,
            workspace_id,
            name: req.name.clone(),
            current_version_id: None,
            created_at: now,
        };
        diesel::insert_into(shift_templates::table)
            .values(&template)
            .execute(&mut *conn)
            .await
            .map_err(AppError::from)?;

        // 2. Create version (change_type = 'create')
        let version = ShiftVersionEntity {
            id: version_id,
            template_id,
            workspace_id,
            version_number: 1,
            name: req.name.clone(),
            report_symbol: req.report_symbol.clone(),
            location_id: req.location_id,
            employer_id: req.employer_id,
            effective_from: now.date_naive(),
            effective_to: None,
            change_type: "create".to_string(),
            change_reason: None,
            superseded_by: None,
            created_by: Uuid::nil(),
            created_at: now,
        };
        diesel::insert_into(shift_versions::table)
            .values(&version)
            .execute(&mut *conn)
            .await
            .map_err(AppError::from)?;

        // 3. Link current version on template
        diesel::update(shift_templates::table.filter(shift_templates::id.eq(template_id)))
            .set(shift_templates::current_version_id.eq(version_id))
            .execute(&mut *conn)
            .await
            .map_err(AppError::from)?;

        // 4. Insert intervals
        if req.intervals.is_empty() {
            return Err(AppError::validation(
                crate::utils::i18n::I18nKey::Validation,
            ));
        }
        for (i, iv) in req.intervals.iter().enumerate() {
            let interval = ShiftDefaultIntervalEntity {
                id: Uuid::new_v4(),
                version_id,
                sort_order: i32::try_from(i).unwrap_or(0),
                interval_name: iv.interval_name.clone(),
                start_time: iv.start_time,
                end_time: iv.end_time,
                break_duration_minutes: iv.break_duration_minutes,
                break_start: iv.break_start,
                break_end: iv.break_end,
                is_break_auto_deduct: iv.is_break_auto_deduct,
                break_is_overtime: iv.break_is_overtime,
                early_punch_in_grace_period_minutes: iv.early_punch_in_grace_period_minutes,
                allow_early_ot: iv.allow_early_ot,
                max_early_ot_minutes: iv.max_early_ot_minutes,
                ot_aggregation_mode: iv.ot_aggregation_mode.clone(),
                is_night_shift: iv.is_night_shift,
                created_at: Some(now),
                updated_at: Some(now),
            };
            diesel::insert_into(shift_default_intervals::table)
                .values(&interval)
                .execute(&mut *conn)
                .await
                .map_err(|e| AppError::Internal(format!("db insert interval: {e}")))?;
        }

        Ok(shift_from(&version, &template))
    }

    // ── get_shift_by_id (reads current version via template) ─────────────────────
    async fn get_shift_by_id(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        template_id: Uuid,
    ) -> Result<Option<Shift>, AppError> {
        let workspace_id = auth_token.0;
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let result: Option<(ShiftTemplateEntity, ShiftVersionEntity)> = shift_templates::table
            .inner_join(
                shift_versions::table
                    .on(shift_templates::current_version_id.eq(shift_versions::id.nullable())),
            )
            .filter(shift_templates::id.eq(template_id))
            .filter(shift_templates::workspace_id.eq(workspace_id))
            .first::<(ShiftTemplateEntity, ShiftVersionEntity)>(&mut *conn)
            .await
            .optional()
            .map_err(AppError::from)?;

        Ok(result.map(|(t, v)| shift_from(&v, &t)))
    }

    // ── get_workspace_shifts (list current versions) ──────────────────────────────
    async fn get_workspace_shifts(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        location_id: Option<Uuid>,
        employer_id: Option<Uuid>,
        page: Option<i64>,
        limit: Option<i64>,
    ) -> Result<(Vec<Shift>, i64), AppError> {
        let workspace_id = auth_token.0;
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        // Base filter — reusable for both count and data queries
        let make_filtered = || {
            let mut q = shift_templates::table
                .inner_join(
                    shift_versions::table
                        .on(shift_templates::current_version_id.eq(shift_versions::id.nullable())),
                )
                .filter(shift_templates::workspace_id.eq(workspace_id))
                .filter(shift_versions::effective_to.is_null())
                .into_boxed();

            if let Some(loc_id) = location_id {
                q = q.filter(
                    shift_versions::location_id
                        .is_null()
                        .or(shift_versions::location_id.eq(loc_id)),
                );
            }
            if let Some(emp_id) = employer_id {
                q = q.filter(
                    shift_versions::employer_id
                        .is_null()
                        .or(shift_versions::employer_id.eq(emp_id)),
                );
            }
            q
        };

        let total = if page.is_some() || limit.is_some() {
            make_filtered()
                .select(diesel::dsl::count_star())
                .first::<i64>(&mut *conn)
                .await
                .map_err(AppError::from)?
        } else {
            0
        };

        // Build data query from scratch (can't reuse make_filtered due to boxing)
        let mut data_q = shift_templates::table
            .inner_join(
                shift_versions::table
                    .on(shift_templates::current_version_id.eq(shift_versions::id.nullable())),
            )
            .filter(shift_templates::workspace_id.eq(workspace_id))
            .filter(shift_versions::effective_to.is_null())
            .select((ShiftTemplateEntity::as_select(), ShiftVersionEntity::as_select()))
            .order(shift_versions::name.asc())
            .into_boxed();

        if let Some(loc_id) = location_id {
            data_q = data_q.filter(
                shift_versions::location_id
                    .is_null()
                    .or(shift_versions::location_id.eq(loc_id)),
            );
        }
        if let Some(emp_id) = employer_id {
            data_q = data_q.filter(
                shift_versions::employer_id
                    .is_null()
                    .or(shift_versions::employer_id.eq(emp_id)),
            );
        }

        if let Some(l) = limit {
            data_q = data_q.limit(l);
        }
        if let (Some(p), Some(l)) = (page, limit) {
            data_q = data_q.offset((p - 1) * l);
        }

        let rows: Vec<(ShiftTemplateEntity, ShiftVersionEntity)> = data_q
            .load::<(ShiftTemplateEntity, ShiftVersionEntity)>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let final_total = if page.is_some() || limit.is_some() {
            total
        } else {
            i64::try_from(rows.len()).unwrap_or(0)
        };

        let shifts: Vec<Shift> = rows
            .iter()
            .map(|(t, v)| shift_from(v, t))
            .collect();

        Ok((shifts, final_total))
    }

    // ── create_new_version ────────────────────────────────────────────────────────
    async fn create_new_version(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        template_id: Uuid,
        req: &crate::models::UpdateShiftRequest,
        change_type: &str,
        effective_date: NaiveDate,
        created_by: Uuid,
        change_reason: Option<String>,
    ) -> Result<Shift, AppError> {
        let workspace_id = auth_token.0;
        let now = Utc::now();
        let new_version_id = Uuid::new_v4();

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        conn.transaction::<_, AppError, _>(|conn| {
            Box::pin(async move {
                // 1. Get current version + template for context
                let (template, current_version): (ShiftTemplateEntity, ShiftVersionEntity) =
                    shift_templates::table
                        .inner_join(
                            shift_versions::table.on(shift_templates::current_version_id
                                .eq(shift_versions::id.nullable())),
                        )
                        .filter(shift_templates::id.eq(template_id))
                        .filter(shift_templates::workspace_id.eq(workspace_id))
                        .first::<(ShiftTemplateEntity, ShiftVersionEntity)>(conn)
                        .await
                        .map_err(|_| {
                            AppError::not_found(crate::utils::i18n::I18nKey::ShiftNotFound)
                        })?;

                let new_version_number = current_version.version_number + 1;

                // 2. Close current version
                let close_date = match change_type {
                    "revise" => effective_date.pred_opt().unwrap_or(effective_date),
                    _ => effective_date, // correction: keep same effective_from
                };
                diesel::update(shift_versions::table.filter(shift_versions::id.eq(current_version.id)))
                    .set((
                        shift_versions::effective_to.eq(close_date),
                        shift_versions::superseded_by.eq(new_version_id),
                    ))
                    .execute(conn)
                    .await
                    .map_err(AppError::from)?;

                // 3. Create new version
                let new_version = ShiftVersionEntity {
                    id: new_version_id,
                    template_id,
                    workspace_id,
                    version_number: new_version_number,
                    name: req.name.clone(),
                    report_symbol: req.report_symbol.clone(),
                    location_id: req.location_id,
                    employer_id: req.employer_id,
                    effective_from: effective_date,
                    effective_to: None,
                    change_type: change_type.to_string(),
                    change_reason,
                    superseded_by: None,
                    created_by,
                    created_at: now,
                };
                diesel::insert_into(shift_versions::table)
                    .values(&new_version)
                    .execute(conn)
                    .await
                    .map_err(AppError::from)?;

                // 4. Update template's current_version_id + name
                diesel::update(
                    shift_templates::table.filter(shift_templates::id.eq(template_id)),
                )
                .set((
                    shift_templates::current_version_id.eq(new_version_id),
                    shift_templates::name.eq(&req.name),
                ))
                .execute(conn)
                .await
                .map_err(AppError::from)?;

                // 5. Insert new intervals (delete old: intervals are immutable per version)
                if req.intervals.is_empty() {
                    return Err(AppError::validation(
                        crate::utils::i18n::I18nKey::Validation,
                    ));
                }
                for (i, iv) in req.intervals.iter().enumerate() {
                    let interval = ShiftDefaultIntervalEntity {
                        id: Uuid::new_v4(),
                        version_id: new_version_id,
                        sort_order: i32::try_from(i).unwrap_or(0),
                        interval_name: iv.interval_name.clone(),
                        start_time: iv.start_time,
                        end_time: iv.end_time,
                        break_duration_minutes: iv.break_duration_minutes,
                        break_start: iv.break_start,
                        break_end: iv.break_end,
                        is_break_auto_deduct: iv.is_break_auto_deduct,
                        break_is_overtime: iv.break_is_overtime,
                        early_punch_in_grace_period_minutes: iv.early_punch_in_grace_period_minutes,
                        allow_early_ot: iv.allow_early_ot,
                        max_early_ot_minutes: iv.max_early_ot_minutes,
                        ot_aggregation_mode: iv.ot_aggregation_mode.clone(),
                        is_night_shift: iv.is_night_shift,
                        created_at: Some(now),
                        updated_at: Some(now),
                    };
                    diesel::insert_into(shift_default_intervals::table)
                        .values(&interval)
                        .execute(conn)
                        .await
                        .map_err(|e| AppError::Internal(format!("db insert interval: {e}")))?;
                }

                let updated_template = ShiftTemplateEntity {
                    current_version_id: Some(new_version_id),
                    name: req.name.clone(),
                    ..template
                };
                Ok(shift_from(&new_version, &updated_template))
            })
        })
        .await
    }

    // ── archive_shift ─────────────────────────────────────────────────────────────
    async fn archive_shift(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        template_id: Uuid,
    ) -> Result<(), AppError> {
        let workspace_id = auth_token.0;
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        conn.transaction::<_, AppError, _>(|conn| {
            Box::pin(async move {
                // Close the current version (set effective_to = today)
                diesel::update(
                    shift_versions::table
                        .filter(shift_versions::template_id.eq(template_id))
                        .filter(shift_versions::effective_to.is_null())
                        .filter(shift_versions::workspace_id.eq(workspace_id)),
                )
                .set(shift_versions::effective_to.eq(Utc::now().date_naive()))
                .execute(conn)
                .await
                .map_err(AppError::from)?;

                // Clear current_version_id on template
                diesel::update(
                    shift_templates::table
                        .filter(shift_templates::id.eq(template_id))
                        .filter(shift_templates::workspace_id.eq(workspace_id)),
                )
                .set(shift_templates::current_version_id.eq(None::<Uuid>))
                .execute(conn)
                .await
                .map_err(AppError::from)?;

                Ok(())
            })
        })
        .await
    }

    // ── assign_employment_shift ───────────────────────────────────────────────────
    async fn assign_employment_shift(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        template_id: Uuid,
        start_date: NaiveDate,
        end_date: Option<NaiveDate>,
    ) -> Result<(), AppError> {
        let _workspace_id = auth_token.0;
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.0)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let new_assignment = crate::models::diesel::roster::EmployeeShiftEntity {
            id: Uuid::new_v4(),
            employment_id,
            template_id,
            start_date,
            end_date,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        diesel::insert_into(employee_shifts::table)
            .values(&new_assignment)
            .execute(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(())
    }

    // ── get_employment_active_shift ────────────────────────────────────────────────
    async fn get_employment_active_shift(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<Shift>, AppError> {
        let workspace_id = auth_token.0;
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let result: Option<(ShiftTemplateEntity, ShiftVersionEntity)> = shift_templates::table
            .inner_join(
                employee_shifts::table
                    .on(shift_templates::id.eq(employee_shifts::template_id)),
            )
            .inner_join(
                shift_versions::table
                    .on(shift_templates::current_version_id.eq(shift_versions::id.nullable())),
            )
            .filter(shift_templates::workspace_id.eq(workspace_id))
            .filter(employee_shifts::employment_id.eq(employment_id))
            .filter(employee_shifts::start_date.le(date))
            .filter(
                employee_shifts::end_date
                    .is_null()
                    .or(employee_shifts::end_date.ge(date)),
            )
            .order(employee_shifts::created_at.desc())
            .select((ShiftTemplateEntity::as_select(), ShiftVersionEntity::as_select()))
            .first::<(ShiftTemplateEntity, ShiftVersionEntity)>(&mut *conn)
            .await
            .optional()
            .map_err(AppError::from)?;

        Ok(result.map(|(t, v)| shift_from(&v, &t)))
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
        let _repo = DieselShiftRepository::new(pools);
    }
}
