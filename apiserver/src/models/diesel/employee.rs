use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::models::Employee;
use crate::schema::employees;

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, AsChangeset)]
#[diesel(table_name = employees)]
pub struct EmployeeEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub user_id: Option<Uuid>,
    pub employee_id_number: String,
    pub is_active: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub tags: Option<Vec<Option<String>>>,
    pub full_name: String,
    pub recruitment_source_id: Option<Uuid>,
    pub hire_date: NaiveDate,
    pub termination_date: Option<NaiveDate>,
    pub original_hire_date: NaiveDate,
}

impl From<EmployeeEntity> for Employee {
    fn from(val: EmployeeEntity) -> Self {
        Employee {
            id: val.id,
            workspace_id: val.workspace_id,
            user_id: val.user_id,
            full_name: val.full_name,
            employee_id_number: val.employee_id_number,
            original_hire_date: val.original_hire_date,
            hire_date: val.hire_date,
            termination_date: val.termination_date,
            tags: val.tags.unwrap_or_default().into_iter().flatten().collect(),
            is_active: val.is_active,
            created_at: val.created_at.unwrap_or_else(Utc::now),
            updated_at: val.updated_at.unwrap_or_else(Utc::now),
            department_name: None,
        }
    }
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = employees)]
pub struct UpdateDieselEmployee {
    pub user_id: Option<Uuid>,
    pub full_name: Option<String>,
    pub employee_id_number: Option<String>,
    pub tags: Option<Vec<Option<String>>>,
    pub is_active: Option<bool>,
    pub hire_date: Option<NaiveDate>,
    pub termination_date: Option<Option<NaiveDate>>,
    pub updated_at: Option<DateTime<Utc>>,
}
