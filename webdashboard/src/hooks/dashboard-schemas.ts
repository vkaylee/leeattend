import { z } from "zod";

const dateString = z.string().min(1);
const count = z.number().finite();

export const workspaceStatsSchema = z.object({
  absent_today: count,
  avg_work_hours_today: count,
  early_departure_today: count,
  estimated_monthly_payroll: z.union([z.string(), count]).transform(String),
  late_today: count,
  on_leave_today: count,
  pending_leave_requests: count,
  present_today: count,
  total_employees: count,
});

export const attendanceTrendSchema = z.array(z.object({
  absent: count,
  date: dateString,
  late: count,
  on_leave: count,
  present: count,
}));

export const departmentAttendanceSchema = z.array(z.object({
  absent: count,
  department_id: dateString,
  department_name: z.string(),
  late: count,
  on_leave: count,
  on_time: count,
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

export const turnoverAnalyticsSchema = z.object({
  by_department: z.array(z.object({ department_name: z.string(), count })),
  by_source: z.array(z.object({ source_name: z.string(), source_type: z.string(), count })),
  period_end: dateString,
  period_start: dateString,
  tenure_distribution: z.array(turnoverBucketSchema),
  total_terminated: count,
});

export const dashboardSchemaByKey = {
  "workspace-stats": workspaceStatsSchema,
  "attendance-trend": attendanceTrendSchema,
  "department-attendance": departmentAttendanceSchema,
  "unexcused-absences": unexcusedAbsencesSchema,
  "overtime-trend": overtimeTrendSchema,
  "peak-times": peakTimesSchema,
  "leave-types-breakdown": leaveTypesBreakdownSchema,
  "lateness-by-day": latenessByDaySchema,
  "recruitment-sources-breakdown": recruitmentSourceBreakdownSchema,
  "turnover-analytics": turnoverAnalyticsSchema,
} as const;

export type DashboardSchemaKey = keyof typeof dashboardSchemaByKey;
