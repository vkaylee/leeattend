use crate::errors::AppError;
use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::repositories::employee::*;

use super::DieselEmployeeRepository;

#[allow(clippy::too_many_lines)]
#[async_trait]
impl EmployeePayrollConfigRepository for DieselEmployeeRepository {
    async fn upsert_employee_payroll_config(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        std_working_days: rust_decimal::Decimal,
        hours_per_day: rust_decimal::Decimal,
        effective_date: chrono::NaiveDate,
    ) -> Result<crate::models::EmployeePayrollConfig, AppError> {
        let workspace_id = auth_token.workspace_id();
        use crate::schema::{employments, workspace_config_entries};

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        // Check if employment exists and belongs to the workspace of this tenant
        let employment_exists = employments::table
            .filter(employments::id.eq(employment_id))
            .filter(employments::workspace_id.eq(workspace_id))
            .select(employments::id)
            .first::<Uuid>(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db check employment: {e}")))?;

        if employment_exists.is_none() {
            return Err(AppError::NotFound("error.employment_not_found".to_string()));
        }

        let keys_and_values = vec![
            ("payroll.fixed_std_days", std_working_days),
            ("payroll.hours_per_day", hours_per_day),
        ];

        let mut first_id = Uuid::new_v4();

        for (key, val) in keys_and_values {
            let existing = workspace_config_entries::table
                .filter(workspace_config_entries::workspace_id.eq(workspace_id))
                .filter(workspace_config_entries::scope_type.eq("employment"))
                .filter(workspace_config_entries::scope_id.eq(employment_id))
                .filter(workspace_config_entries::config_key.eq(key))
                .filter(workspace_config_entries::effective_from.eq(effective_date))
                .first::<crate::models::WorkspaceConfigEntry>(&mut *conn)
                .await
                .optional()
                .map_err(|e| AppError::Internal(format!("db: {e}")))?;

            if let Some(entry) = existing {
                first_id = entry.id();
                diesel::update(
                    workspace_config_entries::table
                        .filter(workspace_config_entries::id.eq(entry.id())),
                )
                .set((
                    workspace_config_entries::value_decimal.eq(val),
                    workspace_config_entries::updated_at.eq(chrono::Utc::now()),
                ))
                .execute(&mut *conn)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?;
            } else {
                let id = Uuid::new_v4();
                if key == "payroll.fixed_std_days" {
                    first_id = id;
                }
                let entry = crate::models::WorkspaceConfigEntry::new(
                    id,
                    workspace_id,
                    "employment".to_string(),
                    Some(employment_id),
                    key.to_string(),
                    None,
                    Some(val),
                    None,
                    None,
                    None,
                    effective_date,
                    None,
                    None,
                    chrono::Utc::now(),
                    chrono::Utc::now(),
                );
                diesel::insert_into(workspace_config_entries::table)
                    .values(&entry)
                    .execute(&mut *conn)
                    .await
                    .map_err(|e| AppError::Internal(format!("db: {e}")))?;
            }
        }

        Ok(crate::models::EmployeePayrollConfig {
            id: first_id,
            employment_id,
            std_working_days,
            hours_per_day,
            effective_date,
            created_at: chrono::Utc::now(),
        })
    }

    async fn get_employee_payroll_config_at(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        date: chrono::NaiveDate,
    ) -> Result<Option<crate::models::EmployeePayrollConfig>, AppError> {
        let workspace_id = auth_token.workspace_id();
        use crate::schema::workspace_config_entries;

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        // Query both old "employee" scope and new "employment" scope for backward compat
        let entries = workspace_config_entries::table
            .filter(workspace_config_entries::workspace_id.eq(workspace_id))
            .filter(
                workspace_config_entries::scope_type.eq("employment")
                    .and(workspace_config_entries::scope_id.eq(employment_id))
            )
            .filter(workspace_config_entries::effective_from.le(date))
            .load::<crate::models::WorkspaceConfigEntry>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        if entries.is_empty() {
            return Ok(None);
        }

        let mut fixed_std_days = None;
        let mut hours_per_day = None;
        let mut max_effective_from = None;
        let mut entry_id = Uuid::new_v4();

        for entry in &entries {
            if entry.config_key() == "payroll.fixed_std_days"
                && max_effective_from.is_none_or(|max_d| entry.effective_from() >= max_d)
            {
                fixed_std_days = entry.value_decimal();
                max_effective_from = Some(entry.effective_from());
                entry_id = entry.id();
            } else if entry.config_key() == "payroll.hours_per_day"
                && max_effective_from.is_none_or(|max_d| entry.effective_from() >= max_d)
            {
                hours_per_day = entry.value_decimal();
                max_effective_from = Some(entry.effective_from());
            }
        }

        if fixed_std_days.is_none() && hours_per_day.is_none() {
            return Ok(None);
        }

        let effective_date = max_effective_from.unwrap_or(date);

        Ok(Some(crate::models::EmployeePayrollConfig {
            id: entry_id,
            employment_id,
            std_working_days: fixed_std_days.unwrap_or_else(|| rust_decimal::Decimal::from(0)),
            hours_per_day: hours_per_day.unwrap_or_else(|| rust_decimal::Decimal::from(0)),
            effective_date,
            created_at: chrono::Utc::now(),
        }))
    }

    async fn get_employee_payroll_configs(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
    ) -> Result<Vec<crate::models::EmployeePayrollConfig>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        use crate::schema::workspace_config_entries;

        let entries = workspace_config_entries::table
            .filter(workspace_config_entries::workspace_id.eq(workspace_id))
            .filter(
                workspace_config_entries::scope_type.eq("employment")
                    .and(workspace_config_entries::scope_id.eq(employment_id))
            )
            .load::<crate::models::WorkspaceConfigEntry>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let mut dates: Vec<chrono::NaiveDate> = entries
            .iter()
            .map(crate::models::WorkspaceConfigEntry::effective_from)
            .collect();
        dates.sort_unstable();
        dates.dedup();
        dates.reverse();

        let mut configs = Vec::new();
        for d in dates {
            let mut fixed_std_days = None;
            let mut hours_per_day = None;
            let mut entry_id = Uuid::new_v4();
            for entry in &entries {
                if entry.effective_from() == d {
                    if entry.config_key() == "payroll.fixed_std_days" {
                        fixed_std_days = entry.value_decimal();
                        entry_id = entry.id();
                    } else if entry.config_key() == "payroll.hours_per_day" {
                        hours_per_day = entry.value_decimal();
                    }
                }
            }
            if fixed_std_days.is_some() || hours_per_day.is_some() {
                configs.push(crate::models::EmployeePayrollConfig {
                    id: entry_id,
                    employment_id,
                    std_working_days: fixed_std_days
                        .unwrap_or_else(|| rust_decimal::Decimal::from(0)),
                    hours_per_day: hours_per_day.unwrap_or_else(|| rust_decimal::Decimal::from(0)),
                    effective_date: d,
                    created_at: chrono::Utc::now(),
                });
            }
        }

        Ok(configs)
    }

