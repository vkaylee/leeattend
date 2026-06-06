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
    pub assignee_user_id: Uuid,
    pub target_employee_id: Option<Uuid>,
    pub target_department_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub assignment_type: String,
}

impl From<BehalfAssignmentEntity> for BehalfAssignment {
    fn from(entity: BehalfAssignmentEntity) -> Self {
        Self::new(
            entity.id,
            entity.workspace_id,
            entity.assignee_user_id,
            entity.target_employee_id,
            entity.target_department_id,
            entity.assignment_type,
            entity.created_at,
            entity.updated_at,
        )
    }
}
