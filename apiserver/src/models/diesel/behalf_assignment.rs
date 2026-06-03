use crate::models::BehalfAssignment;
use crate::schema::behalf_assignments;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Debug, Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = behalf_assignments)]
pub struct BehalfAssignmentEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub assignee_employee_id: Uuid,
    pub target_employee_id: Option<Uuid>,
    pub target_department_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub assignment_type: String,
}

impl From<BehalfAssignmentEntity> for BehalfAssignment {
    fn from(entity: BehalfAssignmentEntity) -> Self {
        Self {
            id: entity.id,
            workspace_id: entity.workspace_id,
            assignee_employee_id: entity.assignee_employee_id,
            target_employee_id: entity.target_employee_id,
            target_department_id: entity.target_department_id,
            assignment_type: entity.assignment_type,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
        }
    }
}
