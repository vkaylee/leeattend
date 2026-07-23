import type { ScheduleTrace } from "@/lib/api";

/** Map schedule trace priority to a human-readable source label. */
export function getDerivedSource(appliedRule?: ScheduleTrace): string {
  if (!appliedRule) return "system";
  switch (appliedRule.priority) {
    case 100:
      return "roster";
    case 80:
      return "pattern";
    case 60:
      return "rules";
    case 40:
      return "department_pattern";
    case 20:
      return "fixed";
    case 10:
      return "department_fixed";
    default:
      return "system";
  }
}

export const SEGMENT_TYPE_COLORS: Record<string, string> = {
  work: "bg-blue-50 text-blue-700 border-blue-200 dark:bg-blue-950/30 dark:text-blue-300 dark:border-blue-800",
  standby: "bg-orange-50 text-orange-700 border-orange-200 dark:bg-orange-950/30 dark:text-orange-300 dark:border-orange-800",
  on_call: "bg-purple-50 text-purple-700 border-purple-200 dark:bg-purple-950/30 dark:text-purple-300 dark:border-purple-800",
  training: "bg-emerald-50 text-emerald-700 border-emerald-200 dark:bg-emerald-950/30 dark:text-emerald-300 dark:border-emerald-800",
  travel: "bg-slate-50 text-slate-600 border-slate-200 dark:bg-slate-950/30 dark:text-slate-300 dark:border-slate-800",
};
