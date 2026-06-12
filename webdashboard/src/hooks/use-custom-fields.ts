import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";

import type {
  CreateCustomFieldDefinitionRequest,
  SetCustomFieldValueRequest,
  UpdateCustomFieldDefinitionRequest,
} from "@/lib/api/types";

import { useWorkspace } from "@/hooks/use-workspace";
import {
  createCustomFieldDefinition,
  deleteCustomFieldDefinition,
  fetchCustomFieldDefinitions,
  fetchCustomFieldValues,
  setCustomFieldValues,
  updateCustomFieldDefinition,
} from "@/lib/api/custom-fields";

export const customFieldKeys = {
  all: ["custom-fields"] as const,
  definitions: (workspaceId: string) => [...customFieldKeys.all, workspaceId, "definitions"] as const,
  definitionList: (workspaceId: string, entityType?: string) =>
    [...customFieldKeys.definitions(workspaceId), { entityType }] as const,
  values: (workspaceId: string, entityType: string, entityId: string) =>
    [...customFieldKeys.all, workspaceId, "values", entityType, entityId] as const,
};

export const useCustomFieldDefinitions = (entityType?: string) => {
  const { workspace } = useWorkspace();
  const workspaceId = workspace?.id;

  return useQuery({
    queryKey: customFieldKeys.definitionList(workspaceId ?? "", entityType),
    queryFn: () => {
      if (!workspaceId) throw new Error("No workspace ID");
      return fetchCustomFieldDefinitions(workspaceId, entityType);
    },
    enabled: !!workspaceId,
  });
};

export const useCreateCustomFieldDefinition = () => {
  const queryClient = useQueryClient();
  const { workspace } = useWorkspace();

  return useMutation({
    mutationFn: (payload: CreateCustomFieldDefinitionRequest) => {
      const workspaceId = workspace?.id;
      if (!workspaceId) throw new Error("No workspace ID");
      return createCustomFieldDefinition(workspaceId, payload);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: customFieldKeys.all });
    },
  });
};

export const useUpdateCustomFieldDefinition = () => {
  const queryClient = useQueryClient();
  const { workspace } = useWorkspace();

  return useMutation({
    mutationFn: ({ id, payload }: { id: string; payload: UpdateCustomFieldDefinitionRequest }) => {
      const workspaceId = workspace?.id;
      if (!workspaceId) throw new Error("No workspace ID");
      return updateCustomFieldDefinition(workspaceId, id, payload);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: customFieldKeys.all });
    },
  });
};

export const useDeleteCustomFieldDefinition = () => {
  const queryClient = useQueryClient();
  const { workspace } = useWorkspace();

  return useMutation({
    mutationFn: (id: string) => {
      const workspaceId = workspace?.id;
      if (!workspaceId) throw new Error("No workspace ID");
      return deleteCustomFieldDefinition(workspaceId, id);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: customFieldKeys.all });
    },
  });
};

export const useCustomFieldValues = (entityType: string, entityId: string) => {
  const { workspace } = useWorkspace();
  const workspaceId = workspace?.id;

  return useQuery({
    queryKey: customFieldKeys.values(workspaceId ?? "", entityType, entityId),
    queryFn: () => {
      if (!workspaceId) throw new Error("No workspace ID");
      return fetchCustomFieldValues(workspaceId, entityType, entityId);
    },
    enabled: !!workspaceId && !!entityType && !!entityId,
  });
};

export const useSetCustomFieldValues = (entityType: string, entityId: string) => {
  const queryClient = useQueryClient();
  const { workspace } = useWorkspace();

  return useMutation({
    mutationFn: (values: SetCustomFieldValueRequest[]) => {
      const workspaceId = workspace?.id;
      if (!workspaceId) throw new Error("No workspace ID");
      return setCustomFieldValues(workspaceId, entityType, entityId, values);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({
        queryKey: customFieldKeys.values(workspace?.id ?? "", entityType, entityId),
      });
      // Also invalidate workspace entity queries to make sure employee list/details refresh
      void queryClient.invalidateQueries({ queryKey: ["employees"] });
      void queryClient.invalidateQueries({ queryKey: ["departments"] });
    },
  });
};
