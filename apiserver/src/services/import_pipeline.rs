use async_trait::async_trait;
use chrono_tz::Tz;
use std::collections::HashMap;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{AttendanceImportJob, ImportJobStatus};
use crate::repositories::import_job_repository::ImportJobRepository;
use crate::repositories::queue_repository::QueueRepository;

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

    /// Optional hook called after ALL rows have been processed.
    /// Use for batch finalization that should run once after the row loop
    /// (e.g., bulk pairing, recalculations).
    /// Only called for non-chunked imports; chunk-based imports finalize
    /// through the separate FinalizeImport handler.
    async fn finalize(
        &mut self,
        _deps: &crate::worker::deps::WorkerDeps,
        _job: &AttendanceImportJob,
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

#[derive(Debug, Clone)]
struct FailedRow {
    data: HashMap<String, String>,
    reason: String,
    row_status: &'static str,
}

/// Per-10-row progress reporter for import pipelines.
/// Non-critical by design: failures are logged at WARN level and never propagated.
struct ProgressTracker<'a> {
    import_repo: &'a dyn ImportJobRepository,
    queue_repo: Option<std::sync::Arc<dyn QueueRepository>>,
    job_id: Uuid,
    workspace_id: Uuid,
    is_chunk: bool,
}

impl ProgressTracker<'_> {
    async fn tick(&self, processed_rows: i32) {
        if self.is_chunk {
            if let Some(ref q) = self.queue_repo {
                if let Err(e) = q.increment_progress(self.job_id, 10).await {
                    tracing::warn!(job_id = %self.job_id, error = %e, "Failed to increment chunk import progress");
                }
            }
        } else {
            let auth = crate::diesel_pool::TenantAuthToken::system_override(self.workspace_id);
            if let Err(e) = self
                .import_repo
                .update_job_progress(
                    &auth,
                    self.job_id,
                    processed_rows,
                    ImportJobStatus::Processing,
                    None,
                )
                .await
            {
                tracing::warn!(job_id = %self.job_id, error = %e, "Failed to update import progress");
            }
        }
    }

    async fn finish(&self, processed_rows: i32, skipped_rows: i32) {
        if !self.is_chunk {
            return;
        }
        let remainder = (processed_rows + skipped_rows) % 10;
        if remainder > 0 {
            if let Some(ref q) = self.queue_repo {
                if let Err(e) = q
                    .increment_progress(self.job_id, u32::try_from(remainder).unwrap_or(0))
                    .await
                {
                    tracing::warn!(job_id = %self.job_id, error = %e, "Failed to flush chunk import progress remainder");
                }
            }
        }
    }
}

