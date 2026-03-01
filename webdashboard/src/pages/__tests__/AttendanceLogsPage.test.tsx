import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import AttendanceLogsPage from "../AttendanceLogsPage";
import { MemoryRouter } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ToastProvider } from "../../components/ui/Toast";
import * as api from "../../lib/api";
import React from "react";

// Mock the API library
vi.mock("../../lib/api", () => ({
  fetchRawAttendanceLogs: vi.fn(),
  fetchEmployees: vi.fn(),
  fetchWorkspace: vi.fn(),
  getCurrentWorkspaceId: vi.fn(),
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

describe("AttendanceLogsPage", () => {
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
    (api.fetchWorkspace as any).mockResolvedValue({ id: "test-ws-id", name: "Test Workspace" });
    (api.fetchEmployees as any).mockResolvedValue({
      data: [
        { id: "emp-1", full_name: "John Doe", employee_id_number: "EMP001", workspace_id: "ws-1", is_active: true, hire_date: "2025-01-01", created_at: "2025-01-01T00:00:00Z", updated_at: "2025-01-01T00:00:00Z" }
      ],
      total: 1,
      page: 1,
      limit: 50,
    });
    (api.fetchRawAttendanceLogs as any).mockResolvedValue({
      data: [
        { id: "log-1", employee_id: "emp-1", employee_name: "John Doe", punch_time: new Date().toISOString(), source: "DEVICE", device_id: "dev-1" }
      ],
      total: 1,
      page: 1,
      page_size: 50
    });
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

  it("renders the page and logs", async () => {
    renderWithProviders(<AttendanceLogsPage />);
    expect(await screen.findByText(/John Doe/i)).toBeDefined();
  });

  it("renders cross-navigation link with preserved date range", async () => {
    renderWithProviders(<AttendanceLogsPage />, ["/attendance-logs?from=2026-01-01&to=2026-01-31"]);
    const link = await screen.findByRole("link", { name: /View Summary Report|attendance_logs.view_reports/i });
    expect(link).toBeDefined();
    expect(link.getAttribute("href")).toBe("/reports?from=2026-01-01&to=2026-01-31");
  });

  it("shows skeleton loading state when fetching data", () => {
    // Override the mock to simulate loading state (slow promise)
    (api.fetchRawAttendanceLogs as any).mockImplementation(() => new Promise(() => { }));

    renderWithProviders(<AttendanceLogsPage />);

    // Check if skeleton rows are rendered (we used animate-pulse class)
    const skeletons = document.querySelectorAll('.animate-pulse');
    expect(skeletons.length).toBeGreaterThan(0);
  });

  it("shows beautiful empty state when no data is returned", async () => {
    // Override the mock to return empty data
    (api.fetchRawAttendanceLogs as any).mockResolvedValue({
      data: [],
      total: 0,
      page: 1,
      page_size: 50
    });

    renderWithProviders(<AttendanceLogsPage />);

    // Check for the new empty state text
    const emptyStateTitle = await screen.findByText(/No Attendance Logs Found|attendance_logs.messages.no_data/i);
    expect(emptyStateTitle).toBeDefined();

    // Check for the helpful subtext we added
    const emptyStateText = await screen.findByText(/Try adjusting your date range filter/i);
    expect(emptyStateText).toBeDefined();
  });
});
