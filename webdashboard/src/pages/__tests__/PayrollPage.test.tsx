import { render, screen, fireEvent, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import PayrollPage from "../PayrollPage";
import { BrowserRouter } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ToastProvider } from "../../components/Toast";
import * as api from "../../lib/api";

// Mock the API library
vi.mock("../../lib/api", () => ({
  fetchEmployees: vi.fn(),
  fetchEmployeePayslips: vi.fn(),
  fetchWorkspacePayslips: vi.fn(),
  calculateBatchPayslips: vi.fn(),
  getCurrentWorkspaceId: vi.fn(() => "test-ws-id"),
  fetchWorkspaceAttendanceAdjustments: vi.fn(),
}));

describe("PayrollPage", () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false, gcTime: 0, staleTime: 0 } },
    });
    
    // Provide default mock values
    (api.fetchEmployees as any).mockResolvedValue([]);
    (api.fetchEmployeePayslips as any).mockResolvedValue([]);
    (api.fetchWorkspacePayslips as any).mockResolvedValue([]);
    (api.fetchWorkspaceAttendanceAdjustments as any).mockResolvedValue([]);
    (api.getCurrentWorkspaceId as any).mockReturnValue("test-ws-id");
  });

  it("renders employee list and summary table", async () => {
    (api.fetchEmployees as any).mockResolvedValue([
      { id: "emp-1", full_name: "John Doe", employee_id_number: "EMP001", hire_date: "2025-01-01" }
    ]);
    (api.fetchWorkspacePayslips as any).mockResolvedValue([]);

    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <BrowserRouter>
            <PayrollPage />
          </BrowserRouter>
        </ToastProvider>
      </QueryClientProvider>
    );

    expect((await screen.findAllByText("John Doe"))[0]).toBeDefined();
    // Match '1' followed by a space. Use findAll to handle potential duplicates during re-renders.
    expect((await screen.findAllByText(/^1 /))[0]).toBeDefined();
  });

  it("shows detail drawer when an employee is clicked", async () => {
    const user = userEvent.setup();
    (api.fetchEmployees as any).mockResolvedValue([
      { id: "emp-1", full_name: "John Doe", employee_id_number: "EMP001", hire_date: "2025-01-01" }
    ]);
    (api.fetchEmployeePayslips as any).mockResolvedValue([
      { id: "ps-1", employee_id: "emp-1", period_start: "2025-12-01", period_end: "2025-12-31", base_pay_amount: "10000000", overtime_pay_amount: "400000", total_deductions: "0", net_pay_amount: "10400000", status: "paid" }
    ]);

    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <BrowserRouter>
            <PayrollPage />
          </BrowserRouter>
        </ToastProvider>
      </QueryClientProvider>
    );

    const employeeText = (await screen.findAllByText("John Doe"))[0];
    const employeeRow = employeeText.closest("tr");
    fireEvent.click(employeeRow!);

    // Should show detail drawer content (history title). Use findAll to be safe.
    expect((await screen.findAllByText(/Lịch sử phiếu lương/i))[0]).toBeDefined();
    expect((await screen.findAllByText(/10[.,]400[.,]000/))[0]).toBeDefined();
    expect((await screen.findAllByText(/ĐÃ CHI TRẢ/i))[0]).toBeDefined();
  });

  it("opens calculation modal and triggers calculation", async () => {
    (api.fetchEmployees as any).mockResolvedValue([
      { id: "emp-1", full_name: "John Doe", employee_id_number: "EMP001", hire_date: "2025-01-01" }
    ]);
    (api.fetchWorkspacePayslips as any).mockResolvedValue([]);
    (api.calculateBatchPayslips as any).mockResolvedValue({ results: [{ employee_id: "emp-1", status: "success" }] });

    const user = userEvent.setup();
    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <BrowserRouter>
            <PayrollPage />
          </BrowserRouter>
        </ToastProvider>
      </QueryClientProvider>
    );

    // Select employee via checkbox in table.
    const checkboxes = await screen.findAllByTestId("select-employee-emp-1");
    fireEvent.click(checkboxes[0]);

    const calculateBtn = await screen.findByTestId("calculate-batch-btn");
    await waitFor(() => expect(calculateBtn).not.toBeDisabled());
    await user.click(calculateBtn);

    // Wait for modal to appear in the document body
    const submitBtn = await screen.findByTestId("batch-calc-submit-btn");
    await user.click(submitBtn);

    await waitFor(() => {
      expect(api.calculateBatchPayslips).toHaveBeenCalled();
    }, { timeout: 2000 });
  });
});
