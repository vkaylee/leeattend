export {
  DEFAULT_DASHBOARD_METRICS_FILTERS,
  dashboardComparisons,
  dashboardPeriods,
  dashboardRiskThresholds,
  parseDashboardComparison,
  parseDashboardPeriod,
  parseDashboardRiskThreshold,
  parseDashboardTopN,
  type DashboardComparison,
  type DashboardMetricsFilters,
  type DashboardPeriod,
  type DashboardRiskThreshold,
} from "./dashboard-metrics-filter-types";

import { canonicalFilterParts, isInvalidCustomRange } from "./dashboard-metrics-filter-helpers";
import {
  DEFAULT_DASHBOARD_METRICS_FILTERS,
  parseDashboardComparison,
  parseDashboardPeriod,
  parseDashboardRiskThreshold,
  parseDashboardTopN,
  type DashboardMetricsFilters,
} from "./dashboard-metrics-filter-types";

export function canonicalizeDashboardMetricsFilters(
  filters: Partial<DashboardMetricsFilters>,
): DashboardMetricsFilters {
  const parts = canonicalFilterParts(filters);
  if (isInvalidCustomRange(parts)) {
    return {
      ...DEFAULT_DASHBOARD_METRICS_FILTERS,
      comparison: parts.comparison,
      ...parts.dimensions,
      riskThreshold: parts.riskThreshold,
      topN: parts.topN,
    };
  }

  return {
    ...parts.dimensions,
    comparison: parts.comparison,
    endDate: parts.period === "custom" ? parts.endDate : undefined,
    period: parts.period,
    riskThreshold: parts.riskThreshold,
    startDate: parts.period === "custom" ? parts.startDate : undefined,
    topN: parts.topN,
  };
}

function queryValues(params: URLSearchParams, key: string): string[] {
  return params.getAll(key);
}

export function parseDashboardMetricsFilters(search: string): DashboardMetricsFilters {
  const params = new URLSearchParams(search);
  return canonicalizeDashboardMetricsFilters({
    businessUnitIds: queryValues(params, "business_unit_ids"),
    comparison: parseDashboardComparison(params.get("comparison")),
    departmentIds: queryValues(params, "department_ids"),
    employeeStatuses: queryValues(params, "employee_statuses"),
    employmentTypes: queryValues(params, "employment_types"),
    endDate: params.get("end_date") ?? undefined,
    locationIds: queryValues(params, "location_ids"),
    managerIds: queryValues(params, "manager_ids"),
    period: parseDashboardPeriod(params.get("period")),
    riskThreshold: parseDashboardRiskThreshold(params.get("risk_threshold")),
    shiftIds: queryValues(params, "shift_ids"),
    startDate: params.get("start_date") ?? undefined,
    topN: parseDashboardTopN(params.get("top_n")),
  });
}

export function serializeDashboardMetricsFilters(
  filters: Partial<DashboardMetricsFilters>,
): string {
  const canonical = canonicalizeDashboardMetricsFilters(filters);
  const params = new URLSearchParams();
  params.set("period", canonical.period);
  if (canonical.comparison !== DEFAULT_DASHBOARD_METRICS_FILTERS.comparison) {
    params.set("comparison", canonical.comparison);
  }
  if (canonical.startDate) params.set("start_date", canonical.startDate);
  if (canonical.endDate) params.set("end_date", canonical.endDate);
  appendValues(params, "department_ids", canonical.departmentIds);
  appendValues(params, "location_ids", canonical.locationIds);
  appendValues(params, "business_unit_ids", canonical.businessUnitIds);
  appendValues(params, "manager_ids", canonical.managerIds);
  appendValues(params, "employment_types", canonical.employmentTypes);
  appendValues(params, "employee_statuses", canonical.employeeStatuses);
  appendValues(params, "shift_ids", canonical.shiftIds);
  if (canonical.topN !== DEFAULT_DASHBOARD_METRICS_FILTERS.topN) {
    params.set("top_n", String(canonical.topN));
  }
  if (canonical.riskThreshold !== DEFAULT_DASHBOARD_METRICS_FILTERS.riskThreshold) {
    params.set("risk_threshold", canonical.riskThreshold);
  }
  return params.toString();
}

function appendValues(params: URLSearchParams, key: string, values: readonly string[]): void {
  values.forEach((value) => {
    params.append(key, value);
  });
}

export function dashboardMetricsQuery(filters: DashboardMetricsFilters): string {
  const query = serializeDashboardMetricsFilters(filters);
  return query ? `?${query}` : "";
}
