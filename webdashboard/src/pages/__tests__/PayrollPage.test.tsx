import { render, screen, fireEvent, waitFor } from "@testing-library/react";
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
  calculate_payroll: vi.fn(),
  getCurrentWorkspaceId: vi.fn(() => "test-ws-id"),
}));

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: false,
    },
  },
});

describe("PayrollPage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    queryClient.clear();
  });

  it("renders employee list and empty state", async () => {
    (api.fetchEmployees as any).mockResolvedValue([
      { id: "emp-1", first_name: "John", last_name: "Doe", employee_id_number: "EMP001" }
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

    expect(await screen.findByText("John Doe")).toBeDefined();
    expect(screen.getByText("Chọn nhân sự để tiếp tục")).toBeDefined();
  });

  it("shows payslips when an employee is selected", async () => {
    (api.fetchEmployees as any).mockResolvedValue([
      { id: "emp-1", first_name: "John", last_name: "Doe", employee_id_number: "EMP001" }
    ]);
    (api.fetchEmployeePayslips as any).mockResolvedValue([
      {
        id: "slip-1",
        period_start: "2026-01-01",
        period_end: "2026-01-31",
        base_pay_amount: "10000000",
        overtime_pay_amount: "500000",
        deductions_amount: "100000",
        net_pay_amount: "10400000",
        status: "paid"
      }
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

    const employeeBtn = await screen.findByText("John Doe");
    fireEvent.click(employeeBtn);

    expect((await screen.findAllByText(/Lịch sử phiếu lương/i))[0]).toBeDefined();
    expect(await screen.findByText("10.400.000")).toBeDefined();
    expect(screen.getByText(/ĐÃ CHI TRẢ/i)).toBeDefined();
  });

  it("opens calculation modal and triggers calculation", async () => {
    (api.fetchEmployees as any).mockResolvedValue([
      { id: "emp-1", first_name: "John", last_name: "Doe", employee_id_number: "EMP001" }
    ]);
    (api.fetchEmployeePayslips as any).mockResolvedValue([]);
    (api.calculate_payroll as any).mockResolvedValue({ id: "new-slip" });

    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <BrowserRouter>
            <PayrollPage />
          </BrowserRouter>
        </ToastProvider>
      </QueryClientProvider>
    );

    fireEvent.click(await screen.findByText("John Doe"));
    fireEvent.click(screen.getByText("Tính lương mới"));

    expect(screen.getByText("Bắt đầu tính lương")).toBeDefined();
    
    fireEvent.click(screen.getByText("Bắt đầu tính lương"));

    await waitFor(() => {
      expect(api.calculate_payroll).toHaveBeenCalledWith("test-ws-id", "emp-1", expect.any(Number), expect.any(Number));
    });
  });
});
