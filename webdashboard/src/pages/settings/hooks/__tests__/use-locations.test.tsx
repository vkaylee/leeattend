import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";
import React from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { fetchLocations } from "@/lib/api/locations";

import { useLocations } from "../use-locations";

vi.mock("@/lib/api/locations", () => ({
  fetchLocations: vi.fn(),
  createLocation: vi.fn(),
  updateLocation: vi.fn(),
  deleteLocation: vi.fn(),
}));

function createWrapper(queryClient: QueryClient) {
  return function QueryClientWrapper({ children }: { children: React.ReactNode }) {
    return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
  };
}

describe("useLocations", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("returns locations for an authorized role", async () => {
    const locations = [{ id: "location-1", name: "HQ" }];
    vi.mocked(fetchLocations).mockResolvedValue(locations as never);
    const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });

    const { result } = renderHook(() => useLocations(), { wrapper: createWrapper(queryClient) });

    await waitFor(() => expect(result.current.data).toEqual(locations));
    expect(fetchLocations).toHaveBeenCalledTimes(1);
  });

  it("keeps forbidden locations optional without retry or error metadata noise", async () => {
    const forbidden = Object.assign(new Error("Forbidden"), { status: 403 });
    vi.mocked(fetchLocations).mockRejectedValue(forbidden);
    const queryClient = new QueryClient({ defaultOptions: { queries: { retry: 3 } } });

    const { result } = renderHook(() => useLocations(), { wrapper: createWrapper(queryClient) });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(result.current.data).toEqual([]);
    expect(fetchLocations).toHaveBeenCalledTimes(1);
    expect(queryClient.getQueryCache().find({ queryKey: ["locations"] })?.meta).toEqual({
      suppressErrorToast: [403],
    });
  });
});
