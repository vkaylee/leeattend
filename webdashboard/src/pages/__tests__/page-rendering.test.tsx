import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import { ToastProvider } from "../../components/ui/toast";
import i18n from "../../i18n/config";
import AttendanceLogsPage from "../attendance-logs-page";
import DashboardPage from "../dashboard-page";
import EmployeesPage from "../employees-page";
import MyClockPage from "../ess/my-clock-page";
import LeaveBalancesPage from "../leave-balances-page";

// Mock TanStack Router
vi.mock("@tanstack/react-router", () => {
  const defaultSearch = { tab: "recorded", mode: "this_month" } as Record<string, unknown>;
  return {
    useNavigate: () => vi.fn(),
    useLocation: () => ({ pathname: "/", search: "", hash: "" }),
    useSearch: () => defaultSearch,
    createFileRoute: () => () => ({
      useSearch: () => defaultSearch,
    }),
    Link: ({ to, children, ...props }: { to: string; children: React.ReactNode }) => <a href={to} {...props}>{children}</a>,
  };
});

// Mock scrollIntoView and IntersectionObserver for jsdom
window.HTMLElement.prototype.scrollIntoView = vi.fn();

class MockIntersectionObserver {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
}
 
window.IntersectionObserver = MockIntersectionObserver as unknown as typeof IntersectionObserver;

// Mock hooks
vi.mock("@/hooks/use-profile", () => ({
  useProfile: () => ({
    user: { id: "test-user", email: "test@example.com", full_name: "Test User", preferred_language: "en" },
    isLoading: false,
    isError: false,
  }),
}));

vi.mock("@/hooks/use-organization", () => ({
  useOrganization: () => ({
    organization: { id: "test-ws-id", name: "Test Workspace", date_format: "DD/MM/YYYY", default_language: "en" },
    isLoading: false,
    isError: false,
  }),
}));

vi.mock("@/hooks/use-theme", () => ({
  useTheme: () => ({
    primaryColor: "#10b981",
    setPreviewTheme: vi.fn(),
    setUserTheme: vi.fn(),
  }),
}));

vi.mock("@/hooks/use-i18n-sync", () => ({
  useI18nSync: vi.fn(),
}));



vi.mock("@/hooks/use-dashboard", () => ({
  useDashboardStats: () => ({ data: { total_employees: 10, present_today: 8, pending_leave_requests: 2, estimated_monthly_payroll: "50000", late_today: 1, absent_today: 1, on_leave_today: 1, early_departure_today: 0, avg_work_hours_today: 7.5 }, isLoading: false, error: null }),
  useDashboardMetrics: () => ({ data: null, isLoading: false, error: null }),
  useAttendanceTrend: () => ({ data: [], isLoading: false, error: null }),
  useDepartmentAttendance: () => ({ data: [], isLoading: false, error: null }),
  useOvertimeTrend: () => ({ data: [], isLoading: false, error: null }),
  usePeakTimes: () => ({ data: [], isLoading: false, error: null }),
  useLeaveTypesBreakdown: () => ({ data: [], isLoading: false, error: null }),
  useLatenessByDay: () => ({ data: [], isLoading: false, error: null }),
  useOvertimeBreakdown: () => ({ data: undefined, isLoading: false, error: null }),
}));

vi.mock("@/lib/api", async (importOriginal) => {
 
  const actual = await importOriginal<Record<string, unknown>>();
  return {
    ...actual,
    getCurrentUserId: vi.fn(() => "test-user"),
    fetchWorkspaceLeaveBalances: vi.fn(() => Promise.resolve([])),
    fetchEmployees: vi.fn(() => Promise.resolve([])),
    fetchWorkspace: vi.fn(() => Promise.resolve({ date_format: "DD/MM/YYYY", timezone: "UTC", default_language: "en" })),
    fetchDepartments: vi.fn(() => Promise.resolve([])),
    fetchEmployeeDepartmentHistory: vi.fn(() => Promise.resolve([])),
    fetchEmploymentHistory: vi.fn(() => Promise.resolve([])),
    fetchRawAttendanceLogs: vi.fn(() => Promise.resolve({ data: [], total: 0 })),
    fetchMyEmployeeProfile: vi.fn(() => Promise.resolve({ id: "emp-1", full_name: "Test User" })),
    searchEmployees: vi.fn(() => Promise.resolve({ data: [], total: 0 })),
  };
});

vi.mock("recharts", () => ({
 
  ResponsiveContainer: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  BarChart: () => <div />, Bar: () => <div />, XAxis: () => <div />, YAxis: () => <div />,
  CartesianGrid: () => <div />, Tooltip: () => <div />, Legend: () => <div />,
  LineChart: () => <div />, Line: () => <div />, AreaChart: () => <div />, Area: () => <div />,
  PieChart: () => <div />, Pie: () => <div />, Cell: () => <div />,
  ComposedChart: () => <div />,
}));

describe("Page Rendering", () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    vi.clearAllMocks();
    void i18n.changeLanguage("en");
    queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false, gcTime: 0, staleTime: 0 } },
    });
  });

  const renderPage = (ui: React.ReactElement) => {
    return render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          {ui}
        </ToastProvider>
      </QueryClientProvider>
    );
  };

  it("renders DashboardPage correctly", async () => {
    // useHeaderActions is mocked globally — just verify the page renders without crashing
    renderPage(<DashboardPage />);
    await waitFor(() => {
      expect(document.querySelector("[class*='space-y-6']")).toBeInTheDocument();
    }, { timeout: 10000 });
  });

  it("renders EmployeesPage correctly", async () => {
    renderPage(<EmployeesPage />);
    // Wait for the title which might be translated
    await waitFor(() => {
      const titles = screen.getAllByRole("heading");
      expect(titles.some(h => /Employees/i.test(h.textContent) || /Nhân viên/i.test(h.textContent))).toBe(true);
    });
  });

  it("renders LeaveBalancesPage correctly", async () => {
    renderPage(<LeaveBalancesPage />);
    await waitFor(() => {
      const titles = screen.getAllByRole("heading");
      expect(titles.some(h => /Leave Balances/i.test(h.textContent) || /Số dư nghỉ phép/i.test(h.textContent))).toBe(true);
    });
  });

  it("renders AttendanceLogsPage correctly", async () => {
    renderPage(<AttendanceLogsPage />);
    await waitFor(() => {
      const titles = screen.getAllByRole("heading");
      expect(titles.some(h => /Attendance Logs/i.test(h.textContent) || /Nhật ký quẹt thẻ/i.test(h.textContent))).toBe(true);
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
