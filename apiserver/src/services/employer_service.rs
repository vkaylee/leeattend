use crate::diesel_pool::TenantAuthToken;
use crate::errors::AppError;
use crate::models::{CreateEmployerRequest, Employer, UpdateEmployerRequest};
use crate::repositories::audit_log_repository::AuditLogRepository;
use crate::repositories::employer_repository::EmployerRepository;
use crate::services::audit_service::{AuditHelper, AuditLogArgs};
use crate::utils::i18n::I18nKey;
use axum::http::HeaderMap;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct EmployerService {
    repo: Arc<dyn EmployerRepository>,
    audit_repo: Arc<dyn AuditLogRepository>,
}

impl EmployerService {
    pub fn new(repo: Arc<dyn EmployerRepository>, audit_repo: Arc<dyn AuditLogRepository>) -> Self {
        Self { repo, audit_repo }
    }

    pub async fn list_employers(
        &self,
        auth_token: &TenantAuthToken,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<Employer>, i64), AppError> {
        self.repo.get_employers(auth_token, page, limit).await
    }

    pub async fn get_employer(&self, auth_token: &TenantAuthToken, id: Uuid) -> Result<Employer, AppError> {
        self.repo
            .get_employer_by_id(auth_token, id)
            .await?
            .ok_or_else(|| AppError::not_found(I18nKey::EmployerNotFound))
    }

    pub async fn create_employer(
        &self,
        auth_token: &TenantAuthToken,
        actor_id: Uuid,
        data: CreateEmployerRequest,
    ) -> Result<Employer, AppError> {
        let workspace_id = auth_token.workspace_id();
        self.validate_parent_employer(auth_token, None, data.parent_id)
            .await?;

        if let Some(ref email) = data.email {
            validate_email(email)?;
        }
        if let Some(ref country) = data.country_code {
            validate_country_code(country)?;
        }
        if let Some(ref currency) = data.currency_code {
            validate_currency_code(currency)?;
        }

        let employer = self.repo.create_employer(auth_token, data).await?;

        let empty_headers = HeaderMap::new();
        AuditHelper::log(AuditLogArgs {
            repo: &self.audit_repo,
            workspace_id,
            actor_id: Some(actor_id),
            actor_type: Some("user".to_string()),
            actor_name: None,
            actor_email: None,
            target_user_id: None,
            target_resource_id: Some(employer.id),
            action: "employer.created",
            details: Some(json!({
                "name": employer.name,
                "tax_code": employer.tax_code,
                "business_type": employer.business_type,
                "registration_number": employer.registration_number,
                "phone": employer.phone,
                "email": employer.email,
                "website": employer.website,
                "incorporation_date": employer.incorporation_date,
                "country_code": employer.country_code,
                "currency_code": employer.currency_code,
            })),
            headers: &empty_headers,
        })
        .await;

        Ok(employer)
    }

    pub async fn update_employer(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
        actor_id: Uuid,
        data: UpdateEmployerRequest,
    ) -> Result<Employer, AppError> {
        let workspace_id = auth_token.workspace_id();
        if let Some(parent_opt) = data.parent_id {
            self.validate_parent_employer(auth_token, Some(id), parent_opt)
                .await?;
        }

        if let Some(ref email) = data.email {
            validate_email(email)?;
        }
        if let Some(ref country) = data.country_code {
            validate_country_code(country)?;
        }
        if let Some(ref currency) = data.currency_code {
            validate_currency_code(currency)?;
        }

        let employer = self.repo.update_employer(auth_token, id, data).await?;

        let empty_headers = HeaderMap::new();
        AuditHelper::log(AuditLogArgs {
            repo: &self.audit_repo,
            workspace_id,
            actor_id: Some(actor_id),
            actor_type: Some("user".to_string()),
            actor_name: None,
            actor_email: None,
            target_user_id: None,
            target_resource_id: Some(employer.id),
            action: "employer.updated",
            details: Some(json!({
                "name": employer.name,
                "tax_code": employer.tax_code,
                "business_type": employer.business_type,
                "registration_number": employer.registration_number,
                "phone": employer.phone,
                "email": employer.email,
                "website": employer.website,
                "incorporation_date": employer.incorporation_date,
                "country_code": employer.country_code,
                "currency_code": employer.currency_code,
            })),
            headers: &empty_headers,
        })
        .await;

        Ok(employer)
    }

