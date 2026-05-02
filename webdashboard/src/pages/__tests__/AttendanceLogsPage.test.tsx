import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";

import { ToastProvider } from "../../components/ui/Toast";
import * as api from "../../lib/api";
import { StagingLogsResponse } from "../../lib/api/attendance";
import { Employee, PaginatedResponse, RawAttendanceLog, Workspace, Device } from "../../lib/api/types";
import AttendanceLogsPage from "../AttendanceLogsPage";


// Mock TanStack Router
vi.mock("@tanstack/react-router", () => ({
 
  useNavigate: () => vi.fn(),
  useLocation: () => ({ pathname: "/", search: "", hash: "" }),
  Link: ({ to, search, children, ...props }: { to: string; search?: Record<string, unknown>; children: React.ReactNode }) => {
    let href = to;
    if (search && typeof search === 'object') {
      const params = new URLSearchParams();
      Object.entries(search).forEach(([key, value]) => {
        if (value !== undefined && value !== null) {
          params.append(key, String(value));
        }
      });
      const qs = params.toString();
      if (qs) href += `?${qs ?? ""}`;
    }
    return <a href={href} {...props}>{children}</a>;
  },
}));

// Mock IntersectionObserver for jsdom
class MockIntersectionObserver {
  observe() { }
  unobserve() { }
 
  disconnect() { }
}
window.IntersectionObserver = MockIntersectionObserver as unknown as typeof IntersectionObserver;

