import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi } from "vitest";

// Mock TanStack Router
 
vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => vi.fn(),
  useLocation: () => ({ pathname: "/", search: "", hash: "" }),
  Link: ({ to, children, ...props }: { to: string; children: React.ReactNode }) => <a href={to} {...props}>{children}</a>,
}));

import { Payslip } from "@/lib/api/types";

import { ToastProvider } from "../../ui/toast";
import PayrollTable from "../payroll-table";

vi.mock("@/hooks/use-permissions", () => ({
  useMyPermissions: () => ({
    hasPermission: () => true,
  }),
}));

const queryClient = new QueryClient({
  defaultOptions: { queries: { retry: false, gcTime: 0, staleTime: 0 } },
});

describe("PayrollTable", () => {
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
      status: "published",
      standard_units: 200,
      overtime_units: 10,
      created_at: "2026-02-01T00:00:00Z",
      updated_at: "2026-02-01T00:00:00Z",
      segments: [
        {
          id: "seg-1",
          start_date: "2026-01-01",
          end_date: "2026-01-31",
          reason: "Salary Change",
          applied_base_salary: 12000000,
          applied_std_days: 26,
          applied_hours_per_day: 8,
          actual_work_days: 22,
          paid_work_days: 22,
          amount: 12000000
        }
      ]
    }
  ];

  const renderWithProviders = (ui: React.ReactElement) => {
    return render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          {ui}
        </ToastProvider>
      </QueryClientProvider>
    );
  };

  it("renders payslip summary rows", () => {
    renderWithProviders(
      <PayrollTable adjustments={[]} employee={null}
        isLoading={false}
        onResolveAdjustments={vi.fn()}
        payslips={mockPayslips}
        organization={null}
      />
    );

    // ResponsiveTable renders both desktop and mobile views
    expect(screen.getAllByTestId("payslip-card-slip-1").length).toBeGreaterThan(0);
    expect(screen.getAllByText(/10[.,]500[.,]000/).length).toBeGreaterThan(0);
    expect(screen.getAllByText(/ĐÃ CHỐT/i).length).toBeGreaterThan(0);
  });

  it("expands card to show segments when clicked", () => {
    renderWithProviders(
      <PayrollTable adjustments={[]} employee={null}
        isLoading={false}
        onResolveAdjustments={vi.fn()}
        payslips={mockPayslips}
        organization={null}
      />
    );

    // Click toggle summary bar - use getAllByTestId because ResponsiveTable renders both views
    const toggleButtons = screen.getAllByTestId("toggle-card-slip-1");
    fireEvent.click(toggleButtons[0]);

    // Now segments should be visible
    expect(screen.getAllByText(/Dòng lương/i).length).toBeGreaterThan(0);
    expect(screen.getAllByText(/Salary Change/i).length).toBeGreaterThan(0);
    expect(screen.getAllByText(/12[.,]000[.,]000/).length).toBeGreaterThan(0);
  });

  it("shows pending adjustments indicator when adjustments are present", () => {
    const mockAdjustments = [
      {
        id: "adj-1",
        employee_id: "emp-1",
        requested_date: "2026-01-15",
        status: "pending"
      }
    ];

    renderWithProviders(
      <PayrollTable adjustments={mockAdjustments as unknown as never} employee={null}
        isLoading={false}
        onResolveAdjustments={vi.fn()}

        payslips={mockPayslips}
        organization={null}
      />
    );

    // Card needs to be expanded to see the banner in the new UI
    const toggleButtons = screen.getAllByTestId("toggle-card-slip-1");
    fireEvent.click(toggleButtons[0]);

    // pending-adj-banner is only in desktop view (PayslipCard), not in mobile card
    expect(screen.getByTestId("pending-adj-banner-slip-1")).toBeDefined();
  });
});
