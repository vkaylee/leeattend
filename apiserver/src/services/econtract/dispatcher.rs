use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::services::econtract::mock::MockEContractProvider;
use crate::services::econtract::provider::EContractProvider;

pub struct EContractDispatcher {
    providers: HashMap<String, Arc<dyn EContractProvider>>,
}

impl EContractDispatcher {
    pub fn new() -> Self {
        let mut providers: HashMap<String, Arc<dyn EContractProvider>> = HashMap::new();
        // Register mock provider
        providers.insert("mock".to_string(), Arc::new(MockEContractProvider));

        Self { providers }
    }

    /// Resolves the appropriate e-Contract provider based on database routing rules.
    #[allow(clippy::unused_async)]
    pub async fn resolve_provider(
        &self,
        employer_id: Uuid,
        signer_country: &str,
        preferred_auth_type: &str,
    ) -> AppResult<Arc<dyn EContractProvider>> {
        tracing::debug!(
            "Resolving e-Contract provider for employer {} (country: {}, auth: {})",
            employer_id,
            signer_country,
            preferred_auth_type
        );

        // In production:
        // 1. Fetch active routing rules for the employer from the DB.
        // 2. Select the rule with the highest priority matching the country & auth type.
        // 3. Resolve the provider instance (FPT, Viettel, DocuSign) using the rule's target provider.

        // For development/mock verification:
        self.providers.get("mock").cloned().ok_or_else(|| {
            AppError::validation_args(
                crate::utils::i18n::I18nKey::Validation,
                vec![(
                    "details",
                    "Mock e-Contract provider not found in registry".to_string(),
                )],
            )
        })
    }
}

impl Default for EContractDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dispatcher_resolve_mock_provider() {
        let dispatcher = EContractDispatcher::new();
        let employer_id = Uuid::new_v4();

        let result = dispatcher
            .resolve_provider(employer_id, "VN", "sms_otp")
            .await;

        assert!(result.is_ok());
        let provider = result.unwrap();

        // 1. Test create_envelope
        let envelope = crate::models::econtract::RequestEnvelope::new(
            "Test Contract".to_string(),
            vec![],
            vec![],
            "https://redirect.com".to_string(),
        );
        let env_id = provider.create_envelope(envelope).await;
        assert!(env_id.is_ok());
        let env_id_str = env_id.unwrap();
        assert!(env_id_str.starts_with("mock-env-"));

        // 2. Test get_signing_url
        let url_res = provider
            .get_signing_url(&env_id_str, "signer-1", "https://redirect.com")
            .await;
        assert!(url_res.is_ok());
        let url_opt = url_res.unwrap();
        assert!(url_opt.is_some());
        assert_eq!(
            url_opt.unwrap(),
            format!("https://mock-provider.com/sign?env={env_id_str}&signer=signer-1&redirect=https://redirect.com")
        );

        // 3. Test parse_webhook
        let headers = HashMap::new();
        let body = b"{\"envelope_id\":\"mock-env-123\",\"status\":\"completed\"}";
        let webhook_res = provider.parse_webhook(&headers, body).await;
        assert!(webhook_res.is_ok());
        let webhook_event = webhook_res.unwrap();
        assert_eq!(webhook_event.envelope_id(), "mock-env-123");
        assert_eq!(webhook_event.status(), "completed");

        // 4. Test download_signed_document
        let doc_res = provider.download_signed_document(&env_id_str).await;
        assert!(doc_res.is_ok());
        assert_eq!(
            doc_res.unwrap(),
            bytes::Bytes::from_static(b"%PDF-1.4 Mock signed document content")
        );
    }
}
