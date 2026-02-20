use sqlx::PgPool;
use tracing::{error, instrument};

use crate::queue::ReportJob;

pub struct HeartbeatHandler;

impl HeartbeatHandler {
    #[instrument(skip(db, job), fields(job_id = %job.job_id))]
    pub async fn handle(db: &PgPool, job: &ReportJob) -> Result<(), String> {
        let agent_id = match &job.agent_id {
            Some(id) => id,
            None => {
                error!("HeartbeatSync job missing agent_id: {:?}", job.job_id);
                return Err("Missing agent_id".to_string());
            }
        };

        let result = sqlx::query!(
            r#"
            UPDATE authorized_agents 
            SET last_seen_at = NOW() 
            WHERE id = $1
            "#,
            agent_id
        )
        .execute(db)
        .await;

        match result {
            Ok(res) => {
                if res.rows_affected() == 0 {
                    // Agent might have been deleted, technically not an error that needs to crash the worker
                    // but we log it for awareness context
                    tracing::debug!("Heartbeat ignored: Agent {} not found or already deleted", agent_id);
                }
                Ok(())
            }
            Err(e) => {
                error!("Failed to update database for heartbeat (agent {}): {}", agent_id, e);
                Err(e.to_string())
            }
        }
    }
}
