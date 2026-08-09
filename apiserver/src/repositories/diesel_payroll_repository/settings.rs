use super::*;
use crate::diesel_pool::{DieselPoolsTenantTxExt, TenantAuthToken};
use crate::errors::AppError;
use crate::models::{NewWorkspaceConfigEntryParams, PayrollSettings};
use chrono::{NaiveDate, Utc};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use uuid::Uuid;

impl DieselPayrollRepository {
    pub(super) async fn get_payroll_settings(
        &self,
        workspace_id: Uuid,
    ) -> Result<Option<PayrollSettings>, AppError> {
        self.get_payroll_settings_at(workspace_id, Utc::now().date_naive())
            .await
    }

    pub(super) async fn get_payroll_settings_at(
        &self,
        workspace_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<PayrollSettings>, AppError> {
        let auth_token = TenantAuthToken::system_override(workspace_id);
        self.diesel_pools
            .with_tenant_tx(&auth_token, |conn: &mut AsyncPgConnection| {
                Box::pin(async move {
                    let entries = workspace_config_entries::table
                        .filter(workspace_config_entries::workspace_id.eq(workspace_id))
                        .filter(workspace_config_entries::scope_type.eq("workspace"))
                        .filter(workspace_config_entries::effective_from.le(date))
                        .load::<crate::models::WorkspaceConfigEntry>(&mut *conn)
                        .await
                        .map_err(AppError::from)?;

                    if entries.is_empty() {
                        return Ok(None);
                    }

                    let resolver = crate::models::WorkspaceConfigResolver::new(entries);
                    let resolved = resolver.resolve_payroll_settings(Uuid::nil(), None, None, date);

                    let max_effective_from = resolver
                        .resolve_key(
                            "payroll.std_working_day_mode",
                            Uuid::nil(),
                            None,
                            None,
                            date,
                        )
                        .map(crate::models::WorkspaceConfigEntry::effective_from)
                        .unwrap_or(date);

                    Ok(Some(PayrollSettings {
                        workspace_id,
                        std_working_day_mode: resolved.std_working_day_mode,
                        fixed_std_days: resolved.fixed_std_days,
                        working_days_per_week: resolved.working_days_per_week,
                        hours_per_day: resolved.hours_per_day,
                        max_ot_hours_per_day: resolved.max_ot_hours_per_day,
                        max_ot_hours_per_month: resolved.max_ot_hours_per_month,
                        effective_date: max_effective_from,
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                    }))
                })
            })
            .await
    }

    pub(super) async fn get_payroll_settings_history(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<PayrollSettings>, AppError> {
        let auth_token = TenantAuthToken::system_override(workspace_id);
        self.diesel_pools
            .with_tenant_tx(&auth_token, |conn: &mut AsyncPgConnection| {
                Box::pin(async move {
                    let entries = workspace_config_entries::table
                        .filter(workspace_config_entries::workspace_id.eq(workspace_id))
                        .filter(workspace_config_entries::scope_type.eq("workspace"))
                        .load::<crate::models::WorkspaceConfigEntry>(&mut *conn)
                        .await
                        .map_err(AppError::from)?;

                    if entries.is_empty() {
                        return Ok(Vec::new());
                    }

                    let mut dates: Vec<NaiveDate> = entries
                        .iter()
                        .map(crate::models::WorkspaceConfigEntry::effective_from)
                        .collect();
                    dates.sort_unstable();
                    dates.dedup();
                    dates.reverse();

                    let mut history = Vec::new();
                    let resolver = crate::models::WorkspaceConfigResolver::new(entries);
                    for d in dates {
                        let resolved =
                            resolver.resolve_payroll_settings(Uuid::nil(), None, None, d);
                        history.push(PayrollSettings {
                            workspace_id,
                            std_working_day_mode: resolved.std_working_day_mode,
                            fixed_std_days: resolved.fixed_std_days,
                            working_days_per_week: resolved.working_days_per_week,
                            hours_per_day: resolved.hours_per_day,
                            max_ot_hours_per_day: resolved.max_ot_hours_per_day,
                            max_ot_hours_per_month: resolved.max_ot_hours_per_month,
                            effective_date: d,
                            created_at: Utc::now(),
                            updated_at: Utc::now(),
                        });
                    }

                    Ok(history)
                })
            })
            .await
    }

    #[allow(clippy::too_many_lines)]
    pub(super) async fn update_payroll_settings(
        &self,
        workspace_id: Uuid,
        req: &crate::models::SavePayrollSettings,
    ) -> Result<PayrollSettings, AppError> {
        let auth_token = TenantAuthToken::system_override(workspace_id);
        self.diesel_pools
            .with_tenant_tx(&auth_token, |conn: &mut AsyncPgConnection| {
                Box::pin(async move {
                    let keys_and_values = vec![
                        (
                            "payroll.std_working_day_mode",
                            None,
                            None,
                            Some(req.std_working_day_mode.clone()),
                            None,
                            None,
                        ),
                        (
                            "payroll.fixed_std_days",
                            None,
                            Some(req.fixed_std_days),
                            None,
                            None,
                            None,
                        ),
                        (
                            "payroll.working_days_per_week",
                            Some(req.working_days_per_week),
                            None,
                            None,
                            None,
                            None,
                        ),
                        (
                            "payroll.hours_per_day",
                            None,
                            Some(req.hours_per_day),
                            None,
                            None,
                            None,
                        ),
                        (
                            "payroll.max_ot_hours_per_day",
                            None,
                            req.max_ot_hours_per_day,
                            None,
                            None,
                            None,
                        ),
                        (
                            "payroll.max_ot_hours_per_month",
                            None,
                            req.max_ot_hours_per_month,
                            None,
                            None,
                            None,
                        ),
                    ];

                    for (key, val_int, val_dec, val_text, val_bool, val_json) in keys_and_values {
                        let has_value = val_int.is_some()
                            || val_dec.is_some()
                            || val_text.is_some()
                            || val_bool.is_some()
                            || val_json.is_some();

                        if has_value {
                            let existing = workspace_config_entries::table
                                .filter(workspace_config_entries::workspace_id.eq(workspace_id))
                                .filter(workspace_config_entries::scope_type.eq("workspace"))
                                .filter(workspace_config_entries::scope_id.is_null())
                                .filter(workspace_config_entries::config_key.eq(key))
                                .filter(
                                    workspace_config_entries::effective_from.eq(req.effective_date),
                                )
                                .first::<crate::models::WorkspaceConfigEntry>(&mut *conn)
                                .await
                                .optional()
                                .map_err(AppError::from)?;

                            if let Some(entry) = existing {
                                diesel::update(
                                    workspace_config_entries::table
                                        .filter(workspace_config_entries::id.eq(entry.id())),
                                )
                                .set((
                                    workspace_config_entries::value_int.eq(val_int),
                                    workspace_config_entries::value_decimal.eq(val_dec),
                                    workspace_config_entries::value_text.eq(val_text),
                                    workspace_config_entries::value_bool.eq(val_bool),
                                    workspace_config_entries::value_json.eq(val_json),
                                    workspace_config_entries::updated_at.eq(Utc::now()),
                                ))
                                .execute(&mut *conn)
                                .await
                                .map_err(AppError::from)?;
                            } else {
                                let id = Uuid::new_v4();
                                let entry = crate::models::WorkspaceConfigEntry::new(
                                    NewWorkspaceConfigEntryParams {
                                        id,
                                        workspace_id,
                                        scope_type: "workspace".to_string(),
                                        scope_id: None,
                                        config_key: key.to_string(),
                                        value_int: val_int,
                                        value_decimal: val_dec,
                                        value_text: val_text,
                                        value_bool: val_bool,
                                        value_json: val_json,
                                        effective_from: req.effective_date,
                                        effective_to: None,
                                        created_by: None,
                                        created_at: Utc::now(),
                                        updated_at: Utc::now(),
                                    },
                                );
                                diesel::insert_into(workspace_config_entries::table)
                                    .values(&entry)
                                    .execute(&mut *conn)
                                    .await
                                    .map_err(AppError::from)?;
                            }
                        } else {
                            diesel::delete(
                                workspace_config_entries::table
                                    .filter(workspace_config_entries::workspace_id.eq(workspace_id))
                                    .filter(workspace_config_entries::scope_type.eq("workspace"))
                                    .filter(workspace_config_entries::scope_id.is_null())
                                    .filter(workspace_config_entries::config_key.eq(key))
                                    .filter(
                                        workspace_config_entries::effective_from
                                            .eq(req.effective_date),
                                    ),
                            )
                            .execute(&mut *conn)
                            .await
                            .map_err(AppError::from)?;
                        }
                    }

                    Ok(())
                })
            })
            .await?;

        let resolved = self
            .get_payroll_settings_at(workspace_id, req.effective_date)
            .await?;
        resolved.ok_or_else(|| {
            AppError::Internal("Failed to resolve newly updated payroll settings".to_string())
        })
    }

    pub(super) async fn get_department_payroll_config_at(
        &self,
        workspace_id: Uuid,
        department_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<crate::models::DepartmentPayrollConfig>, AppError> {
        let auth_token = TenantAuthToken::system_override(workspace_id);
        self.diesel_pools
            .with_tenant_tx(&auth_token, |conn: &mut AsyncPgConnection| {
                Box::pin(async move {
                    use crate::schema::workspace_config_entries;

                    let entries = workspace_config_entries::table
                        .filter(workspace_config_entries::workspace_id.eq(workspace_id))
                        .filter(workspace_config_entries::scope_type.eq("department"))
                        .filter(workspace_config_entries::scope_id.eq(department_id))
                        .filter(workspace_config_entries::effective_from.le(date))
                        .load::<crate::models::WorkspaceConfigEntry>(&mut *conn)
                        .await
                        .map_err(AppError::from)?;

                    if entries.is_empty() {
                        return Ok(None);
                    }

                    let mut fixed_std_days = None;
                    let mut hours_per_day = None;
                    let mut max_effective_from = None;
                    let mut entry_id = Uuid::new_v4();

                    for entry in &entries {
                        if entry.config_key() == "payroll.fixed_std_days"
                            && max_effective_from
                                .is_none_or(|max_d| entry.effective_from() >= max_d)
                        {
                            fixed_std_days = entry.value_decimal();
                            max_effective_from = Some(entry.effective_from());
                            entry_id = entry.id();
                        } else if entry.config_key() == "payroll.hours_per_day"
                            && max_effective_from
                                .is_none_or(|max_d| entry.effective_from() >= max_d)
                        {
                            hours_per_day = entry.value_decimal();
                            max_effective_from = Some(entry.effective_from());
                        }
                    }

                    if fixed_std_days.is_none() && hours_per_day.is_none() {
                        return Ok(None);
                    }

                    let effective_date = max_effective_from.unwrap_or(date);

                    Ok(Some(crate::models::DepartmentPayrollConfig {
                        id: entry_id,
                        department_id,
                        std_working_days: fixed_std_days
                            .unwrap_or_else(|| rust_decimal::Decimal::from(0)),
                        hours_per_day: hours_per_day
                            .unwrap_or_else(|| rust_decimal::Decimal::from(0)),
                        effective_date,
                        created_at: Utc::now(),
                    }))
                })
            })
            .await
    }
}
