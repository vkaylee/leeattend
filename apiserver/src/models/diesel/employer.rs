use crate::models::employer::Employer;
use crate::schema::employers;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = employers)]
pub struct EmployerEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub tax_code: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<EmployerEntity> for Employer {
    fn from(entity: EmployerEntity) -> Self {
        Self {
            id: entity.id,
            workspace_id: entity.workspace_id,
            name: entity.name,
            tax_code: entity.tax_code,
            legal_address: None,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
        }
    }
}
