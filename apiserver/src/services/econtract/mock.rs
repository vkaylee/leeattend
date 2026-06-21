use async_trait::async_trait;
use bytes::Bytes;
use std::collections::HashMap;

use crate::errors::{AppError, AppResult};
use crate::models::econtract::{RequestEnvelope, WebhookEvent};
use crate::services::econtract::provider::EContractProvider;

pub struct MockEContractProvider;

#[async_trait]
impl EContractProvider for MockEContractProvider {
    async fn create_envelope(&self, envelope: RequestEnvelope) -> AppResult<String> {
        let mock_id = format!("mock-env-{}", uuid::Uuid::new_v4());
        tracing::info!(
            "Mock eContract: Created envelope {} with title '{}'",
            mock_id,
            envelope.title()
        );
        Ok(mock_id)
    }

    async fn get_signing_url(
        &self,
        envelope_id: &str,
        signer_id: &str,
        redirect_url: &str,
    ) -> AppResult<Option<String>> {
        let mock_url = format!(
            "https://mock-provider.com/sign?env={envelope_id}&signer={signer_id}&redirect={redirect_url}"
        );
        Ok(Some(mock_url))
    }

    async fn parse_webhook(
        &self,
        _headers: &HashMap<String, String>,
        body: &[u8],
    ) -> AppResult<WebhookEvent> {
        let event: WebhookEvent = serde_json::from_slice(body).map_err(|e| {
            AppError::validation_args(
                crate::utils::i18n::I18nKey::Validation,
                vec![("details", format!("Failed to parse webhook JSON: {e}"))],
            )
        })?;
        Ok(event)
    }

    async fn download_signed_document(&self, envelope_id: &str) -> AppResult<Bytes> {
        tracing::info!(
            "Mock eContract: Downloading completed document {}",
            envelope_id
        );
        Ok(Bytes::from_static(b"%PDF-1.4 Mock signed document content"))
    }
}
