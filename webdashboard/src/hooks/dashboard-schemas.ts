import { dashboardMetricsResponseSchema } from "./dashboard-metric-contract";
import {
  attendanceTrendSchema,
  departmentAttendanceSchema,
  latenessByDaySchema,
  leaveTypesBreakdownSchema,
  organizationStatsSchema,
  overtimeTrendSchema,
  peakTimesSchema,
  recruitmentSourceBreakdownSchema,
  unexcusedAbsencesSchema,
} from "./dashboard-schemas-core";
import {
  overtimeBreakdownSchema,
  recruitmentEffectivenessSchema,
  turnoverAnalyticsSchema,
} from "./dashboard-schemas-advanced";

export {
  attendanceTrendSchema,
  departmentAttendanceSchema,
  latenessByDaySchema,
  leaveTypesBreakdownSchema,
  organizationStatsSchema,
  overtimeTrendSchema,
  peakTimesSchema,
  recruitmentSourceBreakdownSchema,
  unexcusedAbsencesSchema,
} from "./dashboard-schemas-core";
export {
  overtimeBreakdownSchema,
  recruitmentEffectivenessSchema,
  turnoverAnalyticsSchema,
  turnoverRateSchema,
} from "./dashboard-schemas-advanced";

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
