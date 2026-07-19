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
 *  current wall-clock time. The table render uses this; the export handler
 *  maps it down to `NoShowRow`. */
export interface ZonedNoShowRow extends DailyAttendanceReport {
  department_name: string;
  employee_id_number: string;
  zone: Zone;
  /** Minutes of grace remaining (Zone A) or past grace (Zone B); null for C. */
  zoneMinutes: number | null;
}

export interface UseNoShowRowsParams {
  employeeMap: Map<string, { department_name: string; employee_id_number: string }>;
  rows: DailyAttendanceReport[];
  /** The target calendar day ("yyyy-MM-dd") being viewed. */
  viewingDate: string;
  /** IANA workspace timezone, used as fallback when a row has no per-employee
   *  timezone resolved. */
  workspaceTimezone: string;
}

/** Pure core of the enrichment + zone computation. Shared by the reactive hook
 *  (table render) and the imperative export handler so both never disagree —
 *  the hook simply memoises this. */
export function computeNoShowRows(params: UseNoShowRowsParams): ZonedNoShowRow[] {
  const { rows, employeeMap, viewingDate, workspaceTimezone } = params;
  const today = getTodayIso();
  const pastDate = isViewingPastDate(viewingDate, today);

  const out: ZonedNoShowRow[] = [];
  for (const row of rows) {
    if (row.status !== "absent") continue;

    const empInfo = employeeMap.get(row.employee_id) ?? {
      department_name: "—",
      employee_id_number: "—",
    };

    // The shift's assigned calendar day anchors the shift window.
    const shiftDate = toIsoDate(row.date);

    // Use per-employee timezone from the backend, falling back to workspace
    // timezone. This ensures zone computation uses the correct TZ for
    // employees in different locations.
    const empTz = row.timezone ?? workspaceTimezone;

    // Historical date → fixed record, no live clock.
    const zone = pastDate
      ? { zone: "c" as Zone, minutes: null }
      : computeZone({
          scheduledStart: row.scheduled_start,
          scheduledEnd: row.scheduled_end,
          graceMinutes: row.grace_period_minutes,
          referenceTime: new Date(), // Always "now" — the date picker filters what we see
          shiftDate,
          tz: empTz,
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
  // eslint-disable-next-line react-hooks/exhaustive-deps
    [
      params.rows,
      params.employeeMap,
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
