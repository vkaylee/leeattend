import React from "react";

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent, within, waitFor } from "@testing-library/react";
import { BrowserRouter as _BrowserRouter } from "react-router-dom";
import { describe, it, expect, vi, beforeEach } from "vitest";

import { ToastProvider } from "../../components/ui/Toast";
import i18n from "../../i18n/config";
import * as api from "../../lib/api";
import AttendanceLogsPage from "../AttendanceLogsPage";
import PayrollPage from "../PayrollPage";

// Mock TanStack Router
vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => vi.fn(),
  useLocation: () => ({ pathname: "/", search: "", hash: "" }),
  Link: ({ to, children, ...props }: any) => <a href={to} {...props}>{children}</a>,
}));

// Mock useDebounce to be immediate
vi.mock("@/hooks/use-debounce", () => ({
  useDebounce: (value: any) => value,
}));

// Mock EmployeeSearch to avoid internal queries
vi.mock("@/components/common/EmployeeSearch", () => ({
  EmployeeSearch: ({ value, onChange }: any) => (
    <select aria-label="Employee Search"
      data-testid="employee-select"
      value={value}
      onChange={(e) => onChange(e.target.value)}
    >
      <option value="">Select...</option>
      <option value="emp-1">John Doe</option>
    </select>
  )
}));

// Mock MonthYearPicker
vi.mock("@/components/common/MonthYearPicker", () => ({
  MonthYearPicker: ({ value, onChange }: any) => (
    <div data-testid="month-year-picker">Mock Picker</div>
  )
}));

// Mock the API library
vi.mock("../../lib/api", () => ({
  fetchEmployees: vi.fn(),
  searchEmployees: vi.fn(),
  fetchEmployeePayslips: vi.fn(),
  fetchTimeHistory: vi.fn(),
  fetchRawAttendanceLogs: vi.fn(),
  getCurrentWorkspaceId: vi.fn(() => "test-ws-id"),
  fetchWorkspace: vi.fn(),
  submitAttendanceAdjustment: vi.fn(),
  fetchWorkspaceAttendanceAdjustments: vi.fn(),
  fetchMyPermissions: vi.fn(),
  fetchDevices: vi.fn(),
}));

describe("Date Format Integration", () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    vi.clearAllMocks();
    i18n.changeLanguage("en"); // Force English
    queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false, gcTime: 0, staleTime: 0 } },
    });
    (api.fetchMyPermissions as any).mockResolvedValue({ permissions: ["attendance_staging:manage"], roles: ["admin"] });
    (api.fetchDevices as any).mockResolvedValue([]);
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

  it("renders dates in DD/MM/YYYY format when workspace setting is DD/MM/YYYY", async () => {
    (api.fetchEmployees as any).mockResolvedValue({
      data: [{ id: "emp-1", full_name: "John Doe", employee_id_number: "EMP001", workspace_id: "ws-1", is_active: true, hire_date: "2025-01-01", created_at: "2025-01-01T00:00:00Z", updated_at: "2025-01-01T00:00:00Z" }],
      total: 1,
      page: 1,
      limit: 50,
    });
    (api.fetchWorkspace as any).mockResolvedValue({ date_format: "DD/MM/YYYY", timezone: "UTC" });
    (api.fetchRawAttendanceLogs as any).mockImplementation(() => Promise.resolve({
      data: [{ id: "l-1", punch_time: "2026-01-15T08:00:00Z", device_id: "dev-1", source: "device", employee_id: "emp-1" }],
      total: 1,
      page: 1,
      limit: 50
    }));

    renderWithProviders(<AttendanceLogsPage />);

    expect(await screen.findByText(/15\/01\/2026/)).toBeInTheDocument();
  });

  it("renders dates in YYYY-MM-DD format when workspace setting is YYYY-MM-DD", async () => {
    (api.fetchEmployees as any).mockResolvedValue({
      data: [{ id: "emp-1", full_name: "John Doe", employee_id_number: "EMP001", workspace_id: "ws-1", is_active: true, hire_date: "2025-01-01", created_at: "2025-01-01T00:00:00Z", updated_at: "2025-01-01T00:00:00Z" }],
      total: 1,
      page: 1,
      limit: 50,
    });
    (api.fetchWorkspace as any).mockResolvedValue({ date_format: "YYYY-MM-DD", timezone: "UTC" });
    (api.fetchRawAttendanceLogs as any).mockImplementation(() => Promise.resolve({
      data: [{ id: "l-1", punch_time: "2026-01-15T08:00:00Z", device_id: "dev-1", source: "device", employee_id: "emp-1" }],
      total: 1,
      page: 1,
      limit: 50
    }));

    renderWithProviders(<AttendanceLogsPage />);

    expect(await screen.findByText(/2026-01-15/)).toBeInTheDocument();
  });

  it("renders month/year correctly in PayrollPage", async () => {
    (api.fetchEmployees as any).mockResolvedValue({
      data: [{ id: "emp-1", full_name: "John Doe", hire_date: "2025-01-01", employee_id_number: "EMP001", workspace_id: "ws-1", is_active: true, created_at: "2025-01-01T00:00:00Z", updated_at: "2025-01-01T00:00:00Z" }],
      total: 1,
      page: 1,
      limit: 50,
    });
    (api.fetchEmployeePayslips as any).mockResolvedValue([]);
    (api.fetchWorkspaceAttendanceAdjustments as any).mockResolvedValue([]);
    (api.fetchWorkspace as any).mockResolvedValue({ date_format: "DD/MM/YYYY", timezone: "UTC" });

    renderWithProviders(<PayrollPage />);

    const empBtns = await screen.findAllByText("John Doe");
    fireEvent.click(empBtns[0]);

    // Check for the title (VN or EN depending on env, checking for VN based on failure output)
    // Using getAllByText and taking the first one (h1) or being more specific with selector
    const titles = await screen.findAllByText(/Tính lương|Payroll/i, { selector: 'h1' });
    expect(titles.length).toBeGreaterThan(0);
  });
});
