use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct Employer {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub tax_code: Option<String>,
    pub legal_address: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateEmployerRequest {
    pub name: String,
    pub tax_code: Option<String>,
    pub legal_address: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateEmployerRequest {
    pub name: Option<String>,
    pub tax_code: Option<String>,
    pub legal_address: Option<String>,
}
