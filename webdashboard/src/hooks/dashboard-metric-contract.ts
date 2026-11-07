import { z } from "zod";

const decimalString = z.string().regex(/^-?\d+(?:\.\d+)?$/, "Expected decimal value");
const decimalValue = z.union([z.number().finite(), decimalString]);
const date = z.string().regex(/^\d{4}-\d{2}-\d{2}$/, "Expected YYYY-MM-DD");
const rfc3339 = z.string().datetime({ offset: true });
const uuid = z.string().uuid();

const metricStatusValues = ["available", "not_applicable", "data_gap", "restricted"] as const;
const comparisonBasisValues = ["none", "previous_period", "same_period_last_year", "target", "budget", "benchmark"] as const;
const dataQualityStatusValues = ["complete", "partial", "data_gap"] as const;

export const dashboardMetricUnitSchema = z.enum([
  "count",
  "rate",
  "hours",
  "currency",
  "duration",
  "percent",
  "percentage_point_delta",
]);

export const dashboardMetricStatusSchema = z.enum(metricStatusValues);

export const dashboardMetricHealthSchema = z.enum([
  "ok",
  "healthy",
  "warning",
  "critical",
  "neutral",
  "not_applicable",
  "data_gap",
]);

export const dashboardMetricDirectionSchema = z.enum([
  "higher_is_better",
  "lower_is_better",
  "neutral",
]);

export const dashboardMetricQuantitySchema = z.object({
  value: decimalValue,
  unit: z.string().min(1),
}).strict();

export const dashboardMetricFreshnessSchema = z.object({
  as_of: rfc3339,
  lag_seconds: z.number().int().nonnegative().nullable(),
}).strict();

export const dashboardMetricComparisonSchema = z.object({
  basis: z.enum(comparisonBasisValues),
  period_start: date,
  period_end: date,
  value: decimalValue.nullable(),
  delta: decimalValue.nullable(),
  delta_percent: decimalValue.nullable(),
  delta_percentage_points: decimalValue.nullable(),
  status: dashboardMetricStatusSchema,
}).strict();

export const dashboardMetricTargetSchema = z.object({
  value: decimalValue,
  operator: z.enum(["lt", "lte", "eq", "gte", "gt"]),
}).strict();

export const dashboardMetricSchema = z.object({
  key: z.string().min(1),
  value: decimalValue.nullable(),
  unit: dashboardMetricUnitSchema,
  numerator: dashboardMetricQuantitySchema.nullable(),
  denominator: dashboardMetricQuantitySchema.nullable(),
  status: dashboardMetricStatusSchema,
  health: dashboardMetricHealthSchema,
  null_reason: z.string().min(1).nullable(),
  definition_version: z.string().min(1),
  direction: dashboardMetricDirectionSchema,
  freshness: dashboardMetricFreshnessSchema,
  quality_flags: z.array(z.string().min(1)),
  comparison: dashboardMetricComparisonSchema.nullable(),
  target: dashboardMetricTargetSchema.nullable(),
}).strict();

export const dashboardReportingPeriodSchema = z.object({
  key: z.enum(["today", "this_week", "this_month", "this_quarter", "year_to_date", "custom"]),
  start_date: date,
  end_date: date,
  timezone: z.string().min(1),
  as_of: rfc3339,
}).strict();

export const dashboardInsightSchema = z.object({
  id: z.string().min(1),
  type: z.enum(["trend", "threshold", "concentration", "data_quality", "workflow"]),
  severity: z.enum(["info", "watch", "high", "critical"]),
  metric_key: z.string().min(1),
  title_key: z.string().min(1),
  summary_key: z.string().min(1),
  parameters: z.record(z.string(), z.string()),
  requires_permission: z.string().min(1).nullable(),
  scope: z.string().min(1),
  dismissible: z.boolean(),
}).strict();

export const dashboardDataQualitySchema = z.object({
  status: z.enum(dataQualityStatusValues),
  sources: z.array(z.object({
    key: z.string().min(1),
    status: z.enum(dataQualityStatusValues),
    coverage_ratio: z.number().finite().min(0).max(1).nullable(),
    freshness: dashboardMetricFreshnessSchema,
    warnings: z.array(z.string().min(1)),
  }).strict()),
}).strict();

export const dashboardMetricsResponseSchema = z.object({
  contract_version: z.literal("executive-dashboard.v1"),
  workspace: z.object({
    timezone: z.string().min(1),
    currency: z.string().regex(/^[A-Z]{3}$/),
  }).strict(),
  context: z.object({
    period: dashboardReportingPeriodSchema,
    comparison: z.enum(comparisonBasisValues),
    filters: z.record(z.string(), z.string()),
    scope: z.object({
      kind: z.enum(["workspace", "department", "location"]),
      department_ids: z.array(uuid),
      location_ids: z.array(uuid),
      omitted_dimensions: z.array(z.string().min(1)),
    }).strict(),
  }).strict(),
  metrics: z.array(dashboardMetricSchema),
  insights: z.array(dashboardInsightSchema),
  actions: z.object({
    total: z.number().int().nonnegative(),
    overdue: z.number().int().nonnegative(),
    by_type: z.array(z.object({
      type: z.string().min(1),
      count: z.number().int().nonnegative(),
    }).strict()),
  }).strict(),
  data_quality: dashboardDataQualitySchema,
  generated_at: rfc3339,
}).strict();

export type DashboardMetricUnit = z.infer<typeof dashboardMetricUnitSchema>;
export type DashboardMetricStatus = z.infer<typeof dashboardMetricStatusSchema>;
export type DashboardMetricHealth = z.infer<typeof dashboardMetricHealthSchema>;
export type DashboardMetricDirection = z.infer<typeof dashboardMetricDirectionSchema>;
export type DashboardMetricQuantity = z.infer<typeof dashboardMetricQuantitySchema>;
export type DashboardMetricFreshness = z.infer<typeof dashboardMetricFreshnessSchema>;
export type DashboardMetricComparison = z.infer<typeof dashboardMetricComparisonSchema>;
export type DashboardMetricTarget = z.infer<typeof dashboardMetricTargetSchema>;
export type DashboardInsight = z.infer<typeof dashboardInsightSchema>;
export type DashboardMetric = z.infer<typeof dashboardMetricSchema>;
export type DashboardReportingPeriod = z.infer<typeof dashboardReportingPeriodSchema>;
export type DashboardMetricsResponse = z.infer<typeof dashboardMetricsResponseSchema>;
