use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable, ToSchema)]
#[diesel(table_name = crate::schema::storage_configs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct StorageConfig {
    pub id: Uuid,
    pub workspace_id: Uuid,

    pub endpoint_url: String,
    pub region: String,
    pub bucket_name: String,
    pub path_prefix: Option<String>,

    pub access_key_enc: String,
    pub secret_key_enc: String,
    pub encryption_key_version: String,

    pub use_path_style: bool,
    pub presigned_ttl_secs: i32,
    pub max_file_size: i64,

    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, AsChangeset, ToSchema)]
#[diesel(table_name = crate::schema::storage_configs)]
pub struct UpdateStorageConfig {
    pub endpoint_url: Option<String>,
    pub region: Option<String>,
    pub bucket_name: Option<String>,
    pub path_prefix: Option<Option<String>>,

    pub access_key_enc: Option<String>,
    pub secret_key_enc: Option<String>,
    pub encryption_key_version: Option<String>,

    pub use_path_style: Option<bool>,
    pub presigned_ttl_secs: Option<i32>,
    pub max_file_size: Option<i64>,
    pub is_active: Option<bool>,
}
