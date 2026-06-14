use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::models::signature::{SignatureRequest, SignatureStatus};
use crate::schema::signature_requests;

#[derive(Queryable, Insertable, Identifiable, AsChangeset, Debug, Clone)]
#[diesel(table_name = signature_requests)]
pub struct SignatureRequestEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub document_id: Uuid,
    pub signer_id: Uuid,
    pub status: String,
    pub signed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<SignatureRequest> for SignatureRequestEntity {
    fn from(domain: SignatureRequest) -> Self {
        Self {
            id: domain.id,
            workspace_id: domain.workspace_id,
            document_id: domain.document_id,
            signer_id: domain.signer_id,
            status: domain.status.as_ref().to_string(),
            signed_at: domain.signed_at,
            created_at: domain.created_at,
            updated_at: domain.updated_at,
        }
    }
}

impl From<SignatureRequestEntity> for SignatureRequest {
    fn from(entity: SignatureRequestEntity) -> Self {
        Self {
            id: entity.id,
            workspace_id: entity.workspace_id,
            document_id: entity.document_id,
            signer_id: entity.signer_id,
            status: entity.status.parse().unwrap_or(SignatureStatus::Pending),
            signed_at: entity.signed_at,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
        }
    }
}
