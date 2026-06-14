use async_trait::async_trait;
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::diesel_pool::{DieselPools, TenantAuthToken};
use crate::errors::{AppError, AppResult};
use crate::models::diesel::signature::SignatureRequestEntity;
use crate::models::signature::SignatureRequest;
use crate::repositories::signature_repository::SignatureRepository;
use crate::schema::signature_requests;

pub struct DieselSignatureRepository {
    diesel_pools: std::sync::Arc<DieselPools>,
}

impl DieselSignatureRepository {
    pub fn new(diesel_pools: std::sync::Arc<DieselPools>) -> Self {
        Self { diesel_pools }
    }
}

#[async_trait]
impl SignatureRepository for DieselSignatureRepository {
    async fn get_requests_by_document(
        &self,
        auth_token: &TenantAuthToken,
        document_id: Uuid,
    ) -> AppResult<Vec<SignatureRequest>> {
        let mut conn = self.diesel_pools.tenant_session_conn(auth_token).await?;

        let entities = signature_requests::table
            .filter(signature_requests::document_id.eq(document_id))
            .filter(signature_requests::workspace_id.eq(auth_token.workspace_id()))
            .load::<SignatureRequestEntity>(&mut conn)
            .await?;

        Ok(entities.into_iter().map(SignatureRequest::from).collect())
    }

    async fn get_requests_by_signer(
        &self,
        auth_token: &TenantAuthToken,
        signer_id: Uuid,
    ) -> AppResult<Vec<SignatureRequest>> {
        let mut conn = self.diesel_pools.tenant_session_conn(auth_token).await?;

        let entities = signature_requests::table
            .filter(signature_requests::signer_id.eq(signer_id))
            .filter(signature_requests::workspace_id.eq(auth_token.workspace_id()))
            .load::<SignatureRequestEntity>(&mut conn)
            .await?;

        Ok(entities.into_iter().map(SignatureRequest::from).collect())
    }

    async fn get_signature_request(
        &self,
        auth_token: &TenantAuthToken,
        request_id: Uuid,
    ) -> AppResult<Option<SignatureRequest>> {
        let mut conn = self.diesel_pools.tenant_session_conn(auth_token).await?;

        let entity = signature_requests::table
            .filter(signature_requests::id.eq(request_id))
            .filter(signature_requests::workspace_id.eq(auth_token.workspace_id()))
            .first::<SignatureRequestEntity>(&mut conn)
            .await
            .optional()?;

        Ok(entity.map(SignatureRequest::from))
    }

    async fn create_signature_request(
        &self,
        auth_token: &TenantAuthToken,
        request: &SignatureRequest,
    ) -> AppResult<()> {
        let mut conn = self.diesel_pools.tenant_session_conn(auth_token).await?;
        let mut entity = SignatureRequestEntity::from(request.clone());
        entity.workspace_id = auth_token.workspace_id();

        diesel::insert_into(signature_requests::table)
            .values(&entity)
            .execute(&mut conn)
            .await?;

        Ok(())
    }

    async fn update_signature_status(
        &self,
        auth_token: &TenantAuthToken,
        request_id: Uuid,
        status: &str,
        signed_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> AppResult<SignatureRequest> {
        let mut conn = self.diesel_pools.tenant_session_conn(auth_token).await?;

        let entity = diesel::update(
            signature_requests::table
                .filter(signature_requests::id.eq(request_id))
                .filter(signature_requests::workspace_id.eq(auth_token.workspace_id())),
        )
        .set((
            signature_requests::status.eq(status),
            signature_requests::signed_at.eq(signed_at),
            signature_requests::updated_at.eq(Utc::now()),
        ))
        .get_result::<SignatureRequestEntity>(&mut conn)
        .await
        .map_err(|e| match e {
            diesel::result::Error::NotFound => {
                AppError::not_found_raw(format!("Signature request {request_id} not found"))
            }
            _ => AppError::from(e),
        })?;

        Ok(SignatureRequest::from(entity))
    }
}
