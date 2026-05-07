use async_trait::async_trait;
use chrono_tz::Tz;
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

use crate::models::{AttendanceImportJob, ImportJobStatus};
use crate::repositories::import_job_repository::ImportJobRepository;

#[async_trait]
pub trait ImportStrategy: Send + Sync {
    type RowData: Send + Sync;

    /// Expected headers in the Excel file
    fn expected_headers(&self) -> &'static [&'static str];

    /// Map a row of data to the generic type `RowData`
    fn map_row(
        &mut self,
        row_data: &HashMap<String, String>,
        current_line: usize,
        workspace_tz: Tz,
    ) -> Result<Self::RowData, String>;

    /// Process a single valid row. Returns Ok(Some(id)) for success, Ok(None) for skipped, Err(reason) for failure.
    async fn process_row(
        &mut self,
        pool: &PgPool,
        job: &AttendanceImportJob,
        row: &Self::RowData,
    ) -> Result<Option<Uuid>, String>;

    /// Optional hook to initialize caching or do pre-flight checks
    async fn init(&mut self, _pool: &PgPool, _job: &AttendanceImportJob) -> Result<(), String> {
        Ok(())
    }
}

pub struct ImportPipeline<'a, S: ImportStrategy> {
    pub strategy: S,
    pub pool: PgPool,
    pub import_repo: &'a dyn ImportJobRepository,
    pub job: &'a AttendanceImportJob,
    pub file_bytes: Vec<u8>,
    pub workspace_tz: Tz,
}

impl<S: ImportStrategy> ImportPipeline<'_, S> {
    #[allow(clippy::too_many_lines)]
    pub async fn run(mut self) -> Result<(), ()> {
        if let Err(e) = self.strategy.init(&self.pool, self.job).await {
            let _ = self
                .import_repo
                .update_job_progress(self.job.id, 0, ImportJobStatus::Failed, Some(e))
                .await;
            return Err(());
        }

        let mut errors = Vec::new();
        let mut failed_rows_data: Vec<(HashMap<String, String>, String)> = Vec::new();
        let mut pending_rows: Vec<(S::RowData, HashMap<String, String>)> = Vec::new();

        match crate::utils::file_parser::extract_mapped_rows(
            &self.file_bytes,
            self.strategy.expected_headers(),
        ) {
            Ok(rows) => {
                for (current_line, row_data) in rows {
                    match self.strategy.map_row(&row_data, current_line, self.workspace_tz) {
                        Ok(mapped_data) => {
                            pending_rows.push((mapped_data, row_data));
                        }
                        Err(e) => {
                            errors.push(e.clone());
                            failed_rows_data.push((row_data, e));
                        }
                    }
                }
            }
            Err(e) => {
                errors.push(e);
            }
        }

        let total_rows = pending_rows.len() as i32 + failed_rows_data.len() as i32;
        let _ = self
            .import_repo
            .update_job_total_rows(self.job.id, total_rows)
            .await;

        let mut processed_rows = 0;
        let mut skipped_rows = 0;

        for (row_data_parsed, raw_row_data) in pending_rows {
            match self
                .strategy
                .process_row(&self.pool, self.job, &row_data_parsed)
                .await
            {
                Ok(Some(_id)) => {
                    processed_rows += 1;
                }
                Ok(None) => {
                    skipped_rows += 1;
                    failed_rows_data.push((raw_row_data, "Bỏ qua (Trùng lặp hoặc đã tồn tại)".to_string()));
                }
                Err(e) => {
                    errors.push(e.clone());
                    failed_rows_data.push((raw_row_data, e));
                }
            }

            if (processed_rows + skipped_rows) % 10 == 0 {
                let _ = self
                    .import_repo
                    .update_job_progress(
                        self.job.id,
                        processed_rows,
                        ImportJobStatus::Processing,
                        None,
                    )
                    .await;
            }
        }

        let status = if (errors.is_empty() && skipped_rows == 0) || processed_rows > 0 {
            ImportJobStatus::Completed
        } else {
            ImportJobStatus::Failed
        };

        let mut final_log = if errors.is_empty() {
            None
        } else {
            Some(errors.join("\n"))
        };

        if skipped_rows > 0 {
            let skip_msg = format!("Đã bỏ qua {skipped_rows} dòng");
            final_log = Some(match final_log {
                Some(log) => format!("{log}\n{skip_msg}"),
                None => skip_msg,
            });
        }

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
                let _ = worksheet.write_string_with_format(0, col as u16, *header, &header_format);
            }

            for (row_idx, (data, err)) in failed_rows_data.iter().enumerate() {
                let row = (row_idx + 1) as u32;
                for (col, header) in self.strategy.expected_headers().iter().enumerate() {
                    let _ = worksheet.write_string(
                        row,
                        col as u16,
                        data.get(*header).unwrap_or(&String::new()),
                    );
                }
                // Write error reason in the last column
                let _ = worksheet.write_string(row, all_headers.len() as u16 - 1, err);
            }

            let base_dir = crate::utils::get_storage_dir();
            let error_file_path = format!("{base_dir}/imports/error_{}.xlsx", self.job.id);
            if workbook.save(&error_file_path).is_ok() {
                let download_link = format!(
                    "/api/v1/workspaces/{}/attendance/import-jobs/{}/error-file",
                    self.job.workspace_id, self.job.id
                );
                let link_msg = format!("DL_LINK:{download_link}");
                final_log = Some(match final_log {
                    Some(log) => format!("{link_msg}\n{log}"),
                    None => link_msg,
                });
            }
        }

        let _ = self
            .import_repo
            .update_job_progress(self.job.id, processed_rows, status, final_log)
            .await;

        if let Some(file_path) = &self.job.file_path {
            let _ = tokio::fs::remove_file(file_path).await;
        }

        Ok(())
    }
}
