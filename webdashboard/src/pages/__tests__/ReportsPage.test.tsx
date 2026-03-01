import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import ReportsPage from "../ReportsPage";
import { MemoryRouter } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ToastProvider } from "../../components/ui/Toast";
import * as api from "../../lib/api";
import React from "react";
import i18n from "../../i18n/config";
import { getDaysInMonth } from "date-fns";

// Mock the API library
vi.mock("../../lib/api", () => ({
  fetchAttendanceReport: vi.fn(),
  getCurrentWorkspaceId: vi.fn(),
  fetchWorkspace: vi.fn(),
  reprocessAttendance: vi.fn(),
  fetchEmployees: vi.fn(),
  searchEmployees: vi.fn(),
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
    (api.getCurrentWorkspaceId as any).mockReturnValue("test-ws-id");
    (api.fetchWorkspace as any).mockResolvedValue({ id: "test-ws-id", name: "Test Workspace", work_unit_minutes: 60 });
    (api.fetchEmployees as any).mockResolvedValue({ data: [{ id: "emp-1", full_name: "John Doe" }], total: 1 });
    (api.searchEmployees as any).mockResolvedValue({ data: [{ id: "emp-1", full_name: "John Doe", employee_id_number: "E01" }], total: 1 });
    (api.fetchAttendanceReport as any).mockImplementation(() => Promise.resolve({
      data: [{ employee_id: "emp-1", employee_name: "John Doe", total_std_units: 440, total_ot_units: 40, total_leave_days: 0 }],
      total: 1
    }));
    (api.fetchWorkspaceAttendanceAdjustments as any).mockResolvedValue([]);
    (api.adminConfirmAttendance as any).mockResolvedValue(undefined);
  });

  const renderWithProviders = (ui: React.ReactElement, initialEntries = ["/"]) => {
    return render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <MemoryRouter initialEntries={initialEntries}>
            {ui}
          </MemoryRouter>
        </ToastProvider>
      </QueryClientProvider>
    );
  };

  it("renders report table and defaults to last month", async () => {
    renderWithProviders(<ReportsPage />);
    expect(await screen.findByText(/John Doe/i, {}, { timeout: 30000 })).toBeDefined();
    const lastMonthBtn = await screen.findByText(/Last Month|Tháng trước/i, {}, { timeout: 30000 });
    expect(lastMonthBtn).toBeDefined();
  });

  it("changes date range when clicking 'Tháng trước'", async () => {
    const user = userEvent.setup();
    renderWithProviders(<ReportsPage />);
    const lastMonthBtn = await screen.findByText(/Last Month|Tháng trước/i);
    await user.click(lastMonthBtn);
    expect(api.fetchAttendanceReport).toHaveBeenCalled();
  });

  it("shows custom date inputs when clicking 'Tùy chỉnh'", async () => {
    const user = userEvent.setup();
    renderWithProviders(<ReportsPage />);
    const customBtn = await screen.findByText(/Custom|Tùy chỉnh/i);
    await user.click(customBtn);
    expect(await screen.findByText(/From Date|Từ ngày/i)).toBeDefined();
  });

  it("defaults pageSize to the number of days in last month", async () => {
    renderWithProviders(<ReportsPage />);
    const now = new Date();
    const lastMonth = new Date(now.getFullYear(), now.getMonth() - 1, 1);
    const expectedDays = getDaysInMonth(lastMonth);
    await waitFor(() => {
      const calls = (api.fetchAttendanceReport as any).mock.calls;
      expect(calls.length).toBeGreaterThan(0);
      expect(calls[0][5]).toBe(expectedDays);
    });
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

    (api.fetchAttendanceReport as any).mockImplementation(
      (_wsId: string, _start: string, _end: string, type: string) => {
        if (type === "daily") {
          return Promise.resolve({ data: [mockDailyRow], total: 1 });
        }
        return Promise.resolve({
          data: [{ employee_id: "emp-1", employee_name: "John Doe", total_std_units: 440, total_ot_units: 40, total_leave_days: 0 }],
          total: 1
        });
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
      const calls = (api.fetchAttendanceReport as any).mock.calls;
      expect(calls.length).toBeGreaterThan(0);
      expect(calls[0][3]).toBe("summary");
    });

    const dailyTab = screen.queryByText(/Daily Details|Chi tiết hàng ngày/i);
    expect(dailyTab).toBeNull();
  });

  it("renders a search input that is autofocused", async () => {
    renderWithProviders(<ReportsPage />);
    await screen.findByText(/John Doe/i);

    // Search input should exist
    const searchInput = screen.getByRole("textbox");
    expect(searchInput).toBeDefined();
    // Should have autoFocus
    expect(searchInput).toHaveFocus();
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
    (api.fetchAttendanceReport as any).mockImplementation((_wsId: string, _start: string, _end: string, type: string) => {
      if (type === "daily") return Promise.resolve({ data: [mockDailyRow], total: 1 });
      return Promise.resolve({ data: [{ employee_id: "emp-1", employee_name: "John Doe", total_std_units: 440, total_ot_units: 40, total_leave_days: 0 }], total: 1 });
    });

    renderWithProviders(<ReportsPage />);
    const employeeRow = await screen.findByText(/John Doe/i);

    // Type into search
    const searchInput = screen.getByRole("textbox") as HTMLInputElement;
    await user.type(searchInput, "Test");

    // Test auto-select on focus
    searchInput.blur();
    expect(searchInput.selectionStart).toBe(searchInput.selectionEnd);
    await user.click(searchInput);
    expect(searchInput.selectionStart).toBe(0);
    expect(searchInput.selectionEnd).toBe(4); // length of "Test"

    // Open drawer
    await user.click(employeeRow);
    const sheetPanel = await screen.findByTestId("sheet-panel");
    expect(sheetPanel).toBeInTheDocument();

    // Ensure search input lost focus
    expect(searchInput).not.toHaveFocus();

    // Close drawer
    const xIcon = sheetPanel.querySelector('svg.lucide-x');
    const closeBtn = xIcon?.closest('button');
    await user.click(closeBtn!);

    // wait for the 100ms timeout in focusSearch
    await new Promise(r => setTimeout(r, 150));

    // search should be focused again
    expect(searchInput).toHaveFocus();
    // and its text should be auto-selected because of the onFocus handler
    expect(searchInput.selectionStart).toBe(0);
    expect(searchInput.selectionEnd).toBe(4);
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
    (api.fetchAttendanceReport as any).mockImplementation((_wsId: string, _start: string, _end: string, type: string) => {
      if (type === "daily") return Promise.resolve({ data: [mockDailyRow], total: 1 });
      return Promise.resolve({ data: [{ employee_id: "emp-1", employee_name: "John Doe", total_std_units: 440, total_ot_units: 40, total_leave_days: 0 }], total: 1 });
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

    // We should see the daily report data because the drawer is open
    const rawLogsToggle = await screen.findByRole("button", { name: /Nhật ký thô/i });
    expect(rawLogsToggle).toBeInTheDocument();
  });

  it("renders cross-navigation link to raw logs with preserved date range", async () => {
    renderWithProviders(<ReportsPage />, ["/reports?from=2026-03-01&to=2026-03-31"]);
    // Using string matching that is case-insensitive and covers both EN and VI since i18n isn't mocked the same way
    const link = await screen.findByRole("link", { name: /View Raw Logs|Xem nhật ký thô|REPORTS\.VIEW_RAW_LOGS/i });
    expect(link).toBeDefined();
    expect(link.getAttribute("href")).toBe("/attendance-logs?from=2026-03-01&to=2026-03-31");
  });
});
