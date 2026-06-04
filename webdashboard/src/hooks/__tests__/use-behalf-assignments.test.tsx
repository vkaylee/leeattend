import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor, act } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach, type Mock } from "vitest";

import {
  fetchBehalfAssignments,
  createBehalfAssignment,
  deleteBehalfAssignment,
} from "@/lib/api";

import {
  useBehalfAssignments,
  behalfKeys,
} from "../use-behalf-assignments";

// Mock the API module
vi.mock("@/lib/api", () => ({
  fetchBehalfAssignments: vi.fn(),
  createBehalfAssignment: vi.fn(),
  deleteBehalfAssignment: vi.fn(),
  fetchLeaveBehalfAssignments: vi.fn(),
  createLeaveBehalfAssignment: vi.fn(),
  deleteLeaveBehalfAssignment: vi.fn(),
  fetchAdjustmentBehalfAssignments: vi.fn(),
  createAdjustmentBehalfAssignment: vi.fn(),
  deleteAdjustmentBehalfAssignment: vi.fn(),
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

const MOCK_ASSIGNMENTS = [
  {
    id: "assign-1",
    workspace_id: "ws-test-123",
    assignee_employee_id: "emp-1",
    target_employee_id: "emp-2",
    target_department_id: null,
    assignment_type: "ot",
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
  },
];

describe("useBehalfAssignments", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    (fetchBehalfAssignments as Mock).mockResolvedValue(MOCK_ASSIGNMENTS);
  });

  it("fetches assignments on mount", async () => {
    const { result } = renderHook(() => useBehalfAssignments("ot"), {
      wrapper: createWrapper(),
    });

    await waitFor(() => { expect(result.current.isLoading).toBe(false); });

    expect(fetchBehalfAssignments).toHaveBeenCalledWith("ws-test-123");
    expect(result.current.assignments).toHaveLength(1);
    expect(result.current.assignments[0].id).toBe("assign-1");
  });

  it("creates assignment and invalidates cache", async () => {
    const newAssignment = {
      id: "assign-3",
      assignee_employee_id: "emp-3",
      target_employee_id: "emp-4",
      target_department_id: null,
    };
    (createBehalfAssignment as Mock).mockResolvedValue(newAssignment);

    const { result } = renderHook(() => useBehalfAssignments("ot"), {
      wrapper: createWrapper(),
    });

    await waitFor(() => { expect(result.current.isLoading).toBe(false); });

    await act(async () => {
      await result.current.createAssignment({
        assignee_employee_id: "emp-3",
        target_employee_id: "emp-4",
      });
    });

    expect(createBehalfAssignment).toHaveBeenCalledWith("ws-test-123", {
      assignee_employee_id: "emp-3",
      target_employee_id: "emp-4",
    });
  });

  it("deletes assignment and invalidates cache", async () => {
    (deleteBehalfAssignment as Mock).mockResolvedValue(undefined);

    const { result } = renderHook(() => useBehalfAssignments("ot"), {
      wrapper: createWrapper(),
    });

    await waitFor(() => { expect(result.current.isLoading).toBe(false); });

    await act(async () => {
      await result.current.deleteAssignment("assign-1");
    });

    expect(deleteBehalfAssignment).toHaveBeenCalledWith("ws-test-123", "assign-1");
  });

  it("generates correct query keys", () => {
    expect(behalfKeys.all).toEqual(["behalf-assignments"]);
    expect(behalfKeys.list("ws-abc", "ot")).toEqual([
      "behalf-assignments",
      "list",
      "ws-abc",
      "ot",
    ]);
  });
});
