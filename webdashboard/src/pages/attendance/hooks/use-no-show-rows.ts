import { useMemo } from "react";

import type { DailyAttendanceReport } from "@/lib/api";
import type { NoShowRow } from "@/lib/excel-no-show";

import {
  computeZone,
  isViewingPastDate,
  type Zone,
} from "../lib/no-show-zone";

/** A row with employee data resolved but zone not yet computed. */
export interface EnrichedNoShowRow extends DailyAttendanceReport {
  department_name: string;
  employee_id_number: string;
}

/** An absent report row enriched with employee data and its live A/B/C zone.
 *  The table render uses this; the export handler maps it down to `NoShowRow`. */
export interface ZonedNoShowRow extends DailyAttendanceReport {
  department_name: string;
  employee_id_number: string;
  zone: Zone;
  /** Minutes of grace remaining (Zone A) or past grace (Zone B); null for C. */
  zoneMinutes: number | null;
}

export interface UseNoShowRowsParams {
  rows: DailyAttendanceReport[];
  employeeMap: Map<string, { department_name: string; employee_id_number: string }>;
  now: Date;
  viewingDate: string;
}

/** Pure core of the enrichment + zone computation. Shared by the reactive hook
 *  (table render) and the imperative export handler so both never disagree —
 *  the hook simply memoises this. */
export function computeNoShowRows({
  rows,
  employeeMap,
  now,
  viewingDate,
}: UseNoShowRowsParams): ZonedNoShowRow[] {
  const today = getTodayIso();
  const pastDate = isViewingPastDate(viewingDate, today);

  const out: ZonedNoShowRow[] = [];
  for (const row of rows) {
    if (row.status !== "absent") continue;

    const empInfo = employeeMap.get(row.employee_id) ?? {
      department_name: "—",
      employee_id_number: "—",
    };

    // Historical date → fixed record, no live clock.
    const zone = pastDate
      ? { zone: "c" as Zone, minutes: null }
      : computeZone(
          row.scheduled_start,
          row.scheduled_end,
          row.grace_period_minutes,
          now,
        );

    out.push({
      ...row,
      department_name: empInfo.department_name,
      employee_id_number: empInfo.employee_id_number,
      zone: zone.zone,
      zoneMinutes: zone.minutes,
    });
  }
  return out;
}

export function useNoShowRows(params: UseNoShowRowsParams): ZonedNoShowRow[] {
  return useMemo(() => computeNoShowRows(params), [
    params.rows,
    params.employeeMap,
    params.now,
    params.viewingDate,
  ]);
}

function getTodayIso(): string {
  const d = new Date();
  const mm = String(d.getMonth() + 1).padStart(2, "0");
  const dd = String(d.getDate()).padStart(2, "0");
  return `${d.getFullYear()}-${mm}-${dd}`;
}
