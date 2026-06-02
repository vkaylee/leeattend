import { renderHook, act } from "@testing-library/react";

import { describe, it, expect } from "vitest";

import { useCalendarPreview } from "../use-calendar-preview";

describe("useCalendarPreview", () => {
  it("initializes with current year and month", () => {
    const { result } = renderHook(() => useCalendarPreview());
    
    expect(result.current.activeViewMode).toBe("design");
    expect(result.current.previewYear).toBe(new Date().getFullYear());
    expect(result.current.previewMonth).toBe(new Date().getMonth());
  });

  it("handles next and previous month navigation", () => {
    const { result } = renderHook(() => useCalendarPreview());
    const initialMonth = result.current.previewMonth;
    const initialYear = result.current.previewYear;

    act(() => {
      result.current.handleNextMonth();
    });

    if (initialMonth === 11) {
      expect(result.current.previewMonth).toBe(0);
      expect(result.current.previewYear).toBe(initialYear + 1);
    } else {
      expect(result.current.previewMonth).toBe(initialMonth + 1);
    }

    act(() => {
      result.current.handlePreviousMonth();
    });

    expect(result.current.previewMonth).toBe(initialMonth);
    expect(result.current.previewYear).toBe(initialYear);
  });

  it("generates calendar cells correctly for weekly patterns", () => {
    const { result } = renderHook(() => useCalendarPreview());
    
    // Set fixed date for deterministic testing
    act(() => {
      result.current.setPreviewAnchorDate("2026-05-27");
    });
    
    const cycleLengthDays = 0; // Weekly pattern
    const grid: string[][] = []; 
    
    const cells = result.current.generateCalendarCells(cycleLengthDays, grid);
    
    // Calendar grid is always 42 cells (6 rows * 7 columns)
    expect(cells).toHaveLength(42);
    
    // Check if the current month has the right amount of days
    const daysInCurrentMonth = cells.filter(c => c.isCurrentMonth);
    expect(daysInCurrentMonth.length).toBeGreaterThanOrEqual(28);
    expect(daysInCurrentMonth.length).toBeLessThanOrEqual(31);
  });

  it("resolves cycle index and shifts correctly for rotation patterns", () => {
    const { result } = renderHook(() => useCalendarPreview());
    
    // Set anchor date to today to make math predictable
    const today = new Date();
    const todayStr = today.toISOString().split("T")[0];
    
    act(() => {
      result.current.setPreviewAnchorDate(todayStr);
    });
    
    const cycleLengthDays = 3;
    const grid: string[][] = [["shift-1"], ["shift-2"], ["shift-3"]]; 
    
    const cells = result.current.generateCalendarCells(cycleLengthDays, grid);
    
    const anchorCell = cells.find(c => c.isAnchor);
    expect(anchorCell).toBeDefined();
    if (anchorCell) {
      expect(anchorCell.cycleIndex).toBe(0);
      expect(anchorCell.dayShifts).toEqual(["shift-1"]);
    }
  });
});
