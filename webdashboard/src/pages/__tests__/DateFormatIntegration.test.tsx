import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import AttendanceLogsPage from "../AttendanceLogsPage";
import PayrollPage from "../PayrollPage";
import { BrowserRouter } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ToastProvider } from "../../components/Toast";
import * as api from "../../lib/api";

// Mock API
vi.mock("../../lib/api", async () => {
  const actual = await vi.importActual<typeof import("../../lib/api")>("../../lib/api");
  return {
    ...actual,
    fetchRawAttendanceLogs: vi.fn(),
    fetchEmployees: vi.fn(),
    fetchWorkspace: vi.fn(),
    fetchEmployeePayslips: vi.fn(),
    getCurrentWorkspaceId: vi.fn(() => "ws-1"),
  };
});

// Mock EmployeeSearch
vi.mock("@/components/common/EmployeeSearch", () => ({
  EmployeeSearch: ({ value, onChange, placeholder }: any) => (
    <select 
      data-testid="employee-search-mock" 
      value={typeof value === 'object' ? value?.id : value} 
      onChange={(e) => {
        const id = e.target.value;
        // Smart detection: if onChange's source code contains "employees.find", it wants an object
        // Actually, we can just return based on what's selected. 
        // PayrollPage sidebar uses selectedEmployee object.
        const mockEmp = { id: "emp-1", first_name: "John", last_name: "Doe", employee_id_number: "EMP001" };
        if (id === "emp-1") {
          // Try to satisfy both string-based and object-based handlers
          onChange(mockEmp); 
        } else {
          onChange(id);
        }
      }}
    >
      <option value="">{placeholder || "Select..."}</option>
      <option value="emp-1">John Doe</option>
    </select>
  )
}));

// Mock AttendanceImportDialog
vi.mock("@/components/AttendanceImportDialog", () => ({
  default: () => <div />
}));

const queryClient = new QueryClient({
  defaultOptions: { queries: { retry: false, gcTime: 0 } },
});

const wrapper = ({ children }: { children: React.ReactNode }) => (
  <QueryClientProvider client={queryClient}>
    <ToastProvider>
      <BrowserRouter>{children}</BrowserRouter>
    </ToastProvider>
  </QueryClientProvider>
);

describe("Date Format Integration", () => {
  beforeEach(() => {
    queryClient.clear();
    vi.clearAllMocks();
    const mockEmp = { id: "emp-1", first_name: "John", last_name: "Doe", employee_id_number: "EMP001" };
    (api.fetchEmployees as any).mockResolvedValue([mockEmp]);
    (api.fetchWorkspace as any).mockResolvedValue({ date_format: "DD/MM/YYYY", timezone: "UTC" });
    (api.fetchRawAttendanceLogs as any).mockResolvedValue({
      data: [{ id: "1", employee_id: "emp-1", punch_time: "2026-01-15T08:00:00Z", source: "device" }],
      total: 1
    });
    const currentMonth = new Date().getMonth() + 1;
    const currentYear = new Date().getFullYear();
    const periodStart = `${currentYear}-${currentMonth.toString().padStart(2, '0')}-01`;
    (api.fetchEmployeePayslips as any).mockResolvedValue([{
      id: "slip-1",
      period_start: periodStart,
      period_end: periodStart,
      base_pay_amount: "1000",
      overtime_pay_amount: "0",
      deductions_amount: "0",
      net_pay_amount: "1000",
      status: "draft"
    }]);
  });

  it("renders dates in DD/MM/YYYY format when workspace setting is DD/MM/YYYY", async () => {
    (api.fetchWorkspace as any).mockResolvedValue({ date_format: "DD/MM/YYYY", timezone: "UTC" });
    render(<AttendanceLogsPage />, { wrapper });
    
    fireEvent.change(screen.getByTestId("employee-search-mock"), { target: { value: "emp-1" } });
    await waitFor(() => expect(screen.getByText("15/01/2026")).toBeDefined());
  });

  it("renders dates in YYYY-MM-DD format when workspace setting is YYYY-MM-DD", async () => {
    (api.fetchWorkspace as any).mockResolvedValue({ date_format: "YYYY-MM-DD", timezone: "UTC" });
    render(<AttendanceLogsPage />, { wrapper });
    
    fireEvent.change(screen.getByTestId("employee-search-mock"), { target: { value: "emp-1" } });
    await waitFor(() => expect(screen.getByText("2026-01-15")).toBeDefined());
  });

  it("renders month/year correctly in PayrollPage", async () => {
    (api.fetchWorkspace as any).mockResolvedValue({ date_format: "DD/MM/YYYY", timezone: "UTC" });
    
    const user = userEvent.setup();
    render(<PayrollPage />, { wrapper });
    
    const currentMonth = new Date().getMonth() + 1;
    const currentYear = new Date().getFullYear();
    const monthStr = currentMonth.toString().padStart(2, '0');
    const yearStr = currentYear.toString();
    
    // Select employee from the sidebar directly
    // Use role button to be more precise
    const empBtn = await screen.findByRole("button", { name: /John Doe/i });
    await user.click(empBtn);
    
    // Wait for the details to render - payslips should show Month/Year
    await waitFor(() => {
      const text = document.body.textContent || "";
      expect(text).toContain(monthStr);
      expect(text).toContain(yearStr);
    }, { timeout: 5000 });
  });
});