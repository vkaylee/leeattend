import { z } from "zod";

import { dashboardMetricsResponseSchema } from "./dashboard-metric-contract";

const dateString = z.string().min(1);
const count = z.number().finite();

/** A percentage rate plus its raw numerator/denominator. `value` is null when the denominator is zero. */
export const rateQuantitySchema = z.object({
  value: z.string().nullable(),
  numerator: count,
  denominator: count,
  unit: z.string().min(1),
}).strict();

export const organizationStatsSchema = z.object({
  absent_today: count,
  avg_work_hours_today: count,
  early_departure_today: count,
  estimated_monthly_payroll: z.union([z.string(), count]).transform(String),
  late_today: count,
  on_leave_today: count,
  pending_leave_requests: count,
  present_today: count,
  total_employees: count,
  attendance_rate: rateQuantitySchema,
  absence_rate: rateQuantitySchema,
  late_rate: rateQuantitySchema,
});

export const attendanceTrendSchema = z.array(z.object({
  absent: count,
  date: dateString,
  late: count,
  on_leave: count,
  present: count,
  eligible: count,
  attendance_rate: rateQuantitySchema,
  absence_rate: rateQuantitySchema,
  late_rate: rateQuantitySchema,
}));

export const departmentAttendanceSchema = z.array(z.object({
  absent: count,
  department_id: dateString,
  department_name: z.string(),
  late: count,
  on_leave: count,
  on_time: count,
  total_employees: count,
  attendance_rate: rateQuantitySchema,
  absence_rate: rateQuantitySchema,
  late_rate: rateQuantitySchema,
}));

export const unexcusedAbsencesSchema = z.array(z.object({
  consecutive_count: count,
  department_id: dateString,
  department_name: z.string(),
  employee_id: dateString.optional(),
  employee_name: z.string().optional(),
  total_absent_count: count,
  total_leave_count: count,
  unexcused_count: count,
}));

export const overtimeTrendSchema = z.array(z.object({
  approved_ot_hours: count,
  date: dateString,
  system_ot_hours: count,
}));

export const peakTimesSchema = z.array(z.object({
  check_in: count,
  check_out: count,
  time_slot: z.string(),
}));

export const leaveTypesBreakdownSchema = z.array(z.object({
  count,
  leave_type: z.string(),
}));

export const latenessByDaySchema = z.array(z.object({
  avg_late_minutes: count,
  day: z.string(),
  late_count: count,
}));

export const recruitmentSourceBreakdownSchema = z.array(z.object({
  count,
  source_name: z.string(),
  source_type: z.string(),
}));

const turnoverBucketSchema = z.object({
  bucket: z.string(),
  count,
});

/** Same shape as rateQuantitySchema — reused for turnover rates. */
export const turnoverRateSchema = rateQuantitySchema;

const turnoverComparisonSchema = z.object({
  basis: z.enum(["none", "previous_period", "same_period_last_year", "target", "budget", "benchmark"]),
  period_start: dateString,
  period_end: dateString,
  rate: turnoverRateSchema.nullable(),
  delta_percentage_points: z.string().nullable(),
});

const turnoverWarningSchema = z.object({
  code: z.string(),
  message: z.string(),
});

const turnoverReasonRowSchema = z.object({
  reason: z.string(),
  count,
});

const turnoverClassificationSchema = z.object({
  classification: z.string(),
  count,
});

const turnoverByEmploymentTypeSchema = z.object({
  employment_type: z.string(),
  count,
});

const turnoverByLocationSchema = z.object({
  location_name: z.string(),
  count,
});

const turnoverByManagerSchema = z.object({
  manager_name: z.string(),
  count,
});

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

const turnoverTargetSchema = z.object({
  value: z.string(),
  operator: z.string(),
  source: z.string(),
});

export const turnoverAnalyticsSchema = z.object({
  by_department: z.array(z.object({ department_name: z.string(), count })),
  by_source: z.array(z.object({ source_name: z.string(), source_type: z.string(), count })),
  period_end: dateString,
  period_start: dateString,
  tenure_distribution: z.array(turnoverBucketSchema),
  total_terminated: count,
  // New retention & workforce-movement fields (additive — old servers omit them).
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

const nullableCount = z.number().finite().nullable();
const nullableDecimal = z.string().nullable();

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

const recruitmentTrendPointSchema = z.object({
  month: dateString,
  hires: count,
});

const recruitmentDataGapSchema = z.object({
  metric: z.string(),
  reason: z.string(),
});

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
  organization: z.object({
    timezone: z.string().min(1),
    currency: z.string().min(1),
  }).strict(),
  rows: z.array(z.object({
    department_id: z.string().min(1),
    department_name: z.string().min(1),
    system_ot_hours: z.number().finite(),
    approved_ot_hours: z.number().finite(),
    unapproved_ot_hours: z.number().finite(),
    overtime_cost: z.string().nullable(),
    cost_available: z.boolean(),
  })),
  generated_at: z.string().min(1),
});

export const dashboardSchemaByKey = {
  "organization-stats": organizationStatsSchema,
  "attendance-trend": attendanceTrendSchema,
  "department-attendance": departmentAttendanceSchema,
  "unexcused-absences": unexcusedAbsencesSchema,
  "overtime-trend": overtimeTrendSchema,
  "peak-times": peakTimesSchema,
  "leave-types-breakdown": leaveTypesBreakdownSchema,
  "lateness-by-day": latenessByDaySchema,
  "recruitment-sources-breakdown": recruitmentSourceBreakdownSchema,
  "turnover-analytics": turnoverAnalyticsSchema,
  "recruitment-effectiveness": recruitmentEffectivenessSchema,
  "executive-overview": dashboardMetricsResponseSchema,
  "overtime-breakdown": overtimeBreakdownSchema,
} as const;

export type DashboardSchemaKey = keyof typeof dashboardSchemaByKey;
