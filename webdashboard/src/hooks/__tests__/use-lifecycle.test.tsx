import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach, type Mock } from "vitest";

import {
  useLifecycleTemplates,
  useCreateLifecycleTemplate,
  useUpdateLifecycleTemplate,
  useDeleteLifecycleTemplate,
  useLifecycleTemplateItems,
  useCreateLifecycleTemplateItem,
  useUpdateLifecycleTemplateItem,
  useDeleteLifecycleTemplateItem,
  useLifecycleChecklists,
  useCreateLifecycleChecklist,
  useUpdateLifecycleChecklistItem,
  useCompletePendingTask,
  useMyPendingTasks,
  lifecycleKeys,
} from "../use-lifecycle";

import { useWorkspace } from "@/hooks/use-workspace";

const mockUseWorkspace = vi.hoisted(() => vi.fn());

vi.mock("@/hooks/use-workspace", () => ({
  useWorkspace: mockUseWorkspace,
}));

const api = vi.hoisted(() => ({
  fetchLifecycleTemplates: vi.fn(),
  createLifecycleTemplate: vi.fn(),
  updateLifecycleTemplate: vi.fn(),
  deleteLifecycleTemplate: vi.fn(),
  fetchLifecycleTemplateItems: vi.fn(),
  createLifecycleTemplateItem: vi.fn(),
  updateLifecycleTemplateItem: vi.fn(),
  deleteLifecycleTemplateItem: vi.fn(),
  fetchLifecycleChecklists: vi.fn(),
  createLifecycleChecklist: vi.fn(),
  updateChecklistItemStatus: vi.fn(),
  fetchMyPendingTasks: vi.fn(),
}));

vi.mock("@/lib/api/lifecycle", () => api);

const WS_ID = "ws-test-123";
const withWorkspace = { workspace: { id: WS_ID }, isLoading: false, isError: false };
const withoutWorkspace = { workspace: undefined, isLoading: false, isError: false };

let queryClient: QueryClient;

function createWrapper() {
  queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  const Wrapper = ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
  Wrapper.displayName = "QueryClientWrapper";
  return Wrapper;
}

describe("lifecycleKeys", () => {
  it("generates stable query keys for all lifecycle domains", () => {
    expect(lifecycleKeys.all).toEqual(["lifecycle"]);
    expect(lifecycleKeys.templates("ws-1")).toEqual(["lifecycle", "ws-1", "templates"]);
    expect(lifecycleKeys.templateItems("ws-1", "tpl-1")).toEqual([
      "lifecycle",
      "ws-1",
      "templates",
      "tpl-1",
      "items",
    ]);
    expect(lifecycleKeys.checklists("ws-1", "emp-1")).toEqual([
      "lifecycle",
      "ws-1",
      "checklists",
      "emp-1",
    ]);
    expect(lifecycleKeys.checklist("ws-1", "id-1")).toEqual([
      "lifecycle",
      "ws-1",
      "checklist",
      "id-1",
    ]);
    expect(lifecycleKeys.pendingTasks("ws-1")).toEqual([
      "lifecycle",
      "ws-1",
      "pending-tasks",
    ]);
  });
});

describe("useLifecycleTemplates", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUseWorkspace.mockReturnValue(withWorkspace);
    (api.fetchLifecycleTemplates as Mock).mockResolvedValue([{ id: "tpl-1", name: "Onboarding" }]);
  });

  it("fetches templates for the active workspace", async () => {
    const { result } = renderHook(() => useLifecycleTemplates(), { wrapper: createWrapper() });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    expect(api.fetchLifecycleTemplates).toHaveBeenCalledWith(WS_ID);
    expect(result.current.data).toEqual([{ id: "tpl-1", name: "Onboarding" }]);
  });

  it("stays disabled and does not fetch without a workspace", () => {
    mockUseWorkspace.mockReturnValue(withoutWorkspace);

    const { result } = renderHook(() => useLifecycleTemplates(), { wrapper: createWrapper() });

    expect(result.current.isFetching).toBe(false);
    expect(api.fetchLifecycleTemplates).not.toHaveBeenCalled();
  });
});

