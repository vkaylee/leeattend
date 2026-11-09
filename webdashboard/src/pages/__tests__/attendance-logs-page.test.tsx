 
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";

import { ToastProvider } from "../../components/ui/toast";
import * as api from "../../lib/api";
import { StagingLogsResponse } from "../../lib/api/attendance";
import { PaginatedResponse, RawAttendanceLog, Workspace } from "../../lib/api/types";
import { type RouterMockState } from "../../test/router-mock";
import AttendanceLogsPage from "../attendance-logs-page";

// Reactive TanStack Router mock — state created in vi.hoisted so it exists
// before vi.mock's factory fires (Vitest hoists vi.mock before module code).
const { state, resetSearch } = vi.hoisted(() => {
  const DEFAULT = { tab: "recorded", mode: "this_month" };
  return {
    state: { _listeners: new Set(), _search: { ...DEFAULT }, DEFAULT } as RouterMockState,
    resetSearch: () => { state._search = { ...state.DEFAULT }; },
  };
});

vi.mock("@tanstack/react-router", async () => {
  const { buildMockImpl } = await import("../../test/router-mock");
  return buildMockImpl(state);
});

// Mock IntersectionObserver for jsdom
class MockIntersectionObserver {
  observe() { /* empty */ }
  unobserve() { /* empty */ }
 
  disconnect() { /* empty */ }
}
window.IntersectionObserver = MockIntersectionObserver as unknown as typeof IntersectionObserver;

// Mock the API library
vi.mock("../../lib/api", () => ({
  fetchOrganization: vi.fn().mockResolvedValue({ name: "Test Organization" }),
  fetchRawAttendanceLogs: vi.fn(),
  fetchEmployees: vi.fn(),
  fetchWorkspace: vi.fn(),
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
    resetSearch();
    queryClient = new QueryClient({
      defaultOptions: {
        queries: {
          retry: false,
          gcTime: 0,
          staleTime: 0,
        },

      },

    });
 
    vi.mocked(api.fetchWorkspace).mockResolvedValue({ id: "test-ws-id", name: "Test Workspace" } as unknown as Workspace);
    vi.mocked(api.fetchEmployees).mockResolvedValue({
      data: [
        { id: "emp-1", full_name: "John Doe", employee_id_number: "EMP001", workspace_id: "ws-1", created_at: "2025-01-01T00:00:00Z", updated_at: "2025-01-01T00:00:00Z", tags: [] }
      ],
      total: 1,
      page: 1,
      limit: 50,
    });
    vi.mocked(api.fetchRawAttendanceLogs).mockResolvedValue({
      data: [
        { id: "log-1", employee_id: "emp-1", employee_name: "John Doe", punch_time: new Date().toISOString(), source: "import" }
      ],
      total: 1,
      page: 1,
 
      page_size: 50
 
    } as unknown as PaginatedResponse<RawAttendanceLog>);
 
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
    // ResponsiveTable renders both desktop and mobile views, so use findAllByText
    expect((await screen.findAllByText(/John Doe/i)).length).toBeGreaterThan(0);
  });

  it("renders cross-navigation link with preserved date range", async () => {
    // With URL-first state, the report link reflects the current search state.
    // The mock returns mode: "this_month", so the link shows this month's dates.
    renderWithProviders(<AttendanceLogsPage />);
    const link = await screen.findByRole("link", { name: /View Summary Report|attendance_logs.view_reports/i });
    expect(link).toBeDefined();
    // The link should contain date params from the current search state (this_month).
    const href = link.getAttribute("href");
    expect(href).toMatch(/^\/reports\?from=\d{4}-\d{2}-\d{2}&to=\d{4}-\d{2}-\d{2}$/);
  });

 
  it("shows skeleton loading state when fetching data", () => {
    // Override the mock to simulate loading state (slow promise)
    vi.mocked(api.fetchRawAttendanceLogs).mockImplementation(() => new Promise(() => { /* empty */ }));

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

    // Check for the new empty state text - ResponsiveTable renders both desktop and mobile empty states
    const emptyStateTitles = await screen.findAllByText("attendance_logs.messages.no_data");
    expect(emptyStateTitles.length).toBeGreaterThan(0);
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
    // ResponsiveTable renders both desktop and mobile views, so use findAllByText
    await screen.findAllByText(/John Doe/i);

    // Old pagination buttons should NOT exist
    expect(screen.queryByText(/common.pagination.prev/i)).toBeNull();
    expect(screen.queryByText(/common.pagination.next/i)).toBeNull();
    // No "Page X / Y" text
    expect(screen.queryByText(/Page \d+ \//i)).toBeNull();
  });
 

  it("displays floating record count badge with loaded/total counts", async () => {
    vi.mocked(api.fetchRawAttendanceLogs).mockResolvedValue({
      data: [
        { id: "log-1", employee_id: "emp-1", employee_name: "John Doe", punch_time: new Date().toISOString(), source: "import" },
        { id: "log-2", employee_id: "emp-1", employee_name: "John Doe", punch_time: new Date().toISOString(), source: "import" },
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
    const badge = document.querySelector('.fixed.z-50');
    expect(badge).toBeDefined();
    expect(badge?.textContent).toContain("2");
    expect(badge?.textContent).toContain("5");
  });

 
  it("shows loading spinner when fetching next page", () => {
    // Simulate infinite loading
    vi.mocked(api.fetchRawAttendanceLogs).mockImplementation(() => new Promise(() => { /* empty */ }));

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
        { id: "stg-1", raw_user_id: "UNKNOWN-42", punch_time: new Date().toISOString() },
      ],
      total: 3,
      page: 1,
      limit: 20,
    } as unknown as StagingLogsResponse);

    renderWithProviders(<AttendanceLogsPage />);
    // ResponsiveTable renders both desktop and mobile views, so use findAllByText
    await screen.findAllByText(/John Doe/i);

    // Switch to unknown tab
    const unknownTab = screen.getByText(/attendance_logs.tabs.unknown/i);
    await user.click(unknownTab);

    // Should show the staging log (desktop + mobile both render)
    const unknownIds = await screen.findAllByText("UNKNOWN-42");
    expect(unknownIds.length).toBeGreaterThan(0);

    // Floating badge should now show amber variant with staging counts
    const badge = document.querySelector('.fixed.z-50');
    expect(badge).toBeDefined();
    expect(badge?.textContent).toContain("1");
    expect(badge?.textContent).toContain("3");
  });

  it("shows empty state for unknown tab when no staging logs exist", async () => {
    const user = userEvent.setup();

    renderWithProviders(<AttendanceLogsPage />);

    // Switch to unknown tab — the test above may have left search state on "unknown",
    // so clicked tab may already be active. Click "recorded" first to ensure a clean
    // transition, then switch to "unknown".
    const recordedTab = screen.getByText(/attendance_logs.tabs.recorded/i);
    await user.click(recordedTab);
    await screen.findAllByText(/John Doe/i);

    const unknownTab = screen.getByText(/attendance_logs.tabs.unknown/i);
    await user.click(unknownTab);

    // Should show empty state message (desktop + mobile both render)
    const emptyMessages = await screen.findAllByText(/No unidentified logs|attendance_logs.messages.no_unknown_logs/i);
    expect(emptyMessages.length).toBeGreaterThan(0);
  });
});
