//! ACK-sink abstraction: lets worker handlers acknowledge attendance log
//! ingestion outcomes back to the originating agent without coupling the
//! worker module directly to `AgentManager`.
//!
//! Additive 4-state contract: `accepted` / `processed` / `retryable_error` /
//! `permanent_error`. Old agents only understand `success`/`error`; the
//! agent-side decoder treats `processed` as `success` and `retryable_error`
//! as `error` for backward compatibility, so servers may freely emit the new
//! states without breaking older agent builds.

use std::sync::Arc;

pub use crate::agent_manager::AckStatus;

/// Sink that delivers an ACK for a given agent message back to the agent.
///
/// Implementations must not error on an offline agent — that is an expected,
/// recoverable condition (the agent will resend/replay on reconnect against
/// the durable delivery record), not a bug.
#[async_trait::async_trait]
pub trait AckSink: Send + Sync {
    /// Attempt to deliver an ACK. Returns `Err` only for unexpected
    /// serialization failures; an offline/unknown agent is reported via the
    /// `Ok(false)` "not delivered" case so callers can decide whether that is
    /// fatal (it normally is not — the durable record is authoritative).
    async fn ack(
        &self,
        agent_id: &str,
        msg_id: &str,
        status: AckStatus,
        error: Option<String>,
        attempt: Option<u32>,
    ) -> bool;
}

#[async_trait::async_trait]
impl AckSink for crate::agent_manager::AgentManager {
    async fn ack(
        &self,
        agent_id: &str,
        msg_id: &str,
        status: AckStatus,
        error: Option<String>,
        attempt: Option<u32>,
    ) -> bool {
        let msg = crate::agent_manager::CentralMessage::Ack {
            msg_id: msg_id.to_string(),
            status: status.as_str().to_string(),
            error,
            attempt,
            timestamp: Some(chrono::Utc::now().to_rfc3339()),
        };
        self.push_message(agent_id, msg).await.is_ok()
    }
}

/// Convenience type alias used by worker wiring.
pub type SharedAckSink = Arc<dyn AckSink>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_manager::AgentManager;

    #[tokio::test]
    async fn test_ack_status_wire_strings() {
        assert_eq!(AckStatus::Accepted.as_str(), "accepted");
        assert_eq!(AckStatus::Processed.as_str(), "processed");
        assert_eq!(AckStatus::RetryableError.as_str(), "retryable_error");
        assert_eq!(AckStatus::PermanentError.as_str(), "permanent_error");
    }

    #[tokio::test]
    async fn test_ack_offline_agent_returns_false_not_error() {
        let manager = AgentManager::new(None);
        let delivered = manager
            .ack("ghost_agent", "msg1", AckStatus::Processed, None, None)
            .await;
        assert!(!delivered);
    }

    #[tokio::test]
    async fn test_ack_online_agent_delivers_message() {
        use axum::extract::ws::Message;
        let manager = AgentManager::new(None);
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        manager.add_agent("agent1".to_string(), tx).await;

        let delivered = manager
            .ack("agent1", "msg1", AckStatus::Accepted, None, Some(1))
            .await;
        assert!(delivered);

        let msg = rx.recv().await.unwrap();
        if let Message::Text(text) = msg {
            assert!(text.contains("\"status\":\"accepted\""));
            assert!(text.contains("\"msg_id\":\"msg1\""));
        } else {
            panic!("expected text message");
        }
    }
}
