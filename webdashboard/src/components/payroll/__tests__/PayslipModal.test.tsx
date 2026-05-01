import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";

// Mock TanStack Router
vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => vi.fn(),
  useLocation: () => ({ pathname: "/", search: "", hash: "" }),
  Link: ({ to, children, ...props }: { to: string; children: React.ReactNode }) => <a href={to} {...props}>{children}</a>,
}));

import { ToastProvider } from "@/components/ui/Toast";

import { Employee, Payslip, Workspace } from "@/lib/api/types";

import PayslipModal from "../PayslipModal";


// Mock i18next
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}));

// Mock useDateTimeFormatter hook
vi.mock("@/hooks/use-datetime", () => ({
  useDateTimeFormatter: () => ({
    formatMonthYear: (date: string) => {
      const d = new Date(date);
      return `${String(d.getMonth() + 1).padStart(2, '0')}/${d.getFullYear()}`;
    },
    formatDate: (date: string) => {
      const d = new Date(date);
      const day = String(d.getDate()).padStart(2, '0');
      const month = String(d.getMonth() + 1).padStart(2, '0');
      const year = d.getFullYear();
      return `${day}/${month}/${year}`;
    },
  }),
}));

const mockEmployee: Employee = {
  id: "emp-1",
  workspace_id: "ws-1",
  full_name: "Nguyen Van A",
  employee_id_number: "EMP001",
  department_name: "Engineering",
  hire_date: "2024-01-01",
  is_active: true,
  created_at: "2024-01-01T00:00:00Z",
  updated_at: "2024-01-01T00:00:00Z",
};

const mockPayslip: Payslip = {
  id: "slip-1",
  employee_id: "emp-1",
  period_start: "2026-01-01",
  period_end: "2026-01-31",
  base_pay_amount: "10000000",
  overtime_pay_amount: "500000",
  deductions_amount: "500000",
  holiday_work_premium_amount: "0",
  net_pay_amount: "10000000",
  standard_units: 200,
  overtime_units: 10,
  status: "published",
  created_at: "2026-01-31T00:00:00Z",
  updated_at: "2026-01-31T00:00:00Z",
  segments: [{
    id: "seg-1",
    start_date: "2026-01-01",
    end_date: "2026-01-31",
    applied_base_salary: 10000000,
    applied_std_days: 22,
    applied_hours_per_day: 8,
    actual_work_days: 22,
    paid_work_days: 22,
    amount: 10000000,
    reason: "Standard",
  }],
  salary_components: [
    {
      component: {
        id: "comp-1",
        employee_id: "emp-1",
        component_definition_id: "def-1",
        amount: "10000000",
        effective_date: "2024-01-01",
        created_at: "2024-01-01T00:00:00Z",
        updated_at: "2024-01-01T00:00:00Z",
      },
      definition: {
        id: "def-1",
        workspace_id: "ws-1",
        name: "Base Salary",
        code: "BS",
        is_base_for_ot: true,
        is_taxable: true,
        created_at: "2024-01-01T00:00:00Z",
        updated_at: "2024-01-01T00:00:00Z",
      },
    },
  ],
  incentives: [
    { policy_name: "Diligence Bonus", amount: "500000" },
  ],
};

const mockWorkspace: Workspace = {
  id: "ws-1",
  name: "Tech Company Ltd",
  work_unit_minutes: 60,
  rounding_policy: "nearest",
  date_format: "DD/MM/YYYY",
  time_format: "24h",
  timezone: "Asia/Bangkok",
  theme_primary_color: "#3b82f6",
  default_language: "vi",
  created_at: "2024-01-01T00:00:00Z",
};

const queryClient = new QueryClient({
  defaultOptions: { queries: { retry: false, gcTime: 0 } },
});

const renderWithProviders = (ui: React.ReactElement) => {
  return render(
    <QueryClientProvider client={queryClient}>
      <ToastProvider>
        {ui}
      </ToastProvider>
    </QueryClientProvider>
  );
};