describe("useCreateLifecycleTemplate", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUseWorkspace.mockReturnValue(withWorkspace);
    (api.createLifecycleTemplate as Mock).mockResolvedValue({ id: "tpl-new", name: "Offboarding" });
  });

  it("creates a template with the workspace id and invalidates all lifecycle queries", async () => {
    const { result } = renderHook(() => useCreateLifecycleTemplate(), { wrapper: createWrapper() });
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    const payload = { name: "Offboarding" };
    await act(async () => {
      await result.current.mutateAsync(payload);
    });

    expect(api.createLifecycleTemplate).toHaveBeenCalledWith(WS_ID, payload);
    await waitFor(() => {
      expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: lifecycleKeys.all });
    });
  });

  it("rejects with an explicit error when no workspace is active", async () => {
    mockUseWorkspace.mockReturnValue(withoutWorkspace);

    const { result } = renderHook(() => useCreateLifecycleTemplate(), { wrapper: createWrapper() });

    await expect(result.current.mutateAsync({ name: "x" } as never)).rejects.toThrow("No workspace ID");
    expect(api.createLifecycleTemplate).not.toHaveBeenCalled();
  });
});

describe("useUpdateLifecycleTemplate", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUseWorkspace.mockReturnValue(withWorkspace);
    (api.updateLifecycleTemplate as Mock).mockResolvedValue({ id: "tpl-1", name: "Updated" });
  });

  it("updates a template and invalidates the lifecycle cache", async () => {
    const { result } = renderHook(() => useUpdateLifecycleTemplate(), { wrapper: createWrapper() });
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    await act(async () => {
      await result.current.mutateAsync({ id: "tpl-1", payload: { name: "Updated" } });
    });

    expect(api.updateLifecycleTemplate).toHaveBeenCalledWith(WS_ID, "tpl-1", { name: "Updated" });
    await waitFor(() => {
      expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: lifecycleKeys.all });
    });
  });

  it("rejects without a workspace", async () => {
    mockUseWorkspace.mockReturnValue(withoutWorkspace);

    const { result } = renderHook(() => useUpdateLifecycleTemplate(), { wrapper: createWrapper() });

    await expect(
      result.current.mutateAsync({ id: "tpl-1", payload: { name: "x" } }),
    ).rejects.toThrow("No workspace ID");
  });
});

describe("useDeleteLifecycleTemplate", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUseWorkspace.mockReturnValue(withWorkspace);
    (api.deleteLifecycleTemplate as Mock).mockResolvedValue(undefined);
  });

  it("deletes a template and invalidates the lifecycle cache", async () => {
    const { result } = renderHook(() => useDeleteLifecycleTemplate(), { wrapper: createWrapper() });
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    await act(async () => {
      await result.current.mutateAsync("tpl-1");
    });

    expect(api.deleteLifecycleTemplate).toHaveBeenCalledWith(WS_ID, "tpl-1");
    await waitFor(() => {
      expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: lifecycleKeys.all });
    });
  });

  it("rejects without a workspace", async () => {
    mockUseWorkspace.mockReturnValue(withoutWorkspace);

    const { result } = renderHook(() => useDeleteLifecycleTemplate(), { wrapper: createWrapper() });

    await expect(result.current.mutateAsync("tpl-1")).rejects.toThrow("No workspace ID");
  });
});

describe("useLifecycleTemplateItems", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUseWorkspace.mockReturnValue(withWorkspace);
    (api.fetchLifecycleTemplateItems as Mock).mockResolvedValue([{ id: "item-1", title: "Verify ID" }]);
  });

  it("fetches items for a template", async () => {
    const { result } = renderHook(() => useLifecycleTemplateItems("tpl-1"), { wrapper: createWrapper() });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    expect(api.fetchLifecycleTemplateItems).toHaveBeenCalledWith(WS_ID, "tpl-1");
    expect(result.current.data).toHaveLength(1);
  });

  it("does not fetch when no template id is provided", () => {
    const { result } = renderHook(() => useLifecycleTemplateItems(""), { wrapper: createWrapper() });

    expect(result.current.isFetching).toBe(false);
    expect(api.fetchLifecycleTemplateItems).not.toHaveBeenCalled();
  });
});

