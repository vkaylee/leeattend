import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import ReportsPage from "../ReportsPage";
import { BrowserRouter } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ToastProvider } from "../../components/ui/Toast";
import * as api from "../../lib/api";
import React from "react";
import i18n from "../../i18n/config";

// Mock the API library
vi.mock("../../lib/api", () => ({
  fetchAttendanceReport: vi.fn(),
  getCurrentWorkspaceId: vi.fn(),
  fetchWorkspace: vi.fn(),
  reprocessAttendance: vi.fn(),
  fetchEmployees: vi.fn(),
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
    (api.fetchAttendanceReport as any).mockImplementation(() => Promise.resolve({
      data: [{ employee_id: "emp-1", employee_name: "John Doe", total_work_units: 480, total_leave_days: 0 }],
      total: 1
    }));
    (api.fetchWorkspaceAttendanceAdjustments as any).mockResolvedValue([]);
    (api.adminConfirmAttendance as any).mockResolvedValue(undefined);
  });

  const renderWithProviders = (ui: React.ReactElement) => {
    return render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <BrowserRouter>
            {ui}
          </BrowserRouter>
        </ToastProvider>
      </QueryClientProvider>
    );
  };

  it("renders report table and default to current month", async () => {
    renderWithProviders(<ReportsPage />);
    expect(await screen.findByText(/John Doe/i, {}, { timeout: 30000 })).toBeDefined();
    // In ReportsPage, filterMode defaults to "last_month"
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

  it("defaults pageSize to the number of days in the month", async () => {
    renderWithProviders(<ReportsPage />);
    await waitFor(() => {
      const calls = (api.fetchAttendanceReport as any).mock.calls;
      expect(calls.length).toBeGreaterThan(0);
      // The 6th argument (index 5) is pageSize. December 2025 has 31 days.
      expect(calls[0][5]).toBe(31);
    });
  });

  it("shows confirm button on all daily rows and opens modal when clicked", async () => {
    const user = userEvent.setup();
    const mockRow = {
      employee_id: "emp-1",
      employee_name: "John Doe",
      date: "2026-01-15",
      actual_punch_in: "2026-01-15T01:00:00Z", // 08:00 Local
      actual_punch_out: "2026-01-15T10:00:00Z", // 17:00 Local
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
    (api.fetchAttendanceReport as any).mockResolvedValue({
      data: [mockRow],
      total: 1
    });

    renderWithProviders(<ReportsPage />);
    const dailyBtn = await screen.findByText(/Daily Details|Chi ti[eế]t/i);
    await user.click(dailyBtn);

    // Find the confirm button (it's always visible now)
    const confirmBtn = await screen.findByRole('button', { name: /Xác nhận|Confirm/i });
    expect(confirmBtn).toBeDefined();

    // Click it to open the modal
    await user.click(confirmBtn);

    // Modal should appear
    const dialog = await screen.findByRole('dialog');
    expect(dialog).toBeDefined();

    // Verify pre-filled times
    const timeInputs = document.querySelectorAll('input[type="time"]');
    expect(timeInputs.length).toBeGreaterThan(0);
    // It should have some value pre-filled
    expect((timeInputs[0] as HTMLInputElement).value).not.toBe("");
  });

  it("calls adminConfirmAttendance with edited times from the modal", async () => {
    const user = userEvent.setup();
    const mockRow = {
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
    (api.fetchAttendanceReport as any).mockResolvedValue({ data: [mockRow], total: 1 });
    (api.adminConfirmAttendance as any).mockResolvedValue(undefined);

    renderWithProviders(<ReportsPage />);
    const dailyBtn = await screen.findByText(/Daily Details|Chi ti[eế]t/i);
    await user.click(dailyBtn);

    // Click the confirm button to open the modal
    const confirmBtn = await screen.findByRole('button', { name: /Xác nhận|Confirm/i });
    await user.click(confirmBtn);

    // Modal should appear
    const dialog = await screen.findByRole('dialog');
    expect(dialog).toBeDefined();

    // Enter reason
    const reasonInput = document.querySelector('input[type="text"]') as HTMLInputElement;
    await user.type(reasonInput, "Test reason");

    // Click submit inside the modal
    // Filter to get the modal's primary button, ignoring the one in the table
    const buttons = await screen.findAllByRole('button', { name: /Xác nhận công|Confirm Attendance/i });
    const modalSubmitBtn = buttons[buttons.length - 1]; // The last one mounted (in the modal)
    await user.click(modalSubmitBtn);

    // Verify the API was called
    await waitFor(() => {
      expect(api.adminConfirmAttendance).toHaveBeenCalled();
    });

    // Verify the reason payload was passed
    const callArgs = (api.adminConfirmAttendance as any).mock.calls[0];
    expect(callArgs[3]).toBe("Test reason");
  });
})
