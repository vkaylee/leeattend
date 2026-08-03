import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach, type Mock } from "vitest";

import {
  useOvertimeRequests,
  useOvertimeRequest,
  useMyOtAssignments,
  useWorkspaceQuotaSummary,
  useEmployeeQuota,
  useCreateOvertimeRequest,
  useSubmitOvertimeRequest,
  useCancelOvertimeRequest,
  useEmployeeConsent,
} from "../use-overtime-requests";

const mockUseWorkspace = vi.hoisted(() => vi.fn());

vi.mock("@/hooks/use-workspace", () => ({
  useWorkspace: mockUseWorkspace,
}));

const api = vi.hoisted(() => ({
  listRequests: vi.fn(),
  getRequest: vi.fn(),
  createRequest: vi.fn(),
  submitRequest: vi.fn(),
  cancelRequest: vi.fn(),
  employeeConsent: vi.fn(),
  getMyAssignments: vi.fn(),
  getWorkspaceQuotaSummary: vi.fn(),
  getEmployeeQuota: vi.fn(),
}));

vi.mock("@/lib/api/overtime-requests", () => ({
  overtimeRequestsApi: api,
}));

const WS_ID = "ws-gap-456";
const withWorkspace = { workspace: { id: WS_ID }, isLoading: false, isError: false };
const withoutWorkspace = { workspace: undefined, isLoading: false, isError: false };

const MOCK_REQUEST = {
  id: "ot-1",
  workspace_id: WS_ID,
  status: "pending_approval",
  planned_date: "2026-08-10",
  planned_minutes: 120,
  reason: "Release",
  requested_by_user_id: "user-1",
  created_at: "2026-08-01T00:00:00Z",
  updated_at: "2026-08-01T00:00:00Z",
  current_step: 1,
};

let queryClient: QueryClient;

function createWrapper() {
  queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  const Wrapper = ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
  Wrapper.displayName = "QueryClientWrapper";
  return Wrapper;
}

// ── Gap-fill: Error & pending states ─────────────────────────────────────────

describe("useOvertimeRequests – error handling", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUseWorkspace.mockReturnValue(withWorkspace);
  });

  it("propagates isError when listRequests rejects", async () => {
    (api.listRequests as Mock).mockRejectedValue(new Error("Server Down"));
    const { result } = renderHook(() => useOvertimeRequests(), { wrapper: createWrapper() });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(result.current.error?.message).toBe("Server Down");
  });
});

describe("useOvertimeRequest – detail error", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUseWorkspace.mockReturnValue(withWorkspace);
  });

  it("propagates isError when getRequest rejects", async () => {
    (api.getRequest as Mock).mockRejectedValue(new Error("Not Found"));
    const { result } = renderHook(() => useOvertimeRequest("ot-missing"), { wrapper: createWrapper() });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(result.current.error?.message).toBe("Not Found");
  });

  it("does not fetch when workspace is absent even with a valid id", () => {
    mockUseWorkspace.mockReturnValue(withoutWorkspace);

    const { result } = renderHook(() => useOvertimeRequest("ot-1"), { wrapper: createWrapper() });

    expect(result.current.isFetching).toBe(false);
    expect(api.getRequest).not.toHaveBeenCalled();
  });
});

// ── Mutation pending states ──────────────────────────────────────────────────

describe("useCreateOvertimeRequest – pending state", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUseWorkspace.mockReturnValue(withWorkspace);
  });

  it("reports isPending while creation is in flight", async () => {
    let resolveCreate: (v: unknown) => void;
    const deferred = new Promise((resolve) => { resolveCreate = resolve; });
    (api.createRequest as Mock).mockReturnValue(deferred);

    const { result } = renderHook(() => useCreateOvertimeRequest(), { wrapper: createWrapper() });

    act(() => {
      result.current.mutateAsync({
        employees: ["emp-1"],
        planned_date: "2026-08-10",
        planned_minutes: 120,
        reason: "Release",
      });
    });

    await waitFor(() => expect(result.current.isPending).toBe(true));

    await act(async () => {
      resolveCreate?.(MOCK_REQUEST);
    });
    await waitFor(() => expect(result.current.isPending).toBe(false));
  });
});

