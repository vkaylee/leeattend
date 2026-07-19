import type { EffectiveSchedule } from "@/lib/api/types";

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
