import { describe, expect, it } from "vitest";

import type { DailyAttendanceReport, PaginatedResponse } from "@/lib/api";

import { getNextNoShowPageParam } from "../../components/hooks/use-unrecorded-shifts-data";
import { computeNoShowRows } from "../use-no-show-rows";

function localTodayIso(): string {
  const now = new Date();
  const month = String(now.getMonth() + 1).padStart(2, "0");
  const day = String(now.getDate()).padStart(2, "0");
  return `${now.getFullYear()}-${month}-${day}`;
}

function absentRow(date: string): DailyAttendanceReport {
  return {
    date,
    employee_id: "employee-1",
    employment_id: "employment-1",
    employee_name: "Test Employee",
    scheduled_start: "08:00:00",
    scheduled_end: "17:00:00",
    grace_period_minutes: 15,
    status: "absent",
    timezone: "UTC",
  } as DailyAttendanceReport;
}

function page(total: number, limit = 30): PaginatedResponse<DailyAttendanceReport> {
  return { data: [], limit, page: 1, total };
}

describe("no-show regression coverage", () => {
  it("moves an absent employee from pending grace to action required", () => {
    const date = localTodayIso();
    const params = {
      employeeMap: new Map(),
      rows: [absentRow(date)],
      viewingDate: date,
      organizationTimezone: "UTC",
    };

    const pending = computeNoShowRows(params, new Date(`${date}T08:10:00.000Z`));
    const overdue = computeNoShowRows(params, new Date(`${date}T08:16:00.000Z`));

    expect(pending[0]).toMatchObject({ zone: "a", zoneMinutes: 5 });
    expect(overdue[0]).toMatchObject({ zone: "b", zoneMinutes: 1 });
  });

  it("continues after an empty intermediate no-show page", () => {
    expect(getNextNoShowPageParam(page(75), 1)).toBe(2);
    expect(getNextNoShowPageParam(page(75), 2)).toBe(3);
  });

  it("stops exactly after every candidate employee page was scanned", () => {
    expect(getNextNoShowPageParam(page(75), 3)).toBeUndefined();
    expect(getNextNoShowPageParam(page(0), 1)).toBeUndefined();
    expect(getNextNoShowPageParam(page(10, 0), 1)).toBeUndefined();
  });
});
