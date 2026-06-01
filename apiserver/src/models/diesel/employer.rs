use crate::models::employer::Employer;
use crate::schema::employers;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(
    Debug,
    Clone,
    Queryable,
    Selectable,
    Insertable,
    Identifiable,
    AsChangeset,
    Serialize,
    Deserialize,
)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_employer_entity_from() {
        let id = Uuid::new_v4();
        let ws_id = Uuid::new_v4();
        let now = Utc::now();
        let entity = EmployerEntity {
            id,
            workspace_id: ws_id,
            name: "Test Employer".to_string(),
            tax_code: Some("123456".to_string()),
            created_at: now,
            updated_at: now,
        };

        let employer: Employer = entity.into();
        assert_eq!(employer.id, id);
        assert_eq!(employer.workspace_id, ws_id);
        assert_eq!(employer.name, "Test Employer");
        assert_eq!(employer.tax_code, Some("123456".to_string()));
        assert_eq!(employer.legal_address, None);
        assert_eq!(employer.created_at, now);
        assert_eq!(employer.updated_at, now);
    }
}
