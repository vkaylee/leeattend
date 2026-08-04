#[cfg(test)]
mod tests {
    use crate::diesel_pool::TenantAuthToken;
    use crate::errors::AppError;
    use crate::models::{
        ConditionTypeDefinition, CreateConditionTypeRequest, NewConditionTypeParams,
        UpdateConditionTypeRequest,
    };
    use crate::repositories::condition_type_repository::MockConditionTypeRepository;
    use crate::services::condition_type_service::ConditionTypeService;
    use std::sync::Arc;
    use uuid::Uuid;

    fn make_token() -> TenantAuthToken {
        TenantAuthToken::system_override(Uuid::new_v4())
    }

    fn make_definition(
        id: Uuid,
        workspace_id: Uuid,
        code: &str,
        is_system: bool,
    ) -> ConditionTypeDefinition {
        ConditionTypeDefinition::new(NewConditionTypeParams {
            id: id,
            workspace_id: workspace_id,
            code: code.to_string(),
            name_i18n: serde_json::json!({"en": "Test Type", "vi": "Loại Test"}),
            category: "general".to_string(),
            default_minute_allowance: 30,
            default_is_paid: true,
            affects_rules: serde_json::json!({"authorized_early_leave_minutes": true}),
            icon: Some("Clock".to_string()),
            color: Some("blue".to_string()),
            is_system: is_system,
            sort_order: 0,
            is_active: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }

    fn make_create_req(code: &str) -> CreateConditionTypeRequest {
        CreateConditionTypeRequest {
            code: code.to_string(),
            name_i18n: serde_json::json!({"en": "Test"}),
            category: "general".to_string(),
            default_minute_allowance: 30,
            default_is_paid: false,
            affects_rules: serde_json::json!({"authorized_early_leave_minutes": true}),
            icon: None,
            color: None,
            sort_order: 0,
        }
    }

    // ── Code validation ──────────────────────────────────────────────

    #[tokio::test]
    async fn validate_code_valid() {
        let mut mock = MockConditionTypeRepository::new();
        let ws_id = Uuid::new_v4();

        mock.expect_get_condition_type_by_code()
            .returning(|_, _| Box::pin(async { Ok(None) }));
        mock.expect_create_condition_type().returning(|_, d| {
            let cloned = d.clone();
            Box::pin(async move { Ok(cloned) })
        });

        let svc = ConditionTypeService::new(Arc::new(mock));
        let result = svc
            .create(
                &TenantAuthToken::system_override(ws_id),
                make_create_req("EARLY_LEAVE"),
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn validate_code_with_digits_and_underscores() {
        let mut mock = MockConditionTypeRepository::new();
        let ws_id = Uuid::new_v4();

        mock.expect_get_condition_type_by_code()
            .returning(|_, _| Box::pin(async { Ok(None) }));
        mock.expect_create_condition_type().returning(|_, d| {
            let cloned = d.clone();
            Box::pin(async move { Ok(cloned) })
        });

        let svc = ConditionTypeService::new(Arc::new(mock));
        let result = svc
            .create(
                &TenantAuthToken::system_override(ws_id),
                make_create_req("TYPE_2A"),
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn validate_code_rejects_lowercase() {
        let mock = MockConditionTypeRepository::new();
        let svc = ConditionTypeService::new(Arc::new(mock));
        let result = svc
            .create(&make_token(), make_create_req("early_leave"))
            .await;
        assert!(matches!(result.unwrap_err(), AppError::Validation(_)));
    }

    #[tokio::test]
    async fn validate_code_rejects_special_chars() {
        let mock = MockConditionTypeRepository::new();
        let svc = ConditionTypeService::new(Arc::new(mock));
        let result = svc
            .create(&make_token(), make_create_req("EARLY-LEAVE"))
            .await;
        assert!(matches!(result.unwrap_err(), AppError::Validation(_)));
    }

    // ── name_i18n validation ──────────────────────────────────────────

    #[tokio::test]
    async fn validate_name_i18n_requires_en_key() {
        let mut mock = MockConditionTypeRepository::new();
        mock.expect_get_condition_type_by_code()
            .returning(|_, _| Box::pin(async { Ok(None) }));

        let svc = ConditionTypeService::new(Arc::new(mock));
        let mut req = make_create_req("VALID");
        req.name_i18n = serde_json::json!({"vi": "Thiếu EN"});
        let result = svc.create(&make_token(), req).await;
        assert!(matches!(result.unwrap_err(), AppError::Validation(_)));
    }

    #[tokio::test]
    async fn validate_name_i18n_rejects_non_object() {
        let mut mock = MockConditionTypeRepository::new();
        mock.expect_get_condition_type_by_code()
            .returning(|_, _| Box::pin(async { Ok(None) }));

        let svc = ConditionTypeService::new(Arc::new(mock));
        let mut req = make_create_req("VALID");
        req.name_i18n = serde_json::json!("not an object");
        let result = svc.create(&make_token(), req).await;
        assert!(matches!(result.unwrap_err(), AppError::Validation(_)));
    }

    // ── affects_rules validation ──────────────────────────────────────

    #[tokio::test]
    async fn validate_affects_rules_accepts_all_supported_effects() {
        let mut mock = MockConditionTypeRepository::new();
        mock.expect_get_condition_type_by_code()
            .returning(|_, _| Box::pin(async { Ok(None) }));
        mock.expect_create_condition_type()
            .returning(|_, definition| {
                let definition = definition.clone();
                Box::pin(async move { Ok(definition) })
            });

        let mut req = make_create_req("FULL_EFFECTS");
        req.affects_rules = serde_json::json!({
            "authorized_early_leave_minutes": true,
            "is_early_leave_paid": true,
            "grace_period_minutes": true,
            "is_break_auto_deduct": false,
            "is_break_overtime": true,
            "allow_early_ot": false,
            "early_arrival_strategy": "Block"
        });

        let result = ConditionTypeService::new(Arc::new(mock))
            .create(&make_token(), req)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn validate_affects_rules_rejects_no_op_and_invalid_values() {
        for affects_rules in [
            serde_json::json!({}),
            serde_json::json!({"unknown_policy": true}),
            serde_json::json!({"is_break_overtime": "yes"}),
            serde_json::json!({"early_arrival_strategy": "Later"}),
        ] {
            let mut req = make_create_req("INVALID_EFFECTS");
            req.affects_rules = affects_rules;
            let result = ConditionTypeService::new(Arc::new(MockConditionTypeRepository::new()))
                .create(&make_token(), req)
                .await;
            assert!(matches!(result.unwrap_err(), AppError::Validation(_)));
        }
    }

    #[tokio::test]
    async fn validate_affects_rules_requires_positive_allowance_for_minute_effects() {
        let mut req = make_create_req("NO_ALLOWANCE");
        req.default_minute_allowance = 0;
        req.affects_rules = serde_json::json!({"grace_period_minutes": true});

        let result = ConditionTypeService::new(Arc::new(MockConditionTypeRepository::new()))
            .create(&make_token(), req)
            .await;

        assert!(matches!(result.unwrap_err(), AppError::Validation(_)));
    }

    // ── Duplicate code ────────────────────────────────────────────────

    #[tokio::test]
    async fn create_duplicate_code_returns_conflict() {
        let ws = Uuid::new_v4();
        let existing = make_definition(Uuid::new_v4(), ws, "EARLY_LEAVE", true);

        let mut mock = MockConditionTypeRepository::new();
        let existing_clone = existing.clone();
        mock.expect_get_condition_type_by_code()
            .returning(move |_, _| {
                Box::pin({
                    let ex = existing_clone.clone();
                    async move { Ok(Some(ex)) }
                })
            });

        let svc = ConditionTypeService::new(Arc::new(mock));
        let result = svc
            .create(
                &TenantAuthToken::system_override(ws),
                make_create_req("EARLY_LEAVE"),
            )
            .await;
        assert!(matches!(result.unwrap_err(), AppError::Conflict(_, _)));
    }

    // ── Delete system type ────────────────────────────────────────────

    #[tokio::test]
    async fn delete_system_type_returns_forbidden() {
        let ws = Uuid::new_v4();
        let id = Uuid::new_v4();
        let sys_type = make_definition(id, ws, "EARLY_LEAVE", true);

        let mut mock = MockConditionTypeRepository::new();
        mock.expect_get_condition_type_by_id()
            .returning(move |_, _| {
                Box::pin({
                    let st = sys_type.clone();
                    async move { Ok(Some(st)) }
                })
            });

        let svc = ConditionTypeService::new(Arc::new(mock));
        let result = svc.delete(&TenantAuthToken::system_override(ws), id).await;
        assert!(matches!(result.unwrap_err(), AppError::Forbidden(_)));
    }

    #[tokio::test]
    async fn delete_custom_type_succeeds() {
        let ws = Uuid::new_v4();
        let id = Uuid::new_v4();
        let custom = make_definition(id, ws, "CUSTOM", false);

        let mut mock = MockConditionTypeRepository::new();
        mock.expect_get_condition_type_by_id()
            .returning(move |_, _| {
                Box::pin({
                    let ct = custom.clone();
                    async move { Ok(Some(ct)) }
                })
            });
        mock.expect_delete_condition_type()
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let svc = ConditionTypeService::new(Arc::new(mock));
        let result = svc.delete(&TenantAuthToken::system_override(ws), id).await;
        assert!(result.is_ok());
    }

    // ── Update not found ──────────────────────────────────────────────

    #[tokio::test]
    async fn update_not_found_returns_not_found() {
        let mut mock = MockConditionTypeRepository::new();
        mock.expect_get_condition_type_by_id()
            .returning(|_, _| Box::pin(async { Ok(None) }));

        let svc = ConditionTypeService::new(Arc::new(mock));
        let result = svc
            .update(
                &make_token(),
                Uuid::new_v4(),
                UpdateConditionTypeRequest {
                    name_i18n: None,
                    category: None,
                    default_minute_allowance: None,
                    default_is_paid: None,
                    affects_rules: None,
                    icon: None,
                    color: None,
                    sort_order: None,
                    is_active: None,
                },
            )
            .await;
        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }

    // ── List ──────────────────────────────────────────────────────────

    #[tokio::test]
    async fn list_returns_types() {
        let ws = Uuid::new_v4();
        let def = make_definition(Uuid::new_v4(), ws, "TYPE_A", false);

        let mut mock = MockConditionTypeRepository::new();
        mock.expect_list_condition_types().returning(move |_, _| {
            Box::pin({
                let d = def.clone();
                async move { Ok(vec![d]) }
            })
        });

        let svc = ConditionTypeService::new(Arc::new(mock));
        let result = svc.list(&TenantAuthToken::system_override(ws), false).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }
}
