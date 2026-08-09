import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, act, waitFor } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import type { Department } from "@/lib/api";

import { useDepartmentForm } from "../../hooks/use-department-form";

// ── Mock API functions ─────────────────────────────────────────────────────
const mockCreateDepartment = vi.fn();
const mockUpdateDepartment = vi.fn();

vi.mock("@/lib/api", () => ({
  createDepartment: (...args: unknown[]) => mockCreateDepartment(...args),
  updateDepartment: (...args: unknown[]) => mockUpdateDepartment(...args),
}));

// ── Mock toast ─────────────────────────────────────────────────────────────
const mockToastSuccess = vi.fn();
const mockToastError = vi.fn();

vi.mock("@/components/ui/toast", () => ({
  useToast: () => ({
    success: mockToastSuccess,
    error: mockToastError,
    info: vi.fn(),
    warning: vi.fn(),
  }),
}));

// ── Helpers ────────────────────────────────────────────────────────────────

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  const Wrapper = ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
  Wrapper.displayName = "QueryClientWrapper";
  return Wrapper;
}

function mockT(key: string, defaultValue?: string): string {
  return defaultValue ?? key;
}

const MOCK_DEPARTMENT: Department = {
  id: "dept-1",
  name: "Engineering",
  description: "Engineering department",
  manager_id: "emp-1",
  workspace_id: "ws-test-123",
  created_at: "2026-01-01T00:00:00Z",
  updated_at: "2026-01-01T00:00:00Z",
};

beforeEach(() => {
  vi.clearAllMocks();
  localStorage.clear();
  localStorage.setItem("active_workspace_id", "ws-test-123");
  mockCreateDepartment.mockResolvedValue({ id: "dept-new", name: "New Dept" });
  mockUpdateDepartment.mockResolvedValue({ ...MOCK_DEPARTMENT, name: "Updated" });
});

// ── Tests ─────────────────────────────────────────────────────────────────

