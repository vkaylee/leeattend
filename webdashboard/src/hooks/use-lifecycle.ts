import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";

import type {
  CreateLifecycleChecklistTemplateRequest,
  UpdateLifecycleChecklistTemplateRequest,
  CreateLifecycleChecklistTemplateItemRequest,
  UpdateLifecycleChecklistTemplateItemRequest,
  CreateChecklistRequest,
  UpdateChecklistItemStatusRequest,
} from "@/lib/api/types";

import { useWorkspace } from "@/hooks/use-workspace";
import {
  fetchLifecycleTemplates,
  createLifecycleTemplate,
  getLifecycleTemplate,
  updateLifecycleTemplate,
  deleteLifecycleTemplate,
  fetchLifecycleTemplateItems,
  createLifecycleTemplateItem,
  getLifecycleTemplateItem,
  updateLifecycleTemplateItem,
  deleteLifecycleTemplateItem,
  fetchLifecycleChecklists,
  createLifecycleChecklist,
  getLifecycleChecklist,
  updateChecklistItemStatus,
  fetchMyPendingTasks,
} from "@/lib/api/lifecycle";

export const lifecycleKeys = {
  all: ["lifecycle"] as const,
  templates: (workspaceId: string) => [...lifecycleKeys.all, workspaceId, "templates"] as const,
  templateItems: (workspaceId: string, templateId: string) =>
    [...lifecycleKeys.templates(workspaceId), templateId, "items"] as const,
  checklists: (workspaceId: string, employmentId: string) =>
    [...lifecycleKeys.all, workspaceId, "checklists", employmentId] as const,
  checklist: (workspaceId: string, id: string) => [...lifecycleKeys.all, workspaceId, "checklist", id] as const,
  pendingTasks: (workspaceId: string) => [...lifecycleKeys.all, workspaceId, "pending-tasks"] as const,
};

// Templates Hooks
export const useLifecycleTemplates = () => {
  const { workspace } = useWorkspace();
  const workspaceId = workspace?.id;

  return useQuery({
    queryKey: lifecycleKeys.templates(workspaceId ?? ""),
    queryFn: () => {
      if (!workspaceId) throw new Error("No workspace ID");
      return fetchLifecycleTemplates(workspaceId);
    },
    enabled: !!workspaceId,
  });
};

export const useCreateLifecycleTemplate = () => {
  const queryClient = useQueryClient();
  const { workspace } = useWorkspace();

  return useMutation({
    mutationFn: (payload: CreateLifecycleChecklistTemplateRequest) => {
      const workspaceId = workspace?.id;
      if (!workspaceId) throw new Error("No workspace ID");
      return createLifecycleTemplate(workspaceId, payload);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: lifecycleKeys.all });
    },
  });
};

export const useUpdateLifecycleTemplate = () => {
  const queryClient = useQueryClient();
  const { workspace } = useWorkspace();

  return useMutation({
    mutationFn: ({ id, payload }: { id: string; payload: UpdateLifecycleChecklistTemplateRequest }) => {
      const workspaceId = workspace?.id;
      if (!workspaceId) throw new Error("No workspace ID");
      return updateLifecycleTemplate(workspaceId, id, payload);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: lifecycleKeys.all });
    },
  });
};

export const useDeleteLifecycleTemplate = () => {
  const queryClient = useQueryClient();
  const { workspace } = useWorkspace();

  return useMutation({
    mutationFn: (id: string) => {
      const workspaceId = workspace?.id;
      if (!workspaceId) throw new Error("No workspace ID");
      return deleteLifecycleTemplate(workspaceId, id);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: lifecycleKeys.all });
    },
  });
};

// Template Items Hooks
export const useLifecycleTemplateItems = (templateId: string) => {
  const { workspace } = useWorkspace();
  const workspaceId = workspace?.id;

  return useQuery({
    queryKey: lifecycleKeys.templateItems(workspaceId ?? "", templateId),
    queryFn: () => {
      if (!workspaceId) throw new Error("No workspace ID");
      return fetchLifecycleTemplateItems(workspaceId, templateId);
    },
    enabled: !!workspaceId && !!templateId,
  });
};

