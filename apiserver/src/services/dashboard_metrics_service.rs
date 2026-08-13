use crate::errors::AppResult;
use crate::models::{
    DashboardActionsSummary, DashboardDataQuality, DashboardMetric, DashboardMetricContext,
    DashboardMetricDirection, DashboardMetricHealth, DashboardMetricScope, DashboardMetricStatus,
    DashboardMetricUnit, DashboardMetricsQuery, DashboardMetricsResponse, DashboardReportingPeriod,
    DashboardWorkspaceContext, NormalizedDashboardMetricsQuery,
};
use crate::repositories::WorkspaceRepository;
use crate::utils::tz;
use chrono::{DateTime, Utc};
use std::collections::BTreeMap;
use std::sync::Arc;
use uuid::Uuid;

#[allow(clippy::implicit_hasher, clippy::too_many_lines)]
pub async fn build_dashboard_metrics(
    workspace_repo: &Arc<dyn WorkspaceRepository>,
    workspace_id: Uuid,
    query: &DashboardMetricsQuery,
    now: DateTime<Utc>,
    #[allow(clippy::implicit_hasher)] managed_department_ids: Option<
        &std::collections::HashSet<Uuid>,
    >,
) -> AppResult<DashboardMetricsResponse> {
    let workspace = workspace_repo
        .get_workspace_by_id(workspace_id)
        .await?
        .ok_or_else(|| {
            crate::errors::AppError::not_found(crate::utils::i18n::I18nKey::WorkspaceNotFound)
        })?;
    let workspace_tz = tz::validate_tz(workspace.timezone())?;
    let normalized = query.normalize(tz::utc_now_date_in_tz(&workspace_tz))?;
    validate_department_scope(&normalized, managed_department_ids)?;

    let period = DashboardReportingPeriod::new(
        normalized.period.clone(),
        normalized.start_date,
        normalized.end_date,
        workspace_tz.name(),
        now,
    );
    let scope_department_ids = if normalized.department_ids.is_empty() {
        managed_department_ids
            .map(|ids| ids.iter().copied().collect())
            .unwrap_or_default()
    } else {
        normalized.department_ids.clone()
    };
    let scope_kind = if scope_department_ids.is_empty() {
        "workspace"
    } else {
        "department"
    };
    let mut filters = BTreeMap::new();
    filters.insert(
        "department_ids".to_string(),
        join_uuids(&normalized.department_ids),
    );
    filters.insert(
        "location_ids".to_string(),
        join_uuids(&normalized.location_ids),
    );
    filters.insert(
        "business_unit_ids".to_string(),
        join_uuids(&normalized.business_unit_ids),
    );
    filters.insert(
        "manager_ids".to_string(),
        join_uuids(&normalized.manager_ids),
    );
    filters.insert(
        "employment_types".to_string(),
        normalized.employment_types.join(","),
    );
    filters.insert(
        "employee_statuses".to_string(),
        normalized.employee_statuses.join(","),
    );
    filters.insert("shift_ids".to_string(), join_uuids(&normalized.shift_ids));
    filters.insert("top_n".to_string(), normalized.top_n.to_string());
    filters.insert(
        "risk_threshold".to_string(),
        normalized.risk_threshold.clone(),
    );

    let omitted_dimensions = vec![
        "business_unit".to_string(),
        "manager".to_string(),
        "employment_type".to_string(),
        "employee_status".to_string(),
        "shift".to_string(),
    ];
    let context = DashboardMetricContext::new(
        period,
        normalized.comparison,
        filters,
        DashboardMetricScope::new(
            scope_kind,
            scope_department_ids,
            normalized.location_ids,
            omitted_dimensions,
        ),
    );
    let freshness = crate::models::DashboardMetricFreshness::new(now, Some(0));
    let metric = DashboardMetric::new(
        "executive_overview",
        None,
        DashboardMetricUnit::Count,
        None,
        None,
        DashboardMetricStatus::DataGap,
        DashboardMetricHealth::DataGap,
        Some("source_unavailable".to_string()),
        "executive-overview.v1",
        DashboardMetricDirection::Neutral,
        freshness,
        vec!["no_authoritative_executive_source".to_string()],
        None,
        None,
    );

    Ok(DashboardMetricsResponse::new(
        "executive-dashboard.v1",
        DashboardWorkspaceContext::new(workspace_tz.name(), "XXX"),
        context,
        vec![metric],
        Vec::new(),
        DashboardActionsSummary::new(0, 0, Vec::new()),
        DashboardDataQuality::new("data_gap", Vec::new()),
        now,
    ))
}

fn validate_department_scope(
    query: &NormalizedDashboardMetricsQuery,
    managed_department_ids: Option<&std::collections::HashSet<Uuid>>,
) -> AppResult<()> {
    if let Some(allowed) = managed_department_ids {
        if query.department_ids.iter().any(|id| !allowed.contains(id)) {
            return Err(crate::errors::AppError::forbidden(
                crate::utils::i18n::I18nKey::Forbidden,
            ));
        }
    }
    Ok(())
}

fn join_uuids(values: &[Uuid]) -> String {
    values
        .iter()
        .map(Uuid::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
