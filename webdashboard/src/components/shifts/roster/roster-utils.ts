import type { ComplianceViolation } from "@/lib/api/errors";
import type { EffectiveSchedule, EmployeeSchedule } from "@/lib/api/types";

import { maybeNull } from "@/types/api-overrides";

/** Check if weekend day (Sat/Sun) */
export const isWeekend = (date: Date): boolean => {
  const day = date.getDay();
  return day === 0 || day === 6;
};

/** Short display character for shifts */
export const getShiftSymbol = (
  shiftName?: string | null,
  reportSymbol?: string | null,
): string => {
  if (reportSymbol) return reportSymbol;
  if (!shiftName) return "";
  return shiftName.substring(0, 2).toUpperCase();
};

const BASE_CELL_STYLES =
  "group transition-all relative border border-slate-100 dark:border-slate-800 text-center select-none cursor-pointer ";
const SOURCE_CELL_STYLES: Record<string, string> = {
  roster: "bg-teal-50/90 dark:bg-teal-950/40 hover:bg-teal-100 dark:hover:bg-teal-900/50",
  pattern: "bg-blue-50/40 dark:bg-blue-950/20 hover:bg-blue-100/30 dark:hover:bg-blue-900/30",
  assignment_rule: "bg-violet-50/40 dark:bg-violet-950/20 hover:bg-violet-100/30 dark:hover:bg-violet-900/30",
  department_pattern: "bg-slate-50/50 dark:bg-slate-800/30 hover:bg-slate-100/40 dark:hover:bg-slate-700/30",
  fixed: "bg-emerald-50/40 dark:bg-emerald-950/20 hover:bg-emerald-100/30 dark:hover:bg-emerald-900/30",
  department_fixed: "bg-emerald-50/40 dark:bg-emerald-950/20 hover:bg-emerald-100/30 dark:hover:bg-emerald-900/30",
};

/** Get cell color styles based on priority source */
export const getCellStyles = (source?: string, isSelected?: boolean): string => {
  if (isSelected) {
    return BASE_CELL_STYLES + "ring-2 ring-primary-500 ring-inset bg-primary-50 dark:bg-primary-950/40 z-[5]";
  }
  return BASE_CELL_STYLES + (SOURCE_CELL_STYLES[source ?? ""] ?? "hover:bg-slate-50 dark:hover:bg-slate-800/50");
};

/** Priority-ordered source list for determining dominant source */
const SOURCE_PRIORITY = [
  "roster",
  "pattern",
  "assignment_rule",
  "department_pattern",
  "fixed",
  "department_fixed",
] as const;

/** Resolve official lifecycle state for a cell, including empty missing days. */
export type ScheduleLifecycleCellState = "published" | "explicit_off" | "missing" | "closed" | "draft" | "conflict" | "legacy_unverified";

export function getScheduleLifecycleCellState(
  schedules: EffectiveSchedule[],
): ScheduleLifecycleCellState {
  const state = schedules.find((schedule) => schedule.schedule_status)?.schedule_status;
  if (state === "published" || state === "explicit_off" || state === "missing" || state === "closed" || state === "draft" || state === "conflict" || state === "legacy_unverified") {
    return state;
  }
  if (schedules.some((schedule) => schedule.closed)) return "closed";
  if (schedules.some((schedule) => schedule.shift_id)) return "published";
  return "missing";
}

/** Get the highest-priority source from a list of schedules */
export const getHighestPrioritySource = (
  schedules: EffectiveSchedule[],
): string | undefined => {
  const sources = new Set(schedules.map((s) => s.source));
  return SOURCE_PRIORITY.find((s) => sources.has(s));
};

/**
 * Order roster rows to match the user's selection order rather than the
 * organization/API order. Rows whose id is not in `orderedIds` are dropped.
 */
export const orderSchedulesBySelection = <T extends { employee_id?: string }>(
  schedules: T[],
  orderedIds: string[],
): T[] => {
  const byId = new Map(schedules.map((s) => [s.employee_id ?? "", s]));
  return orderedIds
    .map((id) => byId.get(id))
    .filter((s): s is T => s !== undefined);
};

/** Compliance status for a roster cell badge. */
export type ComplianceCellStatus = "ok" | "warning" | "violation";

/** Result entry after translating roster schedules to a target month. */
export interface TranslatedRosterEntry {
  employeeId: string;
  rosterEntries: { date: string; shift_id: string | null }[];
}

function translateScheduleEntries(
  schedules: EffectiveSchedule[],
  currentYearMonth: string,
  validDates: Set<string>,
): { date: string; shift_id: string | null }[] {
  return schedules
    .filter((schedule) => schedule.source === "roster" && Boolean(schedule.shift_id))
    .map((schedule) => {
      const targetDate = `${currentYearMonth}-${schedule.date.substring(8)}`;
      return validDates.has(targetDate)
        ? { date: targetDate, shift_id: schedule.shift_id ?? null }
        : null;
    })
    .filter((entry): entry is { date: string; shift_id: string | null } => entry !== null);
}

/**
 * Translate roster-backed schedules from a previous month into the target
 * month, preserving day-of-month. Only entries whose source is "roster" and
 * that have a shift_id are included; dates outside the valid set are dropped.
 */
export function translateRosterEntriesToMonth(
  prevSchedules: EmployeeSchedule[],
  currentYearMonth: string,
  validDates: Set<string>,
): TranslatedRosterEntry[] {
  return prevSchedules
    .filter((schedule): schedule is EmployeeSchedule & { employee_id: string } => Boolean(schedule.employee_id))
    .map((schedule) => ({
      employeeId: schedule.employee_id,
      rosterEntries: translateScheduleEntries(schedule.schedules, currentYearMonth, validDates),
    }))
    .filter(({ rosterEntries }) => rosterEntries.length > 0);
}

/**
 * Group selected cell keys ("employeeId:dateStr") by employee id.
 * Used by both apply-shift and clear-overrides flows to batch cells per employee
 * before delegating to the bulk mutation handlers.
 */
export function groupCellsByEmployee(
  selectedCells: Set<string>,
): Record<string, string[]> {
  return Array.from(selectedCells).reduce<Record<string, string[]>>(
    (acc, key) => {
      const [empId, date] = key.split(":");
      const list = acc[empId];
      if (maybeNull(list)) {
        list.push(date);
      } else {
        acc[empId] = [date];
      }
      return acc;
    },
    {},
  );
}

/**
 * Determine the worst compliance status for a roster cell based on known
 * violation data from a 409 response. Returns "violation" if any overlap is
 * detected, "warning" for soft violations, or "ok" when no violation matches
 * this cell.
 */
export function getComplianceStatus(
  cellSchedules: EffectiveSchedule[],
  employeeId: string,
  dateStr: string,
  violations: ComplianceViolation[] | undefined,
): ComplianceCellStatus {
  if (!violations || violations.length === 0) return "ok";

  const hasMatchingViolation = violations.some((violation) => (
    violation.details.employment_id === employeeId && violation.details.conflicting_dates.includes(dateStr)
  ));
  if (!hasMatchingViolation) return "ok";

  const hasHardViolation = violations.some((violation) => (
    violation.details.employment_id === employeeId &&
    violation.details.conflicting_dates.includes(dateStr) &&
    (violation.violation_type === "overlap" || violation.severity === "hard")
  ));
  return hasHardViolation ? "violation" : "warning";
}
