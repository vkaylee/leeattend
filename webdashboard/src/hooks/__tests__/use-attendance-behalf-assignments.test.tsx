import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor, act } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach, type Mock } from "vitest";

import {
  fetchAdjustmentBehalfAssignments,
  createAdjustmentBehalfAssignment,
  deleteAdjustmentBehalfAssignment,
} from "@/lib/api";

import {
  useAttendanceBehalfAssignments,
  adjustmentBehalfKeys,
} from "../use-attendance-behalf-assignments";

// Mock the API module
vi.mock("@/lib/api", () => ({
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
    assignment_type: "attendance_adjustment",
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
  },
  {
    id: "assign-2",
    workspace_id: "ws-test-123",
    assignee_employee_id: "emp-1",
    target_employee_id: null,
    target_department_id: "dept-1",
    assignment_type: "attendance_adjustment",
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
  },
];

describe("useAttendanceBehalfAssignments", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    (fetchAdjustmentBehalfAssignments as Mock).mockResolvedValue(MOCK_ASSIGNMENTS);
  });

  it("fetches assignments on mount", async () => {
    const { result } = renderHook(() => useAttendanceBehalfAssignments(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => { expect(result.current.isLoading).toBe(false); });

    expect(fetchAdjustmentBehalfAssignments).toHaveBeenCalledWith("ws-test-123");
    expect(result.current.assignments).toHaveLength(2);
    expect(result.current.assignments[0].id).toBe("assign-1");
  });

  it("returns empty array when no data", async () => {
    (fetchAdjustmentBehalfAssignments as Mock).mockResolvedValue([]);

    const { result } = renderHook(() => useAttendanceBehalfAssignments(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => { expect(result.current.isLoading).toBe(false); });
    expect(result.current.assignments).toEqual([]);
  });

  it("creates assignment and invalidates cache", async () => {
    const newAssignment = {
      id: "assign-3",
      assignee_employee_id: "emp-3",
      target_employee_id: "emp-4",
      target_department_id: null,
    };
    (createAdjustmentBehalfAssignment as Mock).mockResolvedValue(newAssignment);

    const { result } = renderHook(() => useAttendanceBehalfAssignments(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => { expect(result.current.isLoading).toBe(false); });

    await act(async () => {
      await result.current.createAssignment({
        assignee_employee_id: "emp-3",
        target_employee_id: "emp-4",
      });
    });

    expect(createAdjustmentBehalfAssignment).toHaveBeenCalledWith("ws-test-123", {
      assignee_employee_id: "emp-3",
      target_employee_id: "emp-4",
    });
  });

  it("deletes assignment and invalidates cache", async () => {
    (deleteAdjustmentBehalfAssignment as Mock).mockResolvedValue(undefined);

    const { result } = renderHook(() => useAttendanceBehalfAssignments(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => { expect(result.current.isLoading).toBe(false); });

    await act(async () => {
      await result.current.deleteAssignment("assign-1");
    });

    expect(deleteAdjustmentBehalfAssignment).toHaveBeenCalledWith("ws-test-123", "assign-1");
  });

  it("exposes isPending states", async () => {
    const { result } = renderHook(() => useAttendanceBehalfAssignments(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => { expect(result.current.isLoading).toBe(false); });

    expect(result.current.isCreating).toBe(false);
    expect(result.current.isDeleting).toBe(false);
  });

  it("generates correct query keys", () => {
    expect(adjustmentBehalfKeys.all).toEqual(["adjustment-behalf-assignments"]);
    expect(adjustmentBehalfKeys.list("ws-abc")).toEqual([
      "adjustment-behalf-assignments",
      "list",
      "ws-abc",
    ]);
  });
});
