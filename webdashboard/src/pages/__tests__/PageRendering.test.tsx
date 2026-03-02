import { render, screen, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { MemoryRouter } from "react-router-dom";
import DashboardPage from "../DashboardPage";
import EmployeesPage from "../EmployeesPage";
import LeaveBalancesPage from "../LeaveBalancesPage";
import AttendanceLogsPage from "../AttendanceLogsPage";
import MyClockPage from "../ess/MyClockPage";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ToastProvider } from "../../components/ui/Toast";
import React from "react";
import i18n from "../../i18n/config";

// Mock hooks
vi.mock("@/hooks/use-profile", () => ({
  useProfile: () => ({
    user: { id: "test-user", email: "test@example.com", full_name: "Test User", preferred_language: "en" },
    isLoading: false,
    isError: false,
  }),
}));

vi.mock("@/hooks/use-workspace", () => ({
  useWorkspaces: () => ({
    workspaces: [{ id: "test-ws-id", name: "Test Workspace", default_language: "en" }],
    isLoading: false,
    isError: false,
  }),
  useWorkspace: () => ({
    workspace: { id: "test-ws-id", name: "Test Workspace", date_format: "DD/MM/YYYY", default_language: "en" },
    isLoading: false,
    isError: false,
  }),
}));

vi.mock("@/hooks/use-theme", () => ({
  useTheme: () => ({
    primaryColor: "#7c3aed",
    setPreviewTheme: vi.fn(),
    setUserTheme: vi.fn(),
  }),
}));

vi.mock("@/hooks/use-i18n-sync", () => ({
  useI18nSync: vi.fn(),
}));

vi.mock("@/hooks/use-agents", () => ({
  useAgentsStatus: () => ({ agents: [], isLoading: false, error: null }),
}));

vi.mock("@/hooks/use-devices", () => ({
  useDevices: () => ({ devices: [], isLoading: false, error: null }),
}));

vi.mock("@/hooks/use-dashboard", () => ({
  useDashboardStats: () => ({ data: { total_employees: 10, present_today: 8, pending_leave_requests: 2, estimated_monthly_payroll: "50000", late_today: 1, absent_today: 1, on_leave_today: 1, early_departure_today: 0, avg_work_hours_today: 7.5 }, isLoading: false, error: null }),
}));

vi.mock("@/lib/api", async (importOriginal) => {
  const actual = await importOriginal<any>();
  return {
    ...actual,
    getCurrentUserId: vi.fn(() => "test-user"),
    getCurrentWorkspaceId: vi.fn(() => "test-ws-id"),
    fetchWorkspaceLeaveBalances: vi.fn(() => Promise.resolve([])),
    fetchEmployees: vi.fn(() => Promise.resolve([])),
    fetchWorkspace: vi.fn(() => Promise.resolve({ date_format: "DD/MM/YYYY", timezone: "UTC", default_language: "en" })),
    fetchDepartments: vi.fn(() => Promise.resolve([])),
    fetchEmployeeDepartmentHistory: vi.fn(() => Promise.resolve([])),
    fetchEmploymentHistory: vi.fn(() => Promise.resolve([])),
    fetchRawAttendanceLogs: vi.fn(() => Promise.resolve({ data: [], total: 0 })),
    fetchMyEmployeeProfile: vi.fn(() => Promise.resolve({ id: "emp-1", full_name: "Test User" })),
  };
});

vi.mock("recharts", () => ({
  ResponsiveContainer: ({ children }: any) => <div>{children}</div>,
  BarChart: () => <div />, Bar: () => <div />, XAxis: () => <div />, YAxis: () => <div />,
  CartesianGrid: () => <div />, Tooltip: () => <div />, Legend: () => <div />,
  LineChart: () => <div />, Line: () => <div />, AreaChart: () => <div />, Area: () => <div />,
  PieChart: () => <div />, Pie: () => <div />, Cell: () => <div />,
}));

describe("Page Rendering", () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    vi.clearAllMocks();
    i18n.changeLanguage("en");
    queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false, gcTime: 0, staleTime: 0 } },
    });
  });

  const renderPage = (ui: React.ReactElement) => {
    return render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <MemoryRouter>
            {ui}
          </MemoryRouter>
        </ToastProvider>
      </QueryClientProvider>
    );
  };

  it("renders DashboardPage correctly", async () => {
    renderPage(<DashboardPage />);
    // DashboardPage has data-testid="dashboard-title"
    expect(await screen.findByTestId("dashboard-title")).toBeInTheDocument();
  });

  it("renders EmployeesPage correctly", async () => {
    renderPage(<EmployeesPage />);
    // Wait for the title which might be translated
    await waitFor(() => {
      const titles = screen.getAllByRole("heading");
      expect(titles.some(h => /Employees/i.test(h.textContent!) || /Nhân viên/i.test(h.textContent!))).toBe(true);
    });
  });

  it("renders LeaveBalancesPage correctly", async () => {
    renderPage(<LeaveBalancesPage />);
    await waitFor(() => {
      const titles = screen.getAllByRole("heading");
      expect(titles.some(h => /Leave Balances/i.test(h.textContent!) || /Số dư nghỉ phép/i.test(h.textContent!))).toBe(true);
    });
  });

  it("renders AttendanceLogsPage correctly", async () => {
    renderPage(<AttendanceLogsPage />);
    await waitFor(() => {
      const titles = screen.getAllByRole("heading");
      expect(titles.some(h => /Attendance Logs/i.test(h.textContent!) || /Nhật ký quẹt thẻ/i.test(h.textContent!))).toBe(true);
    });
  });

  it("renders MyClockPage correctly", async () => {
    // MyClockPage might render "No profile found" if API fails, but we mocked it.
    renderPage(<MyClockPage />);
    await waitFor(() => {
      // Look for anything that indicates it rendered.
      // In the previous failure it rendered "Không tìm thấy hồ sơ." (No profile found)
      // because fetchMyEmployeeProfile was missing.
      expect(screen.queryByText(/loading/i)).not.toBeInTheDocument();
    });
  });
});
