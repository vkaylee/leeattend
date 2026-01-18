import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import PayrollTable from "../PayrollTable";
import { BrowserRouter } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

const mockPayslips: any[] = [
  {
    id: "slip-1",
    period_start: "2026-01-01",
    period_end: "2026-01-31",
    base_pay_amount: "10000000",
    overtime_pay_amount: "500000",
    deductions_amount: "0",
    net_pay_amount: "10500000",
    status: "finalized",
    segments: [
      {
        start_date: "2026-01-01",
        end_date: "2026-01-15",
        reason: "Standard",
        applied_base_salary: 10000000,
        applied_std_days: 26,
        actual_work_days: 13,
        amount: 5000000
      },
      {
        start_date: "2026-01-16",
        end_date: "2026-01-31",
        reason: "Salary Change",
        applied_base_salary: 12000000,
        applied_std_days: 26,
        actual_work_days: 13,
        amount: 6000000
      }
    ]
  }
];

describe("PayrollTable", () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    queryClient = new QueryClient({
      defaultOptions: {
        queries: {
          retry: false,
        },
      },
    });
  });

  it("renders payslip summary rows", () => {
    render(
      <QueryClientProvider client={queryClient}>
        <BrowserRouter>
          <PayrollTable payslips={mockPayslips} isLoading={false} />
        </BrowserRouter>
      </QueryClientProvider>
    );

    expect(screen.getByTestId("payslip-row-slip-1")).toBeDefined();
    expect(screen.getByText(/10.500.000/)).toBeDefined();
  });

  it("expands row to show segments when toggle is clicked", async () => {
    render(
      <QueryClientProvider client={queryClient}>
        <BrowserRouter>
          <PayrollTable payslips={mockPayslips} isLoading={false} />
        </BrowserRouter>
      </QueryClientProvider>
    );

    // Initial state: segments not visible
    expect(screen.queryByTestId("segments-detail-slip-1")).toBeNull();

    // Click toggle button using data-testid
    const toggleBtn = screen.getByTestId("toggle-segments-slip-1");
    fireEvent.click(toggleBtn);

    // Now segments should be visible
    expect(screen.getByTestId("segments-detail-slip-1")).toBeDefined();
    expect(screen.getByText("Salary Change")).toBeDefined();
    expect(screen.getByText("12.000.000")).toBeDefined();
  });

  it("shows pending adjustments indicator when adjustments are present", () => {
    const mockAdjustments: any[] = [
      {
        id: "adj-1",
        employee_id: "emp-1",
        requested_punch_in_time: "2026-01-10T08:00:00Z",
        status: "pending"
      }
    ];

    render(
      <QueryClientProvider client={queryClient}>
        <BrowserRouter>
          <PayrollTable 
            payslips={mockPayslips} 
            isLoading={false} 
            adjustments={mockAdjustments} 
          />
        </BrowserRouter>
      </QueryClientProvider>
    );

    expect(screen.getByTestId("pending-adj-badge-slip-1")).toBeDefined();
  });
});
