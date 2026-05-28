use crate::models::work_location::WorkLocation;
use crate::schema::work_locations;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = work_locations)]
pub struct WorkLocationEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub address: Option<String>,
    pub timezone: String,
    pub is_headquarters: bool,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<WorkLocationEntity> for WorkLocation {
    fn from(entity: WorkLocationEntity) -> Self {
        Self {
            id: entity.id,
            workspace_id: entity.workspace_id,
            name: entity.name,
            address: entity.address,
            timezone: entity.timezone,
            is_headquarters: entity.is_headquarters,
            status: entity.status,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
        }
    }
}
