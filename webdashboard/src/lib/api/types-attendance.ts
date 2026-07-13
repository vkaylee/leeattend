import { components } from "@/types/api-generated";

import type { TimeEntry } from "./types";

// --- Timekeeping Models ---
export type AttendanceEntry = TimeEntry;

export interface RawAttendanceLog {
  created_at: string;
  device_id?: string;
  device_name?: string;
  employee_full_name?: string;
  employee_id?: string;
  employee_id_number?: string;
  employment_id: string;
  id: string;
  punch_time: string;
  raw_data?: unknown;
  source: string;
  workspace_id: string;
}

export type AttendanceAdjustmentRequest = Omit<components["schemas"]["AttendanceAdjustmentRequest"], "requested_punch_in_time" | "requested_punch_out_time"> & {
  requested_date?: string;
  requested_punch_time: string;
  requested_punch_in_time?: string;
  requested_punch_out_time?: string | null;
  is_break_overtime?: boolean;
};

export type AttendanceReportSort =
  | "name_asc"
  | "name_desc"
  | "work_hours_asc"
  | "work_hours_desc"
  | "leave_days_asc"
  | "leave_days_desc"
  | "ot_hours_asc"
  | "ot_hours_desc"
  | "total_hours_asc"
  | "total_hours_desc";

export interface AttendanceSummary {
  computed_ot_hours: number;
  computed_std_hours: number;
  employee_id: string;
  employee_id_number?: string;
  employee_name: string;
  employer_id?: string;
  employer_name?: string;
  employment_id: string;
  total_leave_days: number;
  total_ot_units: number;
  total_std_units: number;
  unit_coefficient_hours: number;
  unit_type: 'DAY' | 'SHIFT' | 'HOUR' | 'MINUTE';
}

export type AuditLog = Omit<components["schemas"]["AuditLog"], "actor_id" | "target_user_id"> & {
  actor_id?: string;
  target_user_id?: string;
};

export interface DailyAttendanceReport {
  actual_punch_in: string | null;
  actual_punch_out: string | null;
  break_duration_minutes: number | null;
  break_end: string | null;
  break_start: string | null;
  computed_ot_hours: number;
  computed_std_hours: number;
  date: string;
  employee_id: string;
  employee_name: string;
  employer_name?: string | null;
  employment_id: string;
  generation_metadata?: unknown;
  grace_period_minutes?: number | null;
  /** IANA timezone for this employee at this date (e.g. "Asia/Ho_Chi_Minh"),
   *  resolved from the employee's work location. Used for no-show zone computation. */
  timezone?: string | null;
  holiday_name?: string | null;
  interval_id?: string | null;
  interval_name?: string | null;
  is_break_auto_deduct: boolean | null;
  is_holiday: boolean;
  is_leave_paid?: boolean | null;
  is_unscheduled?: boolean;
  ot_claim_status?: string | null;
  ot_work_units: number;
  scheduled_end: string | null;
  scheduled_start: string | null;
  segment_count?: number | null;
  segment_id?: string | null;
  segment_type?: string | null;
  shift_id?: string | null;
  shift_name?: string | null;
  shift_symbol?: string | null;
  sort_order?: number | null;
  status: string;
  std_work_units: number;
  unit_coefficient_hours: number;
  unit_type: 'DAY' | 'SHIFT' | 'HOUR' | 'MINUTE';
}

export type OtStatus = "none" | "pending_employee" | "pending_manager" | "approved" | "rejected";

export type AttendanceDailySummary = Omit<components["schemas"]["AttendanceDailySummary"], "entered_via" | "ot_reason"> & {
  entered_via?: string | null;
  ot_reason?: string | null;
};
