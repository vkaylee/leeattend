import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, act, waitFor } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import * as api from "@/lib/api";

import { useLeaveRequests } from "../use-leave-requests";

// Mock imports
// Mock TanStack Router
vi.mock("@tanstack/react-router", () => {
  const defaultSearch = { tab: "requests", status: "all" } as Record<string, unknown>;
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

vi.mock("@/lib/api", () => ({
  fetchLeaveRequests: vi.fn().mockResolvedValue({ data: [], total: 0, page: 1, limit: 20 }),
  approveLeaveRequest: vi.fn().mockResolvedValue({}),
  fetchLeaveTypes: vi.fn().mockResolvedValue([]),
  fetchLatestLeaveImport: vi.fn().mockResolvedValue(null),
}));

vi.mock("@/hooks/use-employees", () => ({
  useAllEmployees: () => ({ employees: [] }),
}));

vi.mock("@/components/ui/toast", () => ({
  useToast: () => ({ success: vi.fn(), error: vi.fn() }),
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (k: string) => k }),
}));

// Mock localStorage
const localStorageMock = (() => {
  let store: Record<string, string> = {};
  return {
    getItem: vi.fn((key: string) => store[key] || null),
    setItem: vi.fn((key: string, value: string) => {
      store[key] = value.toString();
    }),
    removeItem: vi.fn((key: string) => {
  // eslint-disable-next-line @typescript-eslint/no-dynamic-delete
      delete store[key];
    }),
    clear: vi.fn(() => {
      store = {};
    }),
  };
})();
Object.defineProperty(window, "localStorage", { value: localStorageMock });

const createWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  const Wrapper = ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
  Wrapper.displayName = "QueryClientWrapper";
  return Wrapper;
};

describe("useLeaveRequests - Persistent Banner Dismiss", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    localStorageMock.clear();
  });

  it("does not crash when leave API omits data from a page", async () => {
    vi.mocked(api.fetchLeaveRequests).mockResolvedValue({ total: 0, page: 1, limit: 20 } as unknown as api.PaginatedResponse<api.LeaveRequest>);

    const { result } = renderHook(() => useLeaveRequests(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.requests).toEqual([]);
      expect(result.current.hasNextPage).toBe(false);
    });
  });

  it("should show import banner if job is not dismissed in localStorage", async () => {
    const mockJob = {
      id: "job-123",
      status: "completed",
      total_rows: 5,
      processed_rows: 5,
      error_log: null,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };
    (api.fetchLatestLeaveImport as any).mockResolvedValue(mockJob);

    const { result } = renderHook(() => useLeaveRequests(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.importJob).toEqual(mockJob);
    });

    expect(result.current.importBannerDismissed).toBe(false);
  });

  it("should hide import banner if job is already dismissed in localStorage", async () => {
    const mockJob = {
      id: "job-123",
      status: "completed",
      total_rows: 5,
      processed_rows: 5,
      error_log: null,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };
    (api.fetchLatestLeaveImport as any).mockResolvedValue(mockJob);
    localStorageMock.setItem("dismissed_leave_import_job", "job-123");

    const { result } = renderHook(() => useLeaveRequests(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.importJob).toEqual(mockJob);
    });

    expect(result.current.importBannerDismissed).toBe(true);
  });

  it("should persistently dismiss banner when calling setImportBannerDismissed(true)", async () => {
    const mockJob = {
      id: "job-123",
      status: "completed",
      total_rows: 5,
      processed_rows: 5,
      error_log: null,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };
    (api.fetchLatestLeaveImport as any).mockResolvedValue(mockJob);

    const { result } = renderHook(() => useLeaveRequests(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.importJob).toEqual(mockJob);
    });

    expect(result.current.importBannerDismissed).toBe(false);

    act(() => {
      result.current.setImportBannerDismissed(true);
    });

    expect(result.current.importBannerDismissed).toBe(true);
    expect(localStorageMock.setItem).toHaveBeenCalledWith(
      "dismissed_leave_import_job",
      "job-123"
    );
  });
});