describe("template item mutations", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUseWorkspace.mockReturnValue(withWorkspace);
  });

  it("creates a template item and invalidates the lifecycle cache", async () => {
    (api.createLifecycleTemplateItem as Mock).mockResolvedValue({ id: "item-new", title: "Collect docs" });
    const { result } = renderHook(() => useCreateLifecycleTemplateItem("tpl-1"), { wrapper: createWrapper() });
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    await act(async () => {
      await result.current.mutateAsync({ title: "Collect docs" });
    });

    expect(api.createLifecycleTemplateItem).toHaveBeenCalledWith(WS_ID, "tpl-1", { title: "Collect docs" });
    await waitFor(() => {
      expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: lifecycleKeys.all });
    });
  });

  it("updates a template item and invalidates the lifecycle cache", async () => {
    (api.updateLifecycleTemplateItem as Mock).mockResolvedValue({ id: "item-1", title: "Done" });
    const { result } = renderHook(() => useUpdateLifecycleTemplateItem("tpl-1"), { wrapper: createWrapper() });
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    await act(async () => {
      await result.current.mutateAsync({ id: "item-1", payload: { title: "Done" } });
    });

    expect(api.updateLifecycleTemplateItem).toHaveBeenCalledWith(WS_ID, "tpl-1", "item-1", { title: "Done" });
    await waitFor(() => {
      expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: lifecycleKeys.all });
    });
  });

  it("deletes a template item and invalidates the lifecycle cache", async () => {
    (api.deleteLifecycleTemplateItem as Mock).mockResolvedValue(undefined);
    const { result } = renderHook(() => useDeleteLifecycleTemplateItem("tpl-1"), { wrapper: createWrapper() });
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    await act(async () => {
      await result.current.mutateAsync("item-1");
    });

    expect(api.deleteLifecycleTemplateItem).toHaveBeenCalledWith(WS_ID, "tpl-1", "item-1");
    await waitFor(() => {
      expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: lifecycleKeys.all });
    });
  });

  it("rejects item mutations when no workspace is active", async () => {
    mockUseWorkspace.mockReturnValue(withoutWorkspace);

    const create = renderHook(() => useCreateLifecycleTemplateItem("tpl-1"), { wrapper: createWrapper() });
    await expect(create.result.current.mutateAsync({ title: "x" })).rejects.toThrow("No workspace ID");

    const update = renderHook(() => useUpdateLifecycleTemplateItem("tpl-1"), { wrapper: createWrapper() });
    await expect(
      update.result.current.mutateAsync({ id: "item-1", payload: { title: "x" } }),
    ).rejects.toThrow("No workspace ID");

    const del = renderHook(() => useDeleteLifecycleTemplateItem("tpl-1"), { wrapper: createWrapper() });
    await expect(del.result.current.mutateAsync("item-1")).rejects.toThrow("No workspace ID");
  });
});

describe("useLifecycleChecklists", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUseWorkspace.mockReturnValue(withWorkspace);
    (api.fetchLifecycleChecklists as Mock).mockResolvedValue([{ id: "cl-1", template_name: "Onboarding" }]);
  });

  it("fetches checklists for an employment", async () => {
    const { result } = renderHook(() => useLifecycleChecklists("emp-1"), { wrapper: createWrapper() });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    expect(api.fetchLifecycleChecklists).toHaveBeenCalledWith(WS_ID, "emp-1");
    expect(result.current.data).toHaveLength(1);
  });

  it("does not fetch without an employment id", () => {
    const { result } = renderHook(() => useLifecycleChecklists(""), { wrapper: createWrapper() });

    expect(result.current.isFetching).toBe(false);
    expect(api.fetchLifecycleChecklists).not.toHaveBeenCalled();
  });
});

