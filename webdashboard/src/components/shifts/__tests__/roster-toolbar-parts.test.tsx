import { render, screen } from "@testing-library/react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import "@testing-library/jest-dom";

import { Shift } from "@/lib/api/types";

import { RosterActionBar, formatShiftTimeRange } from "../roster-toolbar-parts";

// ── Mocks ────────────────────────────────────────────────────────────

const STORAGE_KEY = "leeattend_frequent_shifts";

beforeEach(() => {
  localStorage.clear();
  vi.restoreAllMocks();
});

// ── Fixtures ─────────────────────────────────────────────────────────

type IntervalFixture = Partial<{
  end_time: string;
  start_time: string;
  sort_order: number;
}>;

function makeShift(overrides: Partial<Shift> = {}, intervals: IntervalFixture[] = []): Shift {
  const shiftIntervals = intervals.length > 0
    ? intervals.map((iv, i) => ({
        allow_early_ot: false,
        break_duration_minutes: 60,
        break_end: null,
        break_is_overtime: false,
        break_start: null,
        early_punch_in_grace_period_minutes: 0,
        end_time: iv.end_time ?? "17:00:00",
        id: `iv-${i}`,
        interval_name: null,
        is_break_auto_deduct: true,
        is_night_shift: false,
        max_early_ot_minutes: null,
        ot_aggregation_mode: "daily" as const,
        sort_order: iv.sort_order ?? i,
        start_time: iv.start_time ?? "08:00:00",
        version_id: "v-1",
      }))
    : null;

  return {
    id: "shift-1",
    name: "Ca Hành Chính",
    report_symbol: "HC",
    is_active: true,
    workspace_id: "ws-1",
    created_at: "2024-01-01T00:00:00Z",
    updated_at: "2024-01-01T00:00:00Z",
    intervals: shiftIntervals,
    ...overrides,
  } as Shift;
}

const noop = () => undefined;

// ── formatShiftTimeRange tests ──────────────────────────────────────

describe("formatShiftTimeRange", () => {
  it("returns empty string for shift with no intervals", () => {
    const shift = makeShift({ name: "Empty" }, []);
    expect(formatShiftTimeRange(shift)).toBe("");
  });

  it("formats a single-interval shift as HH:MM–HH:MM", () => {
    const shift = makeShift({ name: "Morning" }, [
      { start_time: "08:00:00", end_time: "17:00:00" },
    ]);
    expect(formatShiftTimeRange(shift)).toBe("08:00:00–17:00:00");
  });

  it("joins multiple intervals with / separator", () => {
    const shift = makeShift({ name: "Split" }, [
      { start_time: "08:00:00", end_time: "12:00:00", sort_order: 0 },
      { start_time: "14:00:00", end_time: "18:00:00", sort_order: 1 },
    ]);
    expect(formatShiftTimeRange(shift)).toBe("08:00:00/14:00:00–12:00:00/18:00:00");
  });

  it("formats night shift crossing midnight", () => {
    const shift = makeShift({ name: "Night" }, [
      { start_time: "22:00:00", end_time: "06:00:00" },
    ]);
    expect(formatShiftTimeRange(shift)).toBe("22:00:00–06:00:00");
  });
});

// ── RosterActionBar component tests ─────────────────────────────────

