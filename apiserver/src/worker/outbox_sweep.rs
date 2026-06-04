use std::sync::Arc;
use tokio::time::{interval, Duration};
use tokio_util::sync::CancellationToken;
use tracing::{error, info};
use uuid::Uuid;

use crate::repositories::workspace_repository::WorkspaceRepository;
use crate::services::report_coordinator::CreateJobRequest;
use crate::services::OutboxJobService;
use crate::services::ReportCoordinator;

pub struct OutboxSweepScheduler {
    workspace_repo: Arc<dyn WorkspaceRepository>,
    outbox_service: Arc<OutboxJobService>,
    coordinator: Arc<ReportCoordinator>,
}

impl OutboxSweepScheduler {
    pub fn new(
        workspace_repo: Arc<dyn WorkspaceRepository>,
        outbox_service: Arc<OutboxJobService>,
        coordinator: Arc<ReportCoordinator>,
    ) -> Self {
        Self {
            workspace_repo,
            outbox_service,
            coordinator,
        }
    }

    pub fn start(self, cancel_token: CancellationToken) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            info!("Outbox sweep scheduler started");
            // Check every 30 seconds
            let mut tick = interval(Duration::from_secs(30));

            loop {
                tokio::select! {
                    () = cancel_token.cancelled() => break,
                    _ = tick.tick() => {
                        if let Err(e) = self.sweep().await {
                            error!(error = ?e, "Failed to run outbox sweep");
                        }
                    }
                }
            }
            info!("Outbox sweep scheduler stopped");
        })
    }

    async fn sweep(&self) -> Result<(), crate::errors::AppError> {
        // 1. Get all workspaces
        let workspaces = self.workspace_repo.get_all_workspaces().await?;

        // Consider jobs older than 5 minutes as stuck
        let older_than = chrono::Utc::now() - chrono::Duration::minutes(5);

        for ws in workspaces {
            // 2. Fetch stuck pending/running jobs
            let stuck_jobs = self
                .outbox_service
                .get_stuck_jobs(ws.id, older_than, 10)
                .await?;

            for job in stuck_jobs {
                info!(
                    job_id = %job.id,
                    workspace_id = %ws.id,
                    job_type = %job.job_type,
                    status = %job.status,
                    "Found stuck background job, re-enqueuing..."
                );

                if job.job_type == "AttendanceReport" {
                    // Deserialize payload
                    #[derive(serde::Deserialize)]
                    struct OutboxPayload {
                        request: CreateJobRequest,
                        requester_id: Uuid,
                    }

                    match serde_json::from_value::<OutboxPayload>(job.payload.clone()) {
                        Ok(outbox_payload) => {
                            // Update job updated_at so it doesn't get swept again immediately
                            if let Err(e) = self
                                .outbox_service
                                .update_job_status(ws.id, job.id, "running", None)
                                .await
                            {
                                error!(
                                    job_id = %job.id,
                                    error = ?e,
                                    "Failed to update outbox job status before re-enqueue"
                                );
                                continue;
                            }

                            // Re-enqueue
                            if let Err(e) = self
                                .coordinator
                                .enqueue_report_job_internal(
                                    ws.id,
                                    outbox_payload.requester_id,
                                    job.id,
                                    outbox_payload.request,
                                )
                                .await
                            {
                                error!(
                                    job_id = %job.id,
                                    error = ?e,
                                    "Failed to re-enqueue report job"
                                );
                                // Revert status or update error message
                                let _ = self
                                    .outbox_service
                                    .update_job_status(
                                        ws.id,
                                        job.id,
                                        "failed",
                                        Some(format!("Re-enqueue failed: {e}")),
                                    )
                                    .await;
                            } else {
                                info!(job_id = %job.id, "Successfully re-enqueued report job");
                            }
                        }
                        Err(e) => {
                            error!(
                                job_id = %job.id,
                                error = ?e,
                                "Failed to deserialize outbox payload for AttendanceReport"
                            );
                            let _ = self
                                .outbox_service
                                .update_job_status(
                                    ws.id,
                                    job.id,
                                    "failed",
                                    Some("Invalid job payload".to_string()),
                                )
                                .await;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Workspace;
    use crate::repositories::employee::MockEmployeeRepository;
    use crate::repositories::outbox_job_repository::MockOutboxJobRepository;
    use crate::repositories::queue_repository::MockQueueRepository;
    use crate::repositories::workspace_repository::MockWorkspaceRepository;

    #[tokio::test]
    async fn test_outbox_sweep_re_enqueues_stuck_job() {
        let ws_id = Uuid::new_v4();
        let requester_id = Uuid::new_v4();
        let job_id = Uuid::new_v4();

        let mut mock_workspace_repo = MockWorkspaceRepository::new();
        mock_workspace_repo
            .expect_get_workspace_by_id()
            .with(mockall::predicate::eq(ws_id))
            .returning(move |id| {
                Box::pin(async move {
                    Ok(Some(Workspace {
                        id,
                        name: "Test Workspace".to_string(),
                        created_at: chrono::Utc::now(),
                        date_format: "YYYY-MM-DD".to_string(),
                        time_format: "HH:mm:ss".to_string(),
                        timezone: "UTC".to_string(),
                        theme_primary_color: String::new(),
                        default_language: "en".to_string(),
                        holiday_overnight_strategy: "split".to_string(),
                        default_holiday_ot_multiplier: Default::default(),
                        logo_url: None,
                        tagline: None,
                        custom_db_url_encrypted: None,
                    }))
                })
            });
        let workspace_repo = Arc::new(mock_workspace_repo);
        let mut mock_workspace_repo_for_get = MockWorkspaceRepository::new();
        mock_workspace_repo_for_get
            .expect_get_all_workspaces()
            .returning(move || {
                let ws_id_val = ws_id;
                Box::pin(async move {
                    Ok(vec![Workspace {
                        id: ws_id_val,
                        name: "Test Workspace".to_string(),
                        created_at: chrono::Utc::now(),
                        date_format: "YYYY-MM-DD".to_string(),
                        time_format: "HH:mm:ss".to_string(),
                        timezone: "UTC".to_string(),
                        theme_primary_color: String::new(),
                        default_language: "en".to_string(),
                        holiday_overnight_strategy: "split".to_string(),
                        default_holiday_ot_multiplier: Default::default(),
                        logo_url: None,
                        tagline: None,
                        custom_db_url_encrypted: None,
                    }])
                })
            });
        let workspace_repo_for_sweep = Arc::new(mock_workspace_repo_for_get);

        let mut mock_outbox_repo = MockOutboxJobRepository::new();
        let request = CreateJobRequest {
            report_type: crate::queue::ReportType::Summary,
            start_date: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            end_date: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            employee_ids: None,
            sort: None,
        };

        #[derive(serde::Serialize)]
        struct OutboxPayload {
            request: CreateJobRequest,
            requester_id: Uuid,
        }

        let payload = serde_json::to_value(&OutboxPayload {
            request,
            requester_id,
        })
        .unwrap();

        let job = crate::models::OutboxJob {
            id: job_id,
            workspace_id: ws_id,
            job_type: "AttendanceReport".to_string(),
            payload,
            status: "running".to_string(),
            error_message: None,
            created_at: chrono::Utc::now() - chrono::Duration::minutes(10),
            updated_at: chrono::Utc::now() - chrono::Duration::minutes(10),
        };

        let job_clone = job.clone();
        mock_outbox_repo
            .expect_get_stuck_jobs()
            .with(
                mockall::predicate::always(),
                mockall::predicate::always(),
                mockall::predicate::eq(10),
            )
            .returning(move |_, _, _| Ok(vec![job_clone.clone()]));

        mock_outbox_repo
            .expect_update_job_status()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(job_id),
                mockall::predicate::eq("running"),
                mockall::predicate::eq(None),
            )
            .returning(|_, _, _, _| Ok(()));

        let mut mock_employee_repo = MockEmployeeRepository::new();
        mock_employee_repo
            .expect_get_active_employee_ids()
            .returning(|_, _| Ok(vec![Uuid::new_v4()]));

        let mut mock_queue_repo = MockQueueRepository::new();
        mock_queue_repo
            .expect_create_job_metadata()
            .returning(|_| Ok(()));
        mock_queue_repo.expect_enqueue_jobs().returning(|_| Ok(()));

        let outbox_service = Arc::new(OutboxJobService::new(Arc::new(mock_outbox_repo)));
        let coordinator = Arc::new(ReportCoordinator::new(
            Arc::new(mock_queue_repo),
            Arc::new(mock_employee_repo),
            workspace_repo,
            outbox_service.clone(),
        ));

        let scheduler =
            OutboxSweepScheduler::new(workspace_repo_for_sweep, outbox_service, coordinator);

        let result = scheduler.sweep().await;
        assert!(result.is_ok());
    }
}
