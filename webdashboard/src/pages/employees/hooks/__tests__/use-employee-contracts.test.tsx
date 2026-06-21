import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";
import React from "react";
import { describe, it, expect, vi, beforeEach, type Mock } from "vitest";

import {
  fetchEmployeeContracts,
  createEmployeeContract,
  updateEmployeeContract,
  deleteEmployeeContract,
} from "@/lib/api";

import { useEmployeeContracts } from "../use-employee-contracts";

vi.mock("@/lib/api", () => ({
  fetchEmployeeContracts: vi.fn(),
  createEmployeeContract: vi.fn(),
  updateEmployeeContract: vi.fn(),
  deleteEmployeeContract: vi.fn(),
}));

function createWrapper() {
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
  return { Wrapper, queryClient };
}

const mockContracts = [
  { id: "c-1", contract_number: "HD-001", status: "active", start_date: "2025-01-01" },
  { id: "c-2", contract_number: "HD-002", status: "draft", start_date: "2025-02-01" },
];

describe("useEmployeeContracts hook", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    (fetchEmployeeContracts as Mock).mockResolvedValue(mockContracts);
    (createEmployeeContract as Mock).mockResolvedValue({ id: "c-3" });
    (updateEmployeeContract as Mock).mockResolvedValue({ id: "c-1" });
    (deleteEmployeeContract as Mock).mockResolvedValue({ id: "c-1" });
  });

  it("fetches contracts for given wsId and employeeId", async () => {
    const { Wrapper } = createWrapper();
    const { result } = renderHook(() => useEmployeeContracts("ws-1", "emp-1"), {
      wrapper: Wrapper,
    });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    expect(fetchEmployeeContracts).toHaveBeenCalledWith("ws-1", "emp-1");
    expect(result.current.contracts).toEqual(mockContracts);
    expect(result.current.isError).toBe(false);
  });

  it("does not fetch if wsId or employeeId are empty", () => {
    const { Wrapper } = createWrapper();
    const { result } = renderHook(() => useEmployeeContracts("", ""), {
      wrapper: Wrapper,
    });

    expect(fetchEmployeeContracts).not.toHaveBeenCalled();
    expect(result.current.contracts).toEqual([]);
  });

  it("calls createEmployeeContract and invalidates cache", async () => {
    const { Wrapper, queryClient } = createWrapper();
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    const { result } = renderHook(() => useEmployeeContracts("ws-1", "emp-1"), {
      wrapper: Wrapper,
    });

    const newContractPayload = {
      contract_number: "HD-003",
      contract_type: "probation" as const,
      start_date: "2025-03-01",
    };

    await result.current.createContract.mutateAsync(newContractPayload);

    expect(createEmployeeContract).toHaveBeenCalledWith("ws-1", "emp-1", newContractPayload);
    expect(invalidateSpy).toHaveBeenCalledWith({
      queryKey: ["employee-contracts", "ws-1", "emp-1"],
    });
  });

  it("calls updateEmployeeContract and invalidates cache", async () => {
    const { Wrapper, queryClient } = createWrapper();
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    const { result } = renderHook(() => useEmployeeContracts("ws-1", "emp-1"), {
      wrapper: Wrapper,
    });

    const updatePayload = {
      contract_number: "HD-001-Updated",
    };

    await result.current.updateContract.mutateAsync({
      contractId: "c-1",
      data: updatePayload,
    });

    expect(updateEmployeeContract).toHaveBeenCalledWith("ws-1", "emp-1", "c-1", updatePayload);
    expect(invalidateSpy).toHaveBeenCalledWith({
      queryKey: ["employee-contracts", "ws-1", "emp-1"],
    });
  });

  it("calls deleteEmployeeContract and invalidates cache", async () => {
    const { Wrapper, queryClient } = createWrapper();
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    const { result } = renderHook(() => useEmployeeContracts("ws-1", "emp-1"), {
      wrapper: Wrapper,
    });

    await result.current.deleteContract.mutateAsync("c-1");

    expect(deleteEmployeeContract).toHaveBeenCalledWith("ws-1", "emp-1", "c-1");
    expect(invalidateSpy).toHaveBeenCalledWith({
      queryKey: ["employee-contracts", "ws-1", "emp-1"],
    });
  });
});
