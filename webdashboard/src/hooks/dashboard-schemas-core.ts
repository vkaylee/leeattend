import { z } from "zod";

const dateString = z.string().min(1);
const count = z.number();

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

export const leaveTypesBreakdownSchema = z.array(z.object({ count, leave_type: z.string() }));
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
