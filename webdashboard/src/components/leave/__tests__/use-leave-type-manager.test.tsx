import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor, act } from "@testing-library/react";
import React from "react";

import { vi, describe, it, expect, beforeEach } from "vitest";

import * as api from "@/lib/api/leaves";

import { useLeaveTypeManager } from "../hooks/use-leave-type-manager";

// Mock API
vi.mock("@/lib/api/leaves", () => ({
  fetchLeaveTypes: vi.fn(),
  updateLeaveTypeConfig: vi.fn(),
  createLeaveType: vi.fn(),
  deleteLeaveType: vi.fn(),
  restoreLeaveType: vi.fn(),
}));

// Mock Toast
vi.mock("@/components/ui/toast", () => ({ 
  useToast: () => ({ success: vi.fn(), error: vi.fn() }) 
}));

const queryClient = new QueryClient({ 
  defaultOptions: { 
    queries: { retry: false } 
  } 
});

const wrapper = ({ children }: { children: React.ReactNode }) => (
  <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
);

describe("useLeaveTypeManager Hook", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should fetch leave types on mount", async () => {
    const mockLeaveTypes = [
      { id: "1", code: "AL", name: "Annual Leave", is_paid: true, policy_config: {} }
    ];
    vi.mocked(api.fetchLeaveTypes).mockResolvedValue(mockLeaveTypes as never);

    const { result } = renderHook(() => useLeaveTypeManager(), { wrapper });

    await waitFor(() => {
      expect(result.current.isTypesLoading).toBe(false);
    });
    expect(result.current.leaveTypes).toEqual(mockLeaveTypes);
  });

  it("should manage add/edit/delete states correctly", () => {
    const { result } = renderHook(() => useLeaveTypeManager(), { wrapper });

    expect(result.current.isAddingType).toBe(false);
    expect(result.current.editingType).toBeNull();
    expect(result.current.deletingType).toBeNull();

    // Toggle add modal
    act(() => {
      result.current.setIsAddingType(true);
    });
    expect(result.current.isAddingType).toBe(true);

    // Set editing type
    const mockType = { 
      id: "1", 
      code: "AL", 
      name: "Annual Leave", 
      is_paid: true, 
      policy_config: {},
      created_at: new Date().toISOString(),
      workspace_id: "test-ws-id"
    };
    act(() => {
      result.current.setEditingType(mockType);
    });
    expect(result.current.editingType).toEqual(mockType);

    // Set deleting type
    act(() => {
      result.current.setDeletingType(mockType);
    });
    expect(result.current.deletingType).toEqual(mockType);
  });

  it("should close both modals on successful restore (replaces the create intent)", async () => {
    const { result } = renderHook(() => useLeaveTypeManager(), { wrapper });

    // Simulate the state right before restoring: user is on the Add modal
    // and the structured 409 has surfaced the Restore dialog.
    act(() => {
      result.current.setIsAddingType(true);
    });
    act(() => {
      result.current.setRestoringType({ leave_type_id: "retired-1", name: "Annual Leave" });
    });
    expect(result.current.isAddingType).toBe(true);
    expect(result.current.restoringType).not.toBeNull();

    vi.mocked(api.restoreLeaveType).mockResolvedValue({} as never);

    await act(async () => {
      result.current.restoreLeaveType("retired-1");
      await Promise.resolve();
    });

    // Both modals must close so the user returns to a clean list view.
    expect(result.current.isAddingType).toBe(false);
    expect(result.current.restoringType).toBeNull();
    expect(api.restoreLeaveType).toHaveBeenCalledWith("retired-1");
  });
});
