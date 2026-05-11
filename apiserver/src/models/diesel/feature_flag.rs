use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::models::feature_flag::FeatureFlag;
use crate::schema::feature_flags;

#[derive(Debug, Queryable, Selectable, Identifiable, Insertable)]
#[diesel(table_name = feature_flags)]
pub struct FeatureFlagEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub flag_key: String,
    pub enabled: bool,
    pub description: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<FeatureFlagEntity> for FeatureFlag {
    fn from(e: FeatureFlagEntity) -> Self {
        Self {
            id: e.id,
            workspace_id: e.workspace_id,
            flag_key: e.flag_key,
            enabled: e.enabled,
            description: e.description,
            metadata: e.metadata,
            created_at: e.created_at,
            updated_at: e.updated_at,
        }
    }
}
