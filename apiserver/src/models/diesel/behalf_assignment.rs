use crate::models::{BehalfAssignment, NewBehalfAssignmentParams};
use crate::schema::behalf_assignments;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Debug, Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = behalf_assignments)]
pub struct BehalfAssignmentEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub target_employee_id: Option<Uuid>,
    pub target_department_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub assignment_type: String,
    pub assignee_user_id: Uuid,
    pub employer_id: Option<Uuid>,
}

impl From<BehalfAssignmentEntity> for BehalfAssignment {
    fn from(entity: BehalfAssignmentEntity) -> Self {
        Self::new(NewBehalfAssignmentParams {
            id: entity.id,
            workspace_id: entity.workspace_id,
            assignee_user_id: entity.assignee_user_id,
            target_employee_id: entity.target_employee_id,
            target_department_id: entity.target_department_id,
            assignment_type: entity.assignment_type,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
            employer_id: entity.employer_id,
        })
    }
}
