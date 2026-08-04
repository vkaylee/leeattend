use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkflowCompletedPayload {
    pub entity_type: String,
    pub entity_id: Uuid,
    pub final_status: String,
    pub actor_user_id: Uuid,
    #[serde(rename = "_trace_context")]
    pub trace_context: Option<serde_json::Value>,
}

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

pub struct NewOutboxJobParams {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub job_type: String,
    pub payload: serde_json::Value,
    pub status: String,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl OutboxJob {
    pub fn new(params: NewOutboxJobParams) -> Self {
        Self {
            id: params.id,
            workspace_id: params.workspace_id,
            job_type: params.job_type,
            payload: params.payload,
            status: params.status,
            error_message: params.error_message,
            created_at: params.created_at,
            updated_at: params.updated_at,
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

    pub fn trace_context(&self) -> Option<&str> {
        self.payload.get("_trace_context").and_then(|v| v.as_str())
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
