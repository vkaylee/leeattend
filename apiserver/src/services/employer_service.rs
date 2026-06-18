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
        workspace_id: Uuid,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<Employer>, i64), AppError> {
        let auth_token = TenantAuthToken::system_override(workspace_id);
        self.repo.get_employers(&auth_token, page, limit).await
    }

    pub async fn get_employer(&self, workspace_id: Uuid, id: Uuid) -> Result<Employer, AppError> {
        let auth_token = TenantAuthToken::system_override(workspace_id);
        self.repo
            .get_employer_by_id(&auth_token, id)
            .await?
            .ok_or_else(|| AppError::not_found(I18nKey::EmployerNotFound))
    }

    pub async fn create_employer(
        &self,
        workspace_id: Uuid,
        actor_id: Uuid,
        data: CreateEmployerRequest,
    ) -> Result<Employer, AppError> {
        self.validate_parent_employer(workspace_id, None, data.parent_id)
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

        let auth_token = TenantAuthToken::system_override(workspace_id);
        let employer = self.repo.create_employer(&auth_token, data).await?;

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
        workspace_id: Uuid,
        id: Uuid,
        actor_id: Uuid,
        data: UpdateEmployerRequest,
    ) -> Result<Employer, AppError> {
        if let Some(parent_opt) = data.parent_id {
            self.validate_parent_employer(workspace_id, Some(id), parent_opt)
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

        let auth_token = TenantAuthToken::system_override(workspace_id);
        let employer = self.repo.update_employer(&auth_token, id, data).await?;

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
        workspace_id: Uuid,
        id: Uuid,
        actor_id: Uuid,
    ) -> Result<(), AppError> {
        let auth_token = TenantAuthToken::system_override(workspace_id);
        let existing = self.get_employer(workspace_id, id).await?;

        let employee_count = self
            .repo
            .count_employees_by_employer(&auth_token, id)
            .await?;

        if employee_count > 0 {
            return Err(AppError::validation_args(
                I18nKey::Validation,
                vec![("details", format!(
                    "Cannot delete employer because {employee_count} employees are still assigned to it."
                ))],
            ));
        }

        self.repo.delete_employer(&auth_token, id).await?;

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
        workspace_id: Uuid,
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
            .get_employer(workspace_id, parent_id)
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

                if let Ok(p) = self.get_employer(workspace_id, pid).await {
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
