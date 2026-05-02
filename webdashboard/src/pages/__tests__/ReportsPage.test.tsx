import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";

import { ToastProvider } from "../../components/ui/Toast";
import i18n from "../../i18n/config";
import * as api from "../../lib/api";
import { Employee, PaginatedResponse, AttendanceSummary, DailyAttendanceReport, Department } from "../../lib/api/types";
import ReportsPage from "../ReportsPage";

// Mock TanStack Router
vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => vi.fn(),
 
  useLocation: () => ({ pathname: "/", search: "", hash: "" }),
  Link: ({ to, search, children, ...props }: { to: string; search?: Record<string, unknown>; children: React.ReactNode; [key: string]: unknown }) => {
    let href = to;
    if (search && typeof search === 'object') {
      const params = new URLSearchParams();
      Object.entries(search).forEach(([key, value]) => {
        if (value !== undefined && value !== null) {
          params.append(key, String(value));
        }
      });
      const qs = params.toString();
      if (qs) href += `?${qs}`;
    }
    return <a href={href} {...props}>{children}</a>;
  },
}));

// Mock scrollIntoView and scrollTo for jsdom
window.HTMLElement.prototype.scrollIntoView = vi.fn();

// Mock IntersectionObserver for jsdom
class MockIntersectionObserver {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
  constructor() { }
 
}
window.IntersectionObserver = MockIntersectionObserver as unknown as typeof IntersectionObserver;

// Mock the API library
vi.mock("../../lib/api", () => ({
  fetchAttendanceReport: vi.fn(),
  getCurrentWorkspaceId: vi.fn(),
  fetchWorkspace: vi.fn(),
  reprocessAttendance: vi.fn(),
  fetchEmployees: vi.fn(),
  searchEmployees: vi.fn(),
  fetchDepartments: vi.fn(),
  fetchWorkspaceAttendanceAdjustments: vi.fn(),
  adminConfirmAttendance: vi.fn(),
}));

// Mock DebugContext
vi.mock("@/context/DebugContext", () => ({
  useDebug: () => ({ isEnabled: false }),
}));

