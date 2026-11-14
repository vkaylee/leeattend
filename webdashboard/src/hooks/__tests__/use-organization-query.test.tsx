import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import { useOrganizationInfiniteQuery, useOrganizationMutation, useOrganizationQuery } from "../common/use-organization-query";

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

  it("calls queryFn without injecting an organization identifier", async () => {
    const queryFn = vi.fn().mockResolvedValue({ items: [1, 2] });
    const { result } = renderHook(
      () => useOrganizationQuery({ queryKey: ["test-key"], queryFn }),
      { wrapper: createWrapper() },
    );
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(queryFn).toHaveBeenCalledWith(expect.any(Object));
    expect(result.current.data).toEqual({ items: [1, 2] });
  });

  it("does not run when explicitly disabled", () => {
    const queryFn = vi.fn().mockResolvedValue([1]);
    const { result } = renderHook(
      () => useOrganizationQuery({ queryKey: ["test-key"], queryFn, enabled: false }),
      { wrapper: createWrapper() },
    );
    expect(result.current.fetchStatus).toBe("idle");
    expect(queryFn).not.toHaveBeenCalled();
  });

  it("propagates query errors", async () => {
    const queryFn = vi.fn().mockRejectedValue(new Error("boom"));
    const { result } = renderHook(
      () => useOrganizationQuery({ queryKey: ["test-key"], queryFn }),
      { wrapper: createWrapper() },
    );
    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(result.current.error).toEqual(new Error("boom"));
  });
});

describe("useOrganizationInfiniteQuery", () => {
  beforeEach(() => vi.clearAllMocks());

  it("passes only the page parameter to queryFn", async () => {
    const queryFn = vi.fn().mockImplementation((pageParam: number) =>
      Promise.resolve({ data: [pageParam], total: 2 }),
    );
    const { result } = renderHook(
      () =>
        useOrganizationInfiniteQuery({
          queryKey: ["infinite-key"],
          queryFn,
          getNextPageParam: (lastPage: { data: number[]; total: number }, allPages: { data: number[]; total: number }[]) =>
            allPages.length < lastPage.total ? allPages.length + 1 : undefined,
          initialPageParam: 1,
        }),
      { wrapper: createWrapper() },
    );

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(queryFn).toHaveBeenCalledWith(1);
    expect(result.current.data?.pages).toEqual([{ data: [1], total: 2 }]);
  });
});

describe("useOrganizationMutation", () => {
  beforeEach(() => vi.clearAllMocks());

  it("passes variables without injecting organization context", async () => {
    const mutationFn = vi.fn().mockResolvedValue({ id: "created-1" });
    const payload = { name: "test" };
    const { result } = renderHook(
      () => useOrganizationMutation<typeof payload, { id: string }>({ mutationFn, keys: [["items"]] }),
      { wrapper: createWrapper() },
    );
    const mutationResult = await result.current.mutateAsync(payload);
    expect(mutationFn).toHaveBeenCalledWith(payload, expect.any(Object));
    expect(mutationResult).toEqual({ id: "created-1" });
  });

  it("invalidates supplied keys before custom success callback", async () => {
    const order: string[] = [];
    const queryClient = new QueryClient();
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries").mockImplementation(() => {
      order.push("invalidate");
      return Promise.resolve();
    });

    const mutationFn = vi.fn().mockResolvedValue("ok");
    const wrapper = ({ children }: { children: React.ReactNode }) => (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );

    const { result } = renderHook(
      () => useOrganizationMutation<string, string>({
        mutationFn,
        keys: [["items"], ["dashboard"]],
        onSuccess: () => order.push("custom"),
      }),
      { wrapper },
    );

    await act(() => result.current.mutateAsync("value"));

    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["items"] });
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["dashboard"] });
    expect(order).toEqual(["invalidate", "invalidate", "custom"]);
  });

  it("supports no-payload mutations", async () => {
    const mutationFn = vi.fn().mockResolvedValue("done");
    const { result } = renderHook(
      () => useOrganizationMutation<undefined, string>({ mutationFn, keys: [["items"]] }),
      { wrapper: createWrapper() },
    );
    await act(() => result.current.mutateAsync());
    expect(mutationFn).toHaveBeenCalledWith(undefined, expect.any(Object));
  });
});
