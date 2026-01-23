import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import PayrollSummaryTable from "../PayrollSummaryTable";
import { Employee, Payslip } from "@/lib/api/types";
import { BrowserRouter } from "react-router-dom";

// Mock i18next
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}));

const mockEmployees: Employee[] = [
  {
    id: "emp-1",
    full_name: "John Doe",
    employee_id_number: "EMP001",
    workspace_id: "ws-1",
    hire_date: "2024-01-01",
    is_active: true,
    created_at: "",
    updated_at: ""
  },
  {
    id: "emp-2",
    full_name: "Jane Smith",
    employee_id_number: "EMP002",
    workspace_id: "ws-1",
    hire_date: "2024-01-01",
    is_active: true,
    created_at: "",
    updated_at: ""
  }
];

const mockPayslips: Payslip[] = [
  {
    id: "slip-1",
    employee_id: "emp-1",
    period_start: "2026-01-01",
    period_end: "2026-01-31",
    base_pay_amount: "10000000",
    overtime_pay_amount: "500000",
    deductions_amount: "0",
    net_pay_amount: "10500000",
    standard_units: 200,
    overtime_units: 10,
    status: "published",
    created_at: "",
    updated_at: ""
  }
];

describe("PayrollSummaryTable", () => {
  const onToggleSelect = vi.fn();
  const onToggleSelectAll = vi.fn();
  const onViewDetail = vi.fn();
  const onSearchChange = vi.fn();

  it("renders employee list correctly", () => {
    render(
      <BrowserRouter>
        <PayrollSummaryTable 
          employees={mockEmployees}
          payslips={mockPayslips}
          isLoading={false}
          selectedIds={new Set()}
          onToggleSelect={onToggleSelect}
          onToggleSelectAll={onToggleSelectAll}
          onViewDetail={onViewDetail}
          searchTerm=""
          onSearchChange={onSearchChange}
        />
      </BrowserRouter>
    );

    expect(screen.getByText("John Doe")).toBeDefined();
    expect(screen.getByText("Jane Smith")).toBeDefined();
    expect(screen.getByText("10.500.000")).toBeDefined(); // Net pay for John
  });

  it("filters employees based on search term", () => {
    render(
      <BrowserRouter>
        <PayrollSummaryTable 
          employees={mockEmployees}
          payslips={mockPayslips}
          isLoading={false}
          selectedIds={new Set()}
          onToggleSelect={onToggleSelect}
          onToggleSelectAll={onToggleSelectAll}
          onViewDetail={onViewDetail}
          searchTerm="Jane"
          onSearchChange={onSearchChange}
        />
      </BrowserRouter>
    );

    expect(screen.queryByText("John Doe")).toBeNull();
    expect(screen.getByText("Jane Smith")).toBeDefined();
  });

  it("calls onToggleSelect when checkbox is clicked", () => {
    render(
      <BrowserRouter>
        <PayrollSummaryTable 
          employees={mockEmployees}
          payslips={mockPayslips}
          isLoading={false}
          selectedIds={new Set()}
          onToggleSelect={onToggleSelect}
          onToggleSelectAll={onToggleSelectAll}
          onViewDetail={onViewDetail}
          searchTerm=""
          onSearchChange={onSearchChange}
        />
      </BrowserRouter>
    );

    const checkbox = screen.getByTestId("select-employee-emp-1");
    fireEvent.click(checkbox);
    expect(onToggleSelect).toHaveBeenCalledWith("emp-1");
  });

  it("calls onViewDetail when row is clicked", () => {
    render(
      <BrowserRouter>
        <PayrollSummaryTable 
          employees={mockEmployees}
          payslips={mockPayslips}
          isLoading={false}
          selectedIds={new Set()}
          onToggleSelect={onToggleSelect}
          onToggleSelectAll={onToggleSelectAll}
          onViewDetail={onViewDetail}
          searchTerm=""
          onSearchChange={onSearchChange}
        />
      </BrowserRouter>
    );

    fireEvent.click(screen.getByText("John Doe"));
    expect(onViewDetail).toHaveBeenCalledWith(mockEmployees[0]);
  });
});
