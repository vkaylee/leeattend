use super::*;
use crate::utils::tz::local_to_utc;

impl PayrollService {
    #[tracing::instrument(skip_all)]
    #[allow(clippy::too_many_lines)]
    pub async fn calculate_batch_payslips(
        &self,
        employment_ids: Vec<Uuid>,
        workspace_id: Uuid,
        year: i32,
        month: u32,
    ) -> AppResult<crate::models::BatchCalculatePayrollResponse> {
        // ── 0. Input validation ──────────────────────────────────────
        if employment_ids.is_empty() {
            return Err(AppError::validation(I18nKey::Validation));
        }
        const MAX_BATCH_EMPLOYEES: usize = 500;
        if employment_ids.len() > MAX_BATCH_EMPLOYEES {
            return Err(AppError::validation_args(
                I18nKey::Validation,
                vec![(
                    "details",
                    format!(
                        "Batch size {} exceeds maximum {MAX_BATCH_EMPLOYEES}",
                        employment_ids.len()
                    ),
                )],
            ));
        }

        // Deduplicate but preserve order for deterministic results
        let mut seen = std::collections::HashSet::new();
        let mut deduped_ids = Vec::with_capacity(employment_ids.len());
        for id in employment_ids {
            if seen.insert(id) {
                deduped_ids.push(id);
            }
        }
        let employment_ids = deduped_ids;

        // ── Phase 1: Workspace-level pre-fetch ──────────────────────
        let workspace = self
            .workspace_repo
            .get_workspace_by_id(workspace_id)
            .await?
            .ok_or_else(|| AppError::not_found(I18nKey::WorkspaceNotFound))?;
        let tz: Tz = crate::utils::tz::parse_workspace_tz(workspace.timezone());

        let (start_date, end_date) = month_boundaries(year, month)?;
        let start_dt = local_to_utc(&tz, start_date, 0, 0, 0)
            .map_err(|e| AppError::Internal(format!("Failed to convert start date to UTC: {e}")))?;
        let end_dt = local_to_utc(&tz, end_date, 23, 59, 59)
            .map_err(|e| AppError::Internal(format!("Failed to convert end date to UTC: {e}")))?;
        let token = &crate::diesel_pool::TenantAuthToken::system_override(workspace_id);

        let condition_type_catalog = PolicyResolverService::load_condition_type_catalog(
            self.condition_type_repo.as_ref(),
            token,
        )
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch condition type catalog: {e}")))?;

        let overtime_data = self.fetch_overtime_data(workspace_id).await;
        let holidays_map = self
            .fetch_holidays(token, start_date, end_date, workspace_id)
            .await;

        let roster_data = self
            .roster_provider
            .get_roster_data(workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to fetch roster data: {e}")))?;

        let ws_payroll_settings_history = self
            .payroll_repo
            .get_payroll_settings_history(workspace_id)
            .await
            .map_err(|e| {
                AppError::Internal(format!("Failed to fetch payroll settings history: {e}"))
            })?;

        // month-end payroll settings are optional — absence just means no OT cap
        let month_end_payroll_settings = self
            .payroll_repo
            .get_payroll_settings_at(workspace_id, end_date)
            .await
            .unwrap_or_else(|e| {
                tracing::warn!(workspace_id = %workspace_id, error = %e, "Failed to fetch month end payroll settings, continuing without OT cap");
                None
            });

        let active_policies_with_tiers = self
            .fetch_incentive_policies_with_tiers(workspace_id, start_date, end_date)
            .await;

        let salary_base_groups = self
            .salary_base_repo
            .get_groups(workspace_id)
            .await
            .unwrap_or_else(|e| {
                tracing::warn!(
                    workspace_id = %workspace_id,
                    year = year,
                    month = month,
                    error = %e,
                    "calculate_batch_payslips: get_groups failed — continuing without salary base groups"
                );
                Vec::new()
            });

        // ── Phase 2: Process chunks ─────────────────────────────────
        let shared_data = Arc::new(ChunkSharedData {
            workspace: workspace.clone(),
            tz,
            start_date,
            end_date,
            start_dt,
            end_dt,
            token: *token,
            condition_type_catalog: Arc::new(condition_type_catalog),
            overtime_data: Arc::new(overtime_data),
            holidays_map: Arc::new(holidays_map),
            roster_data: Arc::clone(&roster_data),
            ws_payroll_settings_history: Arc::new(ws_payroll_settings_history),
            month_end_payroll_settings: Arc::new(month_end_payroll_settings),
            active_policies_with_tiers: Arc::new(active_policies_with_tiers),
            salary_base_groups: Arc::new(salary_base_groups),
        });

        let mut results = Vec::new();
        for chunk in employment_ids.chunks(50) {
            let chunk_results = self
                .process_payroll_chunk(chunk.to_vec(), &shared_data, workspace_id)
                .await;
            results.extend(chunk_results);
        }

        Ok(crate::models::BatchCalculatePayrollResponse { results })
    }
}
