import "@testing-library/jest-dom";
import { render, screen, fireEvent } from "@testing-library/react";

import { describe, it, expect, vi } from "vitest";

// Mock TanStack Router
vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => vi.fn(),
  useLocation: () => ({ pathname: "/", search: "", hash: "" }),
 
  Link: ({ to, children, ...props }: { to: string; children: React.ReactNode }) => <a href={to} {...props}>{children}</a>,
}));
import { Employee, Payslip } from "@/lib/api/types";

import PayrollSummaryTable from "../payroll-summary-table";

// Mock i18next
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}));

vi.mock("@/hooks/use-permissions", () => ({
  useMyPermissions: () => ({
    hasPermission: () => true,
  }),
}));

vi.mock("@/components/ui/toast", () => ({
  useToast: () => ({
    success: vi.fn(),
    error: vi.fn(),
    show: vi.fn(),
  }),
}));

const mockEmployees: Employee[] = [
  {
    id: "emp-1",
    full_name: "John Doe",
    employee_id_number: "EMP001",
    workspace_id: "ws-1",
    active_employment_id: "emp-1",
    created_at: "",
    updated_at: "",
    tags: []
  },
  {
    id: "emp-2",
    full_name: "Jane Smith",
    employee_id_number: "EMP002",
    workspace_id: "ws-1",
    active_employment_id: "emp-2",
    created_at: "",
    updated_at: "",
    tags: []
  }
];

const mockPayslips: Payslip[] = [
  {
    id: "slip-1",
    workspace_id: "ws-1",
    employment_id: "emp-1",
    period_start: "2026-01-01",
    period_end: "2026-01-31",
    base_pay_amount: "10000000",
    overtime_pay_amount: "500000",
    deductions_amount: "0",
    net_pay_amount: "10500000",
    holiday_work_premium_amount: "0",
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
        <PayrollSummaryTable
          employees={mockEmployees}
          isLoading={false}
          onSearchChange={onSearchChange}
          onToggleSelect={onToggleSelect}
          onToggleSelectAll={onToggleSelectAll}
          onViewDetail={onViewDetail}
          payslips={mockPayslips}
          searchTerm=""
          selectedIds={new Set()}
        />
    );

    // ResponsiveTable renders both desktop and mobile views
    expect(screen.getAllByText("John Doe").length).toBeGreaterThan(0);
    expect(screen.getAllByText("Jane Smith").length).toBeGreaterThan(0);
    expect(screen.getAllByText("10.500.000").length).toBeGreaterThan(0); // Net pay for John
  });

  it("filters employees based on search term", () => {
    render(
        <PayrollSummaryTable
          employees={mockEmployees}
          isLoading={false}
          onSearchChange={onSearchChange}
          onToggleSelect={onToggleSelect}
          onToggleSelectAll={onToggleSelectAll}
          onViewDetail={onViewDetail}
          payslips={mockPayslips}
          searchTerm="Jane"
          selectedIds={new Set()}
        />
    );

    expect(screen.queryByText("John Doe")).toBeNull();
    // ResponsiveTable renders both desktop and mobile views
    expect(screen.getAllByText("Jane Smith").length).toBeGreaterThan(0);
  });

  it("calls onToggleSelect when checkbox is clicked", () => {
    render(
        <PayrollSummaryTable
          employees={mockEmployees}
          isLoading={false}
          onSearchChange={onSearchChange}
          onToggleSelect={onToggleSelect}
          onToggleSelectAll={onToggleSelectAll}
          onViewDetail={onViewDetail}
          payslips={mockPayslips}
          searchTerm=""
          selectedIds={new Set()}
        />
    );

    const checkbox = screen.getByTestId("select-employee-emp-1");
    fireEvent.click(checkbox);
    expect(onToggleSelect).toHaveBeenCalledWith("emp-1");
  });

  it("calls onViewDetail when row is clicked", () => {
    render(
        <PayrollSummaryTable
          employees={mockEmployees}
          isLoading={false}
          onSearchChange={onSearchChange}
          onToggleSelect={onToggleSelect}
          onToggleSelectAll={onToggleSelectAll}
          onViewDetail={onViewDetail}
          payslips={mockPayslips}
          searchTerm=""
          selectedIds={new Set()}
        />
    );

    // ResponsiveTable renders both desktop and mobile views, click the first one
    const johnDoeElements = screen.getAllByText("John Doe");
    fireEvent.click(johnDoeElements[0]);
    expect(onViewDetail).toHaveBeenCalledWith(mockEmployees[0]);
  });

  it("maintains input focus when switching to loading state (regression test)", () => {
    const { rerender } = render(
        <PayrollSummaryTable
          employees={mockEmployees}
          isLoading={false}
          onSearchChange={onSearchChange}
          onToggleSelect={onToggleSelect}
          onToggleSelectAll={onToggleSelectAll}
          onViewDetail={onViewDetail}
          payslips={mockPayslips}
          searchTerm="test"
          selectedIds={new Set()}
        />
    );

    // 1. Find input and focus it
    const input = screen.getByLabelText("common.search");
    input.focus();
    expect(input).toHaveFocus();

    // 2. Rerender with isLoading=true
    rerender(
        <PayrollSummaryTable
          employees={mockEmployees}
          isLoading={true}
          onSearchChange={onSearchChange}
          onToggleSelect={onToggleSelect}
          onToggleSelectAll={onToggleSelectAll}
          onViewDetail={onViewDetail}
          payslips={mockPayslips}
          searchTerm="test"
          selectedIds={new Set()}
        />
    );

    // 3. Verify input is still there and still focused
    expect(screen.getByLabelText("common.search")).toBeInTheDocument();
    expect(screen.getByLabelText("common.search")).toHaveFocus();

    // 4. Verify loading state is also visible (somewhere else)
    expect(screen.getByText("common.loading")).toBeInTheDocument();
  });

  it("renders the search input on mount", () => {
    render(
        <PayrollSummaryTable
          employees={mockEmployees}
          isLoading={false}
          onSearchChange={onSearchChange}
          onToggleSelect={onToggleSelect}
          onToggleSelectAll={onToggleSelectAll}
          onViewDetail={onViewDetail}
          payslips={mockPayslips}
          searchTerm=""
          selectedIds={new Set()}
        />
    );

    const input = screen.getByLabelText("common.search");
    // autoFocus removed for accessibility (jsx-a11y/no-autofocus)
    expect(input).toBeInTheDocument();
  });
});
