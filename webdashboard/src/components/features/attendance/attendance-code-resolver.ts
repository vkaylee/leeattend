/**
 * attendance-code-resolver.ts
 *
 * Pure resolver engine that maps a DailyAttendanceReport row and day context
 * into an AttendanceCodeConfig according to Vietnamese Labor Standards.
 */

import type { DailyAttendanceReport } from "@/lib/api";

import {
  ATTENDANCE_CODE_REGISTRY,
  getAttendanceCodeConfig,
  type AttendanceCodeConfig,
} from "./attendance-code-registry";

export interface ResolveCodeOptions {
  isWeekend?: boolean;
}

/**
 * Resolves an attendance report row into a standardized attendance code.
 *
 * Rules:
 * 1. Holiday: if row.is_holiday is true -> "L"
 * 2. Custom shift symbol override (e.g. "CT", "TS", "Ô")
 * 3. Leave: if row.status === "leave"
 *    - if row.is_leave_paid is true -> "P" (Phép năm hưởng lương)
 *    - else -> "KL" (Nghỉ không lương)
 * 4. Work hours:
 *    - >= 7.0 standard hours -> "X" (1.0 công đủ)
 *    - >= 3.5 standard hours -> "X/2" (0.5 công)
 *    - > 0 standard hours -> "X/2"
 * 5. Present status with punches even if computed hours are pending -> "X"
 * 6. Absent / No show:
 *    - If weekend -> "Ro" (Nghỉ ca / tuần)
 *    - If weekday -> "V" (Vắng)
 * 7. Fallback / No row:
 *    - If weekend -> "Ro"
 *    - Else -> "-" (Chưa có dữ liệu)
 */
export function resolveDailyAttendanceCode(
  row?: DailyAttendanceReport | null,
  options?: ResolveCodeOptions
): AttendanceCodeConfig {
  const isWeekend = options?.isWeekend ?? false;

  if (!row) {
    if (isWeekend) return getAttendanceCodeConfig("Ro");
    return getAttendanceCodeConfig("-");
  }

  // 1. Holiday takes precedence
  if (row.is_holiday) {
    return getAttendanceCodeConfig("L");
  }

  // 2. Custom shift symbol override (e.g. "CT", "TS", "Ô")
  if (row.shift_symbol && ATTENDANCE_CODE_REGISTRY[row.shift_symbol]) {
    return getAttendanceCodeConfig(row.shift_symbol);
  }

  // 3. Leave records
  if (row.status === "leave") {
    if (row.is_leave_paid === true) {
      return getAttendanceCodeConfig("P");
    }
    return getAttendanceCodeConfig("KL");
  }

  // 4. Standard Work Hours
  const stdHours = row.computed_std_hours ?? (row.std_work_units ? row.std_work_units * 8 : 0);
  if (stdHours >= 7.0) {
    return getAttendanceCodeConfig("X");
  }
  if (stdHours >= 3.5) {
    return getAttendanceCodeConfig("X/2");
  }
  if (stdHours > 0) {
    return getAttendanceCodeConfig("X/2");
  }

  // 5. Present status with punches even if computed hours are pending
  if (row.status === "present" || (row.actual_punch_in && row.actual_punch_out)) {
    return getAttendanceCodeConfig("X");
  }

  // 6. Absent / No show
  if (row.status === "absent" || row.status === "no_show") {
    if (isWeekend) return getAttendanceCodeConfig("Ro");
    return getAttendanceCodeConfig("V");
  }

  // 7. Weekend fallback for non-work days
  if (isWeekend) {
    return getAttendanceCodeConfig("Ro");
  }

  return getAttendanceCodeConfig("V");
}
