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
    pub parent_id: Option<Uuid>,
    pub relationship_type: Option<String>,
    pub business_type: Option<String>,
    pub registration_number: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub incorporation_date: Option<chrono::NaiveDate>,
    pub country_code: Option<String>,
    pub currency_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreateEmployerRequest {
    pub name: String,
    pub tax_code: Option<String>,
    pub legal_address: Option<String>,
    pub parent_id: Option<Uuid>,
    pub relationship_type: Option<String>,
    pub business_type: Option<String>,
    pub registration_number: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub incorporation_date: Option<chrono::NaiveDate>,
    pub country_code: Option<String>,
    pub currency_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct UpdateEmployerRequest {
    pub name: Option<String>,
    pub tax_code: Option<String>,
    pub legal_address: Option<String>,
    #[serde(
        default,
        deserialize_with = "crate::utils::serde_helpers::deserialize_option_option"
    )]
    pub parent_id: Option<Option<Uuid>>,
    #[serde(
        default,
        deserialize_with = "crate::utils::serde_helpers::deserialize_option_option"
    )]
    pub relationship_type: Option<Option<String>>,
    pub business_type: Option<String>,
    pub registration_number: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub incorporation_date: Option<chrono::NaiveDate>,
    pub country_code: Option<String>,
    pub currency_code: Option<String>,
}
