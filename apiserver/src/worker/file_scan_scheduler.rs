use crate::diesel_pool::TenantAuthToken;
use crate::errors::AppError;
use crate::repositories::workspace_repository::WorkspaceRepository;
use crate::repositories::FileRepository;
use crate::services::storage::{FileService, ScanResult, VirusScanner};
use futures_util::stream::{self, StreamExt};
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

pub struct FileScanScheduler {
    file_repo: Arc<dyn FileRepository>,
    file_service: Arc<FileService>,
    scan_service: Arc<dyn VirusScanner>,
    workspace_repo: Arc<dyn WorkspaceRepository>,
}

impl FileScanScheduler {
    pub fn new(
        file_repo: Arc<dyn FileRepository>,
        file_service: Arc<FileService>,
        scan_service: Arc<dyn VirusScanner>,
        workspace_repo: Arc<dyn WorkspaceRepository>,
    ) -> Self {
        Self {
            file_repo,
            file_service,
            scan_service,
            workspace_repo,
        }
    }

    pub fn start(&self, cancel_token: CancellationToken) -> tokio::task::JoinHandle<()> {
        let repo = Arc::clone(&self.file_repo);
        let file_service = Arc::clone(&self.file_service);
        let scan_service = Arc::clone(&self.scan_service);
        let ws_repo = Arc::clone(&self.workspace_repo);

        tokio::spawn(async move {
            tracing::info!("Starting FileScanScheduler...");
            let mut interval = tokio::time::interval(Duration::from_secs(30)); // Run every 30 seconds

            loop {
                tokio::select! {
                    () = cancel_token.cancelled() => {
                        tracing::info!("FileScanScheduler shutting down");
                        break;
                    }
                    _ = interval.tick() => {
                        if let Err(e) = Self::process_pending_files(&repo, &file_service, &scan_service, &ws_repo).await {
                            tracing::error!("Error in FileScanScheduler: {}", e);
                        }
                    }
                }
            }
        })
    }

    #[allow(clippy::too_many_lines)]
    async fn process_pending_files(
        repo: &Arc<dyn FileRepository>,
        file_service: &Arc<FileService>,
        scan_service: &Arc<dyn VirusScanner>,
        ws_repo: &Arc<dyn WorkspaceRepository>,
    ) -> Result<(), AppError> {
        let workspaces = ws_repo.get_all_workspaces().await?;

        let mut workspace_stream = stream::iter(workspaces)
            .map(|ws| {
                let repo = Arc::clone(repo);
                let file_service = Arc::clone(file_service);
                let scan_service = Arc::clone(scan_service);
                async move {
                    let auth_token = TenantAuthToken::system_override(ws.id());
                    let pending_files =
                        match repo.get_pending_scan_workspace_files(&auth_token, 10).await {
                            Ok(files) => files,
                            Err(e) => {
                                tracing::error!(
                                    "Failed to fetch pending files for workspace {}: {}",
                                    ws.id(),
                                    e
                                );
                                return;
                            }
                        };

                    for file in pending_files {
                        let bytes = match file_service
                            .download_file_direct(&auth_token, file.id)
                            .await
                        {
                            Ok(b) => b,
                            Err(e) => {
                                tracing::error!(
                                    "Failed to download file {} for scan: {}",
                                    file.id,
                                    e
                                );
                                continue;
                            }
                        };

                        let scan_result = match scan_service.scan(&bytes).await {
                            Ok(r) => r,
                            Err(e) => {
                                tracing::error!("Failed to scan file {}: {}", file.id, e);
                                continue;
                            }
                        };

                        let status_str = match scan_result {
                            ScanResult::Clean => "Clean",
                            ScanResult::Infected(e) => {
                                tracing::warn!("Infected file detected {}: {}", file.id, e);
                                "Infected"
                            }
                            ScanResult::Skipped(e) => {
                                tracing::info!("Scan skipped for file {}: {}", file.id, e);
                                "Clean"
                            }
                        };

                        if let Err(e) = repo
                            .update_workspace_file_scan_status(&auth_token, file.id, status_str)
                            .await
                        {
                            tracing::error!("Failed to update scan status for {}: {}", file.id, e);
                        }
                    }
                }
            })
            .buffer_unordered(5);

        while workspace_stream.next().await.is_some() {}

        // Also process system files
        let system_files = match repo.get_pending_scan_system_files(10).await {
            Ok(files) => files,
            Err(e) => {
                tracing::error!("Failed to fetch pending system files: {}", e);
                return Ok(());
            }
        };

        for file in system_files {
            let bytes = match file_service.download_system_file_direct(file.id).await {
                Ok(b) => b,
                Err(e) => {
                    tracing::error!("Failed to download system file {} for scan: {}", file.id, e);
                    continue;
                }
            };

            let scan_result = match scan_service.scan(&bytes).await {
                Ok(r) => r,
                Err(e) => {
                    tracing::error!("Failed to scan system file {}: {}", file.id, e);
                    continue;
                }
            };

            let status_str = match scan_result {
                ScanResult::Clean => "Clean",
                ScanResult::Infected(e) => {
                    tracing::warn!("Infected system file detected {}: {}", file.id, e);
                    "Infected"
                }
                ScanResult::Skipped(e) => {
                    tracing::info!("Scan skipped for system file {}: {}", file.id, e);
                    "Clean"
                }
            };

            if let Err(e) = repo
                .update_system_file_scan_status(file.id, status_str)
                .await
            {
                tracing::error!(
                    "Failed to update scan status for system file {}: {}",
                    file.id,
                    e
                );
            }
        }

        Ok(())
    }
}
