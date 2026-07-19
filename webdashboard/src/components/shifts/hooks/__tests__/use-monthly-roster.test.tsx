import { renderHook, act } from "@testing-library/react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import type { useShifts } from "@/pages/shifts/hooks/use-shifts";

import { useMonthlyRoster } from "../use-monthly-roster";

// ── Mocks ──────────────────────────────────────────────────────────────────

const mockEmployees = [
  { id: "emp-1", full_name: "Alice", active_employment_id: "emp-1-emp" },
  { id: "emp-2", full_name: "Bob", active_employment_id: "emp-2-emp" },
];

const mockMonthDays = [
  new Date("2026-05-01T00:00:00"),
  new Date("2026-05-02T00:00:00"),
  new Date("2026-05-03T00:00:00"),
];

const createMockHook = () =>
  ({
    monthDays: mockMonthDays,
    handleMonthBulkAssign: vi.fn(),
    handleMonthDeleteRoster: vi.fn(),
    handleCopyPreviousMonth: vi.fn(),
    employees: mockEmployees,
    monthSchedules: [],
    selectedEmployeeIds: ["emp-1", "emp-2"],
    setSelectedEmployeeIds: vi.fn(),
    selectedEmployees: [],
    setSelectedEmployees: vi.fn(),
  }) as unknown as ReturnType<typeof useShifts>;

const primaryPointer = { button: 0 } as React.PointerEvent;

// End the current drag the same way the real global pointerup listener does.
const releasePointer = () => {
  act(() => {
    window.dispatchEvent(new Event("pointerup"));
  });
};

// ── Tests ──────────────────────────────────────────────────────────────────

describe("useMonthlyRoster drag-select paint mode", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("paints the whole rectangle when dragging from an empty cell", () => {
    const { result } = renderHook(() =>
      useMonthlyRoster({ hook: createMockHook() }),
    );

    act(() => {
      result.current.handleCellPointerDown("emp-1", "2026-05-01", primaryPointer);
    });
    act(() => {
      result.current.handleCellPointerEnter("emp-2", "2026-05-02");
    });

    // Rectangle spans emp-1..emp-2 × 2026-05-01..02 → 4 cells.
    expect(result.current.selectedCells).toEqual(
      new Set([
        "emp-1:2026-05-01",
        "emp-1:2026-05-02",
        "emp-2:2026-05-01",
        "emp-2:2026-05-02",
      ]),
    );
  });

  it("erases the whole rectangle when dragging from a selected cell", () => {
    const { result } = renderHook(() =>
      useMonthlyRoster({ hook: createMockHook() }),
    );

    // First drag selects the full 2×2 block.
    act(() => {
      result.current.handleCellPointerDown("emp-1", "2026-05-01", primaryPointer);
    });
    act(() => {
      result.current.handleCellPointerEnter("emp-2", "2026-05-02");
    });
    releasePointer();

    // Second drag starts on an already-selected cell → remove mode.
    act(() => {
      result.current.handleCellPointerDown("emp-1", "2026-05-01", primaryPointer);
    });
    act(() => {
      result.current.handleCellPointerEnter("emp-2", "2026-05-02");
    });

    expect(result.current.selectedCells).toEqual(new Set());
  });

  it("preserves cells selected outside the drag rectangle", () => {
    const { result } = renderHook(() =>
      useMonthlyRoster({ hook: createMockHook() }),
    );

    // Pre-select a cell that will sit outside the next rectangle.
    act(() => {
      result.current.handleCellPointerDown("emp-2", "2026-05-03", primaryPointer);
    });
    releasePointer();

    // Drag a fresh rectangle that does not include emp-2:2026-05-03.
    act(() => {
      result.current.handleCellPointerDown("emp-1", "2026-05-01", primaryPointer);
    });
    act(() => {
      result.current.handleCellPointerEnter("emp-1", "2026-05-02");
    });

    expect(result.current.selectedCells).toEqual(
      new Set([
        "emp-2:2026-05-03",
        "emp-1:2026-05-01",
        "emp-1:2026-05-02",
      ]),
    );
  });

  it("ignores non-primary pointer buttons", () => {
    const { result } = renderHook(() =>
      useMonthlyRoster({ hook: createMockHook() }),
    );

    act(() => {
      result.current.handleCellPointerDown("emp-1", "2026-05-01", {
        button: 2,
      } as React.PointerEvent);
    });

    expect(result.current.selectedCells).toEqual(new Set());
    expect(result.current.isDragging).toBe(false);
  });

  it("does not extend the selection after the pointer is released", () => {
    const { result } = renderHook(() =>
      useMonthlyRoster({ hook: createMockHook() }),
    );

    act(() => {
      result.current.handleCellPointerDown("emp-1", "2026-05-01", primaryPointer);
    });
    releasePointer();

    // A stray pointerenter after release must not paint anything.
    act(() => {
      result.current.handleCellPointerEnter("emp-2", "2026-05-02");
    });

    expect(result.current.selectedCells).toEqual(new Set(["emp-1:2026-05-01"]));
  });
});
