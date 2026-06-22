import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";
import React from "react";

import { beforeEach, describe, expect, it, vi } from "vitest";

import * as api from "@/lib/api";

import { useMyLeave } from "../use-my-leave";

// Mock the API functions
vi.mock("@/lib/api", () => ({
  fetchEssLeaveBalances: vi.fn(),
  fetchEssLeaveRequests: vi.fn(),
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

describe("useMyLeave", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("fetches leave balances and requests", async () => {
    const mockBalances = [{ leave_type_id: "lt-1", balance_days: 10 }];
    const mockRequests = [{ id: "req-1", reason: "vacation" }];
    vi.mocked(api.fetchEssLeaveBalances).mockResolvedValue(mockBalances as any);
    vi.mocked(api.fetchEssLeaveRequests).mockResolvedValue(mockRequests as any);

    const { Wrapper } = createWrapper();
    const { result } = renderHook(() => useMyLeave(), {
      wrapper: Wrapper,
    });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.balances).toEqual(mockBalances);
    expect(result.current.requests).toEqual(mockRequests);
  });
});