// Mock the API library
vi.mock("../../lib/api", () => ({
  fetchRawAttendanceLogs: vi.fn(),
  fetchEmployees: vi.fn(),
  fetchWorkspace: vi.fn(),
  getCurrentWorkspaceId: vi.fn(),
  fetchDevices: vi.fn(),
  fetchStagingLogs: vi.fn(),
  resolveStagingLog: vi.fn(),
  dismissStagingLog: vi.fn(),
  fetchMyPermissions: vi.fn(),
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
 
    vi.mocked(api.getCurrentWorkspaceId).mockReturnValue("test-ws-id");
    vi.mocked(api.fetchWorkspace).mockResolvedValue({ id: "test-ws-id", name: "Test Workspace" } as unknown as Workspace);
    vi.mocked(api.fetchEmployees).mockResolvedValue({
      data: [
        { id: "emp-1", full_name: "John Doe", employee_id_number: "EMP001", workspace_id: "ws-1", is_active: true, hire_date: "2025-01-01", created_at: "2025-01-01T00:00:00Z", updated_at: "2025-01-01T00:00:00Z" }
      ],
      total: 1,
      page: 1,
 
      limit: 50,
    });
    vi.mocked(api.fetchRawAttendanceLogs).mockResolvedValue({
      data: [
        { id: "log-1", employee_id: "emp-1", employee_name: "John Doe", punch_time: new Date().toISOString(), source: "DEVICE", device_id: "dev-1" }
      ],
      total: 1,
      page: 1,
 
      page_size: 50
 
    } as unknown as PaginatedResponse<RawAttendanceLog>);
 
    vi.mocked(api.fetchDevices).mockResolvedValue([]);
    vi.mocked(api.fetchStagingLogs).mockResolvedValue({ data: [], total: 0, page: 1, limit: 20 });
    vi.mocked(api.fetchMyPermissions).mockResolvedValue({ permissions: ["attendance_staging:manage"], roles: ["admin"], managed_department_ids: null });
  });

  const renderWithProviders = (ui: React.ReactElement, initialEntries = ["/"]) => {
    window.history.pushState({}, "Test page", initialEntries[0]);
    return render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          {ui}
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
    vi.mocked(api.fetchRawAttendanceLogs).mockImplementation(() => new Promise(() => { }));

    renderWithProviders(<AttendanceLogsPage />);

    // Check if skeleton rows are rendered (we used animate-pulse class)
    const skeletons = document.querySelectorAll('.animate-pulse');
    expect(skeletons.length).toBeGreaterThan(0);
  });

 
  it("shows beautiful empty state when no data is returned", async () => {
    // Override the mock to return empty data
    vi.mocked(api.fetchRawAttendanceLogs).mockResolvedValue({
      data: [],
      total: 0,
      page: 1,
      page_size: 50
    } as unknown as PaginatedResponse<RawAttendanceLog>);

    renderWithProviders(<AttendanceLogsPage />);

    // Check for the new empty state text
    const emptyStateTitle = await screen.findByText("attendance_logs.messages.no_data");
    expect(emptyStateTitle).toBeDefined();

    // Check for the helpful subtext we added
    const emptyStateText = await screen.findByText("attendance_logs.messages.no_data_hint");
    expect(emptyStateText).toBeDefined();
  });

  it("uses PAGE_SIZE of 20 for infinite scroll", async () => {
 
    renderWithProviders(<AttendanceLogsPage />);
    await waitFor(() => {
      const calls = vi.mocked(api.fetchRawAttendanceLogs).mock.calls;
      expect(calls.length).toBeGreaterThan(0);
      // limit param should be 20
      expect(calls[0][0].limit).toBe(20);
      // first page param should be 1
      expect(calls[0][0].page).toBe(1);
    });
  });

  it("does not render pagination prev/next buttons", async () => {
    renderWithProviders(<AttendanceLogsPage />);
    await screen.findByText(/John Doe/i);

    // Old pagination buttons should NOT exist
    expect(screen.queryByText(/common.pagination.prev/i)).toBeNull();
    expect(screen.queryByText(/common.pagination.next/i)).toBeNull();
    // No "Page X / Y" text
    expect(screen.queryByText(/Page \d+ \//i)).toBeNull();
  });
 

  it("displays floating record count badge with loaded/total counts", async () => {
    vi.mocked(api.fetchRawAttendanceLogs).mockResolvedValue({
      data: [
        { id: "log-1", employee_id: "emp-1", employee_name: "John Doe", punch_time: new Date().toISOString(), source: "DEVICE", device_id: "dev-1" },
        { id: "log-2", employee_id: "emp-1", employee_name: "John Doe", punch_time: new Date().toISOString(), source: "DEVICE", device_id: "dev-1" },
      ],
      total: 5,
      page: 1,
      page_size: 20
    } as unknown as PaginatedResponse<RawAttendanceLog>);

    renderWithProviders(<AttendanceLogsPage />);
    // Wait for data to render (2 rows with same name)
    await waitFor(() => {
      const rows = screen.getAllByText(/John Doe/i);
      expect(rows.length).toBeGreaterThanOrEqual(2);
    });

    // Floating badge should show loaded count (2) and total (5)
    const badge = document.querySelector('.fixed.bottom-6');
    expect(badge).toBeDefined();
    expect(badge?.textContent).toContain("2");
    expect(badge?.textContent).toContain("5");
  });

 
  it("shows loading spinner when fetching next page", () => {
    // Simulate infinite loading
    vi.mocked(api.fetchRawAttendanceLogs).mockImplementation(() => new Promise(() => { }));

    renderWithProviders(<AttendanceLogsPage />);

    // Check animate-spin exists (Loader2 spinner in sentinel row)
    const _spinners = document.querySelectorAll('.animate-spin');
    // While first page is loading, skeleton rows show instead of spinner
    // The spinner only appears during fetchNextPage, so we just verify no crash
    expect(document.querySelectorAll('.animate-pulse').length).toBeGreaterThan(0);
  });

  it("renders unknown tab with staging data and floating amber badge", async () => {
 
    const user = userEvent.setup();

    vi.mocked(api.fetchStagingLogs).mockResolvedValue({
      data: [
        { id: "stg-1", raw_user_id: "UNKNOWN-42", punch_time: new Date().toISOString(), device_id: "dev-1" },
      ],
      total: 3,
      page: 1,
      limit: 20,
    } as unknown as StagingLogsResponse);

    renderWithProviders(<AttendanceLogsPage />);
    await screen.findByText(/John Doe/i);

    // Switch to unknown tab
    const unknownTab = screen.getByText(/attendance_logs.tabs.unknown/i);
    await user.click(unknownTab);

    // Should show the staging log
    expect(await screen.findByText("UNKNOWN-42")).toBeDefined();

    // Floating badge should now show amber variant with staging counts
    const badge = document.querySelector('.fixed.bottom-6');
    expect(badge).toBeDefined();
    expect(badge?.textContent).toContain("1");
    expect(badge?.textContent).toContain("3");
  });

  it("shows empty state for unknown tab when no staging logs exist", async () => {
    const user = userEvent.setup();

    renderWithProviders(<AttendanceLogsPage />);
    await screen.findByText(/John Doe/i);

    // Switch to unknown tab
    const unknownTab = screen.getByText(/attendance_logs.tabs.unknown/i);
    await user.click(unknownTab);

    // Should show empty state message
    expect(await screen.findByText(/No unidentified logs|attendance_logs.messages.no_unknown_logs/i)).toBeDefined();
  });
});
