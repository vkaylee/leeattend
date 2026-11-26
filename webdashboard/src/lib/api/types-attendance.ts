import type { TimeEntry } from "./api-generated-reexports";

// --- Timekeeping Models ---
export type AttendanceEntry = TimeEntry;

export interface RawAttendanceLog {
  created_at: string;
  employee_full_name?: string;
  employee_id?: string;
  employee_id_number?: string;
  employment_id: string;
  id: string;
  punch?: number;
  punch_time: string;
  raw_data?: unknown;
  source: string;
  status?: number;
}

export type AdjustmentStatus = "pending" | "approved" | "rejected";

export interface AttendanceAdjustmentRequest {
  approved_by_user_id?: string | null;
  attachment_file_ids?: string[] | null;
  created_at: string;
  employment_id: string;
  id: string;
  is_break_overtime?: boolean;
  reason?: string | null;
  requested_by_user_id?: string | null;
  requested_date?: string;
  requested_punch_in_time?: string;
  requested_punch_out_time?: string | null;
  requested_punch_time: string;
  status: string;
  time_entry_id?: string | null;
  updated_at: string;
}

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

export interface AuditLog {
  action: string;
  actor_email?: string | null;
  actor_id?: string | null;
  actor_name?: string | null;
  actor_type?: string | null;
  created_at: string;
  details?: unknown;
  id: string;
  ip_address?: string | null;
  target_resource_id?: string | null;
  target_user_id?: string | null;
  user_agent?: string | null;
}

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
  /** IANA timezone for this employee at this date (e.g. "Asia/Ho_Chi_Minh"),
   *  resolved from the employee's work location. Used for no-show zone computation. */
  timezone?: string | null;
  unit_coefficient_hours: number;
  unit_type: 'DAY' | 'SHIFT' | 'HOUR' | 'MINUTE';
}

export type OtStatus = "none" | "pending_employee" | "pending_manager" | "approved" | "rejected";

export interface AttendanceDailySummary {
  actual_work_minutes: number;
  actual_break_minutes?: number;
  approved_ot_minutes: number;
  break_exception_code?: string | null;
  break_ot_minutes?: number;
  break_provisional?: boolean;
  deducted_break_minutes?: number;
  claim_entered_at?: string | null;
  claim_entered_by_user_id?: string | null;
  claimed_ot_minutes: number;
  created_at: string;
  date: string;
  early_ot_minutes?: number;
  employment_id: string;
  entered_via?: string | null;
  holiday_name?: string | null;
  id: string;
  is_holiday: boolean;
  is_weekend?: boolean;
  late_ot_minutes?: number;
  manager_note?: string | null;
  ot_reason?: string | null;
  ot_status: string;
  paper_form_photo_url?: string | null;
  net_paid_minutes?: number;
  net_work_minutes?: number;
  scheduled_break_minutes?: number;
  standard_minutes?: number;
  system_detected_ot_minutes: number;
  updated_at: string;
}
