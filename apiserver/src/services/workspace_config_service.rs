use crate::errors::{AppError, AppResult};
use crate::models::{EffectivePayrollSettings, WorkspaceConfigEntry, WorkspaceConfigResolver};
use crate::repositories::WorkspaceConfigRepository;
use crate::utils::i18n::I18nKey;
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
        entry.set_workspace_id(auth_token.workspace_id());

        if entry.scope_type() != "workspace"
            && entry.scope_type() != "employer"
            && entry.scope_type() != "department"
            && entry.scope_type() != "employee"
        {
            return Err(AppError::validation(I18nKey::InvalidScopeType));
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

    pub async fn get_effective_payroll_settings(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        department_id: Option<Uuid>,
        employer_id: Option<Uuid>,
        target_date: NaiveDate,
    ) -> AppResult<EffectivePayrollSettings> {
        let entries = self
            .workspace_config_repo
            .list_entries(auth_token, auth_token.workspace_id(), None)
            .await?;
        let resolver = WorkspaceConfigResolver::new(entries);
        Ok(resolver.resolve_payroll_settings(employee_id, department_id, employer_id, target_date))
    }

    #[allow(clippy::similar_names)]
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
            if let Some(text) = entry.value_text() {
                use std::str::FromStr;
                if let Ok(strategy) = crate::models::HolidayOvernightStrategy::from_str(text) {
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
                if let Some(text) = entry.value_text() {
                    use std::str::FromStr;
                    if let Ok(strategy) = crate::models::HolidayOvernightStrategy::from_str(text) {
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
                if let Some(text) = entry.value_text() {
                    use std::str::FromStr;
                    if let Ok(strategy) = crate::models::HolidayOvernightStrategy::from_str(text) {
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
            if let Some(text) = entry.value_text() {
                use std::str::FromStr;
                if let Ok(strategy) = crate::models::HolidayOvernightStrategy::from_str(text) {
                    return Ok(strategy);
                }
            }
        }

        // 5. Fallback Default
        Ok(crate::models::HolidayOvernightStrategy::SplitAtMidnight)
    }

    #[allow(clippy::similar_names)]
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
            if let Some(decimal) = entry.value_decimal() {
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
                if let Some(decimal) = entry.value_decimal() {
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
                if let Some(decimal) = entry.value_decimal() {
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
            if let Some(decimal) = entry.value_decimal() {
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

    fn create_test_entry(
        id: Uuid,
        workspace_id: Uuid,
        scope_type: String,
        scope_id: Option<Uuid>,
        config_key: String,
        value_int: Option<i32>,
        value_decimal: Option<rust_decimal::Decimal>,
        value_text: Option<String>,
        value_bool: Option<bool>,
        value_json: Option<serde_json::Value>,
        effective_from: NaiveDate,
        effective_to: Option<NaiveDate>,
    ) -> WorkspaceConfigEntry {
        WorkspaceConfigEntry::new(
            id,
            workspace_id,
            scope_type,
            scope_id,
            config_key,
            value_int,
            value_decimal,
            value_text,
            value_bool,
            value_json,
            effective_from,
            effective_to,
            None,
            Utc::now(),
            Utc::now(),
        )
    }

    struct MockWorkspaceConfigRepository {
        pub create_entry_called: std::sync::atomic::AtomicBool,
        pub entries: std::sync::Mutex<Vec<WorkspaceConfigEntry>>,
    }

    impl MockWorkspaceConfigRepository {
        fn new() -> Self {
            Self {
                create_entry_called: std::sync::atomic::AtomicBool::new(false),
                entries: std::sync::Mutex::new(vec![]),
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
            self.entries.lock().unwrap().push(entry.clone());
            Ok(entry)
        }

        async fn get_effective_config(
            &self,
            _auth_token: &TenantAuthToken,
            _workspace_id: Uuid,
            config_key: &str,
            scope_type: &str,
            scope_id: Option<Uuid>,
            _target_date: NaiveDate,
        ) -> Result<Option<WorkspaceConfigEntry>, AppError> {
            let entry = self
                .entries
                .lock()
                .unwrap()
                .iter()
                .find(|e| {
                    e.config_key() == config_key
                        && e.scope_type() == scope_type
                        && e.scope_id() == scope_id
                })
                .cloned();
            Ok(entry)
        }

        async fn list_entries(
            &self,
            _auth_token: &TenantAuthToken,
            _workspace_id: Uuid,
            config_key: Option<&str>,
        ) -> Result<Vec<WorkspaceConfigEntry>, AppError> {
            let filtered = self
                .entries
                .lock()
                .unwrap()
                .iter()
                .filter(|e| {
                    if let Some(key) = config_key {
                        e.config_key() == key
                    } else {
                        true
                    }
                })
                .cloned()
                .collect();
            Ok(filtered)
        }

        async fn delete_entry(
            &self,
            _auth_token: &TenantAuthToken,
            entry_id: Uuid,
        ) -> Result<(), AppError> {
            self.entries.lock().unwrap().retain(|e| e.id() != entry_id);
            Ok(())
        }

        async fn delete_by_key(
            &self,
            _auth_token: &TenantAuthToken,
            _workspace_id: Uuid,
            scope_type: &str,
            config_key: &str,
        ) -> Result<(), AppError> {
            self.entries
                .lock()
                .unwrap()
                .retain(|e| !(e.scope_type() == scope_type && e.config_key() == config_key));
            Ok(())
        }

        async fn get_effective_delegations(
            &self,
            _auth_token: &TenantAuthToken,
            _workspace_id: Uuid,
            _delegate_user_id: Uuid,
            _effective_date: NaiveDate,
        ) -> Result<Vec<Uuid>, AppError> {
            Ok(vec![])
        }
    }

    #[tokio::test]
    async fn test_create_entry_invalid_scope() {
        let repo = Arc::new(MockWorkspaceConfigRepository::new());
        let service = WorkspaceConfigService::new(repo.clone());
        let ws_id = Uuid::new_v4();
        let token = TenantAuthToken::system_override(ws_id);

        let entry = create_test_entry(
            Uuid::new_v4(),
            ws_id,
            "invalid_scope".to_string(),
            None,
            "ot_quota".to_string(),
            Some(10),
            None,
            None,
            None,
            None,
            Utc::now().date_naive(),
            None,
        );

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

        let entry = create_test_entry(
            Uuid::new_v4(),
            Uuid::new_v4(), // deliberate mismatch
            "workspace".to_string(),
            None,
            "ot_quota".to_string(),
            Some(10),
            None,
            None,
            None,
            None,
            Utc::now().date_naive(),
            None,
        );

        let result = service.create_entry(&token, entry).await.unwrap();
        // Ensure workspace_id is enforced
        assert_eq!(result.workspace_id(), ws_id);
        assert!(repo
            .create_entry_called
            .load(std::sync::atomic::Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_get_effective_config() {
        let repo = Arc::new(MockWorkspaceConfigRepository::new());
        let service = WorkspaceConfigService::new(repo.clone());
        let ws_id = Uuid::new_v4();
        let token = TenantAuthToken::system_override(ws_id);

        let entry = create_test_entry(
            Uuid::new_v4(),
            ws_id,
            "employee".to_string(),
            Some(Uuid::new_v4()),
            "some_key".to_string(),
            Some(42),
            None,
            None,
            None,
            None,
            Utc::now().date_naive(),
            None,
        );

        repo.entries.lock().unwrap().push(entry.clone());

        let res = service
            .get_effective_config(
                &token,
                "some_key",
                "employee",
                entry.scope_id(),
                Utc::now().date_naive(),
            )
            .await
            .unwrap();

        assert!(res.is_some());
        assert_eq!(res.unwrap().value_int(), Some(42));
    }

    #[tokio::test]
    async fn test_list_entries() {
        let repo = Arc::new(MockWorkspaceConfigRepository::new());
        let service = WorkspaceConfigService::new(repo.clone());
        let ws_id = Uuid::new_v4();
        let token = TenantAuthToken::system_override(ws_id);

        let entry1 = create_test_entry(
            Uuid::new_v4(),
            ws_id,
            "workspace".to_string(),
            None,
            "key1".to_string(),
            Some(1),
            None,
            None,
            None,
            None,
            Utc::now().date_naive(),
            None,
        );

        let entry2 = create_test_entry(
            Uuid::new_v4(),
            ws_id,
            "workspace".to_string(),
            None,
            "key2".to_string(),
            Some(2),
            None,
            None,
            None,
            None,
            Utc::now().date_naive(),
            None,
        );

        repo.entries.lock().unwrap().push(entry1);
        repo.entries.lock().unwrap().push(entry2);

        let res_all = service.list_entries(&token, None).await.unwrap();
        assert_eq!(res_all.len(), 2);

        let res_key1 = service.list_entries(&token, Some("key1")).await.unwrap();
        assert_eq!(res_key1.len(), 1);
        assert_eq!(res_key1[0].config_key(), "key1");
    }

    #[tokio::test]
    async fn test_delete_entry() {
        let repo = Arc::new(MockWorkspaceConfigRepository::new());
        let service = WorkspaceConfigService::new(repo.clone());
        let ws_id = Uuid::new_v4();
        let token = TenantAuthToken::system_override(ws_id);
        let entry_id = Uuid::new_v4();

        let entry = create_test_entry(
            entry_id,
            ws_id,
            "workspace".to_string(),
            None,
            "key1".to_string(),
            Some(1),
            None,
            None,
            None,
            None,
            Utc::now().date_naive(),
            None,
        );

        repo.entries.lock().unwrap().push(entry);

        service.delete_entry(&token, entry_id).await.unwrap();
        {
            let entries = repo.entries.lock().unwrap();
            assert!(entries.is_empty());
            drop(entries);
        }
    }

    #[tokio::test]
    async fn test_get_effective_holiday_strategy() {
        let repo = Arc::new(MockWorkspaceConfigRepository::new());
        let service = WorkspaceConfigService::new(repo.clone());
        let ws_id = Uuid::new_v4();
        let token = TenantAuthToken::system_override(ws_id);

        let emp_id = Uuid::new_v4();
        let dept_id = Uuid::new_v4();
        let employer_id = Uuid::new_v4();
        let target_date = Utc::now().date_naive();
        let config_key = "overtime.holiday_overnight_strategy";

        // 1. Fallback Default
        let strategy = service
            .get_effective_holiday_strategy(
                &token,
                emp_id,
                Some(dept_id),
                Some(employer_id),
                target_date,
            )
            .await
            .unwrap();
        assert_eq!(
            strategy,
            crate::models::HolidayOvernightStrategy::SplitAtMidnight
        );

        // 2. Workspace scope config
        let ws_entry = create_test_entry(
            Uuid::new_v4(),
            ws_id,
            "workspace".to_string(),
            None,
            config_key.to_string(),
            None,
            None,
            Some("by_punch_in_date".to_string()),
            None,
            None,
            target_date,
            None,
        );
        repo.entries.lock().unwrap().push(ws_entry);

        let strategy = service
            .get_effective_holiday_strategy(
                &token,
                emp_id,
                Some(dept_id),
                Some(employer_id),
                target_date,
            )
            .await
            .unwrap();
        assert_eq!(
            strategy,
            crate::models::HolidayOvernightStrategy::ByPunchInDate
        );

        // 3. Employer scope config overrides Workspace
        let employer_entry = create_test_entry(
            Uuid::new_v4(),
            ws_id,
            "employer".to_string(),
            Some(employer_id),
            config_key.to_string(),
            None,
            None,
            Some("by_majority_hours".to_string()),
            None,
            None,
            target_date,
            None,
        );
        repo.entries.lock().unwrap().push(employer_entry);

        let strategy = service
            .get_effective_holiday_strategy(
                &token,
                emp_id,
                Some(dept_id),
                Some(employer_id),
                target_date,
            )
            .await
            .unwrap();
        assert_eq!(
            strategy,
            crate::models::HolidayOvernightStrategy::ByMajorityHours
        );

        // 4. Department scope config overrides Employer
        let dept_entry = create_test_entry(
            Uuid::new_v4(),
            ws_id,
            "department".to_string(),
            Some(dept_id),
            config_key.to_string(),
            None,
            None,
            Some("split_at_midnight".to_string()),
            None,
            None,
            target_date,
            None,
        );
        repo.entries.lock().unwrap().push(dept_entry);

        let strategy = service
            .get_effective_holiday_strategy(
                &token,
                emp_id,
                Some(dept_id),
                Some(employer_id),
                target_date,
            )
            .await
            .unwrap();
        assert_eq!(
            strategy,
            crate::models::HolidayOvernightStrategy::SplitAtMidnight
        );

        // 5. Employee scope config overrides Department
        let emp_entry = create_test_entry(
            Uuid::new_v4(),
            ws_id,
            "employee".to_string(),
            Some(emp_id),
            config_key.to_string(),
            None,
            None,
            Some("by_punch_in_date".to_string()),
            None,
            None,
            target_date,
            None,
        );
        repo.entries.lock().unwrap().push(emp_entry);

        let strategy = service
            .get_effective_holiday_strategy(
                &token,
                emp_id,
                Some(dept_id),
                Some(employer_id),
                target_date,
            )
            .await
            .unwrap();
        assert_eq!(
            strategy,
            crate::models::HolidayOvernightStrategy::ByPunchInDate
        );
    }

    #[tokio::test]
    async fn test_get_effective_holiday_ot_multiplier() {
        let repo = Arc::new(MockWorkspaceConfigRepository::new());
        let service = WorkspaceConfigService::new(repo.clone());
        let ws_id = Uuid::new_v4();
        let token = TenantAuthToken::system_override(ws_id);

        let emp_id = Uuid::new_v4();
        let dept_id = Uuid::new_v4();
        let employer_id = Uuid::new_v4();
        let target_date = Utc::now().date_naive();
        let config_key = "overtime.default_holiday_ot_multiplier";

        // 1. Fallback Default (3.0)
        let mult = service
            .get_effective_holiday_ot_multiplier(
                &token,
                emp_id,
                Some(dept_id),
                Some(employer_id),
                target_date,
            )
            .await
            .unwrap();
        assert_eq!(mult, rust_decimal::Decimal::new(30, 1));

        // 2. Workspace scope config
        let ws_entry = create_test_entry(
            Uuid::new_v4(),
            ws_id,
            "workspace".to_string(),
            None,
            config_key.to_string(),
            None,
            Some(rust_decimal::Decimal::new(15, 1)), // 1.5
            None,
            None,
            None,
            target_date,
            None,
        );
        repo.entries.lock().unwrap().push(ws_entry);

        let mult = service
            .get_effective_holiday_ot_multiplier(
                &token,
                emp_id,
                Some(dept_id),
                Some(employer_id),
                target_date,
            )
            .await
            .unwrap();
        assert_eq!(mult, rust_decimal::Decimal::new(15, 1));

        // 3. Employer scope config overrides Workspace
        let employer_entry = create_test_entry(
            Uuid::new_v4(),
            ws_id,
            "employer".to_string(),
            Some(employer_id),
            config_key.to_string(),
            None,
            Some(rust_decimal::Decimal::new(20, 1)), // 2.0
            None,
            None,
            None,
            target_date,
            None,
        );
        repo.entries.lock().unwrap().push(employer_entry);

        let mult = service
            .get_effective_holiday_ot_multiplier(
                &token,
                emp_id,
                Some(dept_id),
                Some(employer_id),
                target_date,
            )
            .await
            .unwrap();
        assert_eq!(mult, rust_decimal::Decimal::new(20, 1));

        // 4. Department scope config overrides Employer
        let dept_entry = create_test_entry(
            Uuid::new_v4(),
            ws_id,
            "department".to_string(),
            Some(dept_id),
            config_key.to_string(),
            None,
            Some(rust_decimal::Decimal::new(25, 1)), // 2.5
            None,
            None,
            None,
            target_date,
            None,
        );
        repo.entries.lock().unwrap().push(dept_entry);

        let mult = service
            .get_effective_holiday_ot_multiplier(
                &token,
                emp_id,
                Some(dept_id),
                Some(employer_id),
                target_date,
            )
            .await
            .unwrap();
        assert_eq!(mult, rust_decimal::Decimal::new(25, 1));

        // 5. Employee scope config overrides Department
        let emp_entry = create_test_entry(
            Uuid::new_v4(),
            ws_id,
            "employee".to_string(),
            Some(emp_id),
            config_key.to_string(),
            None,
            Some(rust_decimal::Decimal::new(35, 1)), // 3.5
            None,
            None,
            None,
            target_date,
            None,
        );
        repo.entries.lock().unwrap().push(emp_entry);

        let mult = service
            .get_effective_holiday_ot_multiplier(
                &token,
                emp_id,
                Some(dept_id),
                Some(employer_id),
                target_date,
            )
            .await
            .unwrap();
        assert_eq!(mult, rust_decimal::Decimal::new(35, 1));
    }

    #[tokio::test]
    async fn test_get_effective_payroll_settings() {
        let repo = Arc::new(MockWorkspaceConfigRepository::new());
        let service = WorkspaceConfigService::new(repo.clone());
        let ws_id = Uuid::new_v4();
        let token = TenantAuthToken::system_override(ws_id);

        let emp_id = Uuid::new_v4();
        let dept_id = Uuid::new_v4();
        let employer_id = Uuid::new_v4();
        let target_date = Utc::now().date_naive();

        // 1. Fallback Default
        let settings = service
            .get_effective_payroll_settings(
                &token,
                emp_id,
                Some(dept_id),
                Some(employer_id),
                target_date,
            )
            .await
            .unwrap();
        assert_eq!(settings.std_working_day_mode, "fixed");
        assert_eq!(settings.fixed_std_days, rust_decimal::Decimal::from(26));
        assert_eq!(settings.working_days_per_week, 5);
        assert_eq!(settings.hours_per_day, rust_decimal::Decimal::from(8));
        assert!(settings.max_ot_hours_per_day.is_none());
        assert!(settings.max_ot_hours_per_month.is_none());

        // 2. Workspace scope overrides default
        let ws_entry = create_test_entry(
            Uuid::new_v4(),
            ws_id,
            "workspace".to_string(),
            None,
            "payroll.fixed_std_days".to_string(),
            None,
            Some(rust_decimal::Decimal::from(22)),
            None,
            None,
            None,
            target_date,
            None,
        );
        repo.entries.lock().unwrap().push(ws_entry);

        let settings = service
            .get_effective_payroll_settings(
                &token,
                emp_id,
                Some(dept_id),
                Some(employer_id),
                target_date,
            )
            .await
            .unwrap();
        assert_eq!(settings.fixed_std_days, rust_decimal::Decimal::from(22));

        // 3. Employer scope overrides Workspace
        let employer_entry = create_test_entry(
            Uuid::new_v4(),
            ws_id,
            "employer".to_string(),
            Some(employer_id),
            "payroll.fixed_std_days".to_string(),
            None,
            Some(rust_decimal::Decimal::from(24)),
            None,
            None,
            None,
            target_date,
            None,
        );
        repo.entries.lock().unwrap().push(employer_entry);

        let settings = service
            .get_effective_payroll_settings(
                &token,
                emp_id,
                Some(dept_id),
                Some(employer_id),
                target_date,
            )
            .await
            .unwrap();
        assert_eq!(settings.fixed_std_days, rust_decimal::Decimal::from(24));

        // 4. Department scope overrides Employer
        let dept_entry = create_test_entry(
            Uuid::new_v4(),
            ws_id,
            "department".to_string(),
            Some(dept_id),
            "payroll.fixed_std_days".to_string(),
            None,
            Some(rust_decimal::Decimal::from(25)),
            None,
            None,
            None,
            target_date,
            None,
        );
        repo.entries.lock().unwrap().push(dept_entry);

        let settings = service
            .get_effective_payroll_settings(
                &token,
                emp_id,
                Some(dept_id),
                Some(employer_id),
                target_date,
            )
            .await
            .unwrap();
        assert_eq!(settings.fixed_std_days, rust_decimal::Decimal::from(25));

        // 5. Employee scope overrides Department
        let emp_entry = create_test_entry(
            Uuid::new_v4(),
            ws_id,
            "employee".to_string(),
            Some(emp_id),
            "payroll.fixed_std_days".to_string(),
            None,
            Some(rust_decimal::Decimal::from(26)),
            None,
            None,
            None,
            target_date,
            None,
        );
        repo.entries.lock().unwrap().push(emp_entry);

        let settings = service
            .get_effective_payroll_settings(
                &token,
                emp_id,
                Some(dept_id),
                Some(employer_id),
                target_date,
            )
            .await
            .unwrap();
        assert_eq!(settings.fixed_std_days, rust_decimal::Decimal::from(26));
    }
}
