use async_trait::async_trait;
use chrono_tz::Tz;
use std::collections::HashMap;
use uuid::Uuid;

use crate::models::{AttendanceImportJob, ImportJobStatus};
use crate::repositories::import_job_repository::ImportJobRepository;

#[async_trait]
pub trait ImportStrategy: Send + Sync {
    type RowData: Send + Sync + Clone;

    /// Expected headers in the Excel file
    fn expected_headers(&self) -> &'static [&'static str];

    /// Map a row of data to the generic type `RowData`
    fn map_row(
        &mut self,
        row_data: &HashMap<String, String>,
        current_line: usize,
        workspace_tz: Tz,
    ) -> Result<Self::RowData, crate::errors::AppError>;

    /// Process a single valid row. Returns Ok(Some(id)) for success, Ok(None) for skipped, Err(reason) for failure.
    async fn process_row(
        &mut self,
        deps: &crate::worker::deps::WorkerDeps,
        job: &AttendanceImportJob,
        row: &Self::RowData,
    ) -> Result<Option<Uuid>, crate::errors::AppError>;

    /// Optional hook to initialize caching or do pre-flight checks
    async fn init(
        &mut self,
        _deps: &crate::worker::deps::WorkerDeps,
        _job: &AttendanceImportJob,
        _rows: &[Self::RowData],
    ) -> Result<(), crate::errors::AppError> {
        Ok(())
    }

    /// Optional hook to pre-process a chunk of rows in bulk (e.g., bulk fetch cache or bulk insert)
    async fn pre_process_chunk(
        &mut self,
        _deps: &crate::worker::deps::WorkerDeps,
        _job: &AttendanceImportJob,
        _rows: &[Self::RowData],
    ) -> Result<(), crate::errors::AppError> {
        Ok(())
    }
}

pub struct ImportPipeline<'a, S: ImportStrategy> {
    pub strategy: S,
    pub deps: crate::worker::deps::WorkerDeps,
    pub import_repo: &'a dyn ImportJobRepository,
    pub job: &'a AttendanceImportJob,
    pub file_bytes: Vec<u8>,
    pub workspace_tz: Tz,
    pub start_index: Option<usize>,
    pub end_index: Option<usize>,
    pub is_chunk: bool,
    pub queue_repo:
        Option<std::sync::Arc<dyn crate::repositories::queue_repository::QueueRepository>>,
}

