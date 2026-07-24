//! Detail report merge — aggregates batch [`DailyAttendanceReport`] results
//! into a final Excel file with timezone grouping and custom field support.

use crate::errors::{AppError, AppResult};
use crate::models::DailyAttendanceReport;
use crate::services::import_messages::ExcelReportMessages;
use crate::services::ExcelWriter;
use chrono::{NaiveDate, Utc};
use std::collections::HashMap;

impl super::MergeHandler {
    /// Merge detail results from all batches.
    #[allow(clippy::too_many_lines)]
    pub(super) async fn merge_detail_results(
        batch_results: &[(String, String)],
        workspace_name: &str,
        date_range: (NaiveDate, NaiveDate),
        parent_job_id: uuid::Uuid,
        timezone: &str,
        work_unit_minutes: i32,
        msgs: &ExcelReportMessages,
        deps: &crate::worker::WorkerDeps,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> AppResult<(Vec<u8>, String, usize)> {
        let mut all_reports: Vec<DailyAttendanceReport> = Vec::new();

        for (batch_key, json) in batch_results {
            let batch: Vec<DailyAttendanceReport> = serde_json::from_str(json)
                .map_err(|e| format!("Parse error for {batch_key}: {e}"))?;
            all_reports.extend(batch);
        }

        // Sort by employee name, then by date
        all_reports.sort_by(|a, b| {
            a.employee_name
                .cmp(&b.employee_name)
                .then(a.date.cmp(&b.date))
        });

        tracing::info!(total_records = all_reports.len(), "Merged detail results");

        // Fetch custom field definitions and values
        let all_definitions = deps
            .custom_field_repo
            .list_definitions(auth_token, None)
            .await?;
        let exportable_definitions: Vec<_> = all_definitions
            .into_iter()
            .filter(crate::models::CustomFieldDefinition::is_exportable)
            .collect();

        let mut cf_headers = Vec::new();
        let mut cf_values_map = HashMap::new();

        if !exportable_definitions.is_empty() && !all_reports.is_empty() {
            let mut entity_ids = Vec::new();
            for r in &all_reports {
                entity_ids.push(r.employee_id);
                entity_ids.push(r.employment_id);
            }

            let values_map = deps
                .custom_field_repo
                .get_values_for_entities(auth_token, &entity_ids)
                .await?;

            cf_headers = exportable_definitions
                .iter()
                .map(|d| d.field_label().to_string())
                .collect();

            for report in &all_reports {
                let mut row_values = Vec::new();
                for def in &exportable_definitions {
                    let lookup_id = match def.entity_type() {
                        "employee" => Some(report.employee_id),
                        "employment" => Some(report.employment_id),
                        _ => None,
                    };

                    let mut found_val_str = String::new();
                    if let Some(id) = lookup_id {
                        if let Some(pairs) = values_map.get(&id) {
                            if let Some((val, _)) = pairs.iter().find(|(_, d)| d.id() == def.id()) {
                                found_val_str = match def.field_type() {
                                    "text" | "select" | "url" | "email" | "phone" => {
                                        val.value_text().unwrap_or("").to_string()
                                    }
                                    "number" => val
                                        .value_number()
                                        .map(|n| n.to_string())
                                        .unwrap_or_default(),
                                    "date" => {
                                        val.value_date().map(|d| d.to_string()).unwrap_or_default()
                                    }
                                    "boolean" => {
                                        val.value_bool().map(|b| b.to_string()).unwrap_or_default()
                                    }
                                    "multiselect" => val
                                        .value_json()
                                        .map(ToString::to_string)
                                        .unwrap_or_default(),
                                    _ => String::new(),
                                };
                            }
                        }
                    }
                    row_values.push(found_val_str);
                }
                cf_values_map.insert(report.employee_id, row_values);
            }
        }

        let total_records = all_reports.len();
        let ws_name = workspace_name.to_string();
        let tz = timezone.to_string();
        let messages = *msgs;

        // Generate Excel in thread pool
        let excel_data = tokio::task::spawn_blocking(move || {
            ExcelWriter::write_detail(
                &all_reports,
                &ws_name,
                date_range,
                &tz,
                work_unit_minutes,
                &messages,
                &cf_headers,
                &cf_values_map,
            )
        })
        .await
        .map_err(|e| AppError::Internal(format!("Excel generation join error: {e}")))??;

        let file_name = format!(
            "attendance_detail_{}_{}.xlsx",
            parent_job_id,
            Utc::now().format("%Y%m%d_%H%M%S")
        );

        Ok((excel_data, file_name, total_records))
    }
}
