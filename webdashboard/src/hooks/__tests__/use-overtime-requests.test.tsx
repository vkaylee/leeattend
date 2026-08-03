import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach, type Mock } from "vitest";

import {
  useOvertimeRequests,
  useOvertimeRequest,
  useCreateOvertimeRequest,
  useSubmitOvertimeRequest,
  useCancelOvertimeRequest,
  useEmployeeConsent,
  useMyOtAssignments,
  useWorkspaceQuotaSummary,
  useEmployeeQuota,
  overtimeRequestKeys,
} from "../use-overtime-requests";

import { useWorkspace } from "@/hooks/use-workspace";

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

const WS_ID = "ws-test-123";
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
    defaultOptions: { queries: { retry: false } },
  });
  const Wrapper = ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
  Wrapper.displayName = "QueryClientWrapper";
  return Wrapper;
}

describe("overtimeRequestKeys", () => {
  it("generates stable query keys", () => {
    expect(overtimeRequestKeys.all).toEqual(["overtime-requests"]);
    expect(overtimeRequestKeys.lists(WS_ID)).toEqual(["overtime-requests", WS_ID, "list"]);
    expect(overtimeRequestKeys.list(WS_ID, { status: "pending_approval" })).toEqual([
      "overtime-requests",
      WS_ID,
      "list",
      { params: { status: "pending_approval" } },
    ]);
    expect(overtimeRequestKeys.detail(WS_ID, "ot-1")).toEqual([
      "overtime-requests",
      "detail",
      WS_ID,
      "ot-1",
    ]);
    expect(overtimeRequestKeys.assignments("emp-1")).toEqual([
      "overtime-requests",
      "assignments",
      "emp-1",
    ]);
    expect(overtimeRequestKeys.quotaSummaries(WS_ID, "2026-08")).toEqual([
      "overtime-quotas",
      WS_ID,
      "summary",
      "2026-08",
    ]);
    expect(overtimeRequestKeys.quotaDetail(WS_ID, "emp-1", "2026-08-10")).toEqual([
      "overtime-quotas",
      WS_ID,
      "emp-1",
      "2026-08-10",
    ]);
  });
});

describe("useOvertimeRequests", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUseWorkspace.mockReturnValue(withWorkspace);
    (api.listRequests as Mock).mockResolvedValue({ data: [MOCK_REQUEST], total: 1, page: 1, limit: 20 });
  });

  it("fetches the paginated request list with the active workspace and params", async () => {
    const params = { status: "pending_approval" };
    const { result } = renderHook(() => useOvertimeRequests(params), { wrapper: createWrapper() });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    expect(api.listRequests).toHaveBeenCalledWith(WS_ID, params);
    expect(result.current.data?.data).toHaveLength(1);
  });

  it("does not fetch without a workspace", () => {
    mockUseWorkspace.mockReturnValue(withoutWorkspace);

    const { result } = renderHook(() => useOvertimeRequests(), { wrapper: createWrapper() });

    expect(result.current.isFetching).toBe(false);
    expect(api.listRequests).not.toHaveBeenCalled();
  });
});

describe("useOvertimeRequest", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUseWorkspace.mockReturnValue(withWorkspace);
    (api.getRequest as Mock).mockResolvedValue({ ...MOCK_REQUEST, approval_history: [] });
  });

  it("fetches a single request by id", async () => {
    const { result } = renderHook(() => useOvertimeRequest("ot-1"), { wrapper: createWrapper() });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    expect(api.getRequest).toHaveBeenCalledWith(WS_ID, "ot-1");
    expect(result.current.data?.id).toBe("ot-1");
  });

  it("does not fetch without a request id", () => {
    const { result } = renderHook(() => useOvertimeRequest(""), { wrapper: createWrapper() });

    expect(result.current.isFetching).toBe(false);
    expect(api.getRequest).not.toHaveBeenCalled();
  });
});

