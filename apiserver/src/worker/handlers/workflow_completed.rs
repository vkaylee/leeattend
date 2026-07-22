//! Workflow Completed Handler
//!
//! Processes workflow completion callbacks via outbox pattern.
//! When a workflow reaches a terminal state (approved/rejected/returned),
//! this handler invokes the appropriate callback for the entity type.

use crate::models::WorkflowCompletedPayload;
use crate::services::approval_workflow::WorkflowCallbackHandler;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn};

pub struct WorkflowCompletedHandler {
    handlers: HashMap<String, Arc<dyn WorkflowCallbackHandler>>,
}

impl WorkflowCompletedHandler {
    pub fn new(handlers: HashMap<String, Arc<dyn WorkflowCallbackHandler>>) -> Self {
        Self { handlers }
    }

    /// Handle a workflow completed job.
    /// Deserializes the payload and invokes the appropriate callback handler.
    pub async fn handle(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        payload: &WorkflowCompletedPayload,
    ) -> Result<(), String> {
        info!(
            entity_type = %payload.entity_type,
            entity_id = %payload.entity_id,
            final_status = %payload.final_status,
            actor_user_id = %payload.actor_user_id,
            "Processing workflow completed callback"
        );

        // Find the appropriate handler for this entity type
        match self.handlers.get(&payload.entity_type) {
            Some(handler) => {
                handler
                    .on_workflow_completed(
                        auth_token,
                        payload.entity_id,
                        &payload.final_status,
                        payload.actor_user_id,
                    )
                    .await
                    .map_err(|e| format!("Callback handler failed: {e}"))?;

                info!(
                    entity_type = %payload.entity_type,
                    entity_id = %payload.entity_id,
                    "Workflow completion callback executed successfully"
                );
            }
            None => {
                warn!(
                    entity_type = %payload.entity_type,
                    "No callback handler registered for entity type"
                );
                // Not an error - some entity types may not need callbacks
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diesel_pool::TenantAuthToken;
    use crate::errors::AppError;
    use crate::services::approval_workflow::WorkflowCallbackHandler;
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicBool, Ordering};
    use uuid::Uuid;

    struct MockCallbackHandler {
        called: Arc<AtomicBool>,
    }

    #[async_trait]
    impl WorkflowCallbackHandler for MockCallbackHandler {
        async fn on_workflow_completed(
            &self,
            _auth_token: &TenantAuthToken,
            _entity_id: Uuid,
            _status: &str,
            _actor_user_id: Uuid,
        ) -> Result<(), AppError> {
            self.called.store(true, Ordering::SeqCst);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_workflow_completed_handler_invokes_callback() {
        let called = Arc::new(AtomicBool::new(false));
        let handler = MockCallbackHandler {
            called: called.clone(),
        };

        let mut handlers = HashMap::new();
        handlers.insert(
            "leave_request".to_string(),
            Arc::new(handler) as Arc<dyn WorkflowCallbackHandler>,
        );

        let workflow_handler = WorkflowCompletedHandler::new(handlers);

        let payload = WorkflowCompletedPayload {
            entity_type: "leave_request".to_string(),
            entity_id: Uuid::new_v4(),
            final_status: "approved".to_string(),
            actor_user_id: Uuid::new_v4(),
            trace_context: None,
        };

        let ws_id = Uuid::new_v4();
        let auth_token = TenantAuthToken::system_override(ws_id);

        let result = workflow_handler.handle(&auth_token, &payload).await;
        assert!(result.is_ok());
        assert!(called.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_workflow_completed_handler_no_handler() {
        let handlers = HashMap::new();
        let workflow_handler = WorkflowCompletedHandler::new(handlers);

        let payload = WorkflowCompletedPayload {
            entity_type: "unknown_type".to_string(),
            entity_id: Uuid::new_v4(),
            final_status: "approved".to_string(),
            actor_user_id: Uuid::new_v4(),
            trace_context: None,
        };

        let ws_id = Uuid::new_v4();
        let auth_token = TenantAuthToken::system_override(ws_id);

        let result = workflow_handler.handle(&auth_token, &payload).await;
        assert!(result.is_ok()); // Should not error, just warn
    }
}
