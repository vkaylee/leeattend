use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::models::Employee;
use crate::schema::employees;
use diesel::sql_types::{Array, BigInt, Bool, Date, Nullable, Text, Timestamptz, Uuid as UuidType};

#[derive(QueryableByName)]
pub struct CountRow {
    #[diesel(sql_type = BigInt)]
    pub count: i64,
}

#[derive(Debug, diesel::QueryableByName)]
pub struct EmployeeRow {
    #[diesel(sql_type = UuidType)]
    pub id: Uuid,
    #[diesel(sql_type = UuidType)]
    pub workspace_id: Uuid,
    #[diesel(sql_type = Nullable<UuidType>)]
    pub user_id: Option<Uuid>,
    #[diesel(sql_type = Text)]
    pub full_name: String,
    #[diesel(sql_type = Text)]
    pub employee_id_number: String,
    #[diesel(sql_type = Date)]
    pub original_hire_date: chrono::NaiveDate,
    #[diesel(sql_type = Date)]
    pub hire_date: chrono::NaiveDate,
    #[diesel(sql_type = Nullable<Date>)]
    pub termination_date: Option<chrono::NaiveDate>,
    #[diesel(sql_type = Array<Text>)]
    pub tags: Vec<String>,
    #[diesel(sql_type = Bool)]
    pub is_active: bool,
    #[diesel(sql_type = Timestamptz)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[diesel(sql_type = Timestamptz)]
    pub updated_at: chrono::DateTime<chrono::Utc>,
    #[diesel(sql_type = Nullable<Text>)]
    pub department_name: Option<String>,
}

impl From<EmployeeRow> for Employee {
    fn from(row: EmployeeRow) -> Self {
        Self {
            id: row.id,
            workspace_id: row.workspace_id,
            user_id: row.user_id,
            full_name: row.full_name,
            employee_id_number: row.employee_id_number,
            original_hire_date: row.original_hire_date,
            hire_date: row.hire_date,
            termination_date: row.termination_date,
            tags: row.tags,
            is_active: row.is_active,
            created_at: row.created_at,
            updated_at: row.updated_at,
            department_name: row.department_name,
        }
    }
}

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
