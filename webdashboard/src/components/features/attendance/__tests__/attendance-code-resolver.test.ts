import { describe, expect, it } from "vitest";

import type { DailyAttendanceReport } from "@/lib/api";

import { resolveDailyAttendanceCode } from "../attendance-code-resolver";

describe("resolveDailyAttendanceCode", () => {
  it("returns '-' for empty weekday data and 'Ro' for empty weekend", () => {
    expect(resolveDailyAttendanceCode(null, { isWeekend: false }).code).toBe("-");
    expect(resolveDailyAttendanceCode(undefined, { isWeekend: true }).code).toBe("Ro");
  });

  it("prioritizes holiday status with code 'L'", () => {
    const holidayRow = {
      is_holiday: true,
      computed_std_hours: 8,
    } as unknown as DailyAttendanceReport;

    const res = resolveDailyAttendanceCode(holidayRow);
    expect(res.code).toBe("L");
    expect(res.isHoliday).toBe(true);
    expect(res.paidMultiplier).toBe(1.0);
  });

  it("uses custom shift symbol when provided in registry", () => {
    const ctRow = {
      is_holiday: false,
      shift_symbol: "CT",
      status: "present",
    } as unknown as DailyAttendanceReport;

    expect(resolveDailyAttendanceCode(ctRow).code).toBe("CT");
  });

  it("resolves paid leave to 'P' and unpaid leave to 'KL'", () => {
    const paidLeaveRow = {
      is_holiday: false,
      status: "leave",
      is_leave_paid: true,
    } as unknown as DailyAttendanceReport;

    const unpaidLeaveRow = {
      is_holiday: false,
      status: "leave",
      is_leave_paid: false,
    } as unknown as DailyAttendanceReport;

    expect(resolveDailyAttendanceCode(paidLeaveRow).code).toBe("P");
    expect(resolveDailyAttendanceCode(unpaidLeaveRow).code).toBe("KL");
  });

  it("resolves work hours: >= 7h to 'X', 3.5h to 'X/2'", () => {
    const fullDayRow = {
      is_holiday: false,
      status: "present",
      computed_std_hours: 8.0,
    } as unknown as DailyAttendanceReport;

    const halfDayRow = {
      is_holiday: false,
      status: "present",
      computed_std_hours: 4.0,
    } as unknown as DailyAttendanceReport;

    expect(resolveDailyAttendanceCode(fullDayRow).code).toBe("X");
    expect(resolveDailyAttendanceCode(halfDayRow).code).toBe("X/2");
  });

  it("resolves absence to 'V' on weekday and 'Ro' on weekend", () => {
    const absentRow = {
      is_holiday: false,
      status: "absent",
      computed_std_hours: 0,
    } as unknown as DailyAttendanceReport;

    expect(resolveDailyAttendanceCode(absentRow, { isWeekend: false }).code).toBe("V");
    expect(resolveDailyAttendanceCode(absentRow, { isWeekend: true }).code).toBe("Ro");
  });
});
