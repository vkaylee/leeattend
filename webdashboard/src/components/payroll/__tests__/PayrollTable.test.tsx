import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";

// Mock TanStack Router
vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => vi.fn(),
  useLocation: () => ({ pathname: "/", search: "", hash: "" }),
  Link: ({ to, children, ...props }: any) => <a href={to} {...props}>{children}</a>,
}));
import PayrollTable from "../PayrollTable";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ToastProvider } from "../../ui/Toast";
import { Payslip } from "@/lib/api/types";
import React from "react";

const queryClient = new QueryClient({
  defaultOptions: { queries: { retry: false, gcTime: 0, staleTime: 0 } },
});

describe("PayrollTable", () => {
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

  it("renders payslip summary rows", async () => {
    renderWithProviders(
      <PayrollTable employee={null} workspace={null}
        payslips={mockPayslips}
        isLoading={false}
        adjustments={[]}
        onResolveAdjustments={vi.fn()}
      />
    );

    expect(screen.getByTestId("payslip-card-slip-1")).toBeDefined();
    expect(screen.getByText(/10[.,]500[.,]000/)).toBeDefined();
    expect(screen.getByText(/ĐÃ CHỐT/i)).toBeDefined();
  });

  it("expands card to show segments when clicked", async () => {
    renderWithProviders(
      <PayrollTable employee={null} workspace={null}
        payslips={mockPayslips}
        isLoading={false}
        adjustments={[]}
        onResolveAdjustments={vi.fn()}
      />
    );

    // Click toggle summary bar
    const toggleBtn = screen.getByTestId("toggle-card-slip-1");
    fireEvent.click(toggleBtn);

    // Now segments should be visible
    expect(screen.getByText(/Dòng lương/i)).toBeDefined();
    expect(screen.getByText(/Salary Change/i)).toBeDefined();
    expect(screen.getAllByText(/12[.,]000[.,]000/)[0]).toBeDefined();
  });

  it("shows pending adjustments indicator when adjustments are present", async () => {
    const mockAdjustments = [
      {
        id: "adj-1",
        employee_id: "emp-1",
        requested_date: "2026-01-15",
        status: "pending"
      }
    ];

    renderWithProviders(
      <PayrollTable employee={null} workspace={null}
        payslips={mockPayslips}
        isLoading={false}
        adjustments={mockAdjustments as any}
        onResolveAdjustments={vi.fn()}
      />
    );

    // Card needs to be expanded to see the banner in the new UI
    fireEvent.click(screen.getByTestId("toggle-card-slip-1"));

    expect(screen.getByTestId("pending-adj-banner-slip-1")).toBeDefined();
  });
});
