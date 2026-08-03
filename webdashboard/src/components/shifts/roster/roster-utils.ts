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

/** Get cell color styles based on priority source */
export const getCellStyles = (source?: string, isSelected?: boolean): string => {
  const baseStyles =
    "transition-all relative border border-slate-100 dark:border-slate-800 text-center select-none cursor-pointer ";

  if (isSelected) {
    return baseStyles +
      "ring-2 ring-primary-500 ring-inset bg-primary-50 dark:bg-primary-950/40 z-[5]";
  }

  switch (source) {
    case "roster":
      return baseStyles +
        "bg-teal-50/90 dark:bg-teal-950/40 hover:bg-teal-100 dark:hover:bg-teal-900/50";
    case "pattern":
      return baseStyles +
        "bg-blue-50/40 dark:bg-blue-950/20 hover:bg-blue-100/30 dark:hover:bg-blue-900/30";
    case "assignment_rule":
      return baseStyles +
        "bg-violet-50/40 dark:bg-violet-950/20 hover:bg-violet-100/30 dark:hover:bg-violet-900/30";
    case "department_pattern":
      return baseStyles +
        "bg-slate-50/50 dark:bg-slate-800/30 hover:bg-slate-100/40 dark:hover:bg-slate-700/30";
    case "fixed":
    case "department_fixed":
      return baseStyles +
        "bg-emerald-50/40 dark:bg-emerald-950/20 hover:bg-emerald-100/30 dark:hover:bg-emerald-900/30";
    default:
      return baseStyles + "hover:bg-slate-50 dark:hover:bg-slate-800/50";
  }
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

/** Get the highest-priority source from a list of schedules */
export const getHighestPrioritySource = (
  schedules: EffectiveSchedule[],
): string | undefined => {
  const sources = new Set(schedules.map((s) => s.source));
  return SOURCE_PRIORITY.find((s) => sources.has(s));
};

/**
 * Order roster rows to match the user's selection order rather than the
 * workspace/API order. Rows whose id is not in `orderedIds` are dropped.
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
    .map((empSchedule) => {
      const rosterEntries = empSchedule.schedules
        .filter((s: EffectiveSchedule) => s.source === "roster" && !!s.shift_id)
        .map((s: EffectiveSchedule) => {
          const targetDateStr = `${currentYearMonth}-${s.date.substring(8)}`;
          if (validDates.has(targetDateStr)) {
            return { date: targetDateStr, shift_id: s.shift_id ?? null };
          }
          return null;
        })
        .filter((item): item is { date: string; shift_id: string | null } => item !== null);
      return { employeeId: empSchedule.employee_id, rosterEntries };
    })
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
      const [empId, d] = key.split(":");
      const list = acc[empId];
      if (maybeNull(list)) {
        list.push(d);
      } else {
        acc[empId] = [d];
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

  const matchingViolations = violations.filter((v) => {
    const matchesEmployee = v.details.employment_id === employeeId;
    const matchesDate = v.details.conflicting_dates.includes(dateStr);
    return matchesEmployee && matchesDate;
  });

  if (matchingViolations.length === 0) return "ok";

  const hasHardViolation = matchingViolations.some(
    (v) => v.violation_type === "overlap" || v.severity === "hard",
  );
  return hasHardViolation ? "violation" : "warning";
}
