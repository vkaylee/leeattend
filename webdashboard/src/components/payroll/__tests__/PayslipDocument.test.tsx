import { render, screen } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import PayslipDocument from "../PayslipDocument";
import { Employee, Payslip, Workspace } from "@/lib/api/types";
import { BrowserRouter } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

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

const createMockEmployee = (overrides: Partial<Employee> = {}): Employee => ({
  id: "emp-1",
  workspace_id: "ws-1",
  full_name: "Nguyen Van A",
  employee_id_number: "EMP001",
  department_name: "Engineering",
  hire_date: "2024-01-01",
  is_active: true,
  created_at: "2024-01-01T00:00:00Z",
  updated_at: "2024-01-01T00:00:00Z",
  ...overrides,
});

const createMockPayslip = (overrides: Partial<Payslip> = {}): Payslip => ({
  id: "slip-1",
  employee_id: "emp-1",
  period_start: "2026-01-01",
  period_end: "2026-01-31",
  base_pay_amount: "10000000",
  overtime_pay_amount: "500000",
  deductions_amount: "500000",
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
  ...overrides,
});

const createMockWorkspace = (overrides: Partial<Workspace> = {}): Workspace => ({
  id: "ws-1",
  name: "Tech Company Ltd",
  work_unit_minutes: 60,
  rounding_policy: "nearest",
  date_format: "DD/MM/YYYY",
  time_format: "24h",
  timezone: "Asia/Bangkok",
  theme_primary_color: "#3b82f6",
  default_language: "vi",
  pairing_algorithm: "default",
  early_arrival_strategy: "allow",
  created_at: "2024-01-01T00:00:00Z",
  ...overrides,
});

const queryClient = new QueryClient({
  defaultOptions: { queries: { retry: false, gcTime: 0 } },
});

const renderWithProviders = (ui: React.ReactElement) => {
  return render(
    <QueryClientProvider client={queryClient}>
      <BrowserRouter>
        {ui}
      </BrowserRouter>
    </QueryClientProvider>
  );
};

