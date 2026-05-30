use crate::errors::{AppError, AppResult};
use crate::models::WorkspaceConfigEntry;
use crate::repositories::WorkspaceConfigRepository;
use chrono::NaiveDate;
use std::sync::Arc;
use uuid::Uuid;

pub struct WorkspaceConfigService {
    workspace_config_repo: Arc<dyn WorkspaceConfigRepository>,
}

impl WorkspaceConfigService {
    pub fn new(workspace_config_repo: Arc<dyn WorkspaceConfigRepository>) -> Self {
        Self {
            workspace_config_repo,
        }
    }

    pub async fn create_entry(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        mut entry: WorkspaceConfigEntry,
    ) -> AppResult<WorkspaceConfigEntry> {
        // Enforce workspace_id matches auth_token
        entry.workspace_id = auth_token.workspace_id();

        if entry.scope_type != "workspace"
            && entry.scope_type != "employer"
            && entry.scope_type != "department"
            && entry.scope_type != "employee"
        {
            return Err(AppError::Validation("Invalid scope type".to_string()));
        }

        self.workspace_config_repo
            .create_entry(auth_token, entry)
            .await
    }

    pub async fn get_effective_config(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        config_key: &str,
        scope_type: &str,
        scope_id: Option<Uuid>,
        target_date: NaiveDate,
    ) -> AppResult<Option<WorkspaceConfigEntry>> {
        self.workspace_config_repo
            .get_effective_config(
                auth_token,
                auth_token.workspace_id(),
                config_key,
                scope_type,
                scope_id,
                target_date,
            )
            .await
    }

    pub async fn list_entries(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        config_key: Option<&str>,
    ) -> AppResult<Vec<WorkspaceConfigEntry>> {
        self.workspace_config_repo
            .list_entries(auth_token, auth_token.workspace_id(), config_key)
            .await
    }

    pub async fn delete_entry(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        entry_id: Uuid,
    ) -> AppResult<()> {
        self.workspace_config_repo
            .delete_entry(auth_token, entry_id)
            .await
    }

    pub async fn get_effective_holiday_strategy(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        department_id: Option<Uuid>,
        employer_id: Option<Uuid>,
        target_date: NaiveDate,
    ) -> AppResult<crate::models::HolidayOvernightStrategy> {
        let config_key = "overtime.holiday_overnight_strategy";

        // 1. Employee scope
        if let Some(entry) = self
            .workspace_config_repo
            .get_effective_config(
                auth_token,
                auth_token.workspace_id(),
                config_key,
                "employee",
                Some(employee_id),
                target_date,
            )
            .await?
        {
            if let Some(text) = entry.value_text {
                use std::str::FromStr;
                if let Ok(strategy) = crate::models::HolidayOvernightStrategy::from_str(&text) {
                    return Ok(strategy);
                }
            }
        }

        // 2. Department scope
        if let Some(dept_id) = department_id {
            if let Some(entry) = self
                .workspace_config_repo
                .get_effective_config(
                    auth_token,
                    auth_token.workspace_id(),
                    config_key,
                    "department",
                    Some(dept_id),
                    target_date,
                )
                .await?
            {
                if let Some(text) = entry.value_text {
                    use std::str::FromStr;
                    if let Ok(strategy) = crate::models::HolidayOvernightStrategy::from_str(&text) {
                        return Ok(strategy);
                    }
                }
            }
        }

        // 3. Employer scope
        if let Some(emp_id) = employer_id {
            if let Some(entry) = self
                .workspace_config_repo
                .get_effective_config(
                    auth_token,
                    auth_token.workspace_id(),
                    config_key,
                    "employer",
                    Some(emp_id),
                    target_date,
                )
                .await?
            {
                if let Some(text) = entry.value_text {
                    use std::str::FromStr;
                    if let Ok(strategy) = crate::models::HolidayOvernightStrategy::from_str(&text) {
                        return Ok(strategy);
                    }
                }
            }
        }

        // 4. Workspace scope
        if let Some(entry) = self
            .workspace_config_repo
            .get_effective_config(
                auth_token,
                auth_token.workspace_id(),
                config_key,
                "workspace",
                None,
                target_date,
            )
            .await?
        {
            if let Some(text) = entry.value_text {
                use std::str::FromStr;
                if let Ok(strategy) = crate::models::HolidayOvernightStrategy::from_str(&text) {
                    return Ok(strategy);
                }
            }
        }

        // 5. Fallback Default
        Ok(crate::models::HolidayOvernightStrategy::SplitAtMidnight)
    }