    pub async fn delete_employer(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
        actor_id: Uuid,
    ) -> Result<(), AppError> {
        let workspace_id = auth_token.workspace_id();
        let existing = self.get_employer(auth_token, id).await?;

        let employee_count = self
            .repo
            .count_employees_by_employer(auth_token, id)
            .await?;

        if employee_count > 0 {
            return Err(AppError::validation_args(
                I18nKey::Validation,
                vec![("details", format!(
                    "Cannot delete employer because {employee_count} employees are still assigned to it."
                ))],
            ));
        }

        self.repo.delete_employer(auth_token, id).await?;

        let empty_headers = HeaderMap::new();
        AuditHelper::log(AuditLogArgs {
            repo: &self.audit_repo,
            workspace_id,
            actor_id: Some(actor_id),
            actor_type: Some("user".to_string()),
            actor_name: None,
            actor_email: None,
            target_user_id: None,
            target_resource_id: Some(existing.id),
            action: "employer.deleted",
            details: Some(json!({ "name": existing.name })),
            headers: &empty_headers,
        })
        .await;

        Ok(())
    }

    async fn validate_parent_employer(
        &self,
        auth_token: &TenantAuthToken,
        employer_id: Option<Uuid>,
        parent_id: Option<Uuid>,
    ) -> Result<(), AppError> {
        let Some(parent_id) = parent_id else {
            return Ok(());
        };

        if let Some(id) = employer_id {
            if id == parent_id {
                return Err(AppError::validation_args(
                    I18nKey::Validation,
                    vec![(
                        "details",
                        "An employer cannot be its own parent.".to_string(),
                    )],
                ));
            }
        }

        let parent = self
            .get_employer(auth_token, parent_id)
            .await
            .map_err(|e| {
                AppError::validation_args(
                    I18nKey::Validation,
                    vec![(
                        "details",
                        format!("Parent employer not found in this workspace: {e}"),
                    )],
                )
            })?;

        if let Some(id) = employer_id {
            let mut current_parent_id = parent.parent_id;
            let mut visited = std::collections::HashSet::new();
            visited.insert(id);
            visited.insert(parent_id);

            while let Some(pid) = current_parent_id {
                if pid == id {
                    return Err(AppError::validation_args(
                        I18nKey::Validation,
                        vec![("details", "Circular dependency detected: updating the parent would create a cycle.".to_string())],
                    ));
                }
                if visited.contains(&pid) {
                    break;
                }
                visited.insert(pid);

                if let Ok(p) = self.get_employer(auth_token, pid).await {
                    current_parent_id = p.parent_id;
                } else {
                    break;
                }
            }
        }

        Ok(())
    }
}

// --- Service Validation Helpers ---

fn validate_email(email: &str) -> Result<(), AppError> {
    let trimmed = email.trim();
    if trimmed.is_empty() || trimmed.len() > 254 {
        return Err(AppError::validation(I18nKey::InvalidEmailFormat));
    }

    let parts: Vec<&str> = trimmed.splitn(2, '@').collect();
    if parts.len() != 2 {
        return Err(AppError::validation(I18nKey::InvalidEmailFormat));
    }

    let (local, domain) = (parts[0], parts[1]);
    if local.is_empty() || local.len() > 64 {
        return Err(AppError::validation(I18nKey::InvalidEmailFormat));
    }

    if domain.is_empty()
        || !domain.contains('.')
        || domain.starts_with('.')
        || domain.ends_with('.')
        || domain.contains("..")
    {
        return Err(AppError::validation(I18nKey::InvalidEmailFormat));
    }

    Ok(())
}

fn validate_country_code(code: &str) -> Result<(), AppError> {
    let code = code.trim();
    if code.len() != 3 || !code.chars().all(|c| c.is_ascii_alphabetic()) {
        return Err(AppError::validation_args(
            I18nKey::Validation,
            vec![(
                "details",
                "Country code must be a 3-character ISO 3166-1 alpha-3 code.".to_string(),
            )],
        ));
    }
    Ok(())
}