export const useCreateLifecycleTemplateItem = (templateId: string) => {
  const queryClient = useQueryClient();
  const { workspace } = useWorkspace();

  return useMutation({
    mutationFn: (payload: CreateLifecycleChecklistTemplateItemRequest) => {
      const workspaceId = workspace?.id;
      if (!workspaceId) throw new Error("No workspace ID");
      return createLifecycleTemplateItem(workspaceId, templateId, payload);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: lifecycleKeys.all });
    },
  });
};

export const useUpdateLifecycleTemplateItem = (templateId: string) => {
  const queryClient = useQueryClient();
  const { workspace } = useWorkspace();

  return useMutation({
    mutationFn: ({ id, payload }: { id: string; payload: UpdateLifecycleChecklistTemplateItemRequest }) => {
      const workspaceId = workspace?.id;
      if (!workspaceId) throw new Error("No workspace ID");
      return updateLifecycleTemplateItem(workspaceId, templateId, id, payload);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: lifecycleKeys.all });
    },
  });
};

export const useDeleteLifecycleTemplateItem = (templateId: string) => {
  const queryClient = useQueryClient();
  const { workspace } = useWorkspace();

  return useMutation({
    mutationFn: (id: string) => {
      const workspaceId = workspace?.id;
      if (!workspaceId) throw new Error("No workspace ID");
      return deleteLifecycleTemplateItem(workspaceId, templateId, id);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: lifecycleKeys.all });
    },
  });
};

// Checklist Instances Hooks
export const useLifecycleChecklists = (employmentId: string) => {
  const { workspace } = useWorkspace();
  const workspaceId = workspace?.id;

  return useQuery({
    queryKey: lifecycleKeys.checklists(workspaceId ?? "", employmentId),
    queryFn: () => {
      if (!workspaceId) throw new Error("No workspace ID");
      return fetchLifecycleChecklists(workspaceId, employmentId);
    },
    enabled: !!workspaceId && !!employmentId,
  });
};

export const useCreateLifecycleChecklist = (employmentId: string) => {
  const queryClient = useQueryClient();
  const { workspace } = useWorkspace();

  return useMutation({
    mutationFn: (payload: CreateChecklistRequest) => {
      const workspaceId = workspace?.id;
      if (!workspaceId) throw new Error("No workspace ID");
      return createLifecycleChecklist(workspaceId, employmentId, payload);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({
        queryKey: lifecycleKeys.checklists(workspace?.id ?? "", employmentId),
      });
    },
  });
};

export const useUpdateLifecycleChecklistItem = (checklistId: string, employmentId: string) => {
  const queryClient = useQueryClient();
  const { workspace } = useWorkspace();

  return useMutation({
    mutationFn: ({ itemId, payload }: { itemId: string; payload: UpdateChecklistItemStatusRequest }) => {
      const workspaceId = workspace?.id;
      if (!workspaceId) throw new Error("No workspace ID");
      return updateChecklistItemStatus(workspaceId, checklistId, itemId, payload);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({
        queryKey: lifecycleKeys.checklists(workspace?.id ?? "", employmentId),
      });
      void queryClient.invalidateQueries({
        queryKey: lifecycleKeys.pendingTasks(workspace?.id ?? ""),
      });
    },
  });
};

export const useCompletePendingTask = () => {
  const queryClient = useQueryClient();
  const { workspace } = useWorkspace();

  return useMutation({
    mutationFn: ({
      checklistId,
      itemId,
      payload,
    }: {
      checklistId: string;
      itemId: string;
      payload: UpdateChecklistItemStatusRequest;
    }) => {
      const workspaceId = workspace?.id;
      if (!workspaceId) throw new Error("No workspace ID");
      return updateChecklistItemStatus(workspaceId, checklistId, itemId, payload);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: lifecycleKeys.all });
    },
  });
};


// Dashboard Pending Tasks Hooks
export const useMyPendingTasks = () => {
  const { workspace } = useWorkspace();
  const workspaceId = workspace?.id;

  return useQuery({
    queryKey: lifecycleKeys.pendingTasks(workspaceId ?? ""),
    queryFn: () => {
      if (!workspaceId) throw new Error("No workspace ID");
      return fetchMyPendingTasks(workspaceId);
    },
    enabled: !!workspaceId,
  });
};