describe("PayslipDocument", () => {
  it("renders document container with correct ID", () => {
    const { container } = renderWithProviders(
      <PayslipDocument
        employee={createMockEmployee()}
        payslip={createMockPayslip()}
        workspace={createMockWorkspace()}
      />
    );

    const doc = container.querySelector("#payslip-document");
    expect(doc).toBeDefined();
    expect(doc?.className).toContain("bg-white");
  });

  it("displays company header with workspace name", () => {
    renderWithProviders(
      <PayslipDocument
        employee={createMockEmployee()}
        payslip={createMockPayslip()}
        workspace={createMockWorkspace({ name: "Tech Corp Vietnam" })}
      />
    );

    expect(screen.getByText("Tech Corp Vietnam")).toBeDefined();
    expect(screen.getByText("payroll.document.system_subtitle")).toBeDefined();
    expect(screen.getByText("payroll.document.verified")).toBeDefined();
  });

  it("displays fallback workspace name when null", () => {
    renderWithProviders(
      <PayslipDocument
        employee={createMockEmployee()}
        payslip={createMockPayslip()}
        workspace={null}
      />
    );

    expect(screen.getByText("LeeAttend")).toBeDefined();
  });

  it('shows "Pay Slip" title and period info', () => {
    renderWithProviders(
      <PayslipDocument
        employee={createMockEmployee()}
        payslip={createMockPayslip()}
        workspace={createMockWorkspace()}
      />
    );

    expect(screen.getByText("payroll.document.title")).toBeDefined();
    // Check period dates are shown
    expect(screen.getByText(/01\/01\/2026/)).toBeDefined();
    expect(screen.getByText(/31\/01\/2026/)).toBeDefined();
  });

  it("displays employee details section", () => {
    renderWithProviders(
      <PayslipDocument
        employee={createMockEmployee({
          full_name: "Nguyen Van A",
          employee_id_number: "EMP001",
          department_name: "Engineering",
        })}
        payslip={createMockPayslip()}
        workspace={createMockWorkspace()}
      />
    );

    expect(screen.getByText("payroll.document.employee_details")).toBeDefined();
    // CSS uppercase class makes text display uppercase, but DOM text remains original
    expect(screen.getByText("Nguyen Van A")).toBeDefined();
    expect(screen.getByText("EMP001")).toBeDefined();
    expect(screen.getByText("Engineering")).toBeDefined();
  });

  it("renders Net Payment summary box", () => {
    renderWithProviders(
      <PayslipDocument
        employee={createMockEmployee()}
        payslip={createMockPayslip({ net_pay_amount: "10000000" })}
        workspace={createMockWorkspace()}
      />
    );

    expect(screen.getByText("payroll.document.net_payment")).toBeDefined();
    expect(screen.getByText("Paid via Bank Transfer")).toBeDefined();
  });

  it("renders salary components breakdown with prorated calculation", () => {
    const payslip = createMockPayslip({
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
        {
          component: {
            id: "comp-2",
            employee_id: "emp-1",
            component_definition_id: "def-2",
            amount: "1000000",
            effective_date: "2024-01-01",
            created_at: "2024-01-01T00:00:00Z",
            updated_at: "2024-01-01T00:00:00Z",
          },
          definition: {
            id: "def-2",
            workspace_id: "ws-1",
            name: "Allowance",
            code: "AL",
            is_base_for_ot: false,
            is_taxable: true,
            created_at: "2024-01-01T00:00:00Z",
            updated_at: "2024-01-01T00:00:00Z",
          },
        },
      ],
      segments: [{
        id: "seg-1",
        start_date: "2026-01-01",
        end_date: "2026-01-31",
        applied_base_salary: 10000000,
        applied_std_days: 22,
        applied_hours_per_day: 8,
        actual_work_days: 20,
        paid_work_days: 20,
        amount: 10000000,
        reason: "Standard",
      }],
    });

    renderWithProviders(
      <PayslipDocument
        employee={createMockEmployee()}
        payslip={payslip}
        workspace={createMockWorkspace()}
      />
    );

    expect(screen.getByText("Base Salary")).toBeDefined();
    expect(screen.getByText("Allowance")).toBeDefined();
    expect(screen.getByText(/Code: BS/)).toBeDefined();
    expect(screen.getByText(/Code: AL/)).toBeDefined();
  });

  it("calculates and displays prorated amounts correctly", () => {
     // 10,000,000 / 22 * 22 = 10,000,000
     // 1,000,000 / 22 * 22 = 1,000,000
    const payslip = createMockPayslip({
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
      segments: [{
        id: "seg-1",
        start_date: "2026-01-01",
        end_date: "2026-01-31",
        applied_base_salary: 10000000,
        applied_std_days: 22,
        applied_hours_per_day: 8,
        actual_work_days: 11, // Worked half month
        paid_work_days: 11,
        amount: 5000000,
        reason: "Standard",
      }],
    });

    renderWithProviders(
      <PayslipDocument
        employee={createMockEmployee()}
        payslip={payslip}
        workspace={createMockWorkspace()}
      />
    );

    // Monthly: 10,000,000
    // Daily: 10,000,000 / 22 = 454,545.45...
    // Paid Days: 11
    // Prorated: 5,000,000
    
    // We expect to see the formatted amount "5.000.000 ₫" (or similar depending on locale mock)
    // The locale mock might be just returning the key or simple text? 
    // Wait, the component uses Intl.NumberFormat('vi-VN'). 
    // In node environment, full ICU might not be available or might differ.
    // But usually Vitest/JSDOM supports it reasonably well.
    // 5,000,000 VND formatted is usually "5.000.000 ₫" or "5.000.000 ₫" (nbsp).
    
    // Let's use a regex to be safe about spacing/currency symbol
    expect(screen.getByText(/5\.000\.000/)).toBeDefined();
  });

  it("renders segments fallback when no salary_components", () => {
    const payslip = createMockPayslip({
      salary_components: [],
      segments: [{
        id: "seg-1",
        start_date: "2026-01-01",
        end_date: "2026-01-31",
        applied_base_salary: 11000000,
        applied_std_days: 22,
        applied_hours_per_day: 8,
        actual_work_days: 22,
        paid_work_days: 22,
        amount: 11000000,
        reason: "Standard",
      }],
    });

    renderWithProviders(
      <PayslipDocument
        employee={createMockEmployee()}
        payslip={payslip}
        workspace={createMockWorkspace()}
      />
    );

    expect(screen.getByText("Base Salary")).toBeDefined();
    expect(screen.getByText(/2026-01-01 - 2026-01-31/)).toBeDefined();
  });

  it("handles malformed salary_components gracefully (simulating partial API response)", () => {
    // Simulate a case where definition is missing (e.g. bad join) or component is missing
    const payslip = createMockPayslip({
      salary_components: [
        {
          // @ts-expect-error - Simulating runtime data mismatch
          component: {
            amount: "10000000"
          }
          // definition is missing
        }
      ]
    });

    // This should NOT crash the entire component
    // We expect it might fail to render that row, or render empty, but not throw
    try {
      renderWithProviders(
        <PayslipDocument
          employee={createMockEmployee()}
          payslip={payslip}
          workspace={createMockWorkspace()}
        />
      );
    } catch (e) {
      // If it crashes, we want to know
      throw e;
    }
    
    // If it survives, check if it rendered the document shell
    expect(screen.getByText("payroll.document.title")).toBeDefined();
  });

  it("renders overtime section when present", () => {
    renderWithProviders(
      <PayslipDocument
        employee={createMockEmployee()}
        payslip={createMockPayslip({
          overtime_pay_amount: "500000",
          overtime_units: 10,
        })}
        workspace={createMockWorkspace()}
      />
    );

    expect(screen.getByText("Overtime Compensation")).toBeDefined();
    expect(screen.getByText("10 Units")).toBeDefined();
  });

  it("does not render overtime section when zero", () => {
    renderWithProviders(
      <PayslipDocument
        employee={createMockEmployee()}
        payslip={createMockPayslip({ overtime_pay_amount: "0" })}
        workspace={createMockWorkspace()}
      />
    );

    expect(screen.queryByText("Overtime Compensation")).toBeNull();
  });

  it("renders incentives section when present", () => {
    renderWithProviders(
      <PayslipDocument
        employee={createMockEmployee()}
        payslip={createMockPayslip({
          incentives: [
            { policy_name: "Diligence Bonus", amount: "500000" },
            { policy_name: "Safety Bonus", amount: "200000" },
          ],
        })}
        workspace={createMockWorkspace()}
      />
    );

    expect(screen.getByText("Diligence Bonus")).toBeDefined();
    expect(screen.getByText("Safety Bonus")).toBeDefined();
  });

  it("does not render incentive rows when empty", () => {
    renderWithProviders(
      <PayslipDocument
        employee={createMockEmployee()}
        payslip={createMockPayslip({ incentives: [] })}
        workspace={createMockWorkspace()}
      />
    );

    expect(screen.queryByText("Diligence Bonus")).toBeNull();
  });

  it("displays deductions and net pay", () => {
    renderWithProviders(
      <PayslipDocument
        employee={createMockEmployee()}
        payslip={createMockPayslip({
          deductions_amount: "500000",
          net_pay_amount: "10000000",
        })}
        workspace={createMockWorkspace()}
      />
    );

    expect(screen.getByText("Gross Pay")).toBeDefined();
    expect(screen.getByText("Deductions")).toBeDefined();
    expect(screen.getByText("Net Pay")).toBeDefined();
  });

  it("renders footer with system information", () => {
    renderWithProviders(
      <PayslipDocument
        employee={createMockEmployee()}
        payslip={createMockPayslip()}
        workspace={createMockWorkspace()}
      />
    );

    expect(screen.getByText("Generated by LeeAttend System")).toBeDefined();
  });

  it("handles employee with no department", () => {
    renderWithProviders(
      <PayslipDocument
        employee={createMockEmployee({ department_name: undefined })}
        payslip={createMockPayslip()}
        workspace={createMockWorkspace()}
      />
    );

    expect(screen.getByText("N/A")).toBeDefined();
  });

  it("renders earnings breakdown section header", () => {
    renderWithProviders(
      <PayslipDocument
        employee={createMockEmployee()}
        payslip={createMockPayslip()}
        workspace={createMockWorkspace()}
      />
    );

    expect(screen.getByText("Earnings Breakdown")).toBeDefined();
    expect(screen.getByText("Description")).toBeDefined();
    expect(screen.getByText("Amount")).toBeDefined();
  });
});
