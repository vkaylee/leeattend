/**
 * attendance-code-resolver.ts
 *
 * Pure resolver engine that maps a DailyAttendanceReport row and day context
 * into an AttendanceCodeConfig according to Vietnamese Labor Standards.
 */

import type { DailyAttendanceReport } from "@/lib/api";

import {
  getAttendanceCodeConfig,
  type AttendanceCodeConfig,
} from "./attendance-code-registry";

export interface ResolveCodeOptions {
  isWeekend?: boolean;
}

type CodeResolver = () => AttendanceCodeConfig | undefined;

function resolveHoursCode(row: DailyAttendanceReport): AttendanceCodeConfig | undefined {
  const standardHours = row.computed_std_hours || row.std_work_units * 8;
  if (standardHours >= 7) return getAttendanceCodeConfig("X");
  if (standardHours > 0) return getAttendanceCodeConfig("X/2");
  return undefined;
}

function resolveLeaveCode(row: DailyAttendanceReport): AttendanceCodeConfig | undefined {
  if (row.status !== "leave") return undefined;
  return getAttendanceCodeConfig(row.is_leave_paid === true ? "P" : "KL");
}

function resolvePresenceCode(row: DailyAttendanceReport): AttendanceCodeConfig | undefined {
  if (row.status === "present" || (row.actual_punch_in && row.actual_punch_out)) {
    return getAttendanceCodeConfig("X");
  }
  return undefined;
}

function resolveAbsenceCode(
  row: DailyAttendanceReport,
  isWeekend: boolean,
): AttendanceCodeConfig | undefined {
  if (row.status !== "absent" && row.status !== "no_show") return undefined;
  return getAttendanceCodeConfig(isWeekend ? "Ro" : "V");
}

function resolveRowCode(row: DailyAttendanceReport, isWeekend: boolean): AttendanceCodeConfig {
  const resolvers: CodeResolver[] = [
    () => row.is_holiday ? getAttendanceCodeConfig("L") : undefined,
    () => row.shift_symbol ? getAttendanceCodeConfig(row.shift_symbol) : undefined,
    () => resolveLeaveCode(row),
    () => resolveHoursCode(row),
    () => resolvePresenceCode(row),
    () => resolveAbsenceCode(row, isWeekend),
    () => isWeekend ? getAttendanceCodeConfig("Ro") : undefined,
  ];

  for (const resolve of resolvers) {
    const code = resolve();
    if (code) return code;
  }
  return getAttendanceCodeConfig("V");
}

/**
 * Resolves an attendance report row into a standardized attendance code.
 * Holidays, custom shift symbols, leave, work hours, presence, and absence
 * are checked in that order.
 */
export function resolveDailyAttendanceCode(
  row?: DailyAttendanceReport | null,
  options?: ResolveCodeOptions,
): AttendanceCodeConfig {
  const isWeekend = options?.isWeekend ?? false;
  if (!row) return getAttendanceCodeConfig(isWeekend ? "Ro" : "-");
  return resolveRowCode(row, isWeekend);
}