fn validate_currency_code(code: &str) -> Result<(), AppError> {
    let code = code.trim();
    if code.len() != 3 || !code.chars().all(|c| c.is_ascii_alphabetic()) {
        return Err(AppError::validation_args(
            I18nKey::Validation,
            vec![(
                "details",
                "Currency code must be a 3-character uppercase string.".to_string(),
            )],
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::audit_log_repository::MockAuditLogRepository;
    use crate::repositories::employer_repository::MockEmployerRepository;
    use std::sync::Arc;

    fn make_token(ws: Uuid) -> TenantAuthToken {
        TenantAuthToken::system_override(ws)
    }

    fn make_employer(ws: Uuid) -> Employer {
        Employer {
            id: Uuid::new_v4(),
            workspace_id: ws,
            name: "Test Corp".into(),
            tax_code: Some("TAX123".into()),
            legal_address: Some("123 Main St".into()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            parent_id: None,
            relationship_type: None,
            business_type: Some("corporation".into()),
            registration_number: Some("REG456".into()),
            phone: Some("+84123456789".into()),
            email: Some("test@corp.com".into()),
            website: Some("https://corp.com".into()),
            incorporation_date: None,
            country_code: Some("VNM".into()),
            currency_code: Some("VND".into()),
        }
    }

    fn build_service(repo: MockEmployerRepository) -> EmployerService {
        EmployerService::new(
            Arc::new(repo),
            Arc::new(MockAuditLogRepository::new()),
        )
    }

    // ── validate_email tests ──────────────────────────────────────────────

    #[test]
    fn validate_email_valid() {
        assert!(validate_email("test@example.com").is_ok());
        assert!(validate_email("user.name+tag@example.co.uk").is_ok());
        assert!(validate_email("a@b.com").is_ok());
    }

    #[test]
    fn validate_email_empty() {
        assert!(validate_email("").is_err());
        assert!(validate_email("   ").is_err());
    }

    #[test]
    fn validate_email_no_at_sign() {
        assert!(validate_email("invalid-email").is_err());
    }

    #[test]
    fn validate_email_missing_domain() {
        assert!(validate_email("user@").is_err());
    }

    #[test]
    fn validate_email_missing_local() {
        assert!(validate_email("@example.com").is_err());
    }

    #[test]
    fn validate_email_domain_no_dot() {
        assert!(validate_email("user@domain").is_err());
    }

    // ── validate_country_code tests ───────────────────────────────────────

    #[test]
    fn validate_country_code_valid() {
        assert!(validate_country_code("VNM").is_ok());
        assert!(validate_country_code("USA").is_ok());
        assert!(validate_country_code("jpn").is_ok()); // lowercase is fine
    }

    #[test]
    fn validate_country_code_wrong_length() {
        assert!(validate_country_code("VN").is_err());
        assert!(validate_country_code("VIET").is_err());
    }

    #[test]
    fn validate_country_code_contains_numbers() {
        assert!(validate_country_code("VN1").is_err());
    }

    #[test]
    fn validate_country_code_contains_special_chars() {
        assert!(validate_country_code("V M").is_err());
    }

    // ── validate_currency_code tests ──────────────────────────────────────

    #[test]
    fn validate_currency_code_valid() {
        assert!(validate_currency_code("VND").is_ok());
        assert!(validate_currency_code("USD").is_ok());
        assert!(validate_currency_code("eur").is_ok());
    }

    #[test]
    fn validate_currency_code_wrong_length() {
        assert!(validate_currency_code("VN").is_err());
        assert!(validate_currency_code("VNDD").is_err());
    }

    #[test]
    fn validate_currency_code_contains_numbers() {
        assert!(validate_currency_code("VN1").is_err());
    }

    // ── list_employers test ───────────────────────────────────────────────

    #[tokio::test]
    async fn list_employers_success() {
        let ws = Uuid::new_v4();
        let employer = make_employer(ws);
        let expected = vec![employer.clone()];

        let mut repo = MockEmployerRepository::new();
        repo.expect_get_employers()
            .returning(move |_, _, _| Ok((expected.clone(), 1)));

        let svc = build_service(repo);
        let result = svc.list_employers(&make_token(ws), 1, 10).await;

        assert!(result.is_ok());
        let (list, total) = result.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(total, 1);
    }

    // ── get_employer tests ────────────────────────────────────────────────

    #[tokio::test]
    async fn get_employer_found() {
        let ws = Uuid::new_v4();
        let employer = make_employer(ws);
        let emp_id = employer.id;
        let expected = employer.clone();

        let mut repo = MockEmployerRepository::new();
        repo.expect_get_employer_by_id()
            .returning(move |_, _| Ok(Some(expected.clone())));

        let svc = build_service(repo);
        let result = svc.get_employer(&make_token(ws), emp_id).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, emp_id);
    }

    #[tokio::test]
    async fn get_employer_not_found() {
        let ws = Uuid::new_v4();
        let mut repo = MockEmployerRepository::new();
        repo.expect_get_employer_by_id()
            .returning(|_, _| Ok(None));

        let svc = build_service(repo);
        let result = svc.get_employer(&make_token(ws), Uuid::new_v4()).await;

        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }

    // ── delete_employer tests ─────────────────────────────────────────────

    #[tokio::test]
    async fn delete_employer_has_employees() {
        let ws = Uuid::new_v4();
        let employer = make_employer(ws);

        let mut repo = MockEmployerRepository::new();
        repo.expect_get_employer_by_id()
            .returning(move |_, _| Ok(Some(employer.clone())));
        repo.expect_count_employees_by_employer()
            .returning(|_, _| Ok(5)); // has employees

        let svc = build_service(repo);
        let result = svc.delete_employer(&make_token(ws), Uuid::new_v4(), Uuid::new_v4()).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
        assert!(err.to_string().contains("5 employees"));
    }
}