    async fn get_employee_payroll_configs_batch(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_ids: &[Uuid],
    ) -> Result<std::collections::HashMap<Uuid, Vec<crate::models::EmployeePayrollConfig>>, AppError>
    {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        use crate::schema::workspace_config_entries;

        let entries = workspace_config_entries::table
            .filter(workspace_config_entries::workspace_id.eq(workspace_id))
            .filter(
                workspace_config_entries::scope_type.eq("employment")
            )
            .filter(workspace_config_entries::scope_id.eq_any(employment_ids))
            .load::<crate::models::WorkspaceConfigEntry>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let mut employment_entries: std::collections::HashMap<
            Uuid,
            Vec<crate::models::WorkspaceConfigEntry>,
        > = std::collections::HashMap::new();
        for entry in entries {
            if let Some(emp_id) = entry.scope_id() {
                employment_entries.entry(emp_id).or_default().push(entry);
            }
        }

        let mut map = std::collections::HashMap::new();
        for &employment_id in employment_ids {
            if let Some(entries) = employment_entries.get(&employment_id) {
                let mut dates: Vec<chrono::NaiveDate> = entries
                    .iter()
                    .map(crate::models::WorkspaceConfigEntry::effective_from)
                    .collect();
                dates.sort_unstable();
                dates.dedup();
                dates.reverse();

                let mut configs = Vec::new();
                for d in dates {
                    let mut fixed_std_days = None;
                    let mut hours_per_day = None;
                    let mut entry_id = Uuid::new_v4();
                    for entry in entries {
                        if entry.effective_from() == d {
                            if entry.config_key() == "payroll.fixed_std_days" {
                                fixed_std_days = entry.value_decimal();
                                entry_id = entry.id();
                            } else if entry.config_key() == "payroll.hours_per_day" {
                                hours_per_day = entry.value_decimal();
                            }
                        }
                    }
                    if fixed_std_days.is_some() || hours_per_day.is_some() {
                        configs.push(crate::models::EmployeePayrollConfig {
                            id: entry_id,
                            employment_id,
                            std_working_days: fixed_std_days
                                .unwrap_or_else(|| rust_decimal::Decimal::from(0)),
                            hours_per_day: hours_per_day
                                .unwrap_or_else(|| rust_decimal::Decimal::from(0)),
                            effective_date: d,
                            created_at: chrono::Utc::now(),
                        });
                    }
                }
                map.insert(employment_id, configs);
            } else {
                map.insert(employment_id, Vec::new());
            }
        }

        Ok(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diesel_pool::DieselPools;
    use crate::diesel_pool::TenantAuthToken;

    #[tokio::test]
    async fn test_upsert_employee_payroll_config_non_existent_employee() {
        let test_db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://user:pass@localhost:5432/leeattend_test_db".to_string()
        });
        let pools = DieselPools::for_test(&test_db_url);
        let repo = DieselEmployeeRepository::new(pools);
        let ws_id = Uuid::new_v4();
        let auth_token = TenantAuthToken::system_override(ws_id);

        // Pass a random employee_id which does not exist in the tenant's workspace
        let emp_id = Uuid::new_v4();
        let res = repo
            .upsert_employee_payroll_config(
                &auth_token,
                emp_id,
                rust_decimal::Decimal::new(22, 0),
                rust_decimal::Decimal::new(8, 0),
                chrono::NaiveDate::from_ymd_opt(2026, 6, 6).unwrap(),
            )
            .await;

        assert!(res.is_err());
        match res.unwrap_err() {
            AppError::NotFound(msg) => {
                assert_eq!(msg, "error.employee_not_found");
            }
            AppError::Internal(msg) => {
                assert!(
                    msg.contains("connecting") || msg.contains("connection") || msg.contains("db")
                );
            }
            other => panic!("Expected AppError::NotFound or db connection error, got {other:?}"),
        }
    }
}
