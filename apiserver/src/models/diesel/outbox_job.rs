use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::outbox_jobs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OutboxJobEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub job_type: String,
    pub payload: serde_json::Value,
    pub status: String,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<OutboxJobEntity> for crate::models::OutboxJob {
    fn from(entity: OutboxJobEntity) -> Self {
        Self {
            id: entity.id,
            workspace_id: entity.workspace_id,
            job_type: entity.job_type,
            payload: entity.payload,
            status: entity.status,
            error_message: entity.error_message,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::OutboxJob;

    #[test]
    fn test_outbox_job_entity_from() {
        let id = Uuid::new_v4();
        let ws_id = Uuid::new_v4();
        let now = Utc::now();
        let entity = OutboxJobEntity {
            id,
            workspace_id: ws_id,
            job_type: "report".to_string(),
            payload: serde_json::json!({"some": "data"}),
            status: "pending".to_string(),
            error_message: None,
            created_at: now,
            updated_at: now,
        };

        let domain: OutboxJob = entity.into();
        assert_eq!(domain.id, id);
        assert_eq!(domain.workspace_id, ws_id);
        assert_eq!(domain.job_type, "report");
        assert_eq!(domain.status, "pending");
    }
}
