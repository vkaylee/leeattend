use async_trait::async_trait;
use bytes::Bytes;
use std::collections::HashMap;

use crate::errors::AppResult;
use crate::models::econtract::{RequestEnvelope, WebhookEvent};

#[async_trait]
pub trait EContractProvider: Send + Sync {
    /// Uploads documents and creates the external envelope.
    async fn create_envelope(&self, envelope: RequestEnvelope) -> AppResult<String>;

    /// Retrieves a secure, single-use URL for a specific signer (Embedded flow).
    /// Returns `Ok(None)` if the provider only supports Remote Signing (Email/SMS).
    async fn get_signing_url(
        &self,
        envelope_id: &str,
        signer_id: &str,
        redirect_url: &str,
    ) -> AppResult<Option<String>>;

    /// Verifies and parses incoming webhook notifications securely.
    async fn parse_webhook(
        &self,
        headers: &HashMap<String, String>,
        body: &[u8],
    ) -> AppResult<WebhookEvent>;

    /// Downloads the signed PDF from the provider.
    async fn download_signed_document(&self, envelope_id: &str) -> AppResult<Bytes>;
}
