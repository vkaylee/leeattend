import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";

import type {
  CreateLifecycleChecklistTemplateRequest,
  UpdateLifecycleChecklistTemplateRequest,
  CreateLifecycleChecklistTemplateItemRequest,
  UpdateLifecycleChecklistTemplateItemRequest,
  CreateChecklistRequest,
  UpdateChecklistItemStatusRequest,
} from "@/lib/api/types";

import {
  fetchLifecycleTemplates,
  createLifecycleTemplate,
  updateLifecycleTemplate,
  deleteLifecycleTemplate,
  fetchLifecycleTemplateItems,
  createLifecycleTemplateItem,
  updateLifecycleTemplateItem,
  deleteLifecycleTemplateItem,
  fetchLifecycleChecklists,
  createLifecycleChecklist,
  updateChecklistItemStatus,
  fetchMyPendingTasks,
  fetchLifecycleInstances,
} from "@/lib/api/lifecycle";

export const lifecycleKeys = {
  all: ["lifecycle"] as const,
  templates: () => [...lifecycleKeys.all, "templates"] as const,
  templateItems: (templateId: string) => [...lifecycleKeys.templates(), templateId, "items"] as const,
  checklists: (employmentId: string) => [...lifecycleKeys.all, "checklists", employmentId] as const,
  checklist: (id: string) => [...lifecycleKeys.all, "checklist", id] as const,
  pendingTasks: () => [...lifecycleKeys.all, "pending-tasks"] as const,
  instances: () => [...lifecycleKeys.all, "instances"] as const,
};

// Templates Hooks
export const useLifecycleTemplates = () => {
  return useQuery({
    queryKey: lifecycleKeys.templates(),
    queryFn: () => fetchLifecycleTemplates(),
  });
};

export const useCreateLifecycleTemplate = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (payload: CreateLifecycleChecklistTemplateRequest) => createLifecycleTemplate(payload),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: lifecycleKeys.all });
    },
  });
};

export const useUpdateLifecycleTemplate = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ id, payload }: { id: string; payload: UpdateLifecycleChecklistTemplateRequest }) =>
      updateLifecycleTemplate(id, payload),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: lifecycleKeys.all });
    },
  });
};

export const useDeleteLifecycleTemplate = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: string) => deleteLifecycleTemplate(id),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: lifecycleKeys.all });
    },
  });
};

// Template Items Hooks
export const useLifecycleTemplateItems = (templateId: string) => {
  return useQuery({
    queryKey: lifecycleKeys.templateItems(templateId),
    queryFn: () => fetchLifecycleTemplateItems(templateId),
    enabled: !!templateId,
  });
};

export const useCreateLifecycleTemplateItem = (templateId: string) => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (payload: CreateLifecycleChecklistTemplateItemRequest) =>
      createLifecycleTemplateItem(templateId, payload),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: lifecycleKeys.all });
    },
  });
};

export const useUpdateLifecycleTemplateItem = (templateId: string) => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ id, payload }: { id: string; payload: UpdateLifecycleChecklistTemplateItemRequest }) =>
      updateLifecycleTemplateItem(templateId, id, payload),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: lifecycleKeys.all });
    },
  });
};

export const useDeleteLifecycleTemplateItem = (templateId: string) => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: string) => deleteLifecycleTemplateItem(templateId, id),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: lifecycleKeys.all });
    },
  });
};

// Checklist Instances Hooks
export const useLifecycleChecklists = (employmentId: string) => {
  return useQuery({
    queryKey: lifecycleKeys.checklists(employmentId),
    queryFn: () => fetchLifecycleChecklists(employmentId),
    enabled: !!employmentId,
  });
};

export const useCreateLifecycleChecklist = (employmentId: string) => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (payload: CreateChecklistRequest) => createLifecycleChecklist(employmentId, payload),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: lifecycleKeys.checklists(employmentId) });
    },
  });
};

export const useUpdateLifecycleChecklistItem = (checklistId: string, employmentId: string) => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ itemId, payload }: { itemId: string; payload: UpdateChecklistItemStatusRequest }) =>
      updateChecklistItemStatus(checklistId, itemId, payload),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: lifecycleKeys.checklists(employmentId) });
      void queryClient.invalidateQueries({ queryKey: lifecycleKeys.pendingTasks() });
    },
  });
};

export const useCompletePendingTask = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({
      checklistId,
      itemId,
      payload,
    }: {
      checklistId: string;
      itemId: string;
      payload: UpdateChecklistItemStatusRequest;
    }) => updateChecklistItemStatus(checklistId, itemId, payload),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: lifecycleKeys.all });
    },
  });
};


// Dashboard Pending Tasks Hooks
export const useMyPendingTasks = () => {
  return useQuery({
    queryKey: lifecycleKeys.pendingTasks(),
    queryFn: () => fetchMyPendingTasks(),
  });
};

export const useOrganizationLifecycleInstances = () => {
  return useQuery({
    queryKey: lifecycleKeys.instances(),
    queryFn: () => fetchLifecycleInstances(),
  });
};
