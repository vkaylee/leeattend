import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import React from "react";
import { describe, it, expect, vi, beforeEach } from "vitest";

import { useOrganizationQuery, useOrganizationMutation } from "../common/use-workspace-query";

const ORGANIZATION_ID = "standalone";

const createWrapper = (): React.FC<{ children: React.ReactNode }> => {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  const Wrapper = ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
  Wrapper.displayName = "QueryClientWrapper";
  return Wrapper;
};

describe("useOrganizationQuery", () => {
  beforeEach(() => vi.clearAllMocks());

  it("calls queryFn with backend-owned organization context", async () => {
    const queryFn = vi.fn().mockResolvedValue({ items: [1, 2] });
    const { result } = renderHook(
      () => useOrganizationQuery({ queryKey: ["test-key"], queryFn }),
      { wrapper: createWrapper() },
    );
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(queryFn).toHaveBeenCalledWith(ORGANIZATION_ID);
    expect(result.current.data).toEqual({ items: [1, 2] });
  });

  it("does not run when explicitly disabled", () => {
    const queryFn = vi.fn();
    const { result } = renderHook(
      () => useOrganizationQuery({ queryKey: ["test-key"], queryFn, enabled: false }),
      { wrapper: createWrapper() },
    );
    expect(result.current.fetchStatus).toBe("idle");
    expect(queryFn).not.toHaveBeenCalled();
  });

  it("propagates query errors", async () => {
    const queryFn = vi.fn().mockRejectedValue(new Error("Boom!"));
    const { result } = renderHook(
      () => useOrganizationQuery({ queryKey: ["test-key"], queryFn }),
      { wrapper: createWrapper() },
    );
    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(result.current.error?.message).toBe("Boom!");
  });
});

describe("useOrganizationMutation", () => {
  beforeEach(() => vi.clearAllMocks());

  it("passes organization context and variables", async () => {
    const mutationFn = vi.fn().mockResolvedValue({ id: "created-1" });
    const payload = { name: "test" };
    const { result } = renderHook(
      () => useOrganizationMutation<typeof payload, { id: string }>({ mutationFn, keys: [["items"]] }),
      { wrapper: createWrapper() },
    );
    const mutationResult = await result.current.mutateAsync(payload);
    expect(mutationFn).toHaveBeenCalledWith(ORGANIZATION_ID, payload);
    expect(mutationResult).toEqual({ id: "created-1" });
  });

  it("invalidates supplied keys before custom success callback", async () => {
    const mutationFn = vi.fn().mockResolvedValue("ok");
    const order: string[] = [];
    const queryClient = new QueryClient();
    vi.spyOn(queryClient, "invalidateQueries").mockImplementation(async () => {
      order.push("invalidate");
    });
    const Wrapper = ({ children }: { children: React.ReactNode }) => (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );
    Wrapper.displayName = "Wrapper";
    const { result } = renderHook(
      () => useOrganizationMutation<string, string>({
        mutationFn,
        keys: [["items"], ["dashboard"]],
        onSuccess: () => order.push("custom"),
      }),
      { wrapper: Wrapper },
    );
    await act(() => result.current.mutateAsync("x"));
    expect(order).toEqual(["invalidate", "invalidate", "custom"]);
  });

  it("supports no-payload mutations", async () => {
    const mutationFn = vi.fn().mockResolvedValue("done");
    const { result } = renderHook(
      () => useOrganizationMutation<undefined, string>({ mutationFn, keys: [["items"]] }),
      { wrapper: createWrapper() },
    );
    await act(() => result.current.mutateAsync());
    expect(mutationFn).toHaveBeenCalledWith(ORGANIZATION_ID, undefined);
  });
});
