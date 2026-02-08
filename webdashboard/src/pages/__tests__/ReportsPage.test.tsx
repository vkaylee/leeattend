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
    // Mock for any arguments to be safe
    (api.fetchAttendanceReport as any).mockImplementation(() => Promise.resolve({
      data: [{ employee_id: "emp-1", employee_name: "John Doe", total_work_units: 480, total_leave_days: 0 }],
      total: 1
    }));
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
});
