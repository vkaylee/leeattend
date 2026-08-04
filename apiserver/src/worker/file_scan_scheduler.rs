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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::CryptoService;
    use crate::diesel_pool::DieselPools;
    use crate::models::Workspace;
    use crate::repositories::file_repository::MockFileRepository;
    use crate::repositories::storage_config_repo::StorageConfigRepository;
    use crate::repositories::workspace_repository::MockWorkspaceRepository;
    use crate::services::storage::resolver::StorageResolver;
    use crate::services::storage::scanner::mock::MockScanner;

    #[tokio::test]
    async fn test_process_pending_files_empty() {
        let mut ws_repo = MockWorkspaceRepository::new();
        let mut file_repo = MockFileRepository::new();

        // 1. Mock get_all_workspaces to return one workspace
        let ws_id = uuid::Uuid::new_v4();
        let workspace = Workspace::new(NewWorkspaceParams {
            id: ws_id,
            name: "Test Workspace".to_string(),
            date_format: "YYYY-MM-DD".to_string(),
            time_format: "HH:mm:ss".to_string(),
            timezone: "UTC".to_string(),
            theme_primary_color: "#000000".to_string(),
            default_language: "en".to_string(),
            logo_url: None,
            tagline: None,
            holiday_overnight_strategy: "ignore".to_string(),
            default_holiday_ot_multiplier: rust_decimal::Decimal::ZERO,
            created_at: chrono::Utc::now(),
            custom_db_url_encrypted: None,
        });
        ws_repo
            .expect_get_all_workspaces()
            .times(1)
            .returning(move || {
                let ws = workspace.clone();
                Box::pin(async move { Ok(vec![ws]) })
            });

        // 2. Mock get_pending_scan_workspace_files to return empty list
        file_repo
            .expect_get_pending_scan_workspace_files()
            .times(1)
            .returning(|_, _| Ok(vec![]));

        // 3. Mock get_pending_scan_system_files to return empty list
        file_repo
            .expect_get_pending_scan_system_files()
            .times(1)
            .returning(|_| Ok(vec![]));

        // Setup dummy FileService
        let mock_pools =
            DieselPools::for_test("postgres://user:pass@localhost:5432/leeattend_test_db");
        let keys_json = r#"{"active_key": "v1", "keys": {"v1": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="}}"#;
        let crypto = Arc::new(CryptoService::from_json(keys_json).unwrap());
        let config_repo = Arc::new(StorageConfigRepository::new(mock_pools.api.clone(), crypto));

        struct DummyObjectStorage;
        #[async_trait::async_trait]
        impl crate::services::storage::ObjectStorage for DummyObjectStorage {
            async fn put_object(
                &self,
                _key: &str,
                _data: bytes::Bytes,
                _content_type: &str,
            ) -> crate::errors::AppResult<()> {
                Ok(())
            }
            async fn get_object(&self, _key: &str) -> crate::errors::AppResult<bytes::Bytes> {
                Ok(bytes::Bytes::new())
            }
            async fn delete_object(&self, _key: &str) -> crate::errors::AppResult<()> {
                Ok(())
            }
            async fn presigned_get_url(
                &self,
                _key: &str,
                _ttl: std::time::Duration,
            ) -> crate::errors::AppResult<String> {
                Ok(String::new())
            }
            async fn presigned_put_url(
                &self,
                _key: &str,
                _content_type: &str,
                _ttl: std::time::Duration,
            ) -> crate::errors::AppResult<String> {
                Ok(String::new())
            }
            async fn head_object(
                &self,
                _key: &str,
            ) -> crate::errors::AppResult<crate::services::storage::ObjectMeta> {
                Ok(crate::services::storage::ObjectMeta {
                    content_type: None,
                    content_length: 0,
                    last_modified: None,
                })
            }
            async fn health_check(&self) -> crate::errors::AppResult<()> {
                Ok(())
            }
        }

        let central = Arc::new(DummyObjectStorage);
        let storage_resolver = Arc::new(StorageResolver::new(central, config_repo));

        let file_repo_arc: Arc<dyn FileRepository> = Arc::new(file_repo);
        let file_service = Arc::new(FileService::new(
            storage_resolver,
            Arc::clone(&file_repo_arc),
        ));
        let scan_service: Arc<dyn VirusScanner> = Arc::new(MockScanner);
        let ws_repo_arc: Arc<dyn WorkspaceRepository> = Arc::new(ws_repo);

        let result = FileScanScheduler::process_pending_files(
            &file_repo_arc,
            &file_service,
            &scan_service,
            &ws_repo_arc,
        )
        .await;

        assert!(result.is_ok());
    }
}
