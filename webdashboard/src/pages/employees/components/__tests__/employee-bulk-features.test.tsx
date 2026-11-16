import { render, screen, fireEvent, renderHook, act } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi } from "vitest";

import type { Employee } from "@/lib/api/types";

import { useEmployeeBulkSelection } from "../../hooks/use-employee-bulk-selection";
import { EmployeeBulkActionBar } from "../employee-bulk-action-bar";
import { EmployeeQuickTabs } from "../employee-quick-tabs";
import { EmployeeRowActions } from "../employee-row-actions";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (k: string, fallback?: unknown) => (typeof fallback === "string" ? fallback : k) }),
}));

vi.mock("@/components/common/permission-gate", () => ({
  default: ({ children }: { children: React.ReactNode }) => <>{children}</>,
}));

const mockEmployees: Employee[] = [
  {
    id: "emp-1",
    active_employment_id: "active-1",
    created_at: "2025-01-01T00:00:00Z",
    department_name: "Engineering",
    employee_id_number: "EMP001",
    full_name: "Alice Nguyen",
    hire_date: "2024-01-15",
    job_title: "Software Engineer",
    tags: [],
    updated_at: "2025-01-01T00:00:00Z",
    workspace_id: "ws-1",
  },
  {
    id: "emp-2",
    active_employment_id: null,
    created_at: "2025-01-01T00:00:00Z",
    department_name: "HR",
    employee_id_number: "EMP002",
    full_name: "Bob Tran",
    hire_date: "2023-05-10",
    job_title: "HR Specialist",
    tags: [],
    updated_at: "2025-01-01T00:00:00Z",
    workspace_id: "ws-1",
  },
];

describe("useEmployeeBulkSelection", () => {
  it("manages selection correctly", () => {
    const { result } = renderHook(() => useEmployeeBulkSelection(mockEmployees));

    expect(result.current.selectedIds).toEqual([]);
    expect(result.current.isAllSelected).toBe(false);

    // Toggle one
    act(() => {
      result.current.toggleSelect("emp-1");
    });
    expect(result.current.selectedIds).toEqual(["emp-1"]);
    expect(result.current.isAllSelected).toBe(false);

    // Select all
    act(() => {
      result.current.selectAll();
    });
    expect(result.current.selectedIds).toEqual(["emp-1", "emp-2"]);
    expect(result.current.isAllSelected).toBe(true);

    // Clear
    act(() => {
      result.current.clearSelection();
    });
    expect(result.current.selectedIds).toEqual([]);
    expect(result.current.isAllSelected).toBe(false);

    // Toggle all
    act(() => {
      result.current.toggleSelectAll();
    });
    expect(result.current.selectedIds).toEqual(["emp-1", "emp-2"]);

    act(() => {
      result.current.toggleSelectAll();
    });
    expect(result.current.selectedIds).toEqual([]);
  });
});

describe("EmployeeQuickTabs", () => {
  it("renders status tabs and triggers setFilterStatus", () => {
    const setFilterStatus = vi.fn();
    render(
      <EmployeeQuickTabs
        activeCount={10}
        filterStatus="active"
        setFilterStatus={setFilterStatus}
        totalCount={20}
      />
    );

    expect(screen.getByTestId("quick-tab-active")).toBeDefined();
    expect(screen.getByTestId("quick-tab-terminated")).toBeDefined();
    expect(screen.getByTestId("quick-tab-all")).toBeDefined();

    fireEvent.click(screen.getByTestId("quick-tab-terminated"));
    expect(setFilterStatus).toHaveBeenCalledWith("terminated");

    fireEvent.click(screen.getByTestId("quick-tab-all"));
    expect(setFilterStatus).toHaveBeenCalledWith("all");
  });
});

describe("EmployeeBulkActionBar", () => {
  it("returns null when no employees are selected", () => {
    const { container } = render(
      <EmployeeBulkActionBar
        employees={mockEmployees}
        onClearSelection={vi.fn()}
        selectedIds={[]}
      />
    );
    expect(container.firstChild).toBeNull();
  });

  it("renders action bar and handles clear & export", () => {
    const onClear = vi.fn();
    render(
      <EmployeeBulkActionBar
        employees={mockEmployees}
        onClearSelection={onClear}
        selectedIds={["emp-1"]}
      />
    );

    expect(screen.getByTestId("employee-bulk-action-bar")).toBeDefined();

    fireEvent.click(screen.getByTestId("bulk-clear-btn"));
    expect(onClear).toHaveBeenCalled();

    // Export click
    const exportBtn = screen.getByTestId("bulk-export-btn");
    expect(exportBtn).toBeDefined();
  });
});

describe("EmployeeRowActions", () => {
  it("renders dropdown trigger button", () => {
    const onOpenModal = vi.fn();

    render(
      <EmployeeRowActions
        employee={mockEmployees[0]}
        onOpenModal={onOpenModal}
      />
    );

    expect(screen.getByTestId("employee-actions-emp-1")).toBeDefined();
  });
});
