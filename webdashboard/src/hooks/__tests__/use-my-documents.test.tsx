import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";
import React from "react";

import { beforeEach, describe, expect, it, vi } from "vitest";

import * as api from "@/lib/api";

import { useMyDocuments, useMyPayslips } from "../use-my-documents";

// Mock the API functions
vi.mock("@/lib/api", () => ({
  fetchMyDocuments: vi.fn(),
  fetchMyPayslips: vi.fn(),
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

describe("useMyDocuments & useMyPayslips", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("useMyDocuments fetches documents successfully", async () => {
    const mockDocs = [{ id: "doc-1", name: "ID Card" }];
    vi.mocked(api.fetchMyDocuments).mockResolvedValue(mockDocs as any);

    const { Wrapper } = createWrapper();
    const { result } = renderHook(() => useMyDocuments(), {
      wrapper: Wrapper,
    });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.documents).toEqual(mockDocs);
  });

  it("useMyPayslips fetches payslips successfully", async () => {
    const mockPayslips = [{ id: "ps-1", net_pay_amount: "1000" }];
    vi.mocked(api.fetchMyPayslips).mockResolvedValue(mockPayslips as any);

    const { Wrapper } = createWrapper();
    const { result } = renderHook(() => useMyPayslips(), {
      wrapper: Wrapper,
    });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.payslips).toEqual(mockPayslips);
  });
});
