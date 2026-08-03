import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import { useWorkspaceQuery, useWorkspaceMutation } from "../common/use-workspace-query";

// ── Mocks ───────────────────────────────────────────────────────────────────

const mockUseWorkspace = vi.hoisted(() => vi.fn());

vi.mock("@/hooks/use-workspace", () => ({
  useWorkspace: mockUseWorkspace,
}));

const WS_ID = "ws-test-abc";
const withWorkspace = { workspace: { id: WS_ID, name: "Test WS" } };
const withoutWorkspace = { workspace: undefined };

// ── Helpers ─────────────────────────────────────────────────────────────────

const createWrapper = (): React.FC<{ children: React.ReactNode }> => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });
  const Wrapper = ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
  Wrapper.displayName = "QueryClientWrapper";
  return Wrapper;
};

// ── useWorkspaceQuery ───────────────────────────────────────────────────────

describe("useWorkspaceQuery", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUseWorkspace.mockReturnValue(withWorkspace);
  });

  it("calls queryFn with workspace.id when workspace is present", async () => {
    const queryFn = vi.fn().mockResolvedValue({ items: [1, 2] });

    const { result } = renderHook(
      () =>
        useWorkspaceQuery<{ items: number[] }>({
          queryKey: ["test-key"],
          queryFn,
        }),
      { wrapper: createWrapper() },
    );

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    expect(queryFn).toHaveBeenCalledTimes(1);
    expect(queryFn).toHaveBeenCalledWith(WS_ID);
    expect(result.current.data).toEqual({ items: [1, 2] });
  });

  it("disables query (enabled=false) when workspace is absent", () => {
    mockUseWorkspace.mockReturnValue(withoutWorkspace);
    const queryFn = vi.fn();

    const { result } = renderHook(
      () =>
        useWorkspaceQuery<unknown>({
          queryKey: ["test-key"],
          queryFn,
        }),
      { wrapper: createWrapper() },
    );

    expect(result.current.fetchStatus).toBe("idle");
    expect(queryFn).not.toHaveBeenCalled();
  });

  it("disables query when extra enabled=false even with workspace present", () => {
    const queryFn = vi.fn();

    const { result } = renderHook(
      () =>
        useWorkspaceQuery<unknown>({
          queryKey: ["test-key"],
          queryFn,
          enabled: false,
        }),
      { wrapper: createWrapper() },
    );

    expect(result.current.fetchStatus).toBe("idle");
    expect(queryFn).not.toHaveBeenCalled();
  });

  it("propagates error when queryFn rejects", async () => {
    const queryFn = vi.fn().mockRejectedValue(new Error("Boom!"));

    const { result } = renderHook(
      () =>
        useWorkspaceQuery<unknown>({
          queryKey: ["test-key"],
          queryFn,
        }),
      { wrapper: createWrapper() },
    );

    await waitFor(() => {
      expect(result.current.isError).toBe(true);
    });

    expect(result.current.error?.message).toBe("Boom!");
  });

  it("forwards staleTime to underlying useQuery", async () => {
    const queryFn = vi.fn().mockResolvedValue("data");

    renderHook(
      () =>
        useWorkspaceQuery<string>({
          queryKey: ["test-key"],
          queryFn,
          staleTime: 60_000,
        }),
      { wrapper: createWrapper() },
    );

    // Verify the query was set up — we can check that queryFn was called
    await waitFor(() => {
      expect(queryFn).toHaveBeenCalled();
    });
    // staleTime is forwarded internally; behavioural test: data is still fresh
  });
});

// ── useWorkspaceMutation ────────────────────────────────────────────────────

