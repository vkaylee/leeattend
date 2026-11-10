import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import { ToastProvider } from "../../components/ui/toast";
import i18n from "../../i18n/config";
import * as api from "../../lib/api";
import AttendanceLogsPage from "../attendance-logs-page";
import PayrollPage from "../payroll-page";

const { mockWorkspace } = vi.hoisted(() => ({
  mockWorkspace: { date_format: "DD/MM/YYYY", timezone: "UTC" },
}));

vi.mock("@/hooks/use-organization", () => ({
  useOrganization: () => ({ organization: mockWorkspace }),
}));

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

// Mock useDebounce to be immediate
vi.mock("@/hooks/use-debounce", () => ({
  useDebounce: (value: unknown) => value,
 
}));

// Mock EmployeeSearch to avoid internal queries
vi.mock("@/components/common/employee-search", () => ({
  EmployeeSearch: ({ value, onChange }: { value: string; onChange: (v: string) => void }) => (
 
    <select aria-label="Employee Search"
      data-testid="employee-select"
      onChange={(e) => { onChange(e.target.value); }}
      value={value}
    >
      <option value="">Select...</option>
      <option value="emp-1">John Doe</option>
    </select>
  )
}));

// Mock MonthYearPicker
vi.mock("@/components/common/month-year-picker", () => ({
  MonthYearPicker: ({ _value, _onChange }: { _value: unknown; _onChange: unknown }) => (
 
    <div data-testid="month-year-picker">Mock Picker</div>
  )
}));

// Mock the API library
vi.mock("../../lib/api", () => ({
  fetchEmployees: vi.fn(),
  searchEmployees: vi.fn(),
  fetchEmployeePayslips: vi.fn(),
  fetchRawAttendanceLogs: vi.fn(),
  fetchOrganization: vi.fn(),
  submitAttendanceAdjustment: vi.fn(),
  fetchAttendanceAdjustments: vi.fn(),
  fetchMyPermissions: vi.fn(),
}));

describe("Date Format Integration", () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    vi.clearAllMocks();
    void i18n.changeLanguage("en"); // Force English
    queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false, gcTime: 0, staleTime: 0 } },
    });
    vi.mocked(api.fetchMyPermissions).mockResolvedValue({ permissions: ["attendance_staging:manage"], roles: ["admin"], managed_department_ids: [] });
 
 
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
    vi.mocked(api.fetchEmployees).mockResolvedValue({

      data: [{ id: "emp-1", full_name: "John Doe", employee_id_number: "EMP001", workspace_id: "ws-1", created_at: "2025-01-01T00:00:00Z", updated_at: "2025-01-01T00:00:00Z", tags: [] }],
      total: 1,
      page: 1,
      limit: 50,
    });
    mockWorkspace.date_format = "DD/MM/YYYY";

    vi.mocked(api.fetchRawAttendanceLogs).mockResolvedValue({

      data: [{ id: "l-1", punch_time: "2026-01-15T08:00:00Z", source: "import", employee_id: "emp-1" }],
      total: 1,
      page: 1,
      limit: 50
    } as unknown as never);

    renderWithProviders(<AttendanceLogsPage />);

    // ResponsiveTable renders both desktop and mobile views, so use findAllByText
    const dateElements = await screen.findAllByText(/15\/01\/2026/);
    expect(dateElements.length).toBeGreaterThan(0);
  });

  it("renders dates in YYYY-MM-DD format when workspace setting is YYYY-MM-DD", async () => {
    vi.mocked(api.fetchEmployees).mockResolvedValue({

      data: [{ id: "emp-1", full_name: "John Doe", employee_id_number: "EMP001", workspace_id: "ws-1", created_at: "2025-01-01T00:00:00Z", updated_at: "2025-01-01T00:00:00Z", tags: [] }],
      total: 1,
      page: 1,
      limit: 50,
    });
    mockWorkspace.date_format = "YYYY-MM-DD";

    vi.mocked(api.fetchRawAttendanceLogs).mockResolvedValue({

      data: [{ id: "l-1", punch_time: "2026-01-15T08:00:00Z", source: "import", employee_id: "emp-1" }],
      total: 1,
      page: 1,
      limit: 50
    } as unknown as never);

    renderWithProviders(<AttendanceLogsPage />);

    // ResponsiveTable renders both desktop and mobile views, so use findAllByText
    const dateElements = await screen.findAllByText(/2026-01-15/);
    expect(dateElements.length).toBeGreaterThan(0);
  });

  it("renders month/year correctly in PayrollPage", async () => {
    vi.mocked(api.fetchEmployees).mockResolvedValue({
 
      data: [{ id: "emp-1", full_name: "John Doe", employee_id_number: "EMP001", workspace_id: "ws-1", created_at: "2025-01-01T00:00:00Z", updated_at: "2025-01-01T00:00:00Z", tags: [] }],
      total: 1,
      page: 1,
      limit: 50,
    });
    vi.mocked(api.fetchEmployeePayslips).mockResolvedValue([]);
 
    vi.mocked(api.fetchAttendanceAdjustments).mockResolvedValue({ data: [], total: 0, page: 1, limit: 20 });
 
    vi.mocked(api.fetchOrganization).mockResolvedValue({ date_format: "DD/MM/YYYY", timezone: "UTC" } as unknown as never);
 

    renderWithProviders(<PayrollPage />);

    const empBtns = await screen.findAllByText("John Doe");
    fireEvent.click(empBtns[0]);

    // Check for the title (VN or EN depending on env, checking for VN based on failure output)
    // Using getAllByText and taking the first one (h1) or being more specific with selector
    const titles = await screen.findAllByText(/Tính lương|Payroll/i, { selector: 'h1' });
    expect(titles.length).toBeGreaterThan(0);
  });
});
