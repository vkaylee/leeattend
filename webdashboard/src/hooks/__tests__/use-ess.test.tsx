import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";
import React from "react";
import { describe, it, expect, vi, beforeEach } from "vitest";

import * as api from "@/lib/api";

import { useMyClock, essKeys } from "../use-ess";

// Mock the API
vi.mock("@/lib/api", () => ({
  fetchMyTimeEntries: vi.fn(),
  punchInMySelf: vi.fn(),
  punchOutMySelf: vi.fn(),
}));

const createWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  });
  const Wrapper = ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
  Wrapper.displayName = "QueryClientWrapper";
  return { Wrapper, queryClient };
};

describe("useMyClock optimistic updates", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("updates the clock entries cache immediately on successful punch in", async () => {
    const mockEntries = [
      {
        id: "entry-1",
        entry_type: "work",
        punch_in_time: "2026-05-04T00:00:00Z",
        punch_out_time: "2026-05-04T08:00:00Z",
      },
    ];
    const newEntry = {
      id: "entry-2",
      entry_type: "work",
      punch_in_time: "2026-05-05T00:00:00Z",
      punch_out_time: null,
    };
    vi.mocked(api.fetchMyTimeEntries)
      .mockResolvedValueOnce(mockEntries as any)
      .mockResolvedValueOnce([newEntry, ...mockEntries] as any);

    vi.mocked(api.punchInMySelf).mockResolvedValue(newEntry as any);

    const { Wrapper, queryClient } = createWrapper();
    const { result } = renderHook(() => useMyClock(), {
      wrapper: Wrapper,
    });

    // Wait for initial fetch
    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });
    expect(result.current.entries).toEqual(mockEntries);

    // Call punchIn
    await result.current.punchIn({ entry_type: "work" });

    // Expect the optimistic update to be present in the cache immediately
    const cachedData = queryClient.getQueryData(essKeys.clockEntries());
    expect(cachedData).toEqual([newEntry, ...mockEntries]);
  });

  it("updates the clock entries cache immediately on successful punch out", async () => {
    const mockEntries = [
      {
        id: "entry-1",
        entry_type: "work",
        punch_in_time: "2026-05-04T00:00:00Z",
        punch_out_time: null,
      },
    ];
    const updatedEntry = {
      id: "entry-1",
      entry_type: "work",
      punch_in_time: "2026-05-04T00:00:00Z",
      punch_out_time: "2026-05-04T08:00:00Z",
    };
    vi.mocked(api.fetchMyTimeEntries)
      .mockResolvedValueOnce(mockEntries as any)
      .mockResolvedValueOnce([updatedEntry] as any);

    vi.mocked(api.punchOutMySelf).mockResolvedValue(updatedEntry as any);

    const { Wrapper, queryClient } = createWrapper();
    const { result } = renderHook(() => useMyClock(), {
      wrapper: Wrapper,
    });

    // Wait for initial fetch
    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });
    expect(result.current.entries).toEqual(mockEntries);

    // Call punchOut
    await result.current.punchOut({});

    // Expect the optimistic update to replace the old entry
    const cachedData = queryClient.getQueryData(essKeys.clockEntries());
    expect(cachedData).toEqual([updatedEntry]);
  });
});
