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
  overtimeRequestKeys,
} from "../use-overtime-requests";

const api = vi.hoisted(() => ({
  listRequests: vi.fn(),
  getRequest: vi.fn(),
  createRequest: vi.fn(),
  submitRequest: vi.fn(),
  cancelRequest: vi.fn(),
  employeeConsent: vi.fn(),
  getMyAssignments: vi.fn(),
}));

vi.mock("@/lib/api/overtime-requests", () => ({
  overtimeRequestsApi: api,
}));

const MOCK_REQUEST = {
  id: "ot-1",
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
    expect(overtimeRequestKeys.lists()).toEqual(["overtime-requests", "list"]);
    expect(overtimeRequestKeys.list({ status: "pending_approval" })).toEqual([
      "overtime-requests",
      "list",
      { params: { status: "pending_approval" } },
    ]);
    expect(overtimeRequestKeys.detail("ot-1")).toEqual([
      "overtime-requests",
      "detail",
      "ot-1",
    ]);
    expect(overtimeRequestKeys.assignments("emp-1")).toEqual([
      "overtime-requests",
      "assignments",
      "emp-1",
    ]);
  });
});

describe("useOvertimeRequests", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    (api.listRequests as Mock).mockResolvedValue({ data: [MOCK_REQUEST], total: 1, page: 1, limit: 20 });
  });

  it("fetches the paginated request list with params", async () => {
    const params = { status: "pending_approval" };
    const { result } = renderHook(() => useOvertimeRequests(params), { wrapper: createWrapper() });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    expect(api.listRequests).toHaveBeenCalledWith(params);
    expect(result.current.data?.data).toHaveLength(1);
  });
});

describe("useOvertimeRequest", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    (api.getRequest as Mock).mockResolvedValue({ ...MOCK_REQUEST, approval_history: [] });
  });

  it("fetches a single request by id", async () => {
    const { result } = renderHook(() => useOvertimeRequest("ot-1"), { wrapper: createWrapper() });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    expect(api.getRequest).toHaveBeenCalledWith("ot-1");
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

    expect(api.createRequest).toHaveBeenCalledWith(payload);
    await waitFor(() => {
      expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: overtimeRequestKeys.all });
    });
  });
});

describe("useSubmitOvertimeRequest", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    (api.submitRequest as Mock).mockResolvedValue(undefined);
  });

  it("submits a request and invalidates the list plus its detail", async () => {
    const { result } = renderHook(() => useSubmitOvertimeRequest(), { wrapper: createWrapper() });
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    await act(async () => {
      await result.current.mutateAsync("ot-1");
    });

    expect(api.submitRequest).toHaveBeenCalledWith("ot-1");
    await waitFor(() => {
      expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: overtimeRequestKeys.all });
      expect(invalidateSpy).toHaveBeenCalledWith({
        queryKey: overtimeRequestKeys.detail("ot-1"),
      });
    });
  });
});

describe("useCancelOvertimeRequest", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    (api.cancelRequest as Mock).mockResolvedValue(undefined);
  });

  it("cancels a request and invalidates the list plus its detail", async () => {
    const { result } = renderHook(() => useCancelOvertimeRequest(), { wrapper: createWrapper() });
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    await act(async () => {
      await result.current.mutateAsync("ot-1");
    });

    expect(api.cancelRequest).toHaveBeenCalledWith("ot-1");
    await waitFor(() => {
      expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: overtimeRequestKeys.all });
      expect(invalidateSpy).toHaveBeenCalledWith({
        queryKey: overtimeRequestKeys.detail("ot-1"),
      });
    });
  });
});

describe("useEmployeeConsent", () => {
  beforeEach(() => {
    vi.clearAllMocks();
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

    expect(api.employeeConsent).toHaveBeenCalledWith("ot-1", {
      employment_id: "emp-1",
      status: "accepted",
    });
    await waitFor(() => {
      expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: overtimeRequestKeys.all });
      expect(invalidateSpy).toHaveBeenCalledWith({
        queryKey: overtimeRequestKeys.detail("ot-1"),
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

    expect(api.employeeConsent).toHaveBeenCalledWith("ot-1", {
      employment_id: "emp-1",
      status: "declined",
      decline_reason: "Sick",
    });
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
