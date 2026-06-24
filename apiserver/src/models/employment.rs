use crate::models::custom_field::CustomFieldValueResponse;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct Employment {
    id: Uuid,
    workspace_id: Uuid,
    employee_id: Uuid,
    employer_id: Uuid,
    primary_location_id: Option<Uuid>,
    department_id: Option<Uuid>,
    status: String,
    start_date: NaiveDate,
    end_date: Option<NaiveDate>,
    job_title: Option<String>,
    employment_type: String,
    manager_employment_id: Option<Uuid>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub termination_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    custom_fields: Option<Vec<CustomFieldValueResponse>>,
}

impl Employment {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        workspace_id: Uuid,
        employee_id: Uuid,
        employer_id: Uuid,
        primary_location_id: Option<Uuid>,
        department_id: Option<Uuid>,
        status: String,
        start_date: NaiveDate,
        end_date: Option<NaiveDate>,
        job_title: Option<String>,
        employment_type: String,
        manager_employment_id: Option<Uuid>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        termination_reason: Option<String>,
        notes: Option<String>,
    ) -> Self {
        Self {
            id,
            workspace_id,
            employee_id,
            employer_id,
            primary_location_id,
            department_id,
            status,
            start_date,
            end_date,
            job_title,
            employment_type,
            manager_employment_id,
            created_at,
            updated_at,
            termination_reason,
            notes,
            custom_fields: None,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }
    pub fn workspace_id(&self) -> Uuid {
        self.workspace_id
    }
    pub fn custom_fields(&self) -> Option<&[CustomFieldValueResponse]> {
        self.custom_fields.as_deref()
    }
    #[must_use]
    pub fn with_custom_fields(mut self, custom_fields: Vec<CustomFieldValueResponse>) -> Self {
        self.custom_fields = Some(custom_fields);
        self
    }
    pub fn employee_id(&self) -> Uuid {
        self.employee_id
    }
    pub fn employer_id(&self) -> Uuid {
        self.employer_id
    }
    pub fn primary_location_id(&self) -> Option<Uuid> {
        self.primary_location_id
    }
    pub fn department_id(&self) -> Option<Uuid> {
        self.department_id
    }
    pub fn status(&self) -> &str {
        &self.status
    }
    pub fn start_date(&self) -> NaiveDate {
        self.start_date
    }
    pub fn end_date(&self) -> Option<NaiveDate> {
        self.end_date
    }
    pub fn job_title(&self) -> Option<&str> {
        self.job_title.as_deref()
    }
    pub fn employment_type(&self) -> &str {
        &self.employment_type
    }
    pub fn manager_employment_id(&self) -> Option<Uuid> {
        self.manager_employment_id
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
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
    pub job_title: Option<String>,
    pub employment_type: String,
    pub manager_employment_id: Option<Uuid>,
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
    pub job_title: Option<String>,
    pub employment_type: Option<String>,
    #[serde(
        default,
        deserialize_with = "crate::utils::serde_helpers::deserialize_option_option"
    )]
    pub manager_employment_id: Option<Option<Uuid>>,
    /// Reason for termination — required when status is set to "terminated"
    pub termination_reason: Option<String>,
    /// Free-text notes about the termination (optional)
    pub notes: Option<String>,
}
