use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::models::{EmployeePhase, EmploymentPhaseDefinition};
use crate::schema::{employee_phases, employment_phase_definitions};

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable)]
#[diesel(table_name = employment_phase_definitions)]
pub struct EmploymentPhaseDefinitionEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub phase_type: String,
    pub rule_config: serde_json::Value,
    pub is_active: bool,
    pub display_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<EmploymentPhaseDefinitionEntity> for EmploymentPhaseDefinition {
    fn from(val: EmploymentPhaseDefinitionEntity) -> Self {
        EmploymentPhaseDefinition {
            id: val.id,
            workspace_id: val.workspace_id,
            name: val.name,
            description: val.description,
            phase_type: val.phase_type,
            rule_config: val.rule_config,
            is_active: val.is_active,
            display_order: val.display_order,
            created_at: val.created_at,
            updated_at: val.updated_at,
        }
    }
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, AsChangeset)]
#[diesel(table_name = employee_phases)]
pub struct EmployeePhaseEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub phase_definition_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub status: String,
    pub override_config: Option<serde_json::Value>,
    pub evaluation_notes: Option<String>,
    pub evaluated_by: Option<Uuid>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub employment_id: Uuid,
}

impl From<EmployeePhaseEntity> for EmployeePhase {
    fn from(val: EmployeePhaseEntity) -> Self {
        EmployeePhase {
            id: val.id,
            workspace_id: val.workspace_id,
            employment_id: val.employment_id,
            phase_definition_id: val.phase_definition_id,
            start_date: val.start_date,
            end_date: val.end_date,
            status: val.status,
            override_config: val.override_config,
            evaluation_notes: val.evaluation_notes,
            evaluated_by: val.evaluated_by,
            completed_at: val.completed_at,
            created_at: val.created_at,
            updated_at: val.updated_at,
        }
    }
}
