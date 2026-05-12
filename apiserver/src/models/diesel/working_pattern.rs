use crate::models::{EmployeeWorkingPattern, WorkingPattern, WorkingPatternDetail};
use crate::schema::{employee_working_patterns, working_pattern_details, working_patterns};
use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Debug, Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = working_patterns)]
pub struct WorkingPatternEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<WorkingPatternEntity> for WorkingPattern {
    fn from(entity: WorkingPatternEntity) -> Self {
        Self {
            id: entity.id,
            workspace_id: entity.workspace_id,
            name: entity.name,
            description: entity.description,
            is_active: entity.is_active,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
        }
    }
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = working_pattern_details)]
pub struct WorkingPatternDetailEntity {
    pub id: Uuid,
    pub pattern_id: Uuid,
    pub day_of_week: i32,
    pub shift_id: Option<Uuid>,
}

impl From<WorkingPatternDetailEntity> for WorkingPatternDetail {
    fn from(entity: WorkingPatternDetailEntity) -> Self {
        Self {
            id: entity.id,
            pattern_id: entity.pattern_id,
            day_of_week: entity.day_of_week,
            shift_id: entity.shift_id,
        }
    }
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = employee_working_patterns)]
pub struct EmployeeWorkingPatternEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub employee_id: Uuid,
    pub pattern_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<EmployeeWorkingPatternEntity> for EmployeeWorkingPattern {
    fn from(entity: EmployeeWorkingPatternEntity) -> Self {
        Self {
            id: entity.id,
            workspace_id: entity.workspace_id,
            employee_id: entity.employee_id,
            pattern_id: entity.pattern_id,
            start_date: entity.start_date,
            end_date: entity.end_date,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
        }
    }
}
