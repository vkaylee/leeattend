use crate::diesel_pool::TenantAuthToken;
use crate::errors::{AppError, AppResult};
use crate::models::{
    ConditionRuleEffects, ConditionTypeDefinition, CreateConditionTypeRequest,
    UpdateConditionTypeRequest,
};
use crate::repositories::condition_type_repository::ConditionTypeRepository;
use crate::utils::i18n::I18nKey;
use crate::utils::id::new_id;
use chrono::Utc;
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

pub struct ConditionTypeService {
    repo: Arc<dyn ConditionTypeRepository>,
}

impl ConditionTypeService {
    pub fn new(repo: Arc<dyn ConditionTypeRepository>) -> Self {
        Self { repo }
    }

    /// Validate condition type code format: uppercase alphanumeric with underscores, starts with letter.
    fn validate_code(code: &str) -> AppResult<()> {
        let re = regex::Regex::new(r"^[A-Z][A-Z0-9_]*$").expect("valid regex");
        if !re.is_match(code) {
            return Err(AppError::validation_args(
                I18nKey::Validation,
                [(
                    "detail",
                    "Code must start with an uppercase letter and contain only uppercase letters, digits, and underscores",
                )],
            ));
        }
        Ok(())
    }

    /// Validate name_i18n: must be a JSON object with at least "en" key.
    fn validate_name_i18n(name: &serde_json::Value) -> AppResult<()> {
        match name.as_object() {
            Some(obj) if obj.contains_key("en") => Ok(()),
            _ => Err(AppError::validation_args(
                I18nKey::Validation,
                [(
                    "detail",
                    "name_i18n must be a JSON object with at least 'en' key",
                )],
            )),
        }
    }

    fn validate_affects_rules(
        affects_rules: &Value,
        default_minute_allowance: i32,
    ) -> AppResult<()> {
        if !affects_rules.is_object() {
            return Err(AppError::validation_args(
                I18nKey::Validation,
                [("detail", "affects_rules must be a JSON object")],
            ));
        }

        let effects: ConditionRuleEffects =
            serde_json::from_value(affects_rules.clone()).map_err(|_| {
                AppError::validation_args(
                    I18nKey::Validation,
                    [(
                        "detail",
                        "affects_rules contains an unsupported field or invalid value",
                    )],
                )
            })?;

        if !effects.has_effect() {
            return Err(AppError::validation_args(
                I18nKey::Validation,
                [(
                    "detail",
                    "affects_rules must configure at least one policy effect",
                )],
            ));
        }

        if (effects.authorized_early_leave_minutes == Some(true)
            || effects.grace_period_minutes == Some(true))
            && default_minute_allowance <= 0
        {
            return Err(AppError::validation_args(
                I18nKey::Validation,
                [(
                    "detail",
                    "minute_allowance must be greater than zero for minute-derived policy effects",
                )],
            ));
        }

        Ok(())
    }

    fn validate_update_affects_rules(
        affects_rules: &Value,
        existing: &ConditionTypeDefinition,
        default_minute_allowance: Option<i32>,
    ) -> AppResult<()> {
        Self::validate_affects_rules(
            affects_rules,
            default_minute_allowance.unwrap_or_else(|| existing.default_minute_allowance()),
        )
    }

    pub async fn list(
        &self,
        auth_token: &TenantAuthToken,
        active_only: bool,
    ) -> AppResult<Vec<ConditionTypeDefinition>> {
        self.repo
            .list_condition_types(auth_token, active_only)
            .await
    }

    pub async fn get_by_id(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
    ) -> AppResult<Option<ConditionTypeDefinition>> {
        self.repo.get_condition_type_by_id(auth_token, id).await
    }

    pub async fn create(
        &self,
        auth_token: &TenantAuthToken,
        req: CreateConditionTypeRequest,
    ) -> AppResult<ConditionTypeDefinition> {
        Self::validate_code(&req.code)?;
        Self::validate_name_i18n(&req.name_i18n)?;
        Self::validate_affects_rules(&req.affects_rules, req.default_minute_allowance)?;

        let now = Utc::now();
        let definition = ConditionTypeDefinition::new(
            new_id(),
            auth_token.workspace_id(),
            req.code,
            req.name_i18n,
            req.category,
            req.default_minute_allowance,
            req.default_is_paid,
            req.affects_rules,
            req.icon,
            req.color,
            false, // is_system: user-created types are never system
            req.sort_order,
            true, // is_active
            now,
            now,
        );

        // Check for duplicate code
        if let Some(existing) = self
            .repo
            .get_condition_type_by_code(auth_token, definition.code())
            .await?
        {
            return Err(AppError::conflict_args(
                I18nKey::Conflict,
                [(
                    "resource",
                    format!("Condition type code '{}'", existing.code()),
                )],
            ));
        }

        self.repo
            .create_condition_type(auth_token, &definition)
            .await
    }

    pub async fn update(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
        req: UpdateConditionTypeRequest,
    ) -> AppResult<ConditionTypeDefinition> {
        // Validate name_i18n if provided
        if let Some(ref name) = req.name_i18n {
            Self::validate_name_i18n(name)?;
        }

        // Verify the type exists and belongs to this workspace
        let existing = self
            .repo
            .get_condition_type_by_id(auth_token, id)
            .await?
            .ok_or_else(|| AppError::not_found(I18nKey::NotFound))?;

        if let Some(ref affects_rules) = req.affects_rules {
            Self::validate_update_affects_rules(
                affects_rules,
                &existing,
                req.default_minute_allowance,
            )?;
        }

        // Cannot change code of an existing type
        // Cannot change is_system flag

        self.repo.update_condition_type(auth_token, id, &req).await
    }

    pub async fn delete(&self, auth_token: &TenantAuthToken, id: Uuid) -> AppResult<()> {
        // Verify the type exists and check is_system
        let existing = self
            .repo
            .get_condition_type_by_id(auth_token, id)
            .await?
            .ok_or_else(|| AppError::not_found(I18nKey::NotFound))?;

        if existing.is_system() {
            return Err(AppError::forbidden_args(
                I18nKey::Forbidden,
                [("reason", "System condition types cannot be deleted")],
            ));
        }

        self.repo.delete_condition_type(auth_token, id).await
    }
}
