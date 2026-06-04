import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach, type Mock } from "vitest";

import { fetchMyBehalfTargets } from "@/lib/api/behalf-targets";

import { useBehalfEmployees } from "../use-behalf-employees";

// Mock the API function
vi.mock("@/lib/api/behalf-targets", () => ({
  fetchMyBehalfTargets: vi.fn(),
}));

// Mock getCurrentWorkspaceId
vi.mock("@/lib/api/helpers", () => ({
  getCurrentWorkspaceId: vi.fn().mockReturnValue("ws-test-123"),
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
  {
    id: "emp-1",
    workspace_id: "ws-test-123",
    employee_id_number: "EMP001",
    full_name: "Alice Nguyen",
    is_active: true,
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
  },
  {
    id: "emp-2",
    workspace_id: "ws-test-123",
    employee_id_number: "EMP002",
    full_name: "Bob Tran",
    is_active: true,
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
  },
];

describe("useBehalfEmployees", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("leave domain", () => {
    it("returns employees on success", async () => {
      (fetchMyBehalfTargets as Mock).mockResolvedValue(MOCK_EMPLOYEES);

      const { result } = renderHook(() => useBehalfEmployees("leave"), {
        wrapper: createWrapper(),
      });

      // Initially loading
      expect(result.current.isLoading).toBe(true);
      expect(result.current.employees).toEqual([]);
      expect(result.current.hasAssignments).toBe(false);

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });

      expect(fetchMyBehalfTargets).toHaveBeenCalledWith("ws-test-123", "leave");
      expect(result.current.employees).toHaveLength(2);
      expect(result.current.employees[0].full_name).toBe("Alice Nguyen");
      expect(result.current.hasAssignments).toBe(true);
      expect(result.current.isError).toBe(false);
    });

    it("returns hasAssignments=false when API returns empty list", async () => {
      (fetchMyBehalfTargets as Mock).mockResolvedValue([]);

      const { result } = renderHook(() => useBehalfEmployees("leave"), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });

      expect(result.current.employees).toEqual([]);
      expect(result.current.hasAssignments).toBe(false);
    });

    it("returns isError=true when API fails", async () => {
      (fetchMyBehalfTargets as Mock).mockRejectedValue(new Error("Network error"));

      const { result } = renderHook(() => useBehalfEmployees("leave"), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });

      expect(result.current.isError).toBe(true);
      expect(result.current.employees).toEqual([]);
      expect(result.current.hasAssignments).toBe(false);
    });
  });

  describe("attendance_adjustment domain", () => {
    it("fetches with correct domain parameter", async () => {
      (fetchMyBehalfTargets as Mock).mockResolvedValue(MOCK_EMPLOYEES);

      const { result } = renderHook(() => useBehalfEmployees("attendance_adjustment"), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });

      expect(fetchMyBehalfTargets).toHaveBeenCalledWith("ws-test-123", "attendance_adjustment");
      expect(result.current.employees).toHaveLength(2);
      expect(result.current.hasAssignments).toBe(true);
    });
  });

  describe("overtime_request domain", () => {
    it("fetches with correct domain parameter", async () => {
      (fetchMyBehalfTargets as Mock).mockResolvedValue(MOCK_EMPLOYEES);

      const { result } = renderHook(() => useBehalfEmployees("overtime_request"), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });

      expect(fetchMyBehalfTargets).toHaveBeenCalledWith("ws-test-123", "overtime_request");
      expect(result.current.employees).toHaveLength(2);
      expect(result.current.hasAssignments).toBe(true);
    });
  });

  describe("no workspace", () => {
    it("does not fetch when no workspace is set", async () => {
      // Override the mock for this test
      const helpersModule = await import("@/lib/api/helpers");
      vi.spyOn(helpersModule, "getCurrentWorkspaceId").mockReturnValue(null);

      const { result } = renderHook(() => useBehalfEmployees("leave"), {
        wrapper: createWrapper(),
      });

      // Should not fire the query
      expect(result.current.employees).toEqual([]);
      expect(fetchMyBehalfTargets).not.toHaveBeenCalled();
    });
  });
});