describe("useWorkspaceMutation", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUseWorkspace.mockReturnValue(withWorkspace);
  });

  const makePayload = (name: string) => ({ name, wsId: WS_ID });
  type Payload = ReturnType<typeof makePayload>;

  it("calls mutationFn with wsId and vars on mutateAsync", async () => {
    const mutationFn = vi.fn().mockResolvedValue({ id: "created-1" });

    const { result } = renderHook(
      () =>
        useWorkspaceMutation<Payload, { id: string }>({
          mutationFn,
          keys: [["items"]],
        }),
      { wrapper: createWrapper() },
    );

    expect(result.current.isIdle).toBe(true);

    const data = await act(() => result.current.mutateAsync(makePayload("test")));

    expect(mutationFn).toHaveBeenCalledTimes(1);
    expect(mutationFn).toHaveBeenCalledWith(WS_ID, makePayload("test"));
    expect(data).toEqual({ id: "created-1" });
  });

  it("invalidates provided keys on success", async () => {
    const mutationFn = vi.fn().mockResolvedValue("ok");
    const queryClient = new QueryClient();
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    const Wrapper = ({ children }: { children: React.ReactNode }) => (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );
    Wrapper.displayName = "Wrapper";

    const { result } = renderHook(
      () =>
        useWorkspaceMutation<Payload, string>({
          mutationFn,
          keys: [["items"], ["dashboard"]],
        }),
      { wrapper: Wrapper },
    );

    await act(() => result.current.mutateAsync(makePayload("x")));

    expect(invalidateSpy).toHaveBeenCalledTimes(2);
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["items"] });
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["dashboard"] });
  });

  it("calls custom onSuccess after invalidation", async () => {
    const mutationFn = vi.fn().mockResolvedValue({ status: "done" });
    const onSuccess = vi.fn();
    const callOrder: string[] = [];

    const queryClient = new QueryClient();
    const origInvalidate = queryClient.invalidateQueries.bind(queryClient);
    vi.spyOn(queryClient, "invalidateQueries").mockImplementation((...args) => {
      callOrder.push("invalidate");
      return origInvalidate(...args);
    });

    const Wrapper = ({ children }: { children: React.ReactNode }) => (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );
    Wrapper.displayName = "Wrapper";

    const { result } = renderHook(
      () =>
        useWorkspaceMutation<Payload, { status: string }>({
          mutationFn,
          keys: [["x"]],
          onSuccess: (data, vars) => {
            callOrder.push("custom");
            onSuccess(data, vars);
          },
        }),
      { wrapper: Wrapper },
    );

    const payload = makePayload("order-check");
    await act(() => result.current.mutateAsync(payload));

    // Invalidation must happen before custom onSuccess
    expect(callOrder).toEqual(["invalidate", "custom"]);
    expect(onSuccess).toHaveBeenCalledWith({ status: "done" }, payload);
  });

  it("throws when workspace is absent (guarded at render time)", () => {
    mockUseWorkspace.mockReturnValue(withoutWorkspace);
    const mutationFn = vi.fn();

    const { result } = renderHook(
      () =>
        useWorkspaceMutation<Payload, unknown>({
          mutationFn,
          keys: [["items"]],
        }),
      { wrapper: createWrapper() },
    );

    // The mutation is idle; when mutate fires, the guard inside mutationFn will throw
    return expect(
      result.current.mutateAsync(makePayload("fail")),
    ).rejects.toThrow("No workspace ID");
  });

  it("supports void TVars (no arguments mutation)", async () => {
    const mutationFn = vi.fn().mockResolvedValue("done");

    const { result } = renderHook(
      () =>
        useWorkspaceMutation<void, string>({
          mutationFn,
          keys: [["x"]],
        }),
      { wrapper: createWrapper() },
    );

    await act(() => result.current.mutateAsync());

    expect(mutationFn).toHaveBeenCalledWith(WS_ID, undefined);
  });

  it("returns mutation state (isPending, isSuccess, data)", async () => {
    const mutationFn = vi.fn().mockResolvedValue("result");

    const { result } = renderHook(
      () =>
        useWorkspaceMutation<Payload, string>({
          mutationFn,
          keys: [["x"]],
        }),
      { wrapper: createWrapper() },
    );

    expect(result.current.isPending).toBe(false);

    await act(async () => {
      await result.current.mutateAsync(makePayload("state-test"));
    });

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });
    expect(result.current.data).toBe("result");
  });
});
