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
  useOrganizationLifecycleInstances,
  lifecycleKeys,
} from "../use-lifecycle";

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
  fetchLifecycleInstances: vi.fn(),
}));

vi.mock("@/lib/api/lifecycle", () => api);

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
    expect(lifecycleKeys.templates()).toEqual(["lifecycle", "templates"]);
    expect(lifecycleKeys.templateItems("tpl-1")).toEqual([
      "lifecycle",
      "templates",
      "tpl-1",
      "items",
    ]);
    expect(lifecycleKeys.checklists("emp-1")).toEqual([
      "lifecycle",
      "checklists",
      "emp-1",
    ]);
    expect(lifecycleKeys.checklist("id-1")).toEqual([
      "lifecycle",
      "checklist",
      "id-1",
    ]);
    expect(lifecycleKeys.pendingTasks()).toEqual([
      "lifecycle",
      "pending-tasks",
    ]);
    expect(lifecycleKeys.instances()).toEqual([
      "lifecycle",
      "instances",
    ]);
  });
});

describe("useLifecycleTemplates", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    (api.fetchLifecycleTemplates as Mock).mockResolvedValue([{ id: "tpl-1", name: "Onboarding" }]);
  });

  it("fetches templates", async () => {
    const { result } = renderHook(() => useLifecycleTemplates(), { wrapper: createWrapper() });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    expect(api.fetchLifecycleTemplates).toHaveBeenCalled();
    expect(result.current.data).toEqual([{ id: "tpl-1", name: "Onboarding" }]);
  });
});

describe("useCreateLifecycleTemplate", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    (api.createLifecycleTemplate as Mock).mockResolvedValue({ id: "tpl-new", name: "Offboarding" });
  });

  it("creates a template and invalidates the lifecycle cache", async () => {
    const { result } = renderHook(() => useCreateLifecycleTemplate(), { wrapper: createWrapper() });
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    const payload = {
      name: "Offboarding",
      type: "offboarding" as const,
      applicable_role: "engineer",
    };
    await act(async () => {
      await result.current.mutateAsync(payload);
    });

    expect(api.createLifecycleTemplate).toHaveBeenCalledWith(payload);
    await waitFor(() => {
      expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: lifecycleKeys.all });
    });
  });
});

describe("useUpdateLifecycleTemplate", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    (api.updateLifecycleTemplate as Mock).mockResolvedValue({ id: "tpl-1", name: "Updated" });
  });

  it("updates a template and invalidates the lifecycle cache", async () => {
    const { result } = renderHook(() => useUpdateLifecycleTemplate(), { wrapper: createWrapper() });
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    await act(async () => {
      await result.current.mutateAsync({ id: "tpl-1", payload: { name: "Updated" } });
    });

    expect(api.updateLifecycleTemplate).toHaveBeenCalledWith("tpl-1", { name: "Updated" });
    await waitFor(() => {
      expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: lifecycleKeys.all });
    });
  });
});

describe("useDeleteLifecycleTemplate", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    (api.deleteLifecycleTemplate as Mock).mockResolvedValue(undefined);
  });

  it("deletes a template and invalidates the lifecycle cache", async () => {
    const { result } = renderHook(() => useDeleteLifecycleTemplate(), { wrapper: createWrapper() });
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    await act(async () => {
      await result.current.mutateAsync("tpl-1");
    });

    expect(api.deleteLifecycleTemplate).toHaveBeenCalledWith("tpl-1");
    await waitFor(() => {
      expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: lifecycleKeys.all });
    });
  });
});

describe("useLifecycleTemplateItems", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    (api.fetchLifecycleTemplateItems as Mock).mockResolvedValue([{ id: "item-1", title: "Verify ID" }]);
  });

  it("fetches template items by template id", async () => {
    const { result } = renderHook(() => useLifecycleTemplateItems("tpl-1"), { wrapper: createWrapper() });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    expect(api.fetchLifecycleTemplateItems).toHaveBeenCalledWith("tpl-1");
    expect(result.current.data).toHaveLength(1);
  });

  it("does not fetch without a template id", () => {
    const { result } = renderHook(() => useLifecycleTemplateItems(""), { wrapper: createWrapper() });

    expect(result.current.isFetching).toBe(false);
    expect(api.fetchLifecycleTemplateItems).not.toHaveBeenCalled();
  });
});

