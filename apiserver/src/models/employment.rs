use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct Employment {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub employee_id: Uuid,
    pub employer_id: Uuid,
    pub primary_location_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
    pub status: String,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreateEmploymentRequest {
    pub employee_id: Uuid,
    pub employer_id: Uuid,
    pub primary_location_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
    pub status: String,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct UpdateEmploymentRequest {
    pub employer_id: Option<Uuid>,
    #[serde(
        default,
        deserialize_with = "crate::utils::serde_helpers::deserialize_option_option"
    )]
    pub primary_location_id: Option<Option<Uuid>>,
    #[serde(
        default,
        deserialize_with = "crate::utils::serde_helpers::deserialize_option_option"
    )]
    pub department_id: Option<Option<Uuid>>,
    pub status: Option<String>,
    pub start_date: Option<NaiveDate>,
    #[serde(
        default,
        deserialize_with = "crate::utils::serde_helpers::deserialize_option_option"
    )]
    pub end_date: Option<Option<NaiveDate>>,
}
