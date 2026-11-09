import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";
import React from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  fetchAssignmentRules: vi.fn(),
  fetchWorkingPatterns: vi.fn(),
  fetchEmployers: vi.fn(),
  fetchLocations: vi.fn(),
}));

vi.mock("@/lib/api", () => mocks);
vi.mock("@/hooks/use-departments", () => ({
  useDepartments: () => ({ data: [] }),
}));

import { useAssignmentRulesData } from "../use-assignment-rules-data";

describe("useAssignmentRulesData location access", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.fetchAssignmentRules.mockResolvedValue([]);
    mocks.fetchWorkingPatterns.mockResolvedValue([]);
    mocks.fetchEmployers.mockResolvedValue([]);
  });

  it("does not retry forbidden locations or surface them as required data", async () => {
    const forbidden = Object.assign(new Error("Forbidden"), { status: 403 });
    mocks.fetchLocations.mockRejectedValue(forbidden);
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: 3 } },
    });
    const wrapper = ({ children }: { children: React.ReactNode }) => (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );

    const { result } = renderHook(() => useAssignmentRulesData(), { wrapper });

    await waitFor(() => expect(mocks.fetchLocations).toHaveBeenCalledTimes(1));

    expect(result.current.locations).toBeUndefined();
    expect(queryClient.getQueryCache().find({ queryKey: ["locations"] })?.meta).toEqual({
      suppressErrorToast: [403],
    });
  });
});
