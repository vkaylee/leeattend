import { render, screen } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi } from "vitest";

import { AttendanceSummary, Department, Employer } from "@/lib/api/types";

import { ReportsTable } from "../reports-table";

// Mock the child components to simplify testing the DOM layout
vi.mock("@/components/features/reports/report-filters", () => ({
  default: () => <div data-testid="mock-report-filters">Mock Filters</div>
}));

vi.mock("@/components/common/navigable-list", () => ({
  default: () => <div data-testid="mock-navigable-list">Mock Navigable List</div>
}));

describe("ReportsTable", () => {
  const mockDepartments = [
    { id: "d1", name: "Engineering" },
    { id: "d2", name: "HR" }
  ] as Department[];

  const defaultProps = {
    dateRange: { start: "2024-03-01", end: "2024-03-31" },
    departments: mockDepartments,
    employers: [] as Employer[],
    selectedEmployerId: null,
    handleEmployerChange: vi.fn(),
    fetchNextPage: vi.fn(),
    filteredData: [] as AttendanceSummary[],
    filterMode: "this_month" as const,
    handleDateRangeChange: vi.fn(),
    handleDepartmentChange: vi.fn(),
    handleEmployeeClick: vi.fn(),
    handleFilterModeChange: vi.fn(),
    handleSort: vi.fn(),
    handleSpecificMonthChange: vi.fn(),
    hasNextPage: false,
    isFetchingNextPage: false,
    isLoading: false,
    keyDownRef: { current: null },
    quickStats: { otHours: "0", stdHours: "0", totalEmployees: 0 },
    searchInputRef: { current: null },
    searchQuery: "",
    selectedDepartmentId: null,
    setSearchQuery: vi.fn(),
    sortBy: undefined,
    specificMonth: { month: 3, year: 2024 },
    employeeStatus: "active",
    onEmployeeStatusChange: vi.fn(),
  };

  it("renders ReportFilters and NavigableList as siblings in a flex-col container to prevent layout squashing", () => {
    render(<ReportsTable {...defaultProps} />);

    const filters = screen.getByTestId("mock-report-filters");
    const list = screen.getByTestId("mock-navigable-list");

    expect(filters).toBeInTheDocument();
    expect(list).toBeInTheDocument();

    // With ResponsiveToolbar, filters are wrapped in multiple toolbar layers
    // Navigate up to find the toolbar wrapper (div.flex.w-full.items-center.gap-2)
    let toolbarWrapper = filters.parentElement;
    while (toolbarWrapper && !toolbarWrapper.className.includes("w-full")) {
      toolbarWrapper = toolbarWrapper.parentElement;
    }

    // The toolbar wrapper and NavigableList should share the same parent (flex-col container)
    const listParent = list.parentElement;
    expect(toolbarWrapper?.parentElement).toBe(listParent);

    // Verify the grandparent is a flex-col container
    const grandparent = toolbarWrapper?.parentElement;
    expect(grandparent?.className).toContain("flex");
    expect(grandparent?.className).toContain("flex-col");
    expect(grandparent?.className).toMatch(/gap-[46]/);
  });
});