describe("useCreateOvertimeRequest", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUseWorkspace.mockReturnValue(withWorkspace);
    (api.createRequest as Mock).mockResolvedValue(MOCK_REQUEST);
  });

  it("creates a request and invalidates all overtime queries", async () => {
    const { result } = renderHook(() => useCreateOvertimeRequest(), { wrapper: createWrapper() });
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    const payload = {
      employees: ["emp-1"],
      planned_date: "2026-08-10",
      planned_minutes: 120,
      reason: "Release",
    };
    await act(async () => {
      await result.current.mutateAsync(payload);
    });

    expect(api.createRequest).toHaveBeenCalledWith(WS_ID, payload);
    await waitFor(() => {
      expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: overtimeRequestKeys.all });
    });
  });

  it("rejects when no workspace is active", async () => {
    mockUseWorkspace.mockReturnValue(withoutWorkspace);

    const { result } = renderHook(() => useCreateOvertimeRequest(), { wrapper: createWrapper() });

    await expect(
      result.current.mutateAsync({ employees: [], planned_date: "x", planned_minutes: 60, reason: "r" }),
    ).rejects.toThrow("No workspace ID");
  });
});

describe("useSubmitOvertimeRequest", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUseWorkspace.mockReturnValue(withWorkspace);
    (api.submitRequest as Mock).mockResolvedValue(undefined);
  });

  it("submits a request and invalidates the list plus its detail", async () => {
    const { result } = renderHook(() => useSubmitOvertimeRequest(), { wrapper: createWrapper() });
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    await act(async () => {
      await result.current.mutateAsync("ot-1");
    });

    expect(api.submitRequest).toHaveBeenCalledWith(WS_ID, "ot-1");
    await waitFor(() => {
      expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: overtimeRequestKeys.all });
      expect(invalidateSpy).toHaveBeenCalledWith({
        queryKey: overtimeRequestKeys.detail(WS_ID, "ot-1"),
      });
    });
  });

  it("rejects when no workspace is active", async () => {
    mockUseWorkspace.mockReturnValue(withoutWorkspace);

    const { result } = renderHook(() => useSubmitOvertimeRequest(), { wrapper: createWrapper() });

    await expect(result.current.mutateAsync("ot-1")).rejects.toThrow("No workspace ID");
  });
});

describe("useCancelOvertimeRequest", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUseWorkspace.mockReturnValue(withWorkspace);
    (api.cancelRequest as Mock).mockResolvedValue(undefined);
  });

  it("cancels a request and invalidates the list plus its detail", async () => {
    const { result } = renderHook(() => useCancelOvertimeRequest(), { wrapper: createWrapper() });
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    await act(async () => {
      await result.current.mutateAsync("ot-1");
    });

    expect(api.cancelRequest).toHaveBeenCalledWith(WS_ID, "ot-1");
    await waitFor(() => {
      expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: overtimeRequestKeys.all });
      expect(invalidateSpy).toHaveBeenCalledWith({
        queryKey: overtimeRequestKeys.detail(WS_ID, "ot-1"),
      });
    });
  });

  it("rejects when no workspace is active", async () => {
    mockUseWorkspace.mockReturnValue(withoutWorkspace);

    const { result } = renderHook(() => useCancelOvertimeRequest(), { wrapper: createWrapper() });

    await expect(result.current.mutateAsync("ot-1")).rejects.toThrow("No workspace ID");
  });
});

