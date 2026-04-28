use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SalaryBaseGroup {
    pub id: Uuid,
    pub workspace_id: Option<Uuid>,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub is_system: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SalaryBaseGroupMember {
    pub component_definition_id: Uuid,
    pub base_group_id: Uuid,
    pub created_at: DateTime<Utc>,
}