describe("useCreateLifecycleChecklist", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUseWorkspace.mockReturnValue(withWorkspace);
    (api.createLifecycleChecklist as Mock).mockResolvedValue({ id: "cl-new", template_name: "Onboarding" });
  });

  it("creates a checklist and invalidates that employment's checklist cache", async () => {
    const { result } = renderHook(() => useCreateLifecycleChecklist("emp-1"), { wrapper: createWrapper() });
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    const payload = { template_id: "tpl-1" };
    await act(async () => {
      await result.current.mutateAsync(payload);
    });

    expect(api.createLifecycleChecklist).toHaveBeenCalledWith(WS_ID, "emp-1", payload);
    await waitFor(() => {
      expect(invalidateSpy).toHaveBeenCalledWith({
        queryKey: lifecycleKeys.checklists(WS_ID, "emp-1"),
      });
    });
  });

  it("rejects without a workspace", async () => {
    mockUseWorkspace.mockReturnValue(withoutWorkspace);

    const { result } = renderHook(() => useCreateLifecycleChecklist("emp-1"), { wrapper: createWrapper() });

    await expect(result.current.mutateAsync({ template_id: "tpl-1" } as never)).rejects.toThrow(
      "No workspace ID",
    );
  });
});

describe("useUpdateLifecycleChecklistItem", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUseWorkspace.mockReturnValue(withWorkspace);
    (api.updateChecklistItemStatus as Mock).mockResolvedValue({ id: "cl-1", status: "in_progress" });
  });

  it("updates an item status and invalidates checklists plus pending tasks", async () => {
    const { result } = renderHook(() => useUpdateLifecycleChecklistItem("cl-1", "emp-1"), {
      wrapper: createWrapper(),
    });
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    const payload = { status: "in_progress" };
    await act(async () => {
      await result.current.mutateAsync({ itemId: "item-1", payload });
    });

    expect(api.updateChecklistItemStatus).toHaveBeenCalledWith(WS_ID, "cl-1", "item-1", payload);
    await waitFor(() => {
      expect(invalidateSpy).toHaveBeenCalledWith({
        queryKey: lifecycleKeys.checklists(WS_ID, "emp-1"),
      });
      expect(invalidateSpy).toHaveBeenCalledWith({
        queryKey: lifecycleKeys.pendingTasks(WS_ID),
      });
    });
  });

  it("rejects without a workspace", async () => {
    mockUseWorkspace.mockReturnValue(withoutWorkspace);

    const { result } = renderHook(() => useUpdateLifecycleChecklistItem("cl-1", "emp-1"), {
      wrapper: createWrapper(),
    });

    await expect(
      result.current.mutateAsync({ itemId: "item-1", payload: { status: "done" } }),
    ).rejects.toThrow("No workspace ID");
  });
});

describe("useCompletePendingTask", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUseWorkspace.mockReturnValue(withWorkspace);
    (api.updateChecklistItemStatus as Mock).mockResolvedValue({ id: "cl-1", status: "completed" });
  });

  it("completes a pending task and invalidates all lifecycle queries", async () => {
    const { result } = renderHook(() => useCompletePendingTask(), { wrapper: createWrapper() });
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    const payload = { status: "completed" };
    await act(async () => {
      await result.current.mutateAsync({ checklistId: "cl-1", itemId: "item-1", payload });
    });

    expect(api.updateChecklistItemStatus).toHaveBeenCalledWith(WS_ID, "cl-1", "item-1", payload);
    await waitFor(() => {
      expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: lifecycleKeys.all });
    });
  });

  it("rejects without a workspace", async () => {
    mockUseWorkspace.mockReturnValue(withoutWorkspace);

    const { result } = renderHook(() => useCompletePendingTask(), { wrapper: createWrapper() });

    await expect(
      result.current.mutateAsync({ checklistId: "cl-1", itemId: "item-1", payload: { status: "done" } }),
    ).rejects.toThrow("No workspace ID");
  });
});

describe("useMyPendingTasks", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUseWorkspace.mockReturnValue(withWorkspace);
    (api.fetchMyPendingTasks as Mock).mockResolvedValue([{ id: "task-1", title: "Verify documents" }]);
  });

  it("fetches the dashboard pending tasks", async () => {
    const { result } = renderHook(() => useMyPendingTasks(), { wrapper: createWrapper() });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    expect(api.fetchMyPendingTasks).toHaveBeenCalledWith(WS_ID);
    expect(result.current.data).toHaveLength(1);
  });

  it("does not fetch without a workspace", () => {
    mockUseWorkspace.mockReturnValue(withoutWorkspace);

    const { result } = renderHook(() => useMyPendingTasks(), { wrapper: createWrapper() });

    expect(result.current.isFetching).toBe(false);
    expect(api.fetchMyPendingTasks).not.toHaveBeenCalled();
  });
});
