import { useMemo } from "react";

import type { DailyAttendanceReport } from "@/lib/api";
import type { NoShowRow } from "@/lib/excel-no-show";

import {
  computeZone,
  isViewingPastDate,
  type ComputeZoneInput,
  type Zone,
} from "../lib/no-show-zone";

/** A row with employee data resolved but zone not yet computed. */
export interface EnrichedNoShowRow extends DailyAttendanceReport {
  department_name: string;
  employee_id_number: string;
}

/** An absent report row enriched with employee data and its A/B/C zone at the
 *  pinned reference time `A`. The table render uses this; the export handler
 *  maps it down to `NoShowRow`. */
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
  /** The scrubber's pinned reference instant `A`. */
  referenceTime: Date;
  /** The shift's assigned calendar day ("yyyy-MM-dd") used to anchor the shift
   *  window when deciding if a historical row is a fixed record. Shift date
   *  comes from the row itself (`row.date`), so this is only the fallback used
   *  by `isViewingPastDate` when comparing against "today". */
  viewingDate: string;
  /** IANA workspace timezone, e.g. "Asia/Ho_Chi_Minh", so `computeZone` can
   *  resolve shift boundaries in the workspace's wall-clock frame. */
  workspaceTimezone: string;
}

/** Pure core of the enrichment + zone computation. Shared by the reactive hook
 *  (table render) and the imperative export handler so both never disagree —
 *  the hook simply memoises this. */
export function computeNoShowRows(params: UseNoShowRowsParams): ZonedNoShowRow[] {
  const { rows, employeeMap, referenceTime, viewingDate, workspaceTimezone } = params;
  const today = getTodayIso();
  const pastDate = isViewingPastDate(viewingDate, today);

  const out: ZonedNoShowRow[] = [];
  for (const row of rows) {
    if (row.status !== "absent") continue;

    const empInfo = employeeMap.get(row.employee_id) ?? {
      department_name: "—",
      employee_id_number: "—",
    };

    // The shift's assigned calendar day anchors the shift window. The backend
    // returns it as a Date/ISO string; normalise to "yyyy-MM-dd".
    const shiftDate = toIsoDate(row.date);

    // Historical date → fixed record, no live clock.
    const zone = pastDate
      ? { zone: "c" as Zone, minutes: null }
      : computeZone({
          scheduledStart: row.scheduled_start,
          scheduledEnd: row.scheduled_end,
          graceMinutes: row.grace_period_minutes,
          referenceTime,
          shiftDate,
          tz: workspaceTimezone,
        } satisfies ComputeZoneInput);

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
  return useMemo(
    () => computeNoShowRows(params),
    [
      params.rows,
      params.employeeMap,
      params.referenceTime,
      params.viewingDate,
      params.workspaceTimezone,
    ],
  );
}

function getTodayIso(): string {
  const d = new Date();
  const mm = String(d.getMonth() + 1).padStart(2, "0");
  const dd = String(d.getDate()).padStart(2, "0");
  return `${d.getFullYear()}-${mm}-${dd}`;
}

/** `row.date` arrives as a Date or an ISO "yyyy-MM-dd" string from the API.
 *  Normalise to "yyyy-MM-dd" so `computeZone` / `isViewingPastDate` can anchor
 *  the shift window on it. */
function toIsoDate(value: string | Date | undefined | null): string {
  if (!value) return new Date().toISOString().slice(0, 10);
  if (typeof value === "string") {
    return value.slice(0, 10);
  }
  return value.toISOString().slice(0, 10);
}

export type { NoShowRow };