describe("useDepartmentForm", () => {
  // ── Create mode (no editingDept) ────────────────────────────────────────

  it("initializes with empty manager and parent when creating", () => {
    const { result } = renderHook(
      () => useDepartmentForm("ws-test-123", null, vi.fn(), mockT),
      { wrapper: createWrapper() },
    );

    expect(result.current.selectedManagerId).toBe("");
    expect(result.current.selectedParentId).toBe("");
    expect(result.current.isLoading).toBe(false);
  });

  it("calls createDepartment on form submit in create mode", async () => {
    const onClose = vi.fn();
    const { result } = renderHook(
      () => useDepartmentForm("ws-test-123", null, onClose, mockT),
      { wrapper: createWrapper() },
    );

    // Create a mock form element
    const form = document.createElement("form");
    const nameInput = document.createElement("input");
    nameInput.name = "name";
    nameInput.value = "New Engineering";
    const descInput = document.createElement("input");
    descInput.name = "description";
    descInput.value = "A new department";
    form.appendChild(nameInput);
    form.appendChild(descInput);

    act(() => {
      result.current.handleSubmit({ preventDefault: vi.fn(), currentTarget: form } as unknown as React.SyntheticEvent<HTMLFormElement>);
    });

    await waitFor(() => {
      expect(mockCreateDepartment).toHaveBeenCalledWith("ws-test-123", {
        name: "New Engineering",
        description: "A new department",
        manager_id: null,
      });
    });
  });

  it("shows success toast and calls onClose after create", async () => {
    const onClose = vi.fn();
    const { result } = renderHook(
      () => useDepartmentForm("ws-test-123", null, onClose, mockT),
      { wrapper: createWrapper() },
    );

    const form = document.createElement("form");
    const nameInput = document.createElement("input");
    nameInput.name = "name";
    nameInput.value = "Team";
    form.appendChild(nameInput);
    const descInput = document.createElement("input");
    descInput.name = "description";
    descInput.value = "";
    form.appendChild(descInput);

    act(() => {
      result.current.handleSubmit({ preventDefault: vi.fn(), currentTarget: form } as unknown as React.SyntheticEvent<HTMLFormElement>);
    });

    await waitFor(() => {
      expect(mockToastSuccess).toHaveBeenCalledWith("common.success");
      expect(onClose).toHaveBeenCalled();
    });
  });

  it("saves parent ID to localStorage on create when parent is selected", async () => {
    const onClose = vi.fn();
    const { result } = renderHook(
      () => useDepartmentForm("ws-test-123", null, onClose, mockT),
      { wrapper: createWrapper() },
    );

    // Set a parent ID
    act(() => {
      result.current.setSelectedParentId("parent-dept-1");
    });

    mockCreateDepartment.mockResolvedValue({ id: "dept-new-123", name: "New Dept" });

    const form = document.createElement("form");
    const nameInput = document.createElement("input");
    nameInput.name = "name";
    nameInput.value = "Sub Team";
    form.appendChild(nameInput);
    const descInput = document.createElement("input");
    descInput.name = "description";
    descInput.value = "";
    form.appendChild(descInput);

    act(() => {
      result.current.handleSubmit({ preventDefault: vi.fn(), currentTarget: form } as unknown as React.SyntheticEvent<HTMLFormElement>);
    });

    await waitFor(() => {
      const saved = localStorage.getItem("leeattend_dept_parents_ws-test-123");
      expect(saved).toBeTruthy();
      const parsed = JSON.parse(saved!);
      expect(parsed["dept-new-123"]).toBe("parent-dept-1");
    });
  });

  // ── Update mode (editingDept provided) ─────────────────────────────────

  it("initializes with editingDept values in update mode", () => {
    const { result } = renderHook(
      () => useDepartmentForm("ws-test-123", MOCK_DEPARTMENT, vi.fn(), mockT),
      { wrapper: createWrapper() },
    );

    expect(result.current.selectedManagerId).toBe("emp-1");
  });

  it("calls updateDepartment on form submit in update mode", async () => {
    const onClose = vi.fn();
    const { result } = renderHook(
      () =>
        useDepartmentForm("ws-test-123", MOCK_DEPARTMENT, onClose, mockT),
      { wrapper: createWrapper() },
    );

    const form = document.createElement("form");
    const nameInput = document.createElement("input");
    nameInput.name = "name";
    nameInput.value = "Updated Engineering";
    form.appendChild(nameInput);
    const descInput = document.createElement("input");
    descInput.name = "description";
    descInput.value = "Updated desc";
    form.appendChild(descInput);

    act(() => {
      result.current.handleSubmit({
        preventDefault: vi.fn(),
        currentTarget: form,
      } as unknown as React.SyntheticEvent<HTMLFormElement>);
    });

    await waitFor(() => {
      expect(mockUpdateDepartment).toHaveBeenCalledWith(
        "ws-test-123",
        "dept-1",
        {
          name: "Updated Engineering",
          description: "Updated desc",
          manager_id: "emp-1",
        },
      );
    });
  });

  it("shows success toast and calls onClose after update", async () => {
    const onClose = vi.fn();
    const { result } = renderHook(
      () =>
        useDepartmentForm("ws-test-123", MOCK_DEPARTMENT, onClose, mockT),
      { wrapper: createWrapper() },
    );

    const form = document.createElement("form");
    const nameInput = document.createElement("input");
    nameInput.name = "name";
    nameInput.value = "Updated";
    form.appendChild(nameInput);
    const descInput = document.createElement("input");
    descInput.name = "description";
    descInput.value = "";
    form.appendChild(descInput);

    act(() => {
      result.current.handleSubmit({
        preventDefault: vi.fn(),
        currentTarget: form,
      } as unknown as React.SyntheticEvent<HTMLFormElement>);
    });

    await waitFor(() => {
      expect(mockToastSuccess).toHaveBeenCalledWith("common.success");
      expect(onClose).toHaveBeenCalled();
    });
  });

  // ── Form state management ───────────────────────────────────────────────

  it("allows updating selectedManagerId", () => {
    const { result } = renderHook(
      () => useDepartmentForm("ws-test-123", null, vi.fn(), mockT),
      { wrapper: createWrapper() },
    );

    act(() => {
      result.current.setSelectedManagerId("emp-42");
    });

    expect(result.current.selectedManagerId).toBe("emp-42");
  });

  it("allows updating selectedParentId", () => {
    const { result } = renderHook(
      () => useDepartmentForm("ws-test-123", null, vi.fn(), mockT),
      { wrapper: createWrapper() },
    );

    act(() => {
      result.current.setSelectedParentId("parent-dept-99");
    });

    expect(result.current.selectedParentId).toBe("parent-dept-99");
  });

  it("resets manager and parent when editingDept changes", () => {
    const { result, rerender } = renderHook(
      ({ editingDept }: { editingDept: Department | null }) =>
        useDepartmentForm("ws-test-123", editingDept, vi.fn(), mockT),
      {
        wrapper: createWrapper(),
        initialProps: { editingDept: null as Department | null },
      },
    );

    // Set some values
    act(() => {
      result.current.setSelectedManagerId("emp-x");
      result.current.setSelectedParentId("parent-x");
    });

    // Switch to editing a different department
    rerender({
      editingDept: {
        ...MOCK_DEPARTMENT,
        id: "dept-2",
        manager_id: "emp-99",
      },
    });

    expect(result.current.selectedManagerId).toBe("emp-99");
    expect(result.current.selectedParentId).toBe("");
  });

  // ── isLoading ───────────────────────────────────────────────────────────

  it("reports isLoading as false initially", () => {
    const { result } = renderHook(
      () => useDepartmentForm("ws-test-123", null, vi.fn(), mockT),
      { wrapper: createWrapper() },
    );

    expect(result.current.isLoading).toBe(false);
  });

  // ── handleSubmit prevents default ───────────────────────────────────────

  it("prevents default form submission", () => {
    const { result } = renderHook(
      () => useDepartmentForm("ws-test-123", null, vi.fn(), mockT),
      { wrapper: createWrapper() },
    );

    const preventDefault = vi.fn();
    const form = document.createElement("form");
    form.innerHTML = '<input name="name" value="Test" />';

    act(() => {
      result.current.handleSubmit({
        preventDefault,
        currentTarget: form,
      } as unknown as React.SyntheticEvent<HTMLFormElement>);
    });

    expect(preventDefault).toHaveBeenCalled();
  });

  // ── Manager ID null coercion ────────────────────────────────────────────

  it("sends manager_id as null when empty string", async () => {
    const { result } = renderHook(
      () => useDepartmentForm("ws-test-123", null, vi.fn(), mockT),
      { wrapper: createWrapper() },
    );

    // selectedManagerId defaults to "" — it should be coerced to null in the payload
    const form = document.createElement("form");
    form.innerHTML = '<input name="name" value="Test" /><input name="description" value="Desc" />';

    act(() => {
      result.current.handleSubmit({
        preventDefault: vi.fn(),
        currentTarget: form,
      } as unknown as React.SyntheticEvent<HTMLFormElement>);
    });

    await waitFor(() => {
      expect(mockCreateDepartment).toHaveBeenCalledWith("ws-test-123", {
        name: "Test",
        description: "Desc",
        manager_id: null,
      });
    });
  });

  // ── localStorage parent map: read on edit ───────────────────────────────

  it("reads parent ID from localStorage when editing", () => {
    localStorage.setItem(
      "leeattend_dept_parents_ws-test-123",
      JSON.stringify({ "dept-1": "parent-from-storage" }),
    );

    const { result } = renderHook(
      () => useDepartmentForm("ws-test-123", MOCK_DEPARTMENT, vi.fn(), mockT),
      { wrapper: createWrapper() },
    );

    expect(result.current.selectedParentId).toBe("parent-from-storage");
  });

  // ── localStorage: update removes parent when deselected ─────────────────

  it("removes parent from localStorage on update when parent is deselected", async () => {
    localStorage.setItem(
      "leeattend_dept_parents_ws-test-123",
      JSON.stringify({ "dept-1": "old-parent" }),
    );

    const onClose = vi.fn();
    const { result } = renderHook(
      () =>
        useDepartmentForm("ws-test-123", MOCK_DEPARTMENT, onClose, mockT),
      { wrapper: createWrapper() },
    );

    // Deselect the parent (set to empty)
    act(() => {
      result.current.setSelectedParentId("");
    });

    const form = document.createElement("form");
    form.innerHTML = '<input name="name" value="Updated" /><input name="description" value="" />';

    act(() => {
      result.current.handleSubmit({
        preventDefault: vi.fn(),
        currentTarget: form,
      } as unknown as React.SyntheticEvent<HTMLFormElement>);
    });

    await waitFor(() => {
      const saved = localStorage.getItem("leeattend_dept_parents_ws-test-123");
      if (saved) {
        const parsed = JSON.parse(saved);
        expect(parsed["dept-1"]).toBeUndefined();
      }
    });
  });

  // ── Error handling ──────────────────────────────────────────────────────

  it("does not crash when localStorage.getItem throws", () => {
    const originalGetItem = Storage.prototype.getItem;
    Storage.prototype.getItem = vi.fn(() => {
      throw new Error("storage error");
    });

    const { result } = renderHook(
      () => useDepartmentForm("ws-test-123", MOCK_DEPARTMENT, vi.fn(), mockT),
      { wrapper: createWrapper() },
    );

    // Should still work with default empty parent
    expect(result.current.selectedParentId).toBe("");

    Storage.prototype.getItem = originalGetItem;
  });

  it("does not crash when localStorage.setItem throws", async () => {
    const originalSetItem = Storage.prototype.setItem;
    Storage.prototype.setItem = vi.fn(() => {
      throw new Error("storage quota exceeded");
    });

    const onClose = vi.fn();
    const { result } = renderHook(
      () => useDepartmentForm("ws-test-123", null, onClose, mockT),
      { wrapper: createWrapper() },
    );

    act(() => {
      result.current.setSelectedParentId("parent-1");
    });

    mockCreateDepartment.mockResolvedValue({ id: "dept-new", name: "New" });

    const form = document.createElement("form");
    form.innerHTML = '<input name="name" value="Test" /><input name="description" value="" />';

    act(() => {
      result.current.handleSubmit({
        preventDefault: vi.fn(),
        currentTarget: form,
      } as unknown as React.SyntheticEvent<HTMLFormElement>);
    });

    // Should still complete without throwing
    await waitFor(() => {
      expect(onClose).toHaveBeenCalled();
      expect(mockToastSuccess).toHaveBeenCalled();
    });

    Storage.prototype.setItem = originalSetItem;
  });
});
