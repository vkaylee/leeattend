import { render, screen } from "@testing-library/react";

import { afterEach, describe, expect, it, vi } from "vitest";

import ReportFilters from "../report-filters";

// Provide a minimal t() that returns the key as-is
const t = (key: string) => key;

const baseProps = {
  dateRange: { start: "2026-07-01", end: "2026-07-31" },
  employeeStatus: "active",
  filterMode: "this_month" as const,
  onDateRangeChange: vi.fn(),
  onDepartmentChange: vi.fn(),
  onEmployeeStatusChange: vi.fn(),
  onFilterModeChange: vi.fn(),
  onSearchQueryChange: vi.fn(),
  onSpecificMonthChange: vi.fn(),
  searchQuery: "",
  selectedDepartmentId: null,
  specificMonth: { month: 7, year: 2026 },
  t,
};

describe("ReportFilters — day-31 month arithmetic", () => {
  afterEach(() => {
    vi.useRealTimers();
  });

  it("uses one geometry contract for month and status segmented controls", () => {
    render(<ReportFilters {...baseProps} />);

    const monthButton = screen.getAllByText(/Tháng/)[0].closest("button");
    const statusButton = screen.getByText("reports.filters.status.active");

    expect(monthButton).toHaveClass("!h-full", "!px-3");
    expect(statusButton).toHaveClass("!h-full", "!px-3");
    expect(monthButton?.parentElement).toHaveClass("h-[var(--ui-height-sm)]", "p-1");
    expect(statusButton.parentElement).toHaveClass("h-[var(--ui-height-sm)]", "p-1");
  });

  it("shows distinct labels for this_month and last_month when today is July 31", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-07-31T12:00:00"));

    render(<ReportFilters {...baseProps} />);

    // Current month = July (7), previous month = June (6)
    // The bug was: both buttons showed "THÁNG 7/2026"
    expect(screen.getByText(/Tháng 7\/2026/i)).toBeDefined();
    expect(screen.getByText(/Tháng 6\/2026/i)).toBeDefined();
  });

  it("shows distinct labels when today is March 31 (prev = February)", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-03-31T12:00:00"));

    render(<ReportFilters {...baseProps} />);

    expect(screen.getByText(/Tháng 3\/2026/i)).toBeDefined();
    expect(screen.getByText(/Tháng 2\/2026/i)).toBeDefined();
  });

  it("handles year boundary: January 31 → previous December", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-01-31T12:00:00"));

    render(<ReportFilters {...baseProps} />);

    expect(screen.getByText(/Tháng 1\/2026/i)).toBeDefined();
    expect(screen.getByText(/Tháng 12\/2025/i)).toBeDefined();
  });

  it("works correctly on non-31st days too", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-07-15T12:00:00"));

    render(<ReportFilters {...baseProps} />);

    expect(screen.getByText(/Tháng 7\/2026/i)).toBeDefined();
    expect(screen.getByText(/Tháng 6\/2026/i)).toBeDefined();
  });

  it("handles October 31 → September (30-day previous month)", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-10-31T12:00:00"));

    render(<ReportFilters {...baseProps} />);

    expect(screen.getByText(/Tháng 10\/2026/i)).toBeDefined();
    expect(screen.getByText(/Tháng 9\/2026/i)).toBeDefined();
  });
});
