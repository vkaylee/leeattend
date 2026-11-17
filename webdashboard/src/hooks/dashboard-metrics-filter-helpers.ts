import { z } from "zod";

import {
  DEFAULT_DASHBOARD_METRICS_FILTERS,
  type DashboardMetricsFilters,
  type DashboardPeriod,
  type DashboardComparison,
  type DashboardRiskThreshold,
  comparisonSchema,
  periodSchema,
  riskThresholdSchema,
} from "./dashboard-metrics-filter-types";

const datePattern = /^\d{4}-\d{2}-\d{2}$/;
const uuid = z.uuid();

function sortedUniqueValues(values: readonly string[], validate?: (value: string) => boolean): string[] {
  return [
    ...new Set(
      values
        .flatMap((value) => value.split(","))
        .map((value) => value.trim())
        .filter((value) => value && (!validate || validate(value))),
    ),
  ].sort();
}

export function sortedUniqueIds(values: readonly string[]): string[] {
  return sortedUniqueValues(
    values.flatMap((value) => value.split(",")).map((value) => value.toLowerCase()),
    (value) => uuid.safeParse(value).success,
  );
}

function validDate(value: string | undefined): string | undefined {
  if (!value || !datePattern.test(value)) return undefined;
  const [year, month, day] = value.split("-").map(Number);
  const leapYear = year % 4 === 0 && (year % 100 !== 0 || year % 400 === 0);
  const daysInMonth = [31, leapYear ? 29 : 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31][month - 1];
  return year > 0 && daysInMonth && day >= 1 && day <= daysInMonth ? value : undefined;
}

export interface CanonicalFilterParts {
  comparison: DashboardComparison;
  dimensions: Pick<DashboardMetricsFilters, "businessUnitIds" | "departmentIds" | "employeeStatuses" | "employmentTypes" | "locationIds" | "managerIds" | "shiftIds">;
  endDate?: string;
  period: DashboardPeriod;
  riskThreshold: DashboardRiskThreshold;
  startDate?: string;
  topN: number;
}

export function canonicalFilterParts(filters: Partial<DashboardMetricsFilters>): CanonicalFilterParts {
  return {
    comparison: parseEnum(comparisonSchema, filters.comparison, DEFAULT_DASHBOARD_METRICS_FILTERS.comparison),
    dimensions: canonicalDimensions(filters),
    endDate: validDate(filters.endDate),
    period: parseEnum(periodSchema, filters.period, DEFAULT_DASHBOARD_METRICS_FILTERS.period),
    riskThreshold: parseEnum(riskThresholdSchema, filters.riskThreshold, DEFAULT_DASHBOARD_METRICS_FILTERS.riskThreshold),
    startDate: validDate(filters.startDate),
    topN: canonicalTopN(filters.topN),
  };
}

function canonicalDimensions(filters: Partial<DashboardMetricsFilters>): CanonicalFilterParts["dimensions"] {
  return {
    businessUnitIds: sortedUniqueIds(filters.businessUnitIds ?? []),
    departmentIds: sortedUniqueIds(filters.departmentIds ?? []),
    employeeStatuses: sortedUniqueValues(filters.employeeStatuses ?? []),
    employmentTypes: sortedUniqueValues(filters.employmentTypes ?? []),
    locationIds: sortedUniqueIds(filters.locationIds ?? []),
    managerIds: sortedUniqueIds(filters.managerIds ?? []),
    shiftIds: sortedUniqueIds(filters.shiftIds ?? []),
  };
}

function canonicalTopN(value: number | undefined): number {
  const isValid = typeof value === "number" && Number.isInteger(value) && value >= 1 && value <= 100;
  return isValid ? value : DEFAULT_DASHBOARD_METRICS_FILTERS.topN;
}

function parseEnum<T extends string>(schema: z.ZodType<T>, value: unknown, fallback: T): T {
  const parsed = schema.safeParse(value);
  return parsed.success ? parsed.data : fallback;
}

export function isInvalidCustomRange(parts: Pick<CanonicalFilterParts, "period" | "startDate" | "endDate">): boolean {
  return parts.period === "custom" && (!parts.startDate || !parts.endDate || parts.endDate < parts.startDate);
}