describe("useSubmitOvertimeRequest – pending state", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUseWorkspace.mockReturnValue(withWorkspace);
  });

  it("reports isPending while submission is in flight", async () => {
    let resolveSubmit: (v: unknown) => void;
    const deferred = new Promise((resolve) => { resolveSubmit = resolve; });
    (api.submitRequest as Mock).mockReturnValue(deferred);

    const { result } = renderHook(() => useSubmitOvertimeRequest(), { wrapper: createWrapper() });

    act(() => {
      result.current.mutateAsync("ot-1");
    });

    await waitFor(() => expect(result.current.isPending).toBe(true));

    await act(async () => {
      resolveSubmit?.(undefined);
    });
    await waitFor(() => expect(result.current.isPending).toBe(false));
  });
});

describe("useCancelOvertimeRequest – pending state", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUseWorkspace.mockReturnValue(withWorkspace);
  });

  it("reports isPending while cancellation is in flight", async () => {
    let resolveCancel: (v: unknown) => void;
    const deferred = new Promise((resolve) => { resolveCancel = resolve; });
    (api.cancelRequest as Mock).mockReturnValue(deferred);

    const { result } = renderHook(() => useCancelOvertimeRequest(), { wrapper: createWrapper() });

    act(() => {
      result.current.mutateAsync("ot-1");
    });

    await waitFor(() => expect(result.current.isPending).toBe(true));

    await act(async () => {
      resolveCancel?.(undefined);
    });
    await waitFor(() => expect(result.current.isPending).toBe(false));
  });
});

// ── Empty / many results ─────────────────────────────────────────────────────

describe("useOvertimeRequests – empty results", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUseWorkspace.mockReturnValue(withWorkspace);
  });

  it("returns an empty data array when no requests exist", async () => {
    (api.listRequests as Mock).mockResolvedValue({ data: [], total: 0, page: 1, limit: 20 });
    const { result } = renderHook(() => useOvertimeRequests(), { wrapper: createWrapper() });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    expect(result.current.data?.data).toHaveLength(0);
    expect(result.current.data?.total).toBe(0);
  });
});

describe("useMyOtAssignments – error handling", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("propagates isError when getMyAssignments rejects", async () => {
    (api.getMyAssignments as Mock).mockRejectedValue(new Error("Permission Denied"));

    const { result } = renderHook(() => useMyOtAssignments("emp-1"), { wrapper: createWrapper() });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(result.current.error?.message).toBe("Permission Denied");
  });

  it("returns empty array for assignments when data is empty", async () => {
    (api.getMyAssignments as Mock).mockResolvedValue([]);
    const { result } = renderHook(() => useMyOtAssignments("emp-1"), { wrapper: createWrapper() });

    await waitFor(() => expect(result.current.isLoading).toBe(false));
    expect(result.current.data).toEqual([]);
  });
});

describe("useWorkspaceQuotaSummary – error handling", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUseWorkspace.mockReturnValue(withWorkspace);
  });

  it("propagates isError when getWorkspaceQuotaSummary rejects", async () => {
    (api.getWorkspaceQuotaSummary as Mock).mockRejectedValue(new Error("No quotas configured"));

    const { result } = renderHook(() => useWorkspaceQuotaSummary("2026-08"), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(result.current.error?.message).toBe("No quotas configured");
  });

  it("returns an empty array when no quotas exist", async () => {
    (api.getWorkspaceQuotaSummary as Mock).mockResolvedValue([]);
    const { result } = renderHook(() => useWorkspaceQuotaSummary("2026-08"), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isLoading).toBe(false));
    expect(result.current.data).toEqual([]);
  });
});

