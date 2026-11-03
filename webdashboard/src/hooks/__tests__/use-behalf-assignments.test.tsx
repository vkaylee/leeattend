import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor, act } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach, type Mock } from "vitest";

import {
  fetchBehalfAssignments,
  createBehalfAssignment,
  deleteBehalfAssignment,
  fetchLeaveBehalfAssignments,
  createLeaveBehalfAssignment,
  deleteLeaveBehalfAssignment,
  fetchAdjustmentBehalfAssignments,
  createAdjustmentBehalfAssignment,
  deleteAdjustmentBehalfAssignment,
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

// Mock backend-owned organization context.
vi.mock("@/lib/api/helpers", () => ({
  getStandaloneOrganizationId: vi.fn().mockReturnValue("ws-test-123"),
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

const MOCK_OT_ASSIGNMENTS = [
  {
    id: "assign-1",
    workspace_id: "ws-test-123",
    assignee_user_id: "user-1",
    target_employee_id: "emp-2",
    target_department_id: null,
    assignment_type: "ot",
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
  },
];

const MOCK_LEAVE_ASSIGNMENTS = [
  {
    id: "assign-leave-1",
    workspace_id: "ws-test-123",
    assignee_user_id: "user-1",
    target_employee_id: "emp-2",
    target_department_id: null,
    assignment_type: "leave",
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
  },
];

const MOCK_ADJUSTMENT_ASSIGNMENTS = [
  {
    id: "assign-adj-1",
    workspace_id: "ws-test-123",
    assignee_user_id: "user-1",
    target_employee_id: "emp-2",
    target_department_id: null,
    assignment_type: "attendance_adjustment",
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
  },
];

describe("useBehalfAssignments", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("OT assignment type", () => {
    beforeEach(() => {
      (fetchBehalfAssignments as Mock).mockResolvedValue(MOCK_OT_ASSIGNMENTS);
    });

    it("fetches OT assignments on mount", async () => {
      const { result } = renderHook(() => useBehalfAssignments("ot"), {
        wrapper: createWrapper(),
      });

      await waitFor(() => { expect(result.current.isLoading).toBe(false); });

      expect(fetchBehalfAssignments).toHaveBeenCalledWith("ws-test-123");
      expect(result.current.assignments).toHaveLength(1);
      expect(result.current.assignments[0].id).toBe("assign-1");
    });

    it("creates OT assignment and invalidates cache", async () => {
      const newAssignment = {
        id: "assign-ot-new",
        assignee_user_id: "user-3",
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
          assignee_user_id: "user-3",
          target_employee_id: "emp-4",
        });
      });

      expect(createBehalfAssignment).toHaveBeenCalledWith("ws-test-123", {
        assignee_user_id: "user-3",
        target_employee_id: "emp-4",
      });
    });

    it("deletes OT assignment and invalidates cache", async () => {
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
  });

  describe("Leave assignment type", () => {
    beforeEach(() => {
      (fetchLeaveBehalfAssignments as Mock).mockResolvedValue(MOCK_LEAVE_ASSIGNMENTS);
    });

    it("fetches Leave assignments on mount", async () => {
      const { result } = renderHook(() => useBehalfAssignments("leave"), {
        wrapper: createWrapper(),
      });

      await waitFor(() => { expect(result.current.isLoading).toBe(false); });

      expect(fetchLeaveBehalfAssignments).toHaveBeenCalledWith("ws-test-123");
      expect(result.current.assignments).toHaveLength(1);
      expect(result.current.assignments[0].id).toBe("assign-leave-1");
    });

    it("creates Leave assignment and invalidates cache", async () => {
      const newAssignment = {
        id: "assign-leave-new",
        assignee_user_id: "user-3",
        target_employee_id: "emp-4",
        target_department_id: null,
      };
      (createLeaveBehalfAssignment as Mock).mockResolvedValue(newAssignment);

      const { result } = renderHook(() => useBehalfAssignments("leave"), {
        wrapper: createWrapper(),
      });

      await waitFor(() => { expect(result.current.isLoading).toBe(false); });

      await act(async () => {
        await result.current.createAssignment({
          assignee_user_id: "user-3",
          target_employee_id: "emp-4",
        });
      });

      expect(createLeaveBehalfAssignment).toHaveBeenCalledWith("ws-test-123", {
        assignee_user_id: "user-3",
        target_employee_id: "emp-4",
      });
    });

    it("deletes Leave assignment and invalidates cache", async () => {
      (deleteLeaveBehalfAssignment as Mock).mockResolvedValue(undefined);

      const { result } = renderHook(() => useBehalfAssignments("leave"), {
        wrapper: createWrapper(),
      });

      await waitFor(() => { expect(result.current.isLoading).toBe(false); });

      await act(async () => {
        await result.current.deleteAssignment("assign-leave-1");
      });

      expect(deleteLeaveBehalfAssignment).toHaveBeenCalledWith("ws-test-123", "assign-leave-1");
    });
  });

  describe("Attendance Adjustment assignment type", () => {
    beforeEach(() => {
      (fetchAdjustmentBehalfAssignments as Mock).mockResolvedValue(MOCK_ADJUSTMENT_ASSIGNMENTS);
    });

    it("fetches Attendance Adjustment assignments on mount", async () => {
      const { result } = renderHook(() => useBehalfAssignments("attendance_adjustment"), {
        wrapper: createWrapper(),
      });

      await waitFor(() => { expect(result.current.isLoading).toBe(false); });

      expect(fetchAdjustmentBehalfAssignments).toHaveBeenCalledWith("ws-test-123");
      expect(result.current.assignments).toHaveLength(1);
      expect(result.current.assignments[0].id).toBe("assign-adj-1");
    });

    it("creates Attendance Adjustment assignment and invalidates cache", async () => {
      const newAssignment = {
        id: "assign-adj-new",
        assignee_user_id: "user-3",
        target_employee_id: "emp-4",
        target_department_id: null,
      };
      (createAdjustmentBehalfAssignment as Mock).mockResolvedValue(newAssignment);

      const { result } = renderHook(() => useBehalfAssignments("attendance_adjustment"), {
        wrapper: createWrapper(),
      });

      await waitFor(() => { expect(result.current.isLoading).toBe(false); });

      await act(async () => {
        await result.current.createAssignment({
          assignee_user_id: "user-3",
          target_employee_id: "emp-4",
        });
      });

      expect(createAdjustmentBehalfAssignment).toHaveBeenCalledWith("ws-test-123", {
        assignee_user_id: "user-3",
        target_employee_id: "emp-4",
      });
    });

    it("deletes Attendance Adjustment assignment and invalidates cache", async () => {
      (deleteAdjustmentBehalfAssignment as Mock).mockResolvedValue(undefined);

      const { result } = renderHook(() => useBehalfAssignments("attendance_adjustment"), {
        wrapper: createWrapper(),
      });

      await waitFor(() => { expect(result.current.isLoading).toBe(false); });

      await act(async () => {
        await result.current.deleteAssignment("assign-adj-1");
      });

      expect(deleteAdjustmentBehalfAssignment).toHaveBeenCalledWith("ws-test-123", "assign-adj-1");
    });
  });

  describe("Query Keys", () => {
    it("generates correct query keys", () => {
      expect(behalfKeys.all).toEqual(["behalf-assignments"]);
      expect(behalfKeys.list("ws-abc", "ot")).toEqual([
        "behalf-assignments",
        "list",
        "ws-abc",
        "ot",
      ]);
      expect(behalfKeys.list("ws-abc", "leave")).toEqual([
        "behalf-assignments",
        "list",
        "ws-abc",
        "leave",
      ]);
      expect(behalfKeys.list("ws-abc", "attendance_adjustment")).toEqual([
        "behalf-assignments",
        "list",
        "ws-abc",
        "attendance_adjustment",
      ]);
    });
  });
});
