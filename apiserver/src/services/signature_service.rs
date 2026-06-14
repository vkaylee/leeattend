use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

use crate::diesel_pool::TenantAuthToken;
use crate::errors::{AppError, AppResult};
use crate::models::signature::{
    CreateSignatureRequestPayload, SignDocumentPayload, SignatureRequest, SignatureStatus,
};
use crate::repositories::signature_repository::SignatureRepository;
use crate::services::storage::FileService;

pub struct SignatureService {
    signature_repository: Arc<dyn SignatureRepository>,
    #[allow(dead_code)]
    file_service: Arc<FileService>,
}

impl SignatureService {
    pub fn new(
        signature_repository: Arc<dyn SignatureRepository>,
        file_service: Arc<FileService>,
    ) -> Self {
        Self {
            signature_repository,
            file_service,
        }
    }

    /// Create a new signature request for a document
    pub async fn create_request(
        &self,
        auth_token: &TenantAuthToken,
        payload: CreateSignatureRequestPayload,
    ) -> AppResult<SignatureRequest> {
        // Verification that document exists could go here

        let request = SignatureRequest {
            id: Uuid::new_v4(),
            workspace_id: auth_token.workspace_id(),
            document_id: payload.document_id,
            signer_id: payload.signer_id,
            status: SignatureStatus::Pending,
            signed_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.signature_repository
            .create_signature_request(auth_token, &request)
            .await?;

        Ok(request)
    }

    /// Process a signature submission
    pub async fn sign_document(
        &self,
        auth_token: &TenantAuthToken,
        request_id: Uuid,
        payload: SignDocumentPayload,
    ) -> AppResult<SignatureRequest> {
        let request = self
            .signature_repository
            .get_signature_request(auth_token, request_id)
            .await?
            .ok_or_else(|| AppError::not_found_raw("Signature request not found"))?;

        if request.workspace_id != auth_token.workspace_id() {
            return Err(AppError::forbidden_raw(
                "Request does not belong to your workspace",
            ));
        }

        if request.status != SignatureStatus::Pending {
            return Err(AppError::validation_raw("Signature request is not pending"));
        }

        // Ideally, we decode payload.signature_image_base64 and upload it via FileService
        // We'll skip the actual upload in this mocked version for brevity,
        // but normally: self.file_service.upload_file(..., payload.signature_image_base64)

        // Save audit info and update status
        tracing::info!(
            "Document {} signed by {}. IP: {:?}, Agent: {:?}",
            request.document_id,
            request.signer_id,
            payload.ip_address,
            payload.user_agent
        );

        let signed_request = self
            .signature_repository
            .update_signature_status(
                auth_token,
                request.id,
                SignatureStatus::Signed.as_ref(),
                Some(Utc::now()),
            )
            .await?;

        Ok(signed_request)
    }

    pub async fn get_pending_requests_for_signer(
        &self,
        auth_token: &TenantAuthToken,
        signer_id: Uuid,
    ) -> AppResult<Vec<SignatureRequest>> {
        let requests = self
            .signature_repository
            .get_requests_by_signer(auth_token, signer_id)
            .await?;
        let pending = requests
            .into_iter()
            .filter(|r| r.status == SignatureStatus::Pending)
            .collect();
        Ok(pending)
    }
}
