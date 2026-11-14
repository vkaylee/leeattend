import { z } from "zod";

export const dashboardPeriods = [
  "today",
  "this_week",
  "this_month",
  "this_quarter",
  "year_to_date",
  "custom",
] as const;

export const dashboardComparisons = [
  "none",
  "previous_period",
  "same_period_last_year",
  "target",
  "budget",
  "benchmark",
] as const;

export const dashboardRiskThresholds = ["info", "watch", "high", "critical"] as const;

const datePattern = /^\d{4}-\d{2}-\d{2}$/;
const uuid = z.uuid();
const periodSchema = z.enum(dashboardPeriods);
const comparisonSchema = z.enum(dashboardComparisons);
const riskThresholdSchema = z.enum(dashboardRiskThresholds);
const topNSchema = z.number().int().min(1).max(100);

export type DashboardPeriod = (typeof dashboardPeriods)[number];
export type DashboardComparison = (typeof dashboardComparisons)[number];
export type DashboardRiskThreshold = (typeof dashboardRiskThresholds)[number];

export function parseDashboardPeriod(value: string | null | undefined): DashboardPeriod | undefined {
  const parsed = periodSchema.safeParse(value);
  return parsed.success ? parsed.data : undefined;
}

export function parseDashboardComparison(
  value: string | null | undefined,
): DashboardComparison | undefined {
  const parsed = comparisonSchema.safeParse(value);
  return parsed.success ? parsed.data : undefined;
}

export function parseDashboardRiskThreshold(
  value: string | null | undefined,
): DashboardRiskThreshold | undefined {
  const parsed = riskThresholdSchema.safeParse(value);
  return parsed.success ? parsed.data : undefined;
}

export function parseDashboardTopN(value: string | null | undefined): number | undefined {
  if (!value || !/^\d+$/.test(value)) return undefined;
  const parsed = topNSchema.safeParse(Number(value));
  return parsed.success ? parsed.data : undefined;
}

export interface DashboardMetricsFilters {
  businessUnitIds: string[];
  comparison: DashboardComparison;
  departmentIds: string[];
  employeeStatuses: string[];
  employmentTypes: string[];
  endDate?: string;
  locationIds: string[];
  managerIds: string[];
  period: DashboardPeriod;
  riskThreshold: DashboardRiskThreshold;
  shiftIds: string[];
  startDate?: string;
  topN: number;
}

export const DEFAULT_DASHBOARD_METRICS_FILTERS: DashboardMetricsFilters = {
  businessUnitIds: [],
  comparison: "none",
  departmentIds: [],
  employeeStatuses: [],
  employmentTypes: [],
  locationIds: [],
  managerIds: [],
  period: "this_month",
  riskThreshold: "watch",
  shiftIds: [],
  topN: 10,
};

function sortedUniqueIds(values: readonly string[]) {
  return [
    ...new Set(
      values.flatMap((value) => value.split(",")).flatMap((value) => {
        const normalized = value.trim().toLowerCase();
        return uuid.safeParse(normalized).success ? [normalized] : [];
      }),
    ),
  ].sort();
}

function sortedUniqueValues(values: readonly string[]) {
  return [
    ...new Set(
      values
        .flatMap((value) => value.split(","))
        .map((value) => value.trim())
        .filter(Boolean),
    ),
  ].sort();
}

function validDate(value: string | undefined) {
  if (!value || !datePattern.test(value)) return undefined;
  const [year, month, day] = value.split("-").map(Number);
  const leapYear = year % 4 === 0 && (year % 100 !== 0 || year % 400 === 0);
  const daysInMonth = [
    31,
    leapYear ? 29 : 28,
    31,
    30,
    31,
    30,
    31,
    31,
    30,
    31,
    30,
    31,
  ][month - 1];
  return year > 0 && daysInMonth && day >= 1 && day <= daysInMonth ? value : undefined;
}

export function canonicalizeDashboardMetricsFilters(
  filters: Partial<DashboardMetricsFilters>,
): DashboardMetricsFilters {
  const periodResult = periodSchema.safeParse(filters.period);
  const comparisonResult = comparisonSchema.safeParse(filters.comparison);
  const riskThresholdResult = riskThresholdSchema.safeParse(filters.riskThreshold);
  const topNResult = topNSchema.safeParse(filters.topN);
  const period = periodResult.success
    ? periodResult.data
    : DEFAULT_DASHBOARD_METRICS_FILTERS.period;
  const comparison = comparisonResult.success
    ? comparisonResult.data
    : DEFAULT_DASHBOARD_METRICS_FILTERS.comparison;
  const riskThreshold = riskThresholdResult.success
    ? riskThresholdResult.data
    : DEFAULT_DASHBOARD_METRICS_FILTERS.riskThreshold;
  const topN = topNResult.success ? topNResult.data : DEFAULT_DASHBOARD_METRICS_FILTERS.topN;
  const startDate = validDate(filters.startDate);
  const endDate = validDate(filters.endDate);
  const dimensions = {
    businessUnitIds: sortedUniqueIds(filters.businessUnitIds ?? []),
    departmentIds: sortedUniqueIds(filters.departmentIds ?? []),
    employeeStatuses: sortedUniqueValues(filters.employeeStatuses ?? []),
    employmentTypes: sortedUniqueValues(filters.employmentTypes ?? []),
    locationIds: sortedUniqueIds(filters.locationIds ?? []),
    managerIds: sortedUniqueIds(filters.managerIds ?? []),
    shiftIds: sortedUniqueIds(filters.shiftIds ?? []),
  };

  if (period === "custom" && (!startDate || !endDate || endDate < startDate)) {
    return {
      ...DEFAULT_DASHBOARD_METRICS_FILTERS,
      comparison,
      ...dimensions,
      riskThreshold,
      topN,
    };
  }

  return {
    ...dimensions,
    comparison,
    endDate: period === "custom" ? endDate : undefined,
    period,
    riskThreshold,
    startDate: period === "custom" ? startDate : undefined,
    topN,
  };
}

function queryValues(params: URLSearchParams, key: string) {
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
  canonical.departmentIds.forEach((id) => { params.append("department_ids", id); });
  canonical.locationIds.forEach((id) => { params.append("location_ids", id); });
  canonical.businessUnitIds.forEach((id) => { params.append("business_unit_ids", id); });
  canonical.managerIds.forEach((id) => { params.append("manager_ids", id); });
  canonical.employmentTypes.forEach((value) => { params.append("employment_types", value); });
  canonical.employeeStatuses.forEach((value) => { params.append("employee_statuses", value); });
  canonical.shiftIds.forEach((id) => { params.append("shift_ids", id); });
  if (canonical.topN !== DEFAULT_DASHBOARD_METRICS_FILTERS.topN) {
    params.set("top_n", String(canonical.topN));
  }
  if (canonical.riskThreshold !== DEFAULT_DASHBOARD_METRICS_FILTERS.riskThreshold) {
    params.set("risk_threshold", canonical.riskThreshold);
  }
  return params.toString();
}

export function dashboardMetricsQuery(filters: DashboardMetricsFilters): string {
  const query = serializeDashboardMetricsFilters(filters);
  return query ? `?${query}` : "";
}
