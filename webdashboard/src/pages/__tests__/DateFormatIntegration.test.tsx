import { render, screen, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import AttendanceLogsPage from "../AttendanceLogsPage";
import PayrollPage from "../PayrollPage";
import { BrowserRouter } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ToastProvider } from "../../components/Toast";
import * as api from "../../lib/api";

// We DO NOT mock useDateTimeFormatter here because we want to test its integration
vi.mock("../../lib/api", async () => {
  const actual = await vi.importActual<typeof import("../../lib/api")>("../../lib/api");
  return {
    ...actual,
    fetchRawAttendanceLogs: vi.fn(),
    fetchEmployees: vi.fn(),
    fetchWorkspace: vi.fn(),
    fetchEmployeePayslips: vi.fn(),
    getCurrentWorkspaceId: vi.fn(() => "test-ws-id"),
  };
});

describe("Date Format Integration", () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: {
        queries: {
          retry: false,
          gcTime: 0,
        },
      },
    });

    (api.fetchEmployees as any).mockResolvedValue([
      { id: "emp-1", first_name: "John", last_name: "Doe", employee_id_number: "EMP001", last_name_initial: "D" }
    ]);
    
    (api.fetchRawAttendanceLogs as any).mockResolvedValue({
      data: [
        { id: "log-1", employee_id: "emp-1", punch_time: "2026-01-15T08:00:00Z", source: "device", device_id: "D1" }
      ],
      total: 1,
      page: 1,
      limit: 20
    });

    (api.fetchEmployeePayslips as any).mockResolvedValue([
      { 
        id: "slip-1", 
        employee_id: "emp-1", 
        period_start: "2026-01-01", 
        period_end: "2026-01-31",
        base_pay_amount: "1000",
        overtime_pay_amount: "0",
        deductions_amount: "0",
        net_pay_amount: "1000",
        status: "draft"
      }
    ]);
  });

  it("renders dates in DD/MM/YYYY format when workspace setting is DD/MM/YYYY", async () => {
    (api.fetchWorkspace as any).mockResolvedValue({
      id: "test-ws-id",
      name: "VN Workspace",
      date_format: "DD/MM/YYYY",
      time_format: "24h",
      timezone: "UTC"
    });

    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <BrowserRouter>
            <AttendanceLogsPage />
          </BrowserRouter>
        </ToastProvider>
      </QueryClientProvider>
    );

    // Wait for the table to render the date
    // Jan 15, 2026 should be 15/01/2026
    await waitFor(() => {
      expect(screen.getByText("15/01/2026")).toBeDefined();
    });
  });

  it("renders month/year in MM/yyyy format for Payroll when workspace is VN/US", async () => {
    (api.fetchWorkspace as any).mockResolvedValue({
      id: "test-ws-id",
      date_format: "DD/MM/YYYY",
      time_format: "24h",
      timezone: "UTC"
    });

    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <BrowserRouter>
            <PayrollPage />
          </BrowserRouter>
        </ToastProvider>
      </QueryClientProvider>
    );

    // Select employee
    const empBtn = await screen.findByText("John Doe");
    empBtn.click();

    // Jan 2026 should be 01/2026
    await waitFor(() => {
      expect(screen.getByText(/Tháng 01\/2026/i)).toBeDefined();
    });
  });

  it("renders month/year in yyyy-MM format for Payroll when workspace is ISO", async () => {
    (api.fetchWorkspace as any).mockResolvedValue({
      id: "test-ws-id",
      date_format: "YYYY-MM-DD",
      time_format: "24h",
      timezone: "UTC"
    });

    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <BrowserRouter>
            <PayrollPage />
          </BrowserRouter>
        </ToastProvider>
      </QueryClientProvider>
    );

    // Select employee
    const empBtn = await screen.findByText("John Doe");
    empBtn.click();

    // Jan 2026 should be 2026-01
    await waitFor(() => {
      expect(screen.getByText(/Tháng 2026-01/i)).toBeDefined();
    });
  });

  it("renders dates in YYYY-MM-DD format when workspace setting is YYYY-MM-DD", async () => {
    (api.fetchWorkspace as any).mockResolvedValue({
      id: "test-ws-id",
      name: "ISO Workspace",
      date_format: "YYYY-MM-DD",
      time_format: "24h",
      timezone: "UTC"
    });

    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <BrowserRouter>
            <AttendanceLogsPage />
          </BrowserRouter>
        </ToastProvider>
      </QueryClientProvider>
    );

    // Jan 15, 2026 should be 2026-01-15
    await waitFor(() => {
      expect(screen.getByText("2026-01-15")).toBeDefined();
    });
  });

  it("renders dates in MM/DD/YYYY format when workspace setting is MM/DD/YYYY", async () => {
    (api.fetchWorkspace as any).mockResolvedValue({
      id: "test-ws-id",
      name: "US Workspace",
      date_format: "MM/DD/YYYY",
      time_format: "12h",
      timezone: "UTC"
    });

    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <BrowserRouter>
            <AttendanceLogsPage />
          </BrowserRouter>
        </ToastProvider>
      </QueryClientProvider>
    );

    // Jan 15, 2026 should be 01/15/2026
    await waitFor(() => {
      expect(screen.getByText("01/15/2026")).toBeDefined();
    });
  });
});