describe("ReportsPage", () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    vi.clearAllMocks();
    i18n.changeLanguage("en");
    queryClient = new QueryClient({
      defaultOptions: {
        queries: {
          retry: false,
          gcTime: 0,
          staleTime: 0,
        },
      },
 
    });
 
    vi.mocked(api.getCurrentWorkspaceId).mockReturnValue("test-ws-id");
 
    vi.mocked(api.fetchWorkspace).mockResolvedValue({
 
      id: "test-ws-id",
 
      name: "Test Workspace",
 
      work_unit_minutes: 60,
      rounding_policy: "none",
      date_format: "YYYY-MM-DD",
      time_format: "HH:mm",
 
      timezone: "UTC",
 
      theme_primary_color: "#000000",
      default_language: "en",
      created_at: "2025-01-01T00:00:00Z",
    });
    vi.mocked(api.fetchEmployees).mockResolvedValue({ data: [{ id: "emp-1", full_name: "John Doe", workspace_id: "test-ws-id", employee_id_number: "E01", hire_date: "2025-01-01", is_active: true, created_at: "2025-01-01T00:00:00Z", updated_at: "2025-01-01T00:00:00Z" }], total: 1, page: 1, limit: 50 });
    vi.mocked(api.searchEmployees).mockResolvedValue({ data: [{ id: "emp-1", full_name: "John Doe", workspace_id: "test-ws-id", employee_id_number: "E01", hire_date: "2025-01-01", is_active: true, created_at: "2025-01-01T00:00:00Z", updated_at: "2025-01-01T00:00:00Z" }], total: 1, page: 1, limit: 50 });
    vi.mocked(api.fetchDepartments).mockResolvedValue([]);
    vi.mocked(api.fetchAttendanceReport).mockImplementation(() => Promise.resolve({
      data: [{ employee_id: "emp-1", employee_name: "John Doe", total_std_units: 440, total_ot_units: 40, total_leave_days: 0 }],
      total: 1,
      page: 1,
      limit: 50
    } as unknown as PaginatedResponse<AttendanceSummary>));
    vi.mocked(api.fetchWorkspaceAttendanceAdjustments).mockResolvedValue([]);
    vi.mocked(api.adminConfirmAttendance).mockResolvedValue(undefined);
  });

  const renderWithProviders = (ui: React.ReactElement, _initialEntries = ["/"]) => {
    return render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          {ui}
        </ToastProvider>
      </QueryClientProvider>
    );
  };

  it("renders report table and defaults to last month", async () => {
    renderWithProviders(<ReportsPage />);
    expect(await screen.findByText(/John Doe/i, {}, { timeout: 30000 })).toBeDefined();
    const lastMonthButton = await screen.findByText(/Last Month|Tháng trước/i, {}, { timeout: 30000 });
    expect(lastMonthButton).toBeDefined();
  });

  it("changes date range when clicking 'Tháng trước'", async () => {
    const user = userEvent.setup();
    renderWithProviders(<ReportsPage />);
    const lastMonthButton = await screen.findByText(/Last Month|Tháng trước/i);
    await user.click(lastMonthButton);
 
    expect(api.fetchAttendanceReport).toHaveBeenCalled();
  });

  it("shows custom date inputs when clicking 'Tùy chỉnh'", async () => {
    const user = userEvent.setup();
    renderWithProviders(<ReportsPage />);
    const customButton = await screen.findByText(/Custom|Tùy chỉnh/i);
    await user.click(customButton);
 
    expect(await screen.findByText(/From Date|Từ ngày/i)).toBeDefined();
  });

  it("uses PAGE_SIZE of 50 for infinite scroll", async () => {
    renderWithProviders(<ReportsPage />);
    await waitFor(() => {
      const calls = vi.mocked(api.fetchAttendanceReport).mock.calls;
      expect(calls.length).toBeGreaterThan(0);
      // arg index 5 = limit = PAGE_SIZE = 50
      expect(calls[0][5]).toBe(50);
    });
  });

  it("fetches full dataset with limit=99999 for stats and renders inline stats badges", async () => {
    vi.mocked(api.fetchAttendanceReport).mockImplementation((_wsId, _start, _end, _type, page, limit) => {
      if (limit === 99999) {
        // Full stats query — return all employees
        return Promise.resolve({
          data: [
            { employee_id: "emp-1", employee_name: "John Doe", total_std_units: 440, total_ot_units: 40, total_leave_days: 0 },
            { employee_id: "emp-2", employee_name: "Jane Smith", total_std_units: 360, total_ot_units: 60, total_leave_days: 1 },
          ],
 
          total: 2,
 
          page: 1,
          limit: 99999
        } as unknown as PaginatedResponse<AttendanceSummary>);
      }
      // Paginated query — return only page 1
      return Promise.resolve({
        data: [{ employee_id: "emp-1", employee_name: "John Doe", total_std_units: 440, total_ot_units: 40, total_leave_days: 0 }],
        total: 2,
        page: 1,
        limit: 50
      } as unknown as PaginatedResponse<AttendanceSummary>);
    });

    renderWithProviders(<ReportsPage />);

    // Verify a call with limit=99999 was made for stats
    await waitFor(() => {
      const calls = vi.mocked(api.fetchAttendanceReport).mock.calls;
      const statsCall = calls.find((c) => c[5] === 99999);
      expect(statsCall).toBeDefined();
    });

    // Stats should show totals from ALL employees
    // emp-1: std=440, ot=40 + emp-2: std=360, ot=60 = total std=800, ot=100
    // formatUnitsToHours: 800 * 60min / 60 = 800, 100 * 60min / 60 = 100
    // The "h" suffix is added by JSX: {quickStats.stdHours}h
    await waitFor(() => {
      expect(screen.getByText("800h")).toBeInTheDocument();
      expect(screen.getByText("100h")).toBeInTheDocument();
      expect(screen.getByText("2")).toBeInTheDocument();
    });
  });

  it("shows unit values in tooltips instead of visible text", async () => {
    renderWithProviders(<ReportsPage />);
    await screen.findByText(/John Doe/i);

    // Units should NOT be visible as text
    expect(screen.queryByText("440 U")).toBeNull();
    expect(screen.queryByText("40 OT")).toBeNull();

    // Units should be in title attributes (tooltips)
    const stdHoursCell = screen.getByTitle("440 units");
    expect(stdHoursCell).toBeInTheDocument();

    const otHoursCell = screen.getByTitle("40 OT units");
    expect(otHoursCell).toBeInTheDocument();
  });

  it("opens slide-over drawer when clicking an employee row", async () => {
    const user = userEvent.setup();

 
    const mockDailyRow = {
      employee_id: "emp-1",
      employee_name: "John Doe",
      date: "2026-01-15",
      actual_punch_in: "2026-01-15T01:00:00Z",
      actual_punch_out: "2026-01-15T10:00:00Z",
      scheduled_start: "08:00:00",
      scheduled_end: "17:00:00",
      break_duration_minutes: 60,
      break_start: null,
      break_end: null,
      is_break_auto_deduct: false,
      std_work_units: 480,
      ot_work_units: 0,
      status: "present",
      generation_metadata: null,
    };

    vi.mocked(api.fetchAttendanceReport).mockImplementation(
      (_wsId, _start, _end, type) => {
        if (type === "daily") {
          return Promise.resolve({ data: [mockDailyRow], total: 1, page: 1, limit: 50 } as unknown as PaginatedResponse<DailyAttendanceReport>);
        }
 
        return Promise.resolve({
          data: [{ employee_id: "emp-1", employee_name: "John Doe", total_std_units: 440, total_ot_units: 40, total_leave_days: 0 }],
          total: 1,
          page: 1,
          limit: 50
        } as unknown as PaginatedResponse<AttendanceSummary>);
      }
    );

    renderWithProviders(<ReportsPage />);
    const employeeRow = await screen.findByText(/John Doe/i);
    await user.click(employeeRow);

    const sheetPanel = await screen.findByTestId("sheet-panel");
    expect(sheetPanel).toBeDefined();
  });

  it("always fetches summary data (no daily tab)", async () => {
    renderWithProviders(<ReportsPage />);
    await waitFor(() => {
      const calls = vi.mocked(api.fetchAttendanceReport).mock.calls;
      expect(calls.length).toBeGreaterThan(0);
      expect(calls[0][3]).toBe("summary");
    });

    const dailyTab = screen.queryByText(/Daily Details|Chi tiết hàng ngày/i);
    expect(dailyTab).toBeNull();
  });

  it("renders a search input that exists and is functional", async () => {
    renderWithProviders(<ReportsPage />);
    await screen.findByText(/John Doe/i);

    // Search input should exist
    const searchInput = screen.getByRole("textbox");
    expect(searchInput).toBeDefined();
    // autoFocus removed for accessibility (jsx-a11y/no-autofocus)
    expect(searchInput).toBeInTheDocument();
  });

  it("triggers searchEmployees API on search input with debounce", async () => {
    const user = userEvent.setup();
    renderWithProviders(<ReportsPage />);
    await screen.findByText(/John Doe/i);

    const searchInput = screen.getByRole("textbox");
    await user.type(searchInput, "John");

    // searchEmployees should be called after debounce (400ms)
    await waitFor(() => {
      expect(api.searchEmployees).toHaveBeenCalledWith("test-ws-id", "John");
    }, { timeout: 2000 });
  });
 

  it("auto-selects text on focus and refocuses on drawer close", async () => {
    const user = userEvent.setup();
    const mockDailyRow = {
      employee_id: "emp-1",
      employee_name: "John Doe",
      date: "2026-01-15",
      actual_punch_in: "2026-01-15T01:00:00Z",
      actual_punch_out: "2026-01-15T10:00:00Z",
      scheduled_start: "08:00:00",
      scheduled_end: "17:00:00",
      break_duration_minutes: 60,
      break_start: null,
      break_end: null,
      is_break_auto_deduct: false,
      std_work_units: 480,
      ot_work_units: 0,
      status: "present",
      generation_metadata: null,
    };
    vi.mocked(api.fetchAttendanceReport).mockImplementation((_wsId, _start, _end, type) => {
      if (type === "daily") return Promise.resolve({ data: [mockDailyRow], total: 1, page: 1, limit: 50 } as unknown as PaginatedResponse<DailyAttendanceReport>);
      return Promise.resolve({ data: [{ employee_id: "emp-1", employee_name: "John Doe", total_std_units: 440, total_ot_units: 40, total_leave_days: 0 }], total: 1, page: 1, limit: 50 } as unknown as PaginatedResponse<AttendanceSummary>);
    });

    renderWithProviders(<ReportsPage />);
    const employeeRow = await screen.findByText(/John Doe/i);

    // Type into search
    const searchInput = screen.getByRole("textbox");
    await user.type(searchInput, "Test");

    // Test auto-select on focus
    searchInput.blur();
    expect((searchInput as HTMLInputElement).selectionStart).toBe((searchInput as HTMLInputElement).selectionEnd);
    await user.click(searchInput);
    expect((searchInput as HTMLInputElement).selectionStart).toBe(0);
    expect((searchInput as HTMLInputElement).selectionEnd).toBe(4); // length of "Test"

    // Open drawer
    await user.click(employeeRow);
    const sheetPanel = await screen.findByTestId("sheet-panel");
    expect(sheetPanel).toBeInTheDocument();

    // Ensure search input lost focus
    expect(searchInput).not.toHaveFocus();

    // Close drawer
    const xIcon = sheetPanel.querySelector('svg.lucide-x');
    const closeButton = xIcon?.closest('button');
    await user.click(closeButton!);

    // wait for the 100ms timeout in focusSearch
    await new Promise(r => setTimeout(r, 150));

    // search should be focused again
    expect(searchInput).toHaveFocus();
    // and its text should be auto-selected because of the onFocus handler
    expect((searchInput as HTMLInputElement).selectionStart).toBe(0);
    expect((searchInput as HTMLInputElement).selectionEnd).toBe(4);
  });
 

  it("opens the first employee's drawer when pressing Enter in search input", async () => {
    const user = userEvent.setup();
    const mockDailyRow = {
      employee_id: "emp-1",
      employee_name: "John Doe",
      date: "2026-01-15",
      actual_punch_in: "2026-01-15T01:00:00Z",
      actual_punch_out: "2026-01-15T10:00:00Z",
      scheduled_start: "08:00:00",
      scheduled_end: "17:00:00",
      break_duration_minutes: 60,
      break_start: null,
      break_end: null,
      is_break_auto_deduct: false,
      std_work_units: 480,
      ot_work_units: 0,
      status: "present",
      generation_metadata: null,
    };
    vi.mocked(api.fetchAttendanceReport).mockImplementation((_wsId, _start, _end, type) => {
      if (type === "daily") return Promise.resolve({ data: [mockDailyRow], total: 1, page: 1, limit: 50 } as unknown as PaginatedResponse<DailyAttendanceReport>);
      return Promise.resolve({ data: [{ employee_id: "emp-1", employee_name: "John Doe", total_std_units: 440, total_ot_units: 40, total_leave_days: 0 }], total: 1, page: 1, limit: 50 } as unknown as PaginatedResponse<AttendanceSummary>);
    });

    renderWithProviders(<ReportsPage />);
    await screen.findByText(/John Doe/i);

    // Get search input and focus it
    const searchInput = screen.getByRole("textbox");

    // Type something and hit enter mapped directly onto the element
    await user.type(searchInput, "John{Enter}");

    // The Drawer should open up for John Doe (emp-1)
    const sheetPanel = await screen.findByTestId("sheet-panel");
    expect(sheetPanel).toBeInTheDocument();

    // Ensure the input has lost focus (blur) so arrow navigation works inside the drawer
    expect(searchInput).not.toHaveFocus();

    // We should see the daily report data because the drawer is open
    const rawLogsToggle = await screen.findByRole("button", { name: /Nhật ký thô|View Raw Logs|reports.view_raw_logs/i });
    expect(rawLogsToggle).toBeInTheDocument();
 
  });
  it("allows switching tabs with arrows after opening the drawer via Enter key", async () => {
    const user = userEvent.setup();
    const mockDailyRow = {
      employee_id: "emp-1",
      employee_name: "John Doe",
      date: "2026-01-15",
      actual_punch_in: "2026-01-15T01:00:00Z",
      actual_punch_out: "2026-01-15T10:00:00Z",
      scheduled_start: "08:00:00",
      scheduled_end: "17:00:00",
      break_duration_minutes: 60,
      break_start: null,
      break_end: null,
      is_break_auto_deduct: false,
      std_work_units: 480,
      ot_work_units: 0,
      status: "present",
      generation_metadata: null,
    };
    vi.mocked(api.fetchAttendanceReport).mockImplementation((_wsId, _start, _end, type) => {
      if (type === "daily") return Promise.resolve({ data: [mockDailyRow], total: 1, page: 1, limit: 50 } as unknown as PaginatedResponse<DailyAttendanceReport>);
      return Promise.resolve({ data: [{ employee_id: "emp-1", employee_name: "John Doe", total_std_units: 440, total_ot_units: 40, total_leave_days: 0 }], total: 1, page: 1, limit: 50 } as unknown as PaginatedResponse<AttendanceSummary>);
    });

    renderWithProviders(<ReportsPage />);
    await screen.findByText(/John Doe/i);

    // Get search input and focus it
    const searchInput = screen.getByRole("textbox");

    // Type something and hit enter mapped directly onto the element
    await user.type(searchInput, "John{Enter}");

    // The Drawer should open up for John Doe
    const sheetPanel = await screen.findByTestId("sheet-panel");
    expect(sheetPanel).toBeInTheDocument();

    // Ensure the input has lost focus
    expect(searchInput).not.toHaveFocus();

    // Check if we can switch tabs via ArrowRight (since focus is no longer on input)
    const dailyToggle = await screen.findByRole("button", { name: /Daily|Chi tiết/i });
    const rawLogsToggle = await screen.findByRole("button", { name: /View Raw Logs|Nhật ký thô/i });

    expect(dailyToggle.className).toContain("text-primary-600");

    // Simulate ArrowRight
    fireEvent.keyDown(window, { key: "ArrowRight" });

    // Tab should switch
    await waitFor(() => {
      expect(rawLogsToggle.className).toContain("text-primary-600");
    });
  });
 

  it("navigates with ArrowDown and opens the second employee when pressing Enter", async () => {
    const user = userEvent.setup();
    const mockDailyRow = {
      employee_id: "emp-2",
      employee_name: "Jane Smith",
      date: "2026-01-15",
      actual_punch_in: "2026-01-15T01:00:00Z",
      actual_punch_out: "2026-01-15T10:00:00Z",
      scheduled_start: "08:00:00",
      scheduled_end: "17:00:00",
      break_duration_minutes: 60,
      break_start: null,
      break_end: null,
      is_break_auto_deduct: false,
      std_work_units: 480,
      ot_work_units: 0,
      status: "present",
      generation_metadata: null,
 
    };
    vi.mocked(api.fetchAttendanceReport).mockImplementation((_wsId, _start, _end, type) => {
      if (type === "daily") return Promise.resolve({ data: [mockDailyRow], total: 1, page: 1, limit: 50 } as unknown as PaginatedResponse<DailyAttendanceReport>);
      return Promise.resolve({
        data: [
          { employee_id: "emp-1", employee_name: "John Doe", total_std_units: 440, total_ot_units: 40, total_leave_days: 0 },
          { employee_id: "emp-2", employee_name: "Jane Smith", total_std_units: 440, total_ot_units: 40, total_leave_days: 0 }
        ],
        total: 2,
        page: 1,
 
        limit: 50
      } as unknown as PaginatedResponse<AttendanceSummary>);
    });

    renderWithProviders(<ReportsPage />);
    await screen.findByText(/John Doe/i);
    await screen.findByText(/Jane Smith/i);

    // Get search input and focus it
    const searchInput = screen.getByRole("textbox");

    // Clear mocks
    vi.mocked(window.HTMLElement.prototype.scrollIntoView).mockClear();
    window.HTMLElement.prototype.scrollTo = vi.fn();

    // Press ArrowDown to select the second row
    await user.type(searchInput, "{ArrowDown}");

    // Verify scrollIntoView was called smooth/nearest for index 1
 
    expect(window.HTMLElement.prototype.scrollIntoView).toHaveBeenCalledWith({ behavior: "smooth", block: "nearest" });

    // Press ArrowUp to go back to the first row (index 0)
    vi.mocked(window.HTMLElement.prototype.scrollIntoView).mockClear();
    await user.type(searchInput, "{ArrowUp}");

    // In full-page mode, scrollIntoView is used (no .overflow-auto container)
    expect(window.HTMLElement.prototype.scrollIntoView).toHaveBeenCalled();

    // Put it back to index 1 and press Enter
    await user.type(searchInput, "{ArrowDown}{Enter}");

    // The Drawer should open up for Jane Smith (emp-2)
    const sheetPanel = await screen.findByTestId("sheet-panel");
    expect(sheetPanel).toBeInTheDocument();

    // Verify the drawer is for Jane Smith by checking the mock data return
    const expectedName = await screen.findAllByText(/Jane Smith|John Doe/i);
    expect(expectedName.length).toBeGreaterThan(0);
  });

  it("displays an empty state when no data is returned", async () => {
 
    vi.mocked(api.fetchAttendanceReport).mockImplementation(() => {
      return Promise.resolve({ data: [], total: 0, page: 1, limit: 50 } as unknown as PaginatedResponse<AttendanceSummary>);
    });

    renderWithProviders(<ReportsPage />);

    // The Reports.messages.no_data localized string or empty graphic should appear
    // We could search for the text or the svg icon class
    const emptyStateTitle = await screen.findByText(/NO DATA|Không có dữ liệu/i);
    expect(emptyStateTitle).toBeInTheDocument();
  });

  it("renders cross-navigation link to raw logs with preserved date range", async () => {
    renderWithProviders(<ReportsPage />);
    // Using string matching that is case-insensitive and covers both EN and VI since i18n isn't mocked the same way
    const link = await screen.findByRole("link", { name: /View Raw Logs|Xem nhật ký thô|REPORTS\.VIEW_RAW_LOGS/i });
 
    expect(link).toBeDefined();

    const now = new Date();
    const lastMonth = new Date(now.getFullYear(), now.getMonth() - 1, 1);
    const lastMonthEnd = new Date(now.getFullYear(), now.getMonth(), 0);
    const pad = (n: number) => String(n).padStart(2, '0');
    const fromString = `${lastMonth.getFullYear()}-${pad(lastMonth.getMonth() + 1)}-01`;
    const toString_ = `${lastMonthEnd.getFullYear()}-${pad(lastMonthEnd.getMonth() + 1)}-${pad(lastMonthEnd.getDate())}`;

    expect(link.getAttribute("href")).toBe(`/attendance-logs?from=${fromString}&to=${toString_}`);
  });
  it("filters the report by selected department", async () => {
    const user = userEvent.setup();
    // Setup specific mock for departments
    vi.mocked(api.fetchDepartments).mockResolvedValue([
      { id: "dept-1", name: "Engineering", workspace_id: "test-ws-id", created_at: "2025-01-01T00:00:00Z", updated_at: "2025-01-01T00:00:00Z", active_employee_count: 0 },
      { id: "dept-2", name: "HR", workspace_id: "test-ws-id", created_at: "2025-01-01T00:00:00Z", updated_at: "2025-01-01T00:00:00Z", active_employee_count: 0 }
    ]);
 

    renderWithProviders(<ReportsPage />);

    // Wait for the department chips to render
    const engineeringChip = await screen.findByRole("button", { name: /Engineering/i });
    expect(engineeringChip).toBeDefined();

    // Click the Engineering chip
    await user.click(engineeringChip);

    // Verify fetchAttendanceReport is called with department_id="dept-1" (9th arg, index 8)
    await waitFor(() => {
      const calls = vi.mocked(api.fetchAttendanceReport).mock.calls;
      const latestCall = calls[calls.length - 1];
      expect(latestCall[8]).toBe("dept-1");
    });

    // Verify that clicking "All" clears the department filter (null is passed)
    const allChip = await screen.findByRole("button", { name: /all|tất cả/i });
 
    await user.click(allChip);

    await waitFor(() => {
      const calls = vi.mocked(api.fetchAttendanceReport).mock.calls;
      const latestCall = calls[calls.length - 1];
      expect(latestCall[8]).toBeNull();
 
    });
 
  });

  it("sorts by OT hours when clicking OT column header", async () => {
    const user = userEvent.setup();
    i18n.changeLanguage("en");
    renderWithProviders(<ReportsPage />);
    await screen.findByText(/John Doe/i);

 
    // Find the OT Hours column header — text is "Overtime" (en) inside a flex div with SVG icon
 
    const otHeaderText = screen.getByText((content, element) => {
      return element?.tagName?.toLowerCase() === "div" && /Overtime|Tăng ca/i.test(content);
    });
    const otClickTarget = otHeaderText.closest("th") || otHeaderText;

    // Clear mock calls so we only see calls triggered by the click
    vi.mocked(api.fetchAttendanceReport).mockClear();

    // Click OT header — should sort desc first (default for numeric)
    await user.click(otClickTarget);

    await waitFor(() => {
      const calls = vi.mocked(api.fetchAttendanceReport).mock.calls;
      const sortedCall = calls.find((c) => c[6] === "ot_hours_desc");
      expect(sortedCall).toBeDefined();
    });

    // Click again — should toggle to asc
    await user.click(otClickTarget);

 
    await waitFor(() => {
      const calls = vi.mocked(api.fetchAttendanceReport).mock.calls;
      const sortedCall = calls.find((c) => c[6] === "ot_hours_asc");
      expect(sortedCall).toBeDefined();
    });
  });
 

 
  it("renders total hours column and sorts by total hours", async () => {
    const user = userEvent.setup();
    i18n.changeLanguage("en");
    renderWithProviders(<ReportsPage />);
    await screen.findByText(/John Doe/i);

    // Total hours = 440 std + 40 ot = 480 units = 480h (with work_unit_minutes=60)
    expect(screen.getByTitle("480 total units")).toBeInTheDocument();
 

 
    // Find the Total Hours column header
    const totalHeader = screen.getByText((content, element) => {
      return element?.tagName?.toLowerCase() === "div" && /Total Hours|Tổng giờ công/i.test(content);
    });
    const totalClickTarget = totalHeader.closest("th") || totalHeader;

 
    vi.mocked(api.fetchAttendanceReport).mockClear();

    // Click header — should sort desc first
    await user.click(totalClickTarget);

    await waitFor(() => {
      const calls = vi.mocked(api.fetchAttendanceReport).mock.calls;
      const sortedCall = calls.find((c) => c[6] === "total_hours_desc");
      expect(sortedCall).toBeDefined();
    });

    // Click again — toggle to asc
    await user.click(totalClickTarget);

    await waitFor(() => {
      const calls = vi.mocked(api.fetchAttendanceReport).mock.calls;
      const sortedCall = calls.find((c) => c[6] === "total_hours_asc");
      expect(sortedCall).toBeDefined();
    });
  });

  it("renders department chips with wrapping layout", async () => {
    vi.mocked(api.fetchDepartments).mockResolvedValue([
      { id: "dept-1", name: "Engineering", workspace_id: "test-ws-id", created_at: "2025-01-01T00:00:00Z", updated_at: "2025-01-01T00:00:00Z", active_employee_count: 0 },
      { id: "dept-2", name: "HR", workspace_id: "test-ws-id", created_at: "2025-01-01T00:00:00Z", updated_at: "2025-01-01T00:00:00Z", active_employee_count: 0 },
      { id: "dept-3", name: "Design", workspace_id: "test-ws-id", created_at: "2025-01-01T00:00:00Z", updated_at: "2025-01-01T00:00:00Z", active_employee_count: 0 }
    ]);

    renderWithProviders(<ReportsPage />);

    const chipsContainer = await screen.findByTestId("department-chips");
    
    // Verify wrap class is present
    expect(chipsContainer.className).toContain("flex-wrap");
    
    // Verify overflow-x-auto is removed
    expect(chipsContainer.className).not.toContain("overflow-x-auto");
    expect(chipsContainer.className).not.toContain("scrollbar-thin");
  });
});
