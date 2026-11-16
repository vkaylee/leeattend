import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";
import React from "react";

import { beforeEach, describe, expect, it, vi } from "vitest";

import * as api from "@/lib/api";

import { useMyClock, essKeys } from "../use-ess";

// Mock the API
vi.mock("@/lib/api", () => ({
  fetchMyTimeEntries: vi.fn(),
  punchMySelf: vi.fn(),
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
      .mockResolvedValueOnce(mockEntries as unknown as api.TimeEntry[])
      .mockResolvedValueOnce([newEntry, ...mockEntries] as unknown as api.TimeEntry[]);

    vi.mocked(api.punchMySelf).mockResolvedValue(newEntry as unknown as api.TimeEntry);

    const { Wrapper, queryClient } = createWrapper();
    const { result } = renderHook(() => useMyClock(), {
      wrapper: Wrapper,
    });

    // Wait for initial fetch
    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });
    expect(result.current.entries).toEqual(mockEntries);

    // Call punch
    await result.current.punch({});

    // Expect the optimistic update to be present in the cache immediately
    const cachedData = queryClient.getQueryData(essKeys.clockEntries());
    expect(cachedData).toEqual([newEntry, ...mockEntries]);
  });

  it("requests date-only bounds for the clock endpoint", async () => {
    vi.mocked(api.fetchMyTimeEntries).mockResolvedValue([]);

    const { Wrapper } = createWrapper();
    renderHook(() => useMyClock(), { wrapper: Wrapper });

    await waitFor(() => {
      expect(api.fetchMyTimeEntries).toHaveBeenCalledWith({
        start_date: expect.stringMatching(/^\d{4}-\d{2}-\d{2}$/),
        end_date: expect.stringMatching(/^\d{4}-\d{2}-\d{2}$/),
      });
    });
  });

  it("does not fetch clock entries when clock access is disabled", async () => {
    const { Wrapper } = createWrapper();
    const { result } = renderHook(() => useMyClock({ enabled: false }), {
      wrapper: Wrapper,
    });

    expect(result.current.entries).toEqual([]);
    expect(api.fetchMyTimeEntries).not.toHaveBeenCalled();
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
      .mockResolvedValueOnce(mockEntries as unknown as api.TimeEntry[])
      .mockResolvedValueOnce([updatedEntry] as unknown as api.TimeEntry[]);

    vi.mocked(api.punchMySelf).mockResolvedValue(updatedEntry as unknown as api.TimeEntry);

    const { Wrapper, queryClient } = createWrapper();
    const { result } = renderHook(() => useMyClock(), {
      wrapper: Wrapper,
    });

    // Wait for initial fetch
    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });
    expect(result.current.entries).toEqual(mockEntries);

    // Call punch
    await result.current.punch({});

    // Expect the optimistic update to replace the old entry
    const cachedData = queryClient.getQueryData(essKeys.clockEntries());
    expect(cachedData).toEqual([updatedEntry]);
  });
});
