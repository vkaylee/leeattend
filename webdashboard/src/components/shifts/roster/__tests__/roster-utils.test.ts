import { describe, it, expect } from "vitest";

import {
  getCellStyles,
  getHighestPrioritySource,
  getShiftSymbol,
  isWeekend,
  orderSchedulesBySelection,
} from "../roster-utils";

// ── isWeekend ───────────────────────────────────────────────────────

describe("isWeekend", () => {
  it("returns true for Saturday (day 6)", () => {
    // 2026-07-18 is a Saturday
    expect(isWeekend(new Date("2026-07-18"))).toBe(true);
  });

  it("returns true for Sunday (day 0)", () => {
    // 2026-07-19 is a Sunday
    expect(isWeekend(new Date("2026-07-19"))).toBe(true);
  });

  it("returns false for Monday", () => {
    // 2026-07-20 is a Monday
    expect(isWeekend(new Date("2026-07-20"))).toBe(false);
  });

  it("returns false for Friday", () => {
    // 2026-07-17 is a Friday
    expect(isWeekend(new Date("2026-07-17"))).toBe(false);
  });

  it("returns false for Wednesday (midweek)", () => {
    // 2026-07-15 is a Wednesday
    expect(isWeekend(new Date("2026-07-15"))).toBe(false);
  });
});

// ── getShiftSymbol ──────────────────────────────────────────────────

describe("getShiftSymbol", () => {
  it("returns reportSymbol when provided (overrides shift name)", () => {
    expect(getShiftSymbol("Morning Shift", "M")).toBe("M");
  });

  it("returns first 2 chars uppercase when shift name given", () => {
    expect(getShiftSymbol("Morning Shift", null)).toBe("MO");
  });

  it("returns empty string when shift name is null", () => {
    expect(getShiftSymbol(null)).toBe("");
  });

  it("returns empty string when shift name is undefined", () => {
    expect(getShiftSymbol(undefined)).toBe("");
  });

  it("returns empty string when shift name is empty", () => {
    expect(getShiftSymbol("")).toBe("");
  });

  it("returns reportSymbol even for short codes", () => {
    expect(getShiftSymbol("A", "Z")).toBe("Z");
  });

  it("handles single-char shift name", () => {
    expect(getShiftSymbol("A")).toBe("A");
  });
});

// ── getCellStyles ───────────────────────────────────────────────────

describe("getCellStyles", () => {
  it("includes base styles always", () => {
    const result = getCellStyles();
    expect(result).toContain("transition-all");
    expect(result).toContain("border border-slate-100");
    expect(result).toContain("select-none");
    expect(result).toContain("cursor-pointer");
  });

  it("adds selected ring styles when isSelected is true", () => {
    const result = getCellStyles("roster", true);
    expect(result).toContain("ring-2 ring-primary-500");
    expect(result).toContain("bg-primary-50");
    // Selected should take priority over source
    expect(result).not.toContain("bg-teal-50");
  });

  it("returns teal styles for 'roster' source", () => {
    const result = getCellStyles("roster");
    expect(result).toContain("bg-teal-50/90");
    expect(result).toContain("hover:bg-teal-100");
  });

  it("returns blue styles for 'pattern' source", () => {
    const result = getCellStyles("pattern");
    expect(result).toContain("bg-blue-50/40");
    expect(result).toContain("hover:bg-blue-100/30");
  });

  it("returns violet styles for 'assignment_rule' source", () => {
    const result = getCellStyles("assignment_rule");
    expect(result).toContain("bg-violet-50/40");
    expect(result).toContain("hover:bg-violet-100/30");
  });

  it("returns slate styles for 'department_pattern' source", () => {
    const result = getCellStyles("department_pattern");
    expect(result).toContain("bg-slate-50/50");
    expect(result).toContain("hover:bg-slate-100/40");
  });

  it("returns emerald styles for 'fixed' source", () => {
    const result = getCellStyles("fixed");
    expect(result).toContain("bg-emerald-50/40");
    expect(result).toContain("hover:bg-emerald-100/30");
  });

  it("returns emerald styles for 'department_fixed' source", () => {
    const result = getCellStyles("department_fixed");
    expect(result).toContain("bg-emerald-50/40");
    expect(result).toContain("hover:bg-emerald-100/30");
  });

  it("returns default hover style for unknown source", () => {
    const result = getCellStyles("unknown_source");
    expect(result).toContain("hover:bg-slate-50");
    expect(result).not.toContain("bg-teal");
    expect(result).not.toContain("bg-blue");
  });

  it("returns default hover style for undefined source", () => {
    const result = getCellStyles(undefined);
    expect(result).toContain("hover:bg-slate-50");
  });
});

