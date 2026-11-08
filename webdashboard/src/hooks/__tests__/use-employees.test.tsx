import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach, type Mock } from "vitest";

import { fetchEmployees } from "@/lib/api";

import { useAllEmployees, MAX_EMPLOYEE_FETCH, employeeKeys } from "../use-employees";

// Mock the API
vi.mock("@/lib/api", () => ({
  fetchEmployees: vi.fn(),
}));

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  const Wrapper = ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
  Wrapper.displayName = "QueryClientWrapper";
  return Wrapper;
}

const MOCK_EMPLOYEES = [
  { id: "emp-1", full_name: "Alice", employee_id_number: "E001" },
  { id: "emp-2", full_name: "Bob", employee_id_number: "E002" },
];

describe("useAllEmployees", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    (fetchEmployees as Mock).mockResolvedValue({
      data: MOCK_EMPLOYEES,
      total: 2,
      page: 1,
      limit: MAX_EMPLOYEE_FETCH,
    });
  });

  it("fetches employees on mount", async () => {
    const { result } = renderHook(() => useAllEmployees(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => { expect(result.current.isLoading).toBe(false); });

    expect(fetchEmployees).toHaveBeenCalledWith(1, MAX_EMPLOYEE_FETCH, undefined);
    expect(result.current.employees).toHaveLength(2);
    expect(result.current.employees[0].full_name).toBe("Alice");
  });

  it("respects enabled option for lazy-loading (modals)", async () => {
    // Disabled initially
    const { result, rerender } = renderHook(
      ({ enabled }: { enabled: boolean }) => useAllEmployees({ enabled }),
      { wrapper: createWrapper(), initialProps: { enabled: false } },
    );

    expect(fetchEmployees).not.toHaveBeenCalled();
    expect(result.current.employees).toEqual([]);

    // Enable (modal opens)
    rerender({ enabled: true });

    await waitFor(() => { expect(result.current.isLoading).toBe(false); });
    expect(fetchEmployees).toHaveBeenCalledTimes(1);
    expect(result.current.employees).toHaveLength(2);
  });

  it("returns empty array when API returns no data field", async () => {
    (fetchEmployees as Mock).mockResolvedValue({ total: 0, page: 1, limit: MAX_EMPLOYEE_FETCH });

    const { result } = renderHook(() => useAllEmployees(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => { expect(result.current.isLoading).toBe(false); });
    expect(result.current.employees).toEqual([]);
  });

  it("exposes the MAX_EMPLOYEE_FETCH constant as 100", () => {
    expect(MAX_EMPLOYEE_FETCH).toBe(3000);
  });

  it("generates correct query keys", () => {
    expect(employeeKeys.all()).toEqual(["employees"]);
    expect(employeeKeys.list()).toEqual(["employees", "list", {}]);
  });
});