describe("template item mutations", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("creates a template item and invalidates the lifecycle cache", async () => {
    (api.createLifecycleTemplateItem as Mock).mockResolvedValue({ id: "item-new" });
    const { result } = renderHook(() => useCreateLifecycleTemplateItem("tpl-1"), { wrapper: createWrapper() });
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    await act(async () => {
      await result.current.mutateAsync({ title: "Collect docs" });
    });

    expect(api.createLifecycleTemplateItem).toHaveBeenCalledWith("tpl-1", { title: "Collect docs" });
    await waitFor(() => {
      expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: lifecycleKeys.all });
    });
  });

  it("updates a template item and invalidates the lifecycle cache", async () => {
    (api.updateLifecycleTemplateItem as Mock).mockResolvedValue({ id: "item-1" });
    const { result } = renderHook(() => useUpdateLifecycleTemplateItem("tpl-1"), { wrapper: createWrapper() });
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    await act(async () => {
      await result.current.mutateAsync({ id: "item-1", payload: { title: "Done" } });
    });

    expect(api.updateLifecycleTemplateItem).toHaveBeenCalledWith("tpl-1", "item-1", { title: "Done" });
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

    expect(api.deleteLifecycleTemplateItem).toHaveBeenCalledWith("tpl-1", "item-1");
    await waitFor(() => {
      expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: lifecycleKeys.all });
    });
  });
});

describe("useLifecycleChecklists", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    (api.fetchLifecycleChecklists as Mock).mockResolvedValue([{ id: "cl-1", template_name: "Onboarding" }]);
  });

  it("fetches checklists for an employment id", async () => {
    const { result } = renderHook(() => useLifecycleChecklists("emp-1"), { wrapper: createWrapper() });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    expect(api.fetchLifecycleChecklists).toHaveBeenCalledWith("emp-1");
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
    (api.createLifecycleChecklist as Mock).mockResolvedValue({ id: "cl-new", template_name: "Onboarding" });
  });

  it("creates a checklist and invalidates checklist queries for that employment", async () => {
    const { result } = renderHook(() => useCreateLifecycleChecklist("emp-1"), { wrapper: createWrapper() });
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    const payload = { template_id: "tpl-1" };
    await act(async () => {
      await result.current.mutateAsync(payload);
    });

    expect(api.createLifecycleChecklist).toHaveBeenCalledWith("emp-1", payload);
    await waitFor(() => {
      expect(invalidateSpy).toHaveBeenCalledWith({
        queryKey: lifecycleKeys.checklists("emp-1"),
      });
    });
  });
});

describe("useUpdateLifecycleChecklistItem", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    (api.updateChecklistItemStatus as Mock).mockResolvedValue({ id: "cl-1", status: "in_progress" });
  });

  it("updates status and invalidates both checklist and pending tasks", async () => {
    const { result } = renderHook(() => useUpdateLifecycleChecklistItem("cl-1", "emp-1"), {
      wrapper: createWrapper(),
    });
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    const payload = { status: "in_progress" as const };
    await act(async () => {
      await result.current.mutateAsync({ itemId: "item-1", payload });
    });

    expect(api.updateChecklistItemStatus).toHaveBeenCalledWith("cl-1", "item-1", payload);
    await waitFor(() => {
      expect(invalidateSpy).toHaveBeenCalledWith({
        queryKey: lifecycleKeys.checklists("emp-1"),
      });
      expect(invalidateSpy).toHaveBeenCalledWith({
        queryKey: lifecycleKeys.pendingTasks(),
      });
    });
  });
});

describe("useCompletePendingTask", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    (api.updateChecklistItemStatus as Mock).mockResolvedValue({ id: "cl-1", status: "completed" });
  });

  it("completes a pending task and invalidates the lifecycle cache", async () => {
    const { result } = renderHook(() => useCompletePendingTask(), { wrapper: createWrapper() });
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    const payload = { status: "completed" as const };
    await act(async () => {
      await result.current.mutateAsync({ checklistId: "cl-1", itemId: "item-1", payload });
    });

    expect(api.updateChecklistItemStatus).toHaveBeenCalledWith("cl-1", "item-1", payload);
    await waitFor(() => {
      expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: lifecycleKeys.all });
    });
  });
});

describe("useMyPendingTasks", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    (api.fetchMyPendingTasks as Mock).mockResolvedValue([{ id: "task-1", title: "Verify documents" }]);
  });

  it("fetches pending tasks", async () => {
    const { result } = renderHook(() => useMyPendingTasks(), { wrapper: createWrapper() });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    expect(api.fetchMyPendingTasks).toHaveBeenCalled();
    expect(result.current.data).toHaveLength(1);
  });
});

describe("useOrganizationLifecycleInstances", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    (api.fetchLifecycleInstances as Mock).mockResolvedValue([{ id: "inst-1" }]);
  });

  it("fetches lifecycle instances", async () => {
    const { result } = renderHook(() => useOrganizationLifecycleInstances(), { wrapper: createWrapper() });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    expect(api.fetchLifecycleInstances).toHaveBeenCalled();
    expect(result.current.data).toHaveLength(1);
  });
});
