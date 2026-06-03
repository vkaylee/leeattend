use crate::models::OtClaimBehalfAssignment;
use crate::schema::ot_claim_behalf_assignments;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Debug, Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = ot_claim_behalf_assignments)]
pub struct OtClaimBehalfAssignmentEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub assignee_employee_id: Uuid,
    pub target_employee_id: Option<Uuid>,
    pub target_department_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<OtClaimBehalfAssignmentEntity> for OtClaimBehalfAssignment {
    fn from(entity: OtClaimBehalfAssignmentEntity) -> Self {
        Self {
            id: entity.id,
            workspace_id: entity.workspace_id,
            assignee_employee_id: entity.assignee_employee_id,
            target_employee_id: entity.target_employee_id,
            target_department_id: entity.target_department_id,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
        }
    }
}