describe("RosterActionBar", () => {
  it("renders selected count label", () => {
    render(
      <RosterActionBar
        assignLabel="Assign selected"
        cancelLabel="Cancel"
        clearLabel="Clear overrides"
        isMutating={false}
        onApplyShift={noop}
        onAssignSelected={noop}
        onCancel={noop}
        onClearOverrides={noop}
        selectedCount={3}
        selectedLabel="3 cells selected"
        shifts={[]}
      />,
    );
    expect(screen.getByText("3 cells selected")).toBeInTheDocument();
  });

  it("renders quick-shift buttons with shift name and time range", () => {
    const shifts = [
      makeShift(
        { id: "s1", name: "Ca Sáng", report_symbol: "S" },
        [{ start_time: "08:00:00", end_time: "17:00:00" }],
      ),
      makeShift(
        { id: "s2", name: "Ca Tối", report_symbol: "T" },
        [{ start_time: "18:00:00", end_time: "22:00:00" }],
      ),
    ];

    render(
      <RosterActionBar
        assignLabel="Assign selected"
        cancelLabel="Cancel"
        clearLabel="Clear overrides"
        isMutating={false}
        onApplyShift={noop}
        onAssignSelected={noop}
        onCancel={noop}
        onClearOverrides={noop}
        selectedCount={1}
        selectedLabel="1 cell selected"
        shifts={shifts}
      />,
    );

    expect(screen.getByText("Ca Sáng")).toBeInTheDocument();
    expect(screen.getByText("Ca Tối")).toBeInTheDocument();
    expect(screen.getByText("08:00:00–17:00:00")).toBeInTheDocument();
    expect(screen.getByText("18:00:00–22:00:00")).toBeInTheDocument();
  });

  it("renders accent dots for each quick-shift button", () => {
    const shifts = [
      makeShift(
        { id: "s1", name: "Ca Sáng" },
        [{ start_time: "08:00:00", end_time: "12:00:00" }],
      ),
    ];

    render(
      <RosterActionBar
        assignLabel="Assign selected"
        cancelLabel="Cancel"
        clearLabel="Clear overrides"
        isMutating={false}
        onApplyShift={noop}
        onAssignSelected={noop}
        onCancel={noop}
        onClearOverrides={noop}
        selectedCount={1}
        selectedLabel="1 cell selected"
        shifts={shifts}
      />,
    );

    // The dot span has aria-hidden="true" and the accent class
    const dots = document.querySelectorAll('[aria-hidden="true"].rounded-full');
    expect(dots.length).toBeGreaterThanOrEqual(1);
  });

  it("calls onApplyShift with shift ID when quick button is clicked", async () => {
    const user = userEvent.setup();
    const onApplyShift = vi.fn();
    const shifts = [
      makeShift(
        { id: "shift-morning", name: "Ca Sáng" },
        [{ start_time: "08:00:00", end_time: "17:00:00" }],
      ),
    ];

    render(
      <RosterActionBar
        assignLabel="Assign selected"
        cancelLabel="Cancel"
        clearLabel="Clear overrides"
        isMutating={false}
        onApplyShift={onApplyShift}
        onAssignSelected={noop}
        onCancel={noop}
        onClearOverrides={noop}
        selectedCount={1}
        selectedLabel="1 cell selected"
        shifts={shifts}
      />,
    );

    await user.click(screen.getByText("Ca Sáng"));
    expect(onApplyShift).toHaveBeenCalledWith("shift-morning");
  });

  it("renders tooltip with full shift info on quick buttons", () => {
    const shifts = [
      makeShift(
        { id: "s1", name: "Ca Sáng", report_symbol: "S" },
        [{ start_time: "08:00:00", end_time: "17:00:00" }],
      ),
    ];

    render(
      <RosterActionBar
        assignLabel="Assign selected"
        cancelLabel="Cancel"
        clearLabel="Clear overrides"
        isMutating={false}
        onApplyShift={noop}
        onAssignSelected={noop}
        onCancel={noop}
        onClearOverrides={noop}
        selectedCount={1}
        selectedLabel="1 cell selected"
        shifts={shifts}
      />,
    );

    const button = screen.getByLabelText("Assign Ca Sáng shift");
    expect(button).toHaveAttribute("title", "Ca Sáng (S): 08:00:00–17:00:00");
  });

  it("renders aria-label for accessibility", () => {
    const shifts = [
      makeShift(
        { id: "s1", name: "Ca Tối" },
        [{ start_time: "18:00:00", end_time: "22:00:00" }],
      ),
    ];

    render(
      <RosterActionBar
        assignLabel="Assign selected"
        cancelLabel="Cancel"
        clearLabel="Clear overrides"
        isMutating={false}
        onApplyShift={noop}
        onAssignSelected={noop}
        onCancel={noop}
        onClearOverrides={noop}
        selectedCount={1}
        selectedLabel="1 cell selected"
        shifts={shifts}
      />,
    );

    expect(screen.getByLabelText("Assign Ca Tối shift")).toBeInTheDocument();
  });

  it("sorts shifts by frequency from localStorage", () => {
    // Pre-populate localStorage: Ca Tối used 5 times, Ca Sáng used 2 times
    localStorage.setItem(
      STORAGE_KEY,
      JSON.stringify({ "shift-night": 5, "shift-morning": 2 }),
    );

    const shifts = [
      makeShift(
        { id: "shift-morning", name: "Ca Sáng" },
        [{ start_time: "08:00:00", end_time: "12:00:00" }],
      ),
      makeShift(
        { id: "shift-night", name: "Ca Tối" },
        [{ start_time: "18:00:00", end_time: "22:00:00" }],
      ),
      makeShift(
        { id: "shift-afternoon", name: "Ca Chiều" },
        [{ start_time: "12:00:00", end_time: "18:00:00" }],
      ),
    ];

    render(
      <RosterActionBar
        assignLabel="Assign selected"
        cancelLabel="Cancel"
        clearLabel="Clear overrides"
        isMutating={false}
        onApplyShift={noop}
        onAssignSelected={noop}
        onCancel={noop}
        onClearOverrides={noop}
        selectedCount={1}
        selectedLabel="1 cell selected"
        shifts={shifts}
      />,
    );

    // All shift names should appear in the buttons
    const buttons = screen.getAllByRole("button");
    // Find buttons with shift names (they have aria-label starting with "Assign")
    const shiftButtons = buttons.filter(
      (b) => b.getAttribute("aria-label")?.startsWith("Assign ") ?? false,
    );

    expect(shiftButtons).toHaveLength(3);
    // Most frequent (Ca Tối, 5 uses) should be first
    expect(shiftButtons[0]).toHaveAttribute("aria-label", "Assign Ca Tối shift");
    // Second most frequent (Ca Sáng, 2 uses) should be second
    expect(shiftButtons[1]).toHaveAttribute("aria-label", "Assign Ca Sáng shift");
    // Least frequent (Ca Chiều, 0 uses) should be last
    expect(shiftButtons[2]).toHaveAttribute("aria-label", "Assign Ca Chiều shift");
  });

  it("disables quick buttons when isMutating is true", () => {
    const shifts = [
      makeShift(
        { id: "s1", name: "Ca Sáng" },
        [{ start_time: "08:00:00", end_time: "17:00:00" }],
      ),
    ];

    render(
      <RosterActionBar
        assignLabel="Assign selected"
        cancelLabel="Cancel"
        clearLabel="Clear overrides"
        isMutating={true}
        onApplyShift={noop}
        onAssignSelected={noop}
        onCancel={noop}
        onClearOverrides={noop}
        selectedCount={1}
        selectedLabel="1 cell selected"
        shifts={shifts}
      />,
    );

    const button = screen.getByLabelText("Assign Ca Sáng shift");
    expect(button).toBeDisabled();
  });

  it("renders no quick-shift section when shifts array is empty", () => {
    render(
      <RosterActionBar
        assignLabel="Assign selected"
        cancelLabel="Cancel"
        clearLabel="Clear overrides"
        isMutating={false}
        onApplyShift={noop}
        onAssignSelected={noop}
        onCancel={noop}
        onClearOverrides={noop}
        selectedCount={1}
        selectedLabel="1 cell selected"
        shifts={[]}
      />,
    );

    expect(screen.queryByText("Quick:")).not.toBeInTheDocument();
  });
});
