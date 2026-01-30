import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import ReportsPage from "../ReportsPage";
import { BrowserRouter } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ToastProvider } from "../../components/ui/Toast";
import * as api from "../../lib/api";
import React from "react";

// Mock the API library
vi.mock("../../lib/api", () => ({
  fetchAttendanceReport: vi.fn(),
  getCurrentWorkspaceId: vi.fn(),
  fetchWorkspace: vi.fn(),
  reprocessAttendance: vi.fn(),
}));

// Mock DebugContext
vi.mock("@/context/DebugContext", () => ({
  useDebug: () => ({ isEnabled: false }),
}));

// Override i18n
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: {
      changeLanguage: () => Promise.resolve(),
      language: 'en',
    },
  }),
}));

describe("ReportsPage", () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    vi.clearAllMocks();
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
    (api.fetchAttendanceReport as any).mockResolvedValue({
      data: [{ employee_id: "emp-1", employee_name: "John Doe", total_work_units: 480, total_leave_days: 0 }],
      total: 1
    });
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
    expect(await screen.findByText("John Doe")).toBeDefined();
    // In ReportsPage, filterMode defaults to "last_month"
    const lastMonthBtn = screen.getByText(/common.filters.last_month/i);
    expect(lastMonthBtn).toBeDefined();
  });

  it("changes date range when clicking 'Tháng trước'", async () => {
    const user = userEvent.setup();
    renderWithProviders(<ReportsPage />);
    const lastMonthBtn = screen.getByText(/common.filters.last_month/i);
    await user.click(lastMonthBtn);
    expect(api.fetchAttendanceReport).toHaveBeenCalled();
  });

  it("shows custom date inputs when clicking 'Tùy chỉnh'", async () => {
    const user = userEvent.setup();
    renderWithProviders(<ReportsPage />);
    const customBtn = screen.getByText(/common.filters.custom/i);
    await user.click(customBtn);
    expect(await screen.findByText(/common.from_date/i)).toBeDefined();
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