describe("useEmployeeQuota – error handling", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUseWorkspace.mockReturnValue(withWorkspace);
  });

  it("propagates isError when getEmployeeQuota rejects", async () => {
    (api.getEmployeeQuota as Mock).mockRejectedValue(new Error("Employee not found"));

    const { result } = renderHook(() => useEmployeeQuota("emp-1", "2026-08-10"), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(result.current.error?.message).toBe("Employee not found");
  });

  it("does not fetch when only employeeId is missing", () => {
    const { result } = renderHook(() => useEmployeeQuota("", "2026-08-10"), {
      wrapper: createWrapper(),
    });

    expect(result.current.isFetching).toBe(false);
    expect(api.getEmployeeQuota).not.toHaveBeenCalled();
  });

  it("does not fetch when only date is missing", () => {
    const { result } = renderHook(() => useEmployeeQuota("emp-1", ""), {
      wrapper: createWrapper(),
    });

    expect(result.current.isFetching).toBe(false);
    expect(api.getEmployeeQuota).not.toHaveBeenCalled();
  });
});

// ── Mutation error propagation ──────────────────────────────────────────────

describe("useCreateOvertimeRequest – error propagation", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUseWorkspace.mockReturnValue(withWorkspace);
  });

  it("rejects when createRequest fails", async () => {
    (api.createRequest as Mock).mockRejectedValue(new Error("Conflict: duplicate request"));

    const { result } = renderHook(() => useCreateOvertimeRequest(), { wrapper: createWrapper() });

    await expect(
      result.current.mutateAsync({
        employees: ["emp-1"],
        planned_date: "2026-08-10",
        planned_minutes: 60,
        reason: "Duplicate",
      }),
    ).rejects.toThrow("Conflict: duplicate request");
  });
});

describe("useSubmitOvertimeRequest – error propagation", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUseWorkspace.mockReturnValue(withWorkspace);
  });

  it("rejects when submitRequest fails", async () => {
    (api.submitRequest as Mock).mockRejectedValue(new Error("Already submitted"));

    const { result } = renderHook(() => useSubmitOvertimeRequest(), { wrapper: createWrapper() });

    await expect(result.current.mutateAsync("ot-1")).rejects.toThrow("Already submitted");
  });
});

describe("useCancelOvertimeRequest – error propagation", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUseWorkspace.mockReturnValue(withWorkspace);
  });

  it("rejects when cancelRequest fails", async () => {
    (api.cancelRequest as Mock).mockRejectedValue(new Error("Cannot cancel approved request"));

    const { result } = renderHook(() => useCancelOvertimeRequest(), { wrapper: createWrapper() });

    await expect(result.current.mutateAsync("ot-1")).rejects.toThrow("Cannot cancel approved request");
  });
});

describe("useEmployeeConsent – error propagation", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUseWorkspace.mockReturnValue(withWorkspace);
  });

  it("rejects when employeeConsent fails", async () => {
    (api.employeeConsent as Mock).mockRejectedValue(new Error("Already responded"));

    const { result } = renderHook(() => useEmployeeConsent(), { wrapper: createWrapper() });

    await expect(
      result.current.mutateAsync({
        id: "ot-1",
        employment_id: "emp-1",
        status: "accepted",
      }),
    ).rejects.toThrow("Already responded");
  });

  it("reports isPending while consent submission is in flight", async () => {
    let resolveConsent: (v: unknown) => void;
    const deferred = new Promise((resolve) => { resolveConsent = resolve; });
    (api.employeeConsent as Mock).mockReturnValue(deferred);

    const { result } = renderHook(() => useEmployeeConsent(), { wrapper: createWrapper() });

    act(() => {
      result.current.mutateAsync({ id: "ot-1", employment_id: "emp-1", status: "accepted" });
    });

    await waitFor(() => expect(result.current.isPending).toBe(true));

    await act(async () => {
      resolveConsent?.(undefined);
    });
    await waitFor(() => expect(result.current.isPending).toBe(false));
  });
});
