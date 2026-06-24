use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::models::employment::Employment;
use crate::schema::employments;

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, AsChangeset)]
#[diesel(table_name = employments)]
pub struct EmploymentEntity {
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
    pub deleted_at: Option<DateTime<Utc>>,
    pub job_title: Option<String>,
    pub employment_type: String,
    pub manager_employment_id: Option<Uuid>,
    pub notes: Option<String>,
    pub termination_reason: Option<String>,
}

impl From<EmploymentEntity> for Employment {
    fn from(val: EmploymentEntity) -> Self {
        Self::new(
            val.id,
            val.workspace_id,
            val.employee_id,
            val.employer_id,
            val.primary_location_id,
            val.department_id,
            val.status,
            val.start_date,
            val.end_date,
            val.job_title,
            val.employment_type,
            val.manager_employment_id,
            val.created_at,
            val.updated_at,
            val.termination_reason,
            val.notes,
        )
    }
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = employments)]
pub struct UpdateDieselEmployment {
    pub employer_id: Option<Uuid>,
    pub primary_location_id: Option<Option<Uuid>>,
    pub department_id: Option<Option<Uuid>>,
    pub status: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<Option<NaiveDate>>,
    pub job_title: Option<String>,
    pub employment_type: Option<String>,
    pub manager_employment_id: Option<Option<Uuid>>,
    pub notes: Option<Option<String>>,
    pub termination_reason: Option<Option<String>>,
    pub updated_at: Option<DateTime<Utc>>,
}
