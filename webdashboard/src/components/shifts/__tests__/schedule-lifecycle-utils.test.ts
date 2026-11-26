import { describe, expect, it } from "vitest";

import type { EffectiveSchedule } from "@/lib/api/types";

import { getScheduleLifecycleCellState, translateRosterEntriesToMonth } from "../roster/roster-utils";
import { formatScheduleLocalTime } from "../schedule-lifecycle-utils";

describe("formatScheduleLocalTime", () => {
  it("converts evidence timestamps to replan local time", () => {
    expect(formatScheduleLocalTime("2026-11-16T08:00:00.000Z", "Asia/Ho_Chi_Minh")).toBe("15:00:00");
  });

  it("returns null for missing or malformed timestamps", () => {
    expect(formatScheduleLocalTime(undefined, "UTC")).toBeNull();
    expect(formatScheduleLocalTime("not-a-time", "UTC")).toBeNull();
  });
});

describe("schedule lifecycle state", () => {
  const day = (overrides: Partial<EffectiveSchedule>): EffectiveSchedule => ({
    date: "2026-11-01", source: "roster", ...overrides,
  } as EffectiveSchedule);

  it("distinguishes missing, explicit off, provisional and published work", () => {
    expect(getScheduleLifecycleCellState([])).toBe("missing");
    expect(getScheduleLifecycleCellState([day({ schedule_status: "explicit_off" })])).toBe("explicit_off");
    expect(getScheduleLifecycleCellState([day({ shift_id: "shift", provisional: true, schedule_status: "work" })])).toBe("provisional");
    expect(getScheduleLifecycleCellState([day({ shift_id: "shift", schedule_status: "work", provisional: false })])).toBe("published");
    expect(getScheduleLifecycleCellState([day({ shift_id: "shift" })])).toBe("provisional");
  });

  it("copies explicit roster days off but not missing days", () => {
    const result = translateRosterEntriesToMonth([{
      employee_id: "employee", employment_id: "employment", full_name: "Employee",
      employee_id_number: "1", schedules: [
        day({ schedule_status: "explicit_off", shift_id: null }),
        day({ date: "2026-11-02", schedule_status: "missing", shift_id: null }),
      ],
    }], "2026-12", new Set(["2026-12-01", "2026-12-02"]));
    expect(result[0].rosterEntries).toEqual([{ date: "2026-12-01", shift_id: null }]);
  });
});
