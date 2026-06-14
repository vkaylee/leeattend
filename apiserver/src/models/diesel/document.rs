use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::schema::{document_folders, file_acknowledgements, signature_requests};

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, AsChangeset)]
#[diesel(table_name = document_folders)]
pub struct DocumentFolderEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub parent_folder_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, AsChangeset)]
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

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, AsChangeset)]
#[diesel(table_name = file_acknowledgements)]
pub struct FileAcknowledgementEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub document_id: Uuid,
    pub employee_id: Uuid,
    pub status: String,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
