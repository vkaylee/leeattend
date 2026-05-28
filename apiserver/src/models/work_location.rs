use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct WorkLocation {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub address: Option<String>,
    pub timezone: String,
    pub is_headquarters: bool,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct CreateWorkLocationRequest {
    pub name: String,
    pub address: Option<String>,
    pub timezone: Option<String>,
    pub is_headquarters: Option<bool>,
    pub status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct UpdateWorkLocationRequest {
    pub name: Option<String>,
    pub address: Option<String>,
    pub timezone: Option<String>,
    pub is_headquarters: Option<bool>,
    pub status: Option<String>,
}
