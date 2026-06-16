import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";
import React from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import * as api from "@/lib/api";

import {
  useMyAttendance,
  useMyAttendanceSummaries,
  useSubmitOtClaim,
} from "../use-my-attendance";

// Mock API functions
vi.mock("@/lib/api", () => ({
  fetchMyAttendance: vi.fn(),
  fetchMyAttendanceSummaries: vi.fn(),
  submitMyOtClaim: vi.fn(),
  // Mock standard clock endpoints as they may be imported implicitly
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

describe("useMyAttendance", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("fetches attendance entries correctly", async () => {
    const mockEntries = [{ id: "att-1", punch_in_time: "2026-03-01T08:00:00Z" }];
    vi.mocked(api.fetchMyAttendance).mockResolvedValue(mockEntries as any);

    const { Wrapper } = createWrapper();
    const { result } = renderHook(() => useMyAttendance(), {
      wrapper: Wrapper,
    });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.entries).toEqual(mockEntries);
    expect(api.fetchMyAttendance).toHaveBeenCalled();
  });
});

describe("useMyAttendanceSummaries", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("fetches attendance summaries for range", async () => {
    const mockSummaries = [{ date: "2026-03-01", status: "Present" }];
    vi.mocked(api.fetchMyAttendanceSummaries).mockResolvedValue(mockSummaries as any);

    const { Wrapper } = createWrapper();
    const { result } = renderHook(() => useMyAttendanceSummaries("2026-03-01", "2026-03-07"), {
      wrapper: Wrapper,
    });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.summaries).toEqual(mockSummaries);
    expect(api.fetchMyAttendanceSummaries).toHaveBeenCalledWith("2026-03-01", "2026-03-07");
  });
});

describe("useSubmitOtClaim", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("calls submitMyOtClaim mutation", async () => {
    vi.mocked(api.submitMyOtClaim).mockResolvedValue(undefined as any);

    const { Wrapper } = createWrapper();
    const { result } = renderHook(() => useSubmitOtClaim(), {
      wrapper: Wrapper,
    });

    await result.current.submitClaim({ date: "2026-03-01", minutes: 90, reason: "Busy day" });
    expect(api.submitMyOtClaim).toHaveBeenCalledWith("2026-03-01", 90, "Busy day");
  });
});