describe("useEmployeeConsent", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUseWorkspace.mockReturnValue(withWorkspace);
    (api.employeeConsent as Mock).mockResolvedValue(undefined);
  });

  it("records consent and invalidates lists, detail and assignments", async () => {
    const { result } = renderHook(() => useEmployeeConsent(), { wrapper: createWrapper() });
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    await act(async () => {
      await result.current.mutateAsync({
        id: "ot-1",
        employment_id: "emp-1",
        status: "accepted",
      });
    });

    expect(api.employeeConsent).toHaveBeenCalledWith(WS_ID, "ot-1", {
      employment_id: "emp-1",
      status: "accepted",
    });
    await waitFor(() => {
      expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: overtimeRequestKeys.all });
      expect(invalidateSpy).toHaveBeenCalledWith({
        queryKey: overtimeRequestKeys.detail(WS_ID, "ot-1"),
      });
      expect(invalidateSpy).toHaveBeenCalledWith({
        queryKey: overtimeRequestKeys.assignments("emp-1"),
      });
    });
  });

  it("passes the decline reason through to the API", async () => {
    const { result } = renderHook(() => useEmployeeConsent(), { wrapper: createWrapper() });

    await act(async () => {
      await result.current.mutateAsync({
        id: "ot-1",
        employment_id: "emp-1",
        status: "declined",
        decline_reason: "Sick",
      });
    });

    expect(api.employeeConsent).toHaveBeenCalledWith(WS_ID, "ot-1", {
      employment_id: "emp-1",
      status: "declined",
      decline_reason: "Sick",
    });
  });

  it("rejects when no workspace is active", async () => {
    mockUseWorkspace.mockReturnValue(withoutWorkspace);

    const { result } = renderHook(() => useEmployeeConsent(), { wrapper: createWrapper() });

    await expect(
      result.current.mutateAsync({ id: "ot-1", employment_id: "emp-1", status: "accepted" }),
    ).rejects.toThrow("No workspace ID");
  });
});

describe("useMyOtAssignments", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    (api.getMyAssignments as Mock).mockResolvedValue([{ id: "assign-1", status: "pending_approval" }]);
  });

  it("fetches the caller's overtime assignments", async () => {
    const { result } = renderHook(() => useMyOtAssignments("emp-1"), { wrapper: createWrapper() });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    expect(api.getMyAssignments).toHaveBeenCalledWith("emp-1");
    expect(result.current.data).toHaveLength(1);
  });

  it("does not fetch without an employment id", () => {
    const { result } = renderHook(() => useMyOtAssignments(), { wrapper: createWrapper() });

    expect(result.current.isFetching).toBe(false);
    expect(api.getMyAssignments).not.toHaveBeenCalled();
  });
});

describe("useWorkspaceQuotaSummary", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUseWorkspace.mockReturnValue(withWorkspace);
    (api.getWorkspaceQuotaSummary as Mock).mockResolvedValue([
      { employee_id: "emp-1", monthly_used: 5, monthly_limit: 40, yearly_used: 12, yearly_limit: 200 },
    ]);
  });

  it("fetches the workspace quota summary for the given month", async () => {
    const { result } = renderHook(() => useWorkspaceQuotaSummary("2026-08"), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    expect(api.getWorkspaceQuotaSummary).toHaveBeenCalledWith(WS_ID, "2026-08");
    expect(result.current.data).toHaveLength(1);
  });

  it("does not fetch without a month", () => {
    const { result } = renderHook(() => useWorkspaceQuotaSummary(""), { wrapper: createWrapper() });

    expect(result.current.isFetching).toBe(false);
    expect(api.getWorkspaceQuotaSummary).not.toHaveBeenCalled();
  });
});

describe("useEmployeeQuota", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUseWorkspace.mockReturnValue(withWorkspace);
    (api.getEmployeeQuota as Mock).mockResolvedValue({
      employee_id: "emp-1",
      monthly_used: 5,
      monthly_limit: 40,
      yearly_used: 12,
      yearly_limit: 200,
    });
  });

  it("fetches an employee's quota for a specific date", async () => {
    const { result } = renderHook(() => useEmployeeQuota("emp-1", "2026-08-10"), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    expect(api.getEmployeeQuota).toHaveBeenCalledWith(WS_ID, "emp-1", "2026-08-10");
    expect(result.current.data?.employee_id).toBe("emp-1");
  });

  it("does not fetch without an employee id or date", () => {
    const { result } = renderHook(() => useEmployeeQuota("", "2026-08-10"), {
      wrapper: createWrapper(),
    });

    expect(result.current.isFetching).toBe(false);
    expect(api.getEmployeeQuota).not.toHaveBeenCalled();
  });
});