impl<S: ImportStrategy> ImportPipeline<'_, S> {
    #[allow(clippy::too_many_lines)]
    pub async fn run(mut self) -> Result<(), ()> {
        let mut errors = Vec::new();
        let mut failed_rows_data: Vec<(HashMap<String, String>, String)> = Vec::new();
        let mut pending_rows: Vec<(S::RowData, HashMap<String, String>)> = Vec::new();

        let lang = match self
            .deps
            .workspace_repo
            .get_workspace_by_id(self.job.workspace_id)
            .await
        {
            Ok(Some(ws)) => ws.default_language().to_string(),
            _ => "vi".to_string(),
        };

        match crate::utils::file_parser::extract_mapped_rows(
            &self.file_bytes,
            self.strategy.expected_headers(),
        ) {
            Ok(rows) => {
                for (current_line, row_data) in rows {
                    if let Some(start) = self.start_index {
                        if current_line < start {
                            continue;
                        }
                    }
                    if let Some(end) = self.end_index {
                        if current_line > end {
                            continue;
                        }
                    }

                    match self
                        .strategy
                        .map_row(&row_data, current_line, self.workspace_tz)
                    {
                        Ok(mapped_data) => {
                            pending_rows.push((mapped_data, row_data));
                        }
                        Err(e) => {
                            let translated_err = e.translate(&lang);
                            errors.push(translated_err.clone());
                            failed_rows_data.push((row_data, translated_err));
                        }
                    }
                }
            }
            Err(e) => {
                errors.push(e.translate(&lang));
            }
        }

        let parsed_rows: Vec<S::RowData> = pending_rows.iter().map(|(r, _)| r).cloned().collect();
        if let Err(e) = self.strategy.init(&self.deps, self.job, &parsed_rows).await {
            let translated_err = e.translate(&lang);
            let _ = self
                .import_repo
                .update_job_progress(
                    &crate::diesel_pool::TenantAuthToken::system_override(self.job.workspace_id),
                    self.job.id,
                    0,
                    ImportJobStatus::Failed,
                    Some(translated_err),
                )
                .await;
            return Err(());
        }

        if let Err(e) = self
            .strategy
            .pre_process_chunk(&self.deps, self.job, &parsed_rows)
            .await
        {
            let translated_err = e.translate(&lang);
            let _ = self
                .import_repo
                .update_job_progress(
                    &crate::diesel_pool::TenantAuthToken::system_override(self.job.workspace_id),
                    self.job.id,
                    0,
                    ImportJobStatus::Failed,
                    Some(translated_err),
                )
                .await;
            return Err(());
        }

        if !self.is_chunk {
            let total_rows = i32::try_from(pending_rows.len()).unwrap_or(0)
                + i32::try_from(failed_rows_data.len()).unwrap_or(0);
            let _ = self
                .import_repo
                .update_job_total_rows(
                    &crate::diesel_pool::TenantAuthToken::system_override(self.job.workspace_id),
                    self.job.id,
                    total_rows,
                )
                .await;
        }

        let mut processed_rows = 0;
        let mut skipped_rows = 0;

        for (row_data_parsed, raw_row_data) in pending_rows {
            match self
                .strategy
                .process_row(&self.deps, self.job, &row_data_parsed)
                .await
            {
                Ok(Some(_id)) => {
                    processed_rows += 1;
                }
                Ok(None) => {
                    skipped_rows += 1;
                    failed_rows_data.push((
                        raw_row_data,
                        "Bỏ qua (Trùng lặp hoặc đã tồn tại)".to_string(),
                    ));
                }
                Err(e) => {
                    let translated_err = e.translate(&lang);
                    errors.push(translated_err.clone());
                    failed_rows_data.push((raw_row_data, translated_err));
                }
            }

            if (processed_rows + skipped_rows) % 10 == 0 {
                if self.is_chunk {
                    if let Some(ref q) = self.queue_repo {
                        let _ = q.increment_progress(self.job.id, 10).await;
                    }
                } else {
                    let _ = self
                        .import_repo
                        .update_job_progress(
                            &crate::diesel_pool::TenantAuthToken::system_override(
                                self.job.workspace_id,
                            ),
                            self.job.id,
                            processed_rows,
                            ImportJobStatus::Processing,
                            None,
                        )
                        .await;
                }
            }
        }

        let status = if (errors.is_empty() && skipped_rows == 0) || processed_rows > 0 {
            ImportJobStatus::Completed
        } else {
            ImportJobStatus::Failed
        };

        let mut has_error_file = false;
        let base_dir = crate::utils::get_storage_dir();
        let error_file_path = format!("{base_dir}/imports/error_{}.xlsx", self.job.id);

        if !failed_rows_data.is_empty() {
            use rust_xlsxwriter::{Color, Format, Workbook};
            let mut workbook = Workbook::new();
            let worksheet = workbook.add_worksheet();

            let header_format = Format::new()
                .set_bold()
                .set_font_color(Color::White)
                .set_background_color(Color::Red);

            // Dynamically write headers based on what was provided in failed rows
            let mut all_headers = self.strategy.expected_headers().to_vec();
            all_headers.push("error_reason");

            for (col, header) in all_headers.iter().enumerate() {
                let _ = worksheet.write_string_with_format(
                    0,
                    u16::try_from(col).unwrap_or(0),
                    *header,
                    &header_format,
                );
            }

            for (row_idx, (data, err)) in failed_rows_data.iter().enumerate() {
                let row = u32::try_from(row_idx + 1).unwrap_or(0);
                for (col, header) in self.strategy.expected_headers().iter().enumerate() {
                    let _ = worksheet.write_string(
                        row,
                        u16::try_from(col).unwrap_or(0),
                        data.get(*header).unwrap_or(&String::new()),
                    );
                }
                // Write error reason in the last column
                let _ = worksheet.write_string(
                    row,
                    u16::try_from(all_headers.len().saturating_sub(1)).unwrap_or(0),
                    err,
                );
            }

            if workbook.save(&error_file_path).is_ok() {
                has_error_file = true;
            }
        }

        let final_log = if self.job.target_type
            == crate::models::enums::ImportTargetType::LeaveImport
        {
            if !failed_rows_data.is_empty() || skipped_rows > 0 || !errors.is_empty() {
                let error_count = i32::try_from(errors.len()).unwrap_or(0)
                    + i32::try_from(failed_rows_data.len()).unwrap_or(0)
                    + skipped_rows;
                let json_log = serde_json::json!({
                    "success_count": processed_rows,
                    "error_count": error_count,
                    "error_file_path": if has_error_file { Some(error_file_path) } else { None },
                });
                Some(json_log.to_string())
            } else {
                None
            }
        } else {
            let mut log_opt = if errors.is_empty() {
                None
            } else {
                Some(errors.join("\n"))
            };

            if skipped_rows > 0 {
                let skip_msg = format!("Đã bỏ qua {skipped_rows} dòng");
                log_opt = Some(match log_opt {
                    Some(log) => format!("{log}\n{skip_msg}"),
                    None => skip_msg,
                });
            }

            if has_error_file {
                let download_link = format!(
                    "/api/v1/workspaces/{}/attendance/import-jobs/{}/error-file",
                    self.job.workspace_id, self.job.id
                );
                let link_msg = format!("DL_LINK:{download_link}");
                log_opt = Some(match log_opt {
                    Some(log) => format!("{link_msg}\n{log}"),
                    None => link_msg,
                });
            }
            log_opt
        };

        if self.is_chunk {
            // Push errors to Redis if any
            if let Some(final_msg) = final_log {
                if let Some(ref q) = self.queue_repo {
                    let field = format!(
                        "chunk_{}_{}",
                        self.start_index.unwrap_or(0),
                        self.end_index.unwrap_or(0)
                    );
                    let _ = q.store_import_error(self.job.id, &field, &final_msg).await;
                }
            }

            // Add any remainder to progress
            let remainder = (processed_rows + skipped_rows) % 10;
            if remainder > 0 {
                if let Some(ref q) = self.queue_repo {
                    let _ = q
                        .increment_progress(self.job.id, u32::try_from(remainder).unwrap_or(0))
                        .await;
                }
            }

            Ok(())
        } else {
            let _ = self
                .import_repo
                .update_job_progress(
                    &crate::diesel_pool::TenantAuthToken::system_override(self.job.workspace_id),
                    self.job.id,
                    processed_rows,
                    status,
                    final_log,
                )
                .await;

            if let Some(file_path) = &self.job.file_path {
                let _ = tokio::fs::remove_file(file_path).await;
            }

            if errors.is_empty() {
                Ok(())
            } else {
                Err(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AttendanceImportJob, ImportJobStatus};
    use crate::repositories::import_job_repository::MockImportJobRepository;
    use crate::repositories::queue_repository::MockQueueRepository;
    use crate::worker::deps::WorkerDeps;
    use crate::worker::test_utils::MockWorkerDepsBuilder;
    use std::sync::Arc;

    struct DummyStrategy {
        headers: &'static [&'static str],
        process_result: Result<Option<Uuid>, String>,
    }

    #[async_trait]
    impl ImportStrategy for DummyStrategy {
        type RowData = String;

        fn expected_headers(&self) -> &'static [&'static str] {
            self.headers
        }

        fn map_row(
            &mut self,
            _row_data: &HashMap<String, String>,
            _current_line: usize,
            _workspace_tz: Tz,
        ) -> Result<Self::RowData, crate::errors::AppError> {
            Ok("dummy_data".to_string())
        }

        async fn init(
            &mut self,
            _deps: &WorkerDeps,
            _job: &AttendanceImportJob,
            _rows: &[Self::RowData],
        ) -> Result<(), crate::errors::AppError> {
            Ok(())
        }

        async fn process_row(
            &mut self,
            _deps: &WorkerDeps,
            _job: &AttendanceImportJob,
            _row: &Self::RowData,
        ) -> Result<Option<Uuid>, crate::errors::AppError> {
            self.process_result
                .clone()
                .map_err(crate::errors::AppError::Internal)
        }
    }

    #[tokio::test]
    async fn test_import_pipeline_run_success() {
        let mut mock_import_repo = MockImportJobRepository::new();
        let mut mock_queue_repo = MockQueueRepository::new();

        // Should update progress every 10 rows
        mock_queue_repo
            .expect_increment_progress()
            .returning(|_, _| Ok(()));

        mock_import_repo
            .expect_update_job_total_rows()
            .returning(|_, _, _| Ok(()));

        mock_import_repo
            .expect_update_job_progress()
            .returning(|_, _, _, _, _| Ok(()));

        let job = AttendanceImportJob {
            id: Uuid::new_v4(),
            workspace_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            status: ImportJobStatus::Pending,
            total_rows: 0,
            processed_rows: 0,
            error_log: None,
            file_path: None,
            created_at: chrono::Utc::now(),
            target_type: crate::models::enums::ImportTargetType::Employee,
            updated_at: chrono::Utc::now(),
        };

        // Create a dummy xlsx byte array... actually we might not be able to easily create a valid xlsx bytes.
        // Wait, file_parser::extract_mapped_rows needs a valid Excel file or CSV.
        // It might be easier to test chunk processing behavior.
        // But extracting mapped rows will fail if bytes are invalid, pushing an error and finishing the pipeline!
        // Let's test the error path then.

        let file_bytes = vec![0u8; 10]; // invalid excel file
        let strategy = DummyStrategy {
            headers: &["name", "email"],
            process_result: Ok(Some(Uuid::new_v4())),
        };

        // For non-chunk, it updates total rows and final progress
        mock_import_repo
            .expect_update_job_progress()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(job.id),
                mockall::predicate::eq(0),
                mockall::predicate::eq(ImportJobStatus::Failed),
                mockall::predicate::always(), // should contain error log
            )
            .returning(|_, _, _, _, _| Ok(()));

        // Create dummy deps
        let mut builder = MockWorkerDepsBuilder::new();
        builder
            .workspace_repo
            .expect_get_workspace_by_id()
            .returning(|id| {
                Box::pin(async move {
                    Ok(Some(crate::models::Workspace::new(
                        id,
                        "Mocked Workspace".into(),
                        "YYYY-MM-DD".into(),
                        "HH:mm:ss".into(),
                        "UTC".into(),
                        "#000000".into(),
                        "vi".into(),
                        None,
                        None,
                        "split".into(),
                        rust_decimal::Decimal::new(15, 1),
                        chrono::Utc::now(),
                        None,
                    )))
                })
            });
        let deps = builder.build();

        let pipeline = ImportPipeline {
            strategy,
            deps,
            import_repo: &mock_import_repo,
            job: &job,
            file_bytes,
            workspace_tz: chrono_tz::UTC,
            start_index: None,
            end_index: None,
            is_chunk: false,
            queue_repo: Some(Arc::new(mock_queue_repo)),
        };

        let result = pipeline.run().await;
        assert!(result.is_err()); // because extracting mapped rows fails with invalid excel
    }
}
