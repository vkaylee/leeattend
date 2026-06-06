use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct OutboxJob {
    id: Uuid,
    workspace_id: Uuid,
    job_type: String,
    payload: serde_json::Value,
    status: String,
    error_message: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl OutboxJob {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        workspace_id: Uuid,
        job_type: String,
        payload: serde_json::Value,
        status: String,
        error_message: Option<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            workspace_id,
            job_type,
            payload,
            status,
            error_message,
            created_at,
            updated_at,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn workspace_id(&self) -> Uuid {
        self.workspace_id
    }

    pub fn job_type(&self) -> &str {
        &self.job_type
    }

    pub fn payload(&self) -> &serde_json::Value {
        &self.payload
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn error_message(&self) -> Option<&str> {
        self.error_message.as_deref()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}
