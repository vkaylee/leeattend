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

const datePattern = /^\d{4}-\d{2}-\d{2}$/;
const uuid = z.string().uuid();

export type DashboardPeriod = (typeof dashboardPeriods)[number];
export type DashboardComparison = (typeof dashboardComparisons)[number];

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

export interface DashboardMetricsFilters {
  comparison: DashboardComparison;
  departmentIds: string[];
  endDate?: string;
  locationIds: string[];
  period: DashboardPeriod;
  startDate?: string;
}

export const DEFAULT_DASHBOARD_METRICS_FILTERS: DashboardMetricsFilters = {
  comparison: "none",
  departmentIds: [],
  locationIds: [],
  period: "this_month",
};

const periodSchema = z.enum(dashboardPeriods);
const comparisonSchema = z.enum(dashboardComparisons);

function sortedUniqueIds(values: readonly string[]) {
  return [...new Set(values.filter((value) => uuid.safeParse(value).success))].sort();
}

function validDate(value: string | undefined) {
  return value && datePattern.test(value) ? value : undefined;
}

export function canonicalizeDashboardMetricsFilters(
  filters: Partial<DashboardMetricsFilters>,
): DashboardMetricsFilters {
  const parsedPeriod = periodSchema.safeParse(filters.period);
  const parsedComparison = comparisonSchema.safeParse(filters.comparison);
  const period = parsedPeriod.success
    ? parsedPeriod.data
    : DEFAULT_DASHBOARD_METRICS_FILTERS.period;
  const comparison = parsedComparison.success
    ? parsedComparison.data
    : DEFAULT_DASHBOARD_METRICS_FILTERS.comparison;
  const startDate = validDate(filters.startDate);
  const endDate = validDate(filters.endDate);

  if (period === "custom" && (!startDate || !endDate || endDate < startDate)) {
    return {
      ...DEFAULT_DASHBOARD_METRICS_FILTERS,
      comparison,
      departmentIds: sortedUniqueIds(filters.departmentIds ?? []),
      locationIds: sortedUniqueIds(filters.locationIds ?? []),
    };
  }

  return {
    comparison,
    departmentIds: sortedUniqueIds(filters.departmentIds ?? []),
    endDate: period === "custom" ? endDate : undefined,
    locationIds: sortedUniqueIds(filters.locationIds ?? []),
    period,
    startDate: period === "custom" ? startDate : undefined,
  };
}

export function parseDashboardMetricsFilters(search: string): DashboardMetricsFilters {
  const params = new URLSearchParams(search);
  return canonicalizeDashboardMetricsFilters({
    comparison: parseDashboardComparison(params.get("comparison")),
    departmentIds: params.getAll("department_ids"),
    endDate: params.get("end_date") ?? undefined,
    locationIds: params.getAll("location_ids"),
    period: parseDashboardPeriod(params.get("period")),
    startDate: params.get("start_date") ?? undefined,
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
  canonical.departmentIds.forEach((id) => params.append("department_ids", id));
  canonical.locationIds.forEach((id) => params.append("location_ids", id));
  return params.toString();
}

export function dashboardMetricsQuery(filters: DashboardMetricsFilters): string {
  const query = serializeDashboardMetricsFilters(filters);
  return query ? `?${query}` : "";
}
