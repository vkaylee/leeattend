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

export { comparisonSchema, periodSchema, riskThresholdSchema, topNSchema };
