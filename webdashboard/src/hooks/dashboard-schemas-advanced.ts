import { z } from "zod";

import { rateQuantitySchema } from "./dashboard-schemas-core";

const dateString = z.string().min(1);
const count = z.number();
const nullableCount = z.number().nullable();
const nullableDecimal = z.string().nullable();

const turnoverBucketSchema = z.object({ bucket: z.string(), count });
export const turnoverRateSchema = rateQuantitySchema;

const turnoverComparisonSchema = z.object({
  basis: z.enum(["none", "previous_period", "same_period_last_year", "target", "budget", "benchmark"]),
  period_start: dateString,
  period_end: dateString,
  rate: turnoverRateSchema.nullable(),
  delta_percentage_points: z.string().nullable(),
});
const turnoverWarningSchema = z.object({ code: z.string(), message: z.string() });
const turnoverReasonRowSchema = z.object({ reason: z.string(), count });
const turnoverClassificationSchema = z.object({ classification: z.string(), count });
const turnoverByEmploymentTypeSchema = z.object({ employment_type: z.string(), count });
const turnoverByLocationSchema = z.object({ location_name: z.string(), count });
const turnoverByManagerSchema = z.object({ manager_name: z.string(), count });
const turnoverTrendPointSchema = z.object({
  month: dateString,
  headcount: count,
  leavers: count,
  turnover_rate: turnoverRateSchema,
});
const cohortRetentionRowSchema = z.object({
  cohort_month: dateString,
  months_since_hire: count,
  cohort_size: count,
  active_count: count,
  retention_rate: z.string().nullable(),
});
const turnoverTargetSchema = z.object({ value: z.string(), operator: z.string(), source: z.string() });

export const turnoverAnalyticsSchema = z.object({
  by_department: z.array(z.object({ department_name: z.string(), count })),
  by_source: z.array(z.object({ source_name: z.string(), source_type: z.string(), count })),
  period_end: dateString,
  period_start: dateString,
  tenure_distribution: z.array(turnoverBucketSchema),
  total_terminated: count,
  turnover_rate: turnoverRateSchema,
  average_headcount: count,
  voluntary_count: count,
  involuntary_count: count,
  unknown_classification_count: count,
  retention_rate: z.string().nullable(),
  early_attrition: turnoverRateSchema,
  regrettable_attrition: z.string().nullable(),
  comparison: turnoverComparisonSchema.nullable(),
  target: turnoverTargetSchema,
  monthly_trend: z.array(turnoverTrendPointSchema),
  cohort_retention: z.array(cohortRetentionRowSchema),
  termination_reasons: z.array(turnoverReasonRowSchema),
  classification: z.array(turnoverClassificationSchema),
  by_employment_type: z.array(turnoverByEmploymentTypeSchema),
  by_location: z.array(turnoverByLocationSchema),
  by_manager: z.array(turnoverByManagerSchema),
  warnings: z.array(turnoverWarningSchema),
});

const recruitmentFunnelStageSchema = z.object({
  stage: z.string(),
  count: nullableCount,
  data_source: z.string().nullable(),
  conversion_rate: nullableDecimal,
});
const recruitmentEffectivenessSourceSchema = z.object({
  source_id: z.string().nullable(),
  source_type: z.string(),
  source_name: z.string(),
  hires: count,
  hires_ratio: nullableDecimal,
  retained_90d: count,
  retention_90d: nullableDecimal,
  cost_per_hire: nullableDecimal,
  cost_per_hire_available: z.boolean(),
  turnover_in_range: count,
});
const recruitmentEffectivenessOverviewSchema = z.object({
  hires: count,
  average_cost_per_hire: nullableDecimal,
  cost_available: z.boolean(),
  retained_90d: count,
  retention_90d: nullableDecimal,
  turnover_in_range: count,
  comparison_hires: count.nullable(),
  comparison_hires_delta: count.nullable(),
});
const recruitmentTrendPointSchema = z.object({ month: dateString, hires: count });
const recruitmentDataGapSchema = z.object({ metric: z.string(), reason: z.string() });

export const recruitmentEffectivenessSchema = z.object({
  period_start: dateString,
  period_end: dateString,
  total_hires: count,
  overview: recruitmentEffectivenessOverviewSchema,
  funnel: z.array(recruitmentFunnelStageSchema),
  by_source: z.array(recruitmentEffectivenessSourceSchema),
  hires_trend: z.array(recruitmentTrendPointSchema),
  data_gaps: z.array(recruitmentDataGapSchema),
  warnings: z.array(turnoverWarningSchema),
});

export const overtimeBreakdownSchema = z.object({
  contract_version: z.literal("overtime-breakdown.v2"),
  organization: z.object({ timezone: z.string().min(1), currency: z.string().min(1) }).strict(),
  rows: z.array(z.object({
    department_id: z.string().min(1),
    department_name: z.string().min(1),
    system_ot_hours: z.number(),
    approved_ot_hours: z.number(),
    unapproved_ot_hours: z.number(),
    overtime_cost: z.string().nullable(),
    cost_available: z.boolean(),
  })),
  generated_at: z.string().min(1),
});