    pub async fn get_effective_holiday_ot_multiplier(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        department_id: Option<Uuid>,
        employer_id: Option<Uuid>,
        target_date: NaiveDate,
    ) -> AppResult<rust_decimal::Decimal> {
        let config_key = "overtime.default_holiday_ot_multiplier";

        // 1. Employee scope
        if let Some(entry) = self
            .workspace_config_repo
            .get_effective_config(
                auth_token,
                auth_token.workspace_id(),
                config_key,
                "employee",
                Some(employee_id),
                target_date,
            )
            .await?
        {
            if let Some(decimal) = entry.value_decimal {
                return Ok(decimal);
            }
        }

        // 2. Department scope
        if let Some(dept_id) = department_id {
            if let Some(entry) = self
                .workspace_config_repo
                .get_effective_config(
                    auth_token,
                    auth_token.workspace_id(),
                    config_key,
                    "department",
                    Some(dept_id),
                    target_date,
                )
                .await?
            {
                if let Some(decimal) = entry.value_decimal {
                    return Ok(decimal);
                }
            }
        }

        // 3. Employer scope
        if let Some(emp_id) = employer_id {
            if let Some(entry) = self
                .workspace_config_repo
                .get_effective_config(
                    auth_token,
                    auth_token.workspace_id(),
                    config_key,
                    "employer",
                    Some(emp_id),
                    target_date,
                )
                .await?
            {
                if let Some(decimal) = entry.value_decimal {
                    return Ok(decimal);
                }
            }
        }

        // 4. Workspace scope
        if let Some(entry) = self
            .workspace_config_repo
            .get_effective_config(
                auth_token,
                auth_token.workspace_id(),
                config_key,
                "workspace",
                None,
                target_date,
            )
            .await?
        {
            if let Some(decimal) = entry.value_decimal {
                return Ok(decimal);
            }
        }

        // 5. Fallback Default
        Ok(rust_decimal::Decimal::new(30, 1)) // 3.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diesel_pool::TenantAuthToken;
    use crate::errors::AppError;
    use async_trait::async_trait;
    use chrono::{NaiveDate, Utc};

    struct MockWorkspaceConfigRepository {
        pub create_entry_called: std::sync::atomic::AtomicBool,
    }

    impl MockWorkspaceConfigRepository {
        fn new() -> Self {
            Self {
                create_entry_called: std::sync::atomic::AtomicBool::new(false),
            }
        }
    }

    #[async_trait]
    impl WorkspaceConfigRepository for MockWorkspaceConfigRepository {
        async fn create_entry(
            &self,
            _auth_token: &TenantAuthToken,
            entry: WorkspaceConfigEntry,
        ) -> Result<WorkspaceConfigEntry, AppError> {
            self.create_entry_called
                .store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(entry)
        }

        async fn get_effective_config(
            &self,
            _auth_token: &TenantAuthToken,
            _workspace_id: Uuid,
            _config_key: &str,
            _scope_type: &str,
            _scope_id: Option<Uuid>,
            _target_date: NaiveDate,
        ) -> Result<Option<WorkspaceConfigEntry>, AppError> {
            Ok(None)
        }

        async fn list_entries(
            &self,
            _auth_token: &TenantAuthToken,
            _workspace_id: Uuid,
            _config_key: Option<&str>,
        ) -> Result<Vec<WorkspaceConfigEntry>, AppError> {
            Ok(vec![])
        }

        async fn delete_entry(
            &self,
            _auth_token: &TenantAuthToken,
            _entry_id: Uuid,
        ) -> Result<(), AppError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_create_entry_invalid_scope() {
        let repo = Arc::new(MockWorkspaceConfigRepository::new());
        let service = WorkspaceConfigService::new(repo.clone());
        let ws_id = Uuid::new_v4();
        let token = TenantAuthToken::system_override(ws_id);

        let entry = WorkspaceConfigEntry {
            id: Uuid::new_v4(),
            workspace_id: ws_id,
            scope_type: "invalid_scope".to_string(),
            scope_id: None,
            config_key: "ot_quota".to_string(),
            value_int: Some(10),
            value_decimal: None,
            value_text: None,
            value_bool: None,
            value_json: None,
            effective_from: Utc::now().date_naive(),
            effective_to: None,
            created_by: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let result = service.create_entry(&token, entry).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::Validation(_)));
        assert!(!repo
            .create_entry_called
            .load(std::sync::atomic::Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_create_entry_success() {
        let repo = Arc::new(MockWorkspaceConfigRepository::new());
        let service = WorkspaceConfigService::new(repo.clone());
        let ws_id = Uuid::new_v4();
        let token = TenantAuthToken::system_override(ws_id);

        let entry = WorkspaceConfigEntry {
            id: Uuid::new_v4(),
            workspace_id: Uuid::new_v4(), // deliberate mismatch
            scope_type: "workspace".to_string(),
            scope_id: None,
            config_key: "ot_quota".to_string(),
            value_int: Some(10),
            value_decimal: None,
            value_text: None,
            value_bool: None,
            value_json: None,
            effective_from: Utc::now().date_naive(),
            effective_to: None,
            created_by: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let result = service.create_entry(&token, entry).await.unwrap();
        // Ensure workspace_id is enforced
        assert_eq!(result.workspace_id, ws_id);
        assert!(repo
            .create_entry_called
            .load(std::sync::atomic::Ordering::SeqCst));
    }
}