impl<S: ImportStrategy> ImportPipeline<'_, S> {
    pub async fn run(mut self) -> Result<(), AppError> {
        let mut errors: Vec<String> = Vec::new();
        let mut failed_rows_data: Vec<FailedRow> = Vec::new();

        let lang = self.resolve_language().await;

        let pending_rows = self.parse_and_map_rows(&lang, &mut errors, &mut failed_rows_data);

        let parsed_rows: Vec<S::RowData> = pending_rows.iter().map(|(r, _)| r).cloned().collect();
        self.run_strategy_hooks(&lang, &parsed_rows).await?;

        if !self.is_chunk {
            let total_rows = i32::try_from(pending_rows.len()).unwrap_or(0)
                + i32::try_from(failed_rows_data.len()).unwrap_or(0);
            let auth = crate::diesel_pool::TenantAuthToken::system_override(self.job.workspace_id);
            if let Err(e) = self
                .import_repo
                .update_job_total_rows(&auth, self.job.id, total_rows)
                .await
            {
                tracing::warn!(job_id = %self.job.id, error = %e, "Failed to set import total rows");
            }
        }

        let progress = ProgressTracker {
            import_repo: self.import_repo,
            queue_repo: self.queue_repo.clone(),
            job_id: self.job.id,
            workspace_id: self.job.workspace_id,
            is_chunk: self.is_chunk,
        };
        let (processed_rows, skipped_rows) = Self::process_rows_loop(
            &mut self.strategy,
            &self.deps,
            self.job,
            pending_rows,
            &lang,
            &mut errors,
            &mut failed_rows_data,
            &progress,
        )
        .await;
        progress.finish(processed_rows, skipped_rows).await;

        let error_file_path_opt = self.generate_error_file(&failed_rows_data).await;

        self.finalize_and_update_status(
            processed_rows,
            skipped_rows,
            errors,
            failed_rows_data,
            error_file_path_opt,
            &lang,
        )
        .await
    }

    async fn resolve_language(&self) -> String {
        match self
            .deps
            .workspace_repo
            .get_workspace_by_id(self.job.workspace_id)
            .await
        {
            Ok(Some(ws)) => ws.default_language().to_string(),
            _ => "vi".to_string(),
        }
    }

    fn parse_and_map_rows(
        &mut self,
        lang: &str,
        errors: &mut Vec<String>,
        failed_rows_data: &mut Vec<FailedRow>,
    ) -> Vec<(S::RowData, HashMap<String, String>)> {
        let mut pending_rows = Vec::new();
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
                        Ok(mapped_data) => pending_rows.push((mapped_data, row_data)),
                        Err(e) => {
                            let translated_err = e.translate(lang);
                            errors.push(translated_err.clone());
                            failed_rows_data.push(FailedRow {
                                data: row_data,
                                reason: translated_err,
                                row_status: "error",
                            });
                        }
                    }
                }
            }
            Err(e) => {
                errors.push(e.translate(lang));
            }
        }
        pending_rows
    }

    async fn run_strategy_hooks(
        &mut self,
        lang: &str,
        parsed_rows: &[S::RowData],
    ) -> Result<(), AppError> {
        if let Err(e) = self.strategy.init(&self.deps, self.job, parsed_rows).await {
            let translated_err = e.translate(lang);
            tracing::error!(job_id = %self.job.id, error = %translated_err, "Import strategy init failed");
            self.update_job_to_failed(0, &translated_err).await;
            return Err(AppError::Internal(format!(
                "Strategy init failed: {translated_err}"
            )));
        }
        if let Err(e) = self
            .strategy
            .pre_process_chunk(&self.deps, self.job, parsed_rows)
            .await
        {
            let translated_err = e.translate(lang);
            tracing::error!(job_id = %self.job.id, error = %translated_err, "Import strategy pre_process_chunk failed");
            self.update_job_to_failed(0, &translated_err).await;
            return Err(AppError::Internal(format!(
                "Strategy pre_process_chunk failed: {translated_err}"
            )));
        }
        Ok(())
    }

    async fn update_job_to_failed(&self, processed_rows: i32, error_msg: &str) {
        let auth = crate::diesel_pool::TenantAuthToken::system_override(self.job.workspace_id);
        if let Err(e) = self
            .import_repo
            .update_job_progress(
                &auth,
                self.job.id,
                processed_rows,
                ImportJobStatus::Failed,
                Some(error_msg.to_string()),
            )
            .await
        {
            tracing::error!(job_id = %self.job.id, error = %e, "Failed to mark import job as Failed (DB may be unreachable)");
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn process_rows_loop(
        strategy: &mut S,
        deps: &crate::worker::deps::WorkerDeps,
        job: &AttendanceImportJob,
        pending_rows: Vec<(S::RowData, HashMap<String, String>)>,
        lang: &str,
        errors: &mut Vec<String>,
        failed_rows_data: &mut Vec<FailedRow>,
        progress: &ProgressTracker<'_>,
    ) -> (i32, i32) {
        let mut processed = 0i32;
        let mut skipped = 0i32;
        for (row_data_parsed, raw_row_data) in pending_rows {
            match strategy.process_row(deps, job, &row_data_parsed).await {
                Ok(Some(_id)) => processed += 1,
                Ok(None) => {
                    skipped += 1;
                    failed_rows_data.push(FailedRow {
                        data: raw_row_data,
                        reason: "Skipped (duplicate or already exists)".to_string(),
                        row_status: "skipped",
                    });
                }
                Err(e) => {
                    let translated_err = e.translate(lang);
                    errors.push(translated_err.clone());
                    failed_rows_data.push(FailedRow {
                        data: raw_row_data,
                        reason: translated_err,
                        row_status: "error",
                    });
                }
            }
            if (processed + skipped) % 10 == 0 {
                progress.tick(processed).await;
            }
        }
        (processed, skipped)
    }

    async fn generate_error_file(&self, failed_rows_data: &[FailedRow]) -> Option<String> {
        if failed_rows_data.is_empty() {
            return None;
        }
        let base_dir = crate::utils::get_storage_dir();
        let path = format!("{base_dir}/imports/error_{}.xlsx", self.job.id);
        let headers = self.strategy.expected_headers().to_vec();
        let rows = failed_rows_data.to_vec();
        let path_clone = path.clone();

        let result = tokio::task::spawn_blocking(move || {
            use rust_xlsxwriter::{Color, Format, Workbook};
            let mut workbook = Workbook::new();
            let worksheet = workbook.add_worksheet();
            let header_format = Format::new()
                .set_bold()
                .set_font_color(Color::White)
                .set_background_color(Color::Red);
            let mut all_headers = headers.clone();
            all_headers.push("status");
            all_headers.push("error_reason");
            for (col, header) in all_headers.iter().enumerate() {
                let _ = worksheet.write_string_with_format(
                    0,
                    u16::try_from(col).unwrap_or(0),
                    *header,
                    &header_format,
                );
            }
            for (row_idx, failed_row) in rows.iter().enumerate() {
                let row = u32::try_from(row_idx + 1).unwrap_or(0);
                for (col, header) in headers.iter().enumerate() {
                    let _ = worksheet.write_string(
                        row,
                        u16::try_from(col).unwrap_or(0),
                        failed_row.data.get(*header).unwrap_or(&String::new()),
                    );
                }
                let _ = worksheet.write_string(
                    row,
                    u16::try_from(all_headers.len().saturating_sub(2)).unwrap_or(0),
                    failed_row.row_status,
                );
                let _ = worksheet.write_string(
                    row,
                    u16::try_from(all_headers.len().saturating_sub(1)).unwrap_or(0),
                    &failed_row.reason,
                );
            }
            if workbook.save(&path_clone).is_ok() {
                Some(path_clone)
            } else {
                None
            }
        })
        .await;

        match result {
            Ok(Some(p)) => Some(p),
            Ok(None) => {
                tracing::error!(job_id = %self.job.id, "Failed to save error xlsx file to disk");
                None
            }
            Err(join_err) => {
                tracing::error!(job_id = %self.job.id, error = %join_err, "spawn_blocking panicked during error file generation");
                None
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    async fn finalize_and_update_status(
        mut self,
        processed_rows: i32,
        skipped_rows: i32,
        errors: Vec<String>,
        failed_rows_data: Vec<FailedRow>,
        error_file_path_opt: Option<String>,
        lang: &str,
    ) -> Result<(), AppError> {
        if !self.is_chunk {
            if let Err(e) = self.strategy.finalize(&self.deps, self.job).await {
                let translated_err = e.translate(lang);
                tracing::error!(job_id = %self.job.id, error = %translated_err, "Import strategy finalize failed");
                self.update_job_to_failed(processed_rows, &translated_err)
                    .await;
                return Err(AppError::Internal(format!(
                    "Strategy finalize failed: {translated_err}"
                )));
            }
        }

        let status = if errors.is_empty() && skipped_rows == 0 {
            ImportJobStatus::Completed
        } else if processed_rows > 0 {
            ImportJobStatus::PartiallyFailed
        } else {
            ImportJobStatus::Failed
        };

        let final_log = Self::build_final_log(
            self.job,
            processed_rows,
            skipped_rows,
            &errors,
            &failed_rows_data,
            error_file_path_opt.as_ref(),
        );

        if self.is_chunk {
            if let Some(ref final_msg) = final_log {
                if let Some(ref q) = self.queue_repo {
                    let field = format!(
                        "chunk_{}_{}",
                        self.start_index.unwrap_or(0),
                        self.end_index.unwrap_or(0)
                    );
                    if let Err(e) = q.store_import_error(self.job.id, &field, final_msg).await {
                        tracing::error!(job_id = %self.job.id, field = %field, error = %e, "Failed to persist import error — original error may be lost");
                    }
                }
            }
            Ok(())
        } else {
            let auth = crate::diesel_pool::TenantAuthToken::system_override(self.job.workspace_id);
            if let Err(e) = self
                .import_repo
                .update_job_progress(&auth, self.job.id, processed_rows, status, final_log)
                .await
            {
                tracing::error!(job_id = %self.job.id, error = %e, "Failed to update final job status in database");
                return Err(AppError::Internal(format!(
                    "Failed to update final job status: {e}"
                )));
            }
            if let Some(file_path) = &self.job.file_path {
                if let Err(e) = tokio::fs::remove_file(file_path).await {
                    tracing::warn!(job_id = %self.job.id, file_path = %file_path, error = %e, "Failed to delete import file after completion");
                }
            }
            if errors.is_empty() && skipped_rows == 0 {
                Ok(())
            } else {
                Err(AppError::Internal(
                    "Import completed with errors".to_string(),
                ))
            }
        }
    }

    fn build_final_log(
        job: &AttendanceImportJob,
        processed_rows: i32,
        skipped_rows: i32,
        errors: &[String],
        failed_rows_data: &[FailedRow],
        error_file_path_opt: Option<&String>,
    ) -> Option<String> {
        if job.target_type == crate::models::enums::ImportTargetType::LeaveImport {
            if !failed_rows_data.is_empty() || skipped_rows > 0 || !errors.is_empty() {
                let error_count = i32::try_from(failed_rows_data.len()).unwrap_or(0);
                let json_log = serde_json::json!({
                    "success_count": processed_rows,
                    "error_count": error_count,
                    "error_file_path": error_file_path_opt,
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
                let skip_msg = format!("Skipped {skipped_rows} row(s)");
                log_opt = Some(match log_opt {
                    Some(log) => format!("{log}\n{skip_msg}"),
                    None => skip_msg,
                });
            }
            if error_file_path_opt.is_some() {
                let download_link = format!(
                    "/api/v1/workspaces/{}/attendance/import-jobs/{}/error-file",
                    job.workspace_id, job.id
                );
                let link_msg = format!("DL_LINK:{download_link}");
                log_opt = Some(match log_opt {
                    Some(log) => format!("{link_msg}\n{log}"),
                    None => link_msg,
                });
            }
            log_opt
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::*;
    use crate::models::{AttendanceImportJob, ImportJobStatus};
    use crate::repositories::import_job_repository::MockImportJobRepository;
    use crate::repositories::queue_repository::MockQueueRepository;
    use crate::worker::deps::WorkerDeps;
    use crate::worker::test_utils::MockWorkerDepsBuilder;
    use std::sync::Arc;

    // ── Test helpers ──────────────────────────────────────────────────

    fn create_xlsx_bytes(headers: &[&str], rows: &[Vec<&str>]) -> Vec<u8> {
        let mut workbook = rust_xlsxwriter::Workbook::new();
        let sheet = workbook.add_worksheet();
        for (col, header) in headers.iter().enumerate() {
            sheet.write_string(0, col as u16, *header).unwrap();
        }
        for (row_idx, row) in rows.iter().enumerate() {
            for (col_idx, val) in row.iter().enumerate() {
                sheet
                    .write_string((row_idx + 1) as u32, col_idx as u16, *val)
                    .unwrap();
            }
        }
        workbook.save_to_buffer().unwrap()
    }

    fn make_job(target_type: crate::models::enums::ImportTargetType) -> AttendanceImportJob {
        AttendanceImportJob {
            id: Uuid::new_v4(),
            workspace_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            status: ImportJobStatus::Pending,
            total_rows: 0,
            processed_rows: 0,
            error_log: None,
            file_path: None,
            created_at: chrono::Utc::now(),
            target_type,
            updated_at: chrono::Utc::now(),
        }
    }

    fn build_deps_with_workspace() -> WorkerDeps {
        let mut builder = MockWorkerDepsBuilder::new();
        builder
            .workspace_repo
            .expect_get_workspace_by_id()
            .returning(|id| {
                Box::pin(async move {
                    Ok(Some(crate::models::Workspace::new(NewWorkspaceParams {
                        id,
                        name: "Mocked Workspace".into(),
                        date_format: "YYYY-MM-DD".into(),
                        time_format: "HH:mm:ss".into(),
                        timezone: "UTC".into(),
                        theme_primary_color: "#000000".into(),
                        default_language: "vi".into(),
                        logo_url: None,
                        tagline: None,
                        holiday_overnight_strategy: "split".into(),
                        default_holiday_ot_multiplier: rust_decimal::Decimal::new(15, 1),
                        created_at: chrono::Utc::now(),
                        custom_db_url_encrypted: None,
                    })))
                })
            });
        builder.build()
    }

    // ── DummyStrategy (existing) ─────────────────────────────────────

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

    // ── ConfigurableStrategy ─────────────────────────────────────────

    struct ConfigurableStrategy {
        headers: &'static [&'static str],
        init_result: Result<(), String>,
        pre_process_result: Result<(), String>,
        map_row_result: Result<String, String>,
        process_result: Result<Option<Uuid>, String>,
        /// If set, process_row succeeds for this many calls then switches to error.
        process_fail_after: Option<usize>,
        process_call_count: usize,
    }

    impl ConfigurableStrategy {
        fn success(headers: &'static [&'static str]) -> Self {
            Self {
                headers,
                init_result: Ok(()),
                pre_process_result: Ok(()),
                map_row_result: Ok("mapped".to_string()),
                process_result: Ok(Some(Uuid::new_v4())),
                process_fail_after: None,
                process_call_count: 0,
            }
        }

        fn with_init_error(mut self, msg: &str) -> Self {
            self.init_result = Err(msg.to_string());
            self
        }

        fn with_pre_process_error(mut self, msg: &str) -> Self {
            self.pre_process_result = Err(msg.to_string());
            self
        }

        fn with_map_row_error(mut self, msg: &str) -> Self {
            self.map_row_result = Err(msg.to_string());
            self
        }

        fn with_process_skip(mut self) -> Self {
            self.process_result = Ok(None);
            self
        }

        fn with_process_error(mut self, msg: &str) -> Self {
            self.process_result = Err(msg.to_string());
            self
        }

        /// First `fail_after` calls to process_row succeed; remaining calls fail with `msg`.
        fn with_mixed_process(mut self, fail_after: usize, msg: &str) -> Self {
            self.process_fail_after = Some(fail_after);
            self.process_result = Err(msg.to_string());
            self
        }
    }

    #[async_trait]
    impl ImportStrategy for ConfigurableStrategy {
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
            self.map_row_result
                .clone()
                .map_err(crate::errors::AppError::Internal)
        }

        async fn init(
            &mut self,
            _deps: &WorkerDeps,
            _job: &AttendanceImportJob,
            _rows: &[Self::RowData],
        ) -> Result<(), crate::errors::AppError> {
            self.init_result
                .clone()
                .map_err(crate::errors::AppError::Internal)
        }

        async fn pre_process_chunk(
            &mut self,
            _deps: &WorkerDeps,
            _job: &AttendanceImportJob,
            _rows: &[Self::RowData],
        ) -> Result<(), crate::errors::AppError> {
            self.pre_process_result
                .clone()
                .map_err(crate::errors::AppError::Internal)
        }

        async fn process_row(
            &mut self,
            _deps: &WorkerDeps,
            _job: &AttendanceImportJob,
            _row: &Self::RowData,
        ) -> Result<Option<Uuid>, crate::errors::AppError> {
            if let Some(fail_after) = self.process_fail_after {
                if self.process_call_count < fail_after {
                    self.process_call_count += 1;
                    return Ok(Some(Uuid::new_v4()));
                }
                self.process_call_count += 1;
            }
            self.process_result
                .clone()
                .map_err(crate::errors::AppError::Internal)
        }
    }

    // ── Tests: original ──────────────────────────────────────────────

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

        let job = make_job(crate::models::enums::ImportTargetType::Employee);

        let file_bytes = vec![0u8; 10]; // invalid excel file
        let strategy = DummyStrategy {
            headers: &["name", "email"],
            process_result: Ok(Some(Uuid::new_v4())),
        };

        mock_import_repo
            .expect_update_job_progress()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(job.id),
                mockall::predicate::eq(0),
                mockall::predicate::eq(ImportJobStatus::Failed),
                mockall::predicate::always(),
            )
            .returning(|_, _, _, _, _| Ok(()));

        let deps = build_deps_with_workspace();

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
        assert!(result.is_err());
    }

    // ── Tests: init & pre_process failures ───────────────────────────

    #[tokio::test]
    async fn test_init_failure_marks_job_failed() {
        let mut mock_import_repo = MockImportJobRepository::new();
        mock_import_repo
            .expect_update_job_progress()
            .with(
                mockall::predicate::always(),
                mockall::predicate::always(),
                mockall::predicate::eq(0),
                mockall::predicate::eq(ImportJobStatus::Failed),
                mockall::predicate::always(),
            )
            .returning(|_, _, _, _, _| Ok(()));

        let job = make_job(crate::models::enums::ImportTargetType::Log);
        let file_bytes = create_xlsx_bytes(&["name"], &[vec!["Alice"]]);
        let strategy = ConfigurableStrategy::success(&["name"]).with_init_error("init boom");
        let deps = build_deps_with_workspace();

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
            queue_repo: None,
        };

        let result = pipeline.run().await;
        assert!(result.is_err(), "init failure should return Err");
    }

    #[tokio::test]
    async fn test_pre_process_chunk_failure_marks_job_failed() {
        let mut mock_import_repo = MockImportJobRepository::new();
        mock_import_repo
            .expect_update_job_progress()
            .with(
                mockall::predicate::always(),
                mockall::predicate::always(),
                mockall::predicate::eq(0),
                mockall::predicate::eq(ImportJobStatus::Failed),
                mockall::predicate::always(),
            )
            .returning(|_, _, _, _, _| Ok(()));

        let job = make_job(crate::models::enums::ImportTargetType::Log);
        let file_bytes = create_xlsx_bytes(&["name"], &[vec!["Alice"]]);
        let strategy =
            ConfigurableStrategy::success(&["name"]).with_pre_process_error("chunk boom");
        let deps = build_deps_with_workspace();

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
            queue_repo: None,
        };

        let result = pipeline.run().await;
        assert!(result.is_err(), "pre_process failure should return Err");
    }

    // ── Tests: row processing ────────────────────────────────────────

    #[tokio::test]
    async fn test_map_row_error_goes_to_failed_rows() {
        let mut mock_import_repo = MockImportJobRepository::new();
        mock_import_repo
            .expect_update_job_total_rows()
            .returning(|_, _, _| Ok(()));
        // Final status: errors exist + processed_rows=0 → Failed
        mock_import_repo
            .expect_update_job_progress()
            .with(
                mockall::predicate::always(),
                mockall::predicate::always(),
                mockall::predicate::eq(0), // 0 processed
                mockall::predicate::eq(ImportJobStatus::Failed),
                mockall::predicate::always(),
            )
            .returning(|_, _, _, _, _| Ok(()));

        let job = make_job(crate::models::enums::ImportTargetType::Log);
        let file_bytes = create_xlsx_bytes(&["name"], &[vec!["Alice"], vec!["Bob"]]);
        let strategy = ConfigurableStrategy::success(&["name"]).with_map_row_error("bad row");
        let deps = build_deps_with_workspace();

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
            queue_repo: None,
        };

        // Pipeline returns Err because all rows failed mapping
        let result = pipeline.run().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_process_row_skip_increments_skipped() {
        let mut mock_import_repo = MockImportJobRepository::new();
        mock_import_repo
            .expect_update_job_total_rows()
            .returning(|_, _, _| Ok(()));
        // skipped > 0, processed = 0, no errors → status = Failed
        // Pipeline returns Err because skipped_rows > 0
        mock_import_repo
            .expect_update_job_progress()
            .with(
                mockall::predicate::always(),
                mockall::predicate::always(),
                mockall::predicate::eq(0), // 0 processed
                mockall::predicate::eq(ImportJobStatus::Failed),
                mockall::predicate::always(), // contains skip message
            )
            .returning(|_, _, _, _, _| Ok(()));

        let job = make_job(crate::models::enums::ImportTargetType::Log);
        let file_bytes = create_xlsx_bytes(&["name"], &[vec!["Alice"], vec!["Bob"]]);
        let strategy = ConfigurableStrategy::success(&["name"]).with_process_skip();
        let deps = build_deps_with_workspace();

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
            queue_repo: None,
        };

        // Pipeline returns Err because skipped_rows > 0
        let result = pipeline.run().await;
        assert!(
            result.is_err(),
            "skip-only pipeline returns Err (has skips)"
        );
    }

    #[tokio::test]
    async fn test_all_rows_processed_successfully() {
        let mut mock_import_repo = MockImportJobRepository::new();
        mock_import_repo
            .expect_update_job_total_rows()
            .returning(|_, _, _| Ok(()));
        // Final call: processed_rows=2, status=Completed
        mock_import_repo
            .expect_update_job_progress()
            .with(
                mockall::predicate::always(),
                mockall::predicate::always(),
                mockall::predicate::eq(2),
                mockall::predicate::eq(ImportJobStatus::Completed),
                mockall::predicate::always(),
            )
            .returning(|_, _, _, _, _| Ok(()));

        let job = make_job(crate::models::enums::ImportTargetType::Log);
        let file_bytes = create_xlsx_bytes(&["name"], &[vec!["Alice"], vec!["Bob"]]);
        let strategy = ConfigurableStrategy::success(&["name"]);
        let deps = build_deps_with_workspace();

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
            queue_repo: None,
        };

        let result = pipeline.run().await;
        assert!(result.is_ok(), "all rows success should return Ok");
    }

    // ── Tests: chunk mode ────────────────────────────────────────────

    #[tokio::test]
    async fn test_chunk_mode_uses_queue_progress() {
        let mock_import_repo = MockImportJobRepository::new();
        let mut mock_queue_repo = MockQueueRepository::new();

        // Chunk mode should use queue_repo.increment_progress
        mock_queue_repo
            .expect_increment_progress()
            .returning(|_, _| Ok(()));

        let job = make_job(crate::models::enums::ImportTargetType::Log);
        // 12 rows → increment_progress called at row 10, then remainder 2
        let rows: Vec<Vec<&str>> = (0..12).map(|_| vec!["A"]).collect();
        let file_bytes = create_xlsx_bytes(&["name"], &rows);
        let strategy = ConfigurableStrategy::success(&["name"]);
        let deps = build_deps_with_workspace();

        let pipeline = ImportPipeline {
            strategy,
            deps,
            import_repo: &mock_import_repo,
            job: &job,
            file_bytes,
            workspace_tz: chrono_tz::UTC,
            start_index: Some(1),
            end_index: Some(12),
            is_chunk: true,
            queue_repo: Some(Arc::new(mock_queue_repo)),
        };

        let result = pipeline.run().await;
        assert!(result.is_ok(), "chunk mode success should return Ok");
    }

    #[tokio::test]
    async fn test_chunk_mode_stores_errors_in_queue() {
        let mock_import_repo = MockImportJobRepository::new();
        let mut mock_queue_repo = MockQueueRepository::new();

        mock_queue_repo
            .expect_increment_progress()
            .returning(|_, _| Ok(()));
        // When errors exist in chunk mode, store_import_error should be called
        mock_queue_repo
            .expect_store_import_error()
            .returning(|_, _, _| Ok(()));

        let job = make_job(crate::models::enums::ImportTargetType::Log);
        let file_bytes = create_xlsx_bytes(&["name"], &[vec!["Alice"]]);
        let strategy = ConfigurableStrategy::success(&["name"]).with_process_error("row failed");
        let deps = build_deps_with_workspace();

        let pipeline = ImportPipeline {
            strategy,
            deps,
            import_repo: &mock_import_repo,
            job: &job,
            file_bytes,
            workspace_tz: chrono_tz::UTC,
            start_index: Some(1),
            end_index: Some(1),
            is_chunk: true,
            queue_repo: Some(Arc::new(mock_queue_repo)),
        };

        let result = pipeline.run().await;
        // Chunk mode always returns Ok
        assert!(result.is_ok());
    }

    // ── Tests: partial failure ────────────────────────────────────────

    #[tokio::test]
    async fn test_partial_failure_marks_partially_failed() {
        let mut mock_import_repo = MockImportJobRepository::new();
        mock_import_repo
            .expect_update_job_total_rows()
            .returning(|_, _, _| Ok(()));
        // Final: processed_rows=1, 1 error → PartiallyFailed
        mock_import_repo
            .expect_update_job_progress()
            .with(
                mockall::predicate::always(),
                mockall::predicate::always(),
                mockall::predicate::eq(1), // 1 processed
                mockall::predicate::eq(ImportJobStatus::PartiallyFailed),
                mockall::predicate::always(),
            )
            .returning(|_, _, _, _, _| Ok(()));

        let job = make_job(crate::models::enums::ImportTargetType::Log);
        let file_bytes = create_xlsx_bytes(&["name"], &[vec!["Alice"], vec!["Bob"]]);
        // 1st row succeeds, 2nd row fails
        let strategy = ConfigurableStrategy::success(&["name"]).with_mixed_process(1, "row failed");
        let deps = build_deps_with_workspace();

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
            queue_repo: None,
        };

        let result = pipeline.run().await;
        assert!(result.is_err(), "partial failure should return Err");
    }

    #[tokio::test]
    async fn test_leave_import_error_count_no_double_count() {
        let mut mock_import_repo = MockImportJobRepository::new();
        mock_import_repo
            .expect_update_job_total_rows()
            .returning(|_, _, _| Ok(()));
        // Final: 1 processed, 1 error → PartiallyFailed
        mock_import_repo
            .expect_update_job_progress()
            .with(
                mockall::predicate::always(),
                mockall::predicate::always(),
                mockall::predicate::eq(1),
                mockall::predicate::eq(ImportJobStatus::PartiallyFailed),
                mockall::predicate::function(|log: &Option<String>| {
                    if let Some(log) = log {
                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(log) {
                            let error_count = parsed["error_count"].as_i64().unwrap_or(-1);
                            let success_count = parsed["success_count"].as_i64().unwrap_or(-1);
                            // 1 error row, 1 success — no double-counting
                            return error_count == 1 && success_count == 1;
                        }
                    }
                    false
                }),
            )
            .returning(|_, _, _, _, _| Ok(()));

        let job = make_job(crate::models::enums::ImportTargetType::LeaveImport);
        let file_bytes = create_xlsx_bytes(
            &["name", "employee_id"],
            &[vec!["Alice", "E1"], vec!["Bob", "E2"]],
        );
        // 1st row succeeds, 2nd row fails
        let strategy = ConfigurableStrategy::success(&["name", "employee_id"])
            .with_mixed_process(1, "row failed");
        let deps = build_deps_with_workspace();

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
            queue_repo: None,
        };

        let result = pipeline.run().await;
        assert!(result.is_err(), "partial failure should return Err");
    }
}