// ── getHighestPrioritySource ────────────────────────────────────────

describe("getHighestPrioritySource", () => {
  it("returns roster as highest priority", () => {
    const schedules = [
      { source: "pattern" },
      { source: "roster" },
      { source: "fixed" },
    ] as any[];
    expect(getHighestPrioritySource(schedules)).toBe("roster");
  });

  it("returns pattern when roster is absent", () => {
    const schedules = [
      { source: "fixed" },
      { source: "pattern" },
    ] as any[];
    expect(getHighestPrioritySource(schedules)).toBe("pattern");
  });

  it("returns assignment_rule over department_pattern", () => {
    const schedules = [
      { source: "department_pattern" },
      { source: "assignment_rule" },
    ] as any[];
    expect(getHighestPrioritySource(schedules)).toBe("assignment_rule");
  });

  it("returns department_pattern over fixed", () => {
    const schedules = [
      { source: "fixed" },
      { source: "department_pattern" },
    ] as any[];
    expect(getHighestPrioritySource(schedules)).toBe("department_pattern");
  });

  it("returns fixed when no higher source present", () => {
    const schedules = [{ source: "fixed" }] as any[];
    expect(getHighestPrioritySource(schedules)).toBe("fixed");
  });

  it("returns department_fixed as lowest priority", () => {
    const schedules = [{ source: "department_fixed" }] as any[];
    expect(getHighestPrioritySource(schedules)).toBe("department_fixed");
  });

  it("returns fixed over department_fixed (same tier)", () => {
    const schedules = [
      { source: "department_fixed" },
      { source: "fixed" },
    ] as any[];
    expect(getHighestPrioritySource(schedules)).toBe("fixed");
  });

  it("returns undefined for empty schedules", () => {
    expect(getHighestPrioritySource([])).toBeUndefined();
  });

  it("deduplicates sources", () => {
    const schedules = [
      { source: "roster" },
      { source: "roster" },
      { source: "roster" },
    ] as any[];
    expect(getHighestPrioritySource(schedules)).toBe("roster");
  });
});

// ── orderSchedulesBySelection ───────────────────────────────────────

describe("orderSchedulesBySelection", () => {
  const rows = [
    { employee_id: "a", full_name: "Alice" },
    { employee_id: "b", full_name: "Bob" },
    { employee_id: "c", full_name: "Carol" },
  ];

  it("orders rows by selection order, not input order", () => {
    const result = orderSchedulesBySelection(rows, ["c", "a", "b"]);
    expect(result.map((r) => r.employee_id)).toEqual(["c", "a", "b"]);
  });

  it("returns empty array when no ids selected", () => {
    expect(orderSchedulesBySelection(rows, [])).toEqual([]);
  });

  it("drops ids that have no matching schedule", () => {
    const result = orderSchedulesBySelection(rows, ["b", "missing", "a"]);
    expect(result.map((r) => r.employee_id)).toEqual(["b", "a"]);
  });

  it("returns only the selected subset", () => {
    const result = orderSchedulesBySelection(rows, ["c"]);
    expect(result.map((r) => r.employee_id)).toEqual(["c"]);
  });

  it("does not include duplicates when an id is selected twice", () => {
    const result = orderSchedulesBySelection(rows, ["a", "a"]);
    // same reference returned for each occurrence; order preserved
    expect(result.map((r) => r.employee_id)).toEqual(["a", "a"]);
  });
});