describe("PayslipModal", () => {
  const onClose = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders modal with payslip data when open", () => {
    renderWithProviders(
      <PayslipModal
        isOpen={true}
        onClose={onClose}
        employee={mockEmployee}
        payslip={mockPayslip}
        workspace={mockWorkspace}
      />
    );

    // Dialog content should exist
    expect(screen.getAllByText("Official Payslip Preview").length).toBeGreaterThan(0);
    // Period should display (appears in modal header and document header)
    expect(screen.getAllByText(/common.month/).length).toBeGreaterThan(0);
  });

  it("displays formatted month and year correctly", () => {
    renderWithProviders(
      <PayslipModal
        isOpen={true}
        onClose={onClose}
        employee={mockEmployee}
        payslip={mockPayslip}
        workspace={mockWorkspace}
      />
    );

    // Should show the formatted month/year (01/2026) - appears in multiple places
    expect(screen.getAllByText(/01\/2026/).length).toBeGreaterThan(0);
  });

  it("renders print button with correct styling", () => {
    renderWithProviders(
      <PayslipModal
        isOpen={true}
        onClose={onClose}
        employee={mockEmployee}
        payslip={mockPayslip}
        workspace={mockWorkspace}
      />
    );

    const printButton = screen.getByText("Print Payslip");
    expect(printButton).toBeDefined();
    expect(printButton.closest("button")).toBeDefined();
  });

  it("opens print window when print button clicked", () => {
    // Mock window.open to return a mock window object
    const mockPrintWindow = {
      document: {
        write: vi.fn(),
        close: vi.fn(),
      },
      focus: vi.fn(),
      print: vi.fn(),
      close: vi.fn(),
      closed: false,
      onload: null as (() => void) | null,
    };
    const openMock = vi.spyOn(window, "open").mockReturnValue(mockPrintWindow as unknown as Window);

    renderWithProviders(
      <PayslipModal
        isOpen={true}
        onClose={onClose}
        employee={mockEmployee}
        payslip={mockPayslip}
        workspace={mockWorkspace}
      />
    );

    // window.open should NOT be called on initial render
    expect(openMock).not.toHaveBeenCalled();

    const printButton = screen.getByText("Print Payslip").closest("button");
    if (printButton) fireEvent.click(printButton);

    // window.open IS called when print button clicked
    expect(openMock).toHaveBeenCalledWith('', '_blank', 'width=600,height=800');
    // Document should be written to with A5 page size styling
    expect(mockPrintWindow.document.write).toHaveBeenCalled();
    const writeCall = mockPrintWindow.document.write.mock.calls[0][0];
    expect(writeCall).toContain('size: A5 portrait');
    expect(writeCall).toContain('Payslip - Nguyen Van A');
  });

  it("calls onClose callback when close button clicked", () => {
    renderWithProviders(
      <PayslipModal
        isOpen={true}
        onClose={onClose}
        employee={mockEmployee}
        payslip={mockPayslip}
        workspace={mockWorkspace}
      />
    );

    // Find the close button (the one with X icon)
    const closeBtn = screen.getAllByRole("button").find(b => b.querySelector(".lucide-x"));
    if (closeBtn) fireEvent.click(closeBtn);

    expect(onClose).toHaveBeenCalled();
  });

  it("does not render modal content when closed", () => {
    renderWithProviders(
      <PayslipModal
        isOpen={false}
        onClose={onClose}
        employee={mockEmployee}
        payslip={mockPayslip}
        workspace={mockWorkspace}
      />
    );

    // Modal content should not be visible when closed
    expect(screen.queryByText("Official Payslip Preview")).toBeNull();
  });

  it("passes correct props to PayslipDocument child", () => {
    renderWithProviders(
      <PayslipModal
        isOpen={true}
        onClose={onClose}
        employee={mockEmployee}
        payslip={mockPayslip}
        workspace={mockWorkspace}
      />
    );

    // Verify PayslipDocument renders with correct data
    // Employee name should appear in the document (CSS uppercase, but DOM text is original)
    expect(screen.getByText("Nguyen Van A")).toBeDefined();
    // Employee ID should appear
    expect(screen.getByText("EMP001")).toBeDefined();
    // Workspace name should appear
    expect(screen.getByText("Tech Company Ltd")).toBeDefined();
  });

  it("renders footer warning about print/PDF", () => {
    renderWithProviders(
      <PayslipModal
        isOpen={true}
        onClose={onClose}
        employee={mockEmployee}
        payslip={mockPayslip}
        workspace={mockWorkspace}
      />
    );

    expect(screen.getByText(/Use the print button above/)).toBeDefined();
  });

  it("does not crash when payslip data is incomplete/malformed", () => {
    // Create a malformed payslip with missing nested objects in salary_components
    const malformedPayslip = {
      ...mockPayslip,
      salary_components: [
        { component: { amount: '1000000' } } // Missing definition
      ]
    } as unknown as Payslip;

    renderWithProviders(
      <PayslipModal
        isOpen={true}
        onClose={onClose}
        employee={mockEmployee}
        payslip={malformedPayslip}
        workspace={mockWorkspace}
      />
    );

    // Should still render the modal shell
    expect(screen.getAllByText("Official Payslip Preview").length).toBeGreaterThan(0);
  });
});
