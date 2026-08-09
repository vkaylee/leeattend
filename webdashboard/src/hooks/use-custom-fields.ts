import type {
  CreateCustomFieldDefinitionRequest,
  SetCustomFieldValueRequest,
  UpdateCustomFieldDefinitionRequest,
} from "@/lib/api/types";

import { useWorkspaceQuery, useWorkspaceMutation } from "@/hooks/common/use-workspace-query";
import { useWorkspace } from "@/hooks/use-workspace";
import { departmentKeys, employeeKeys } from "@/lib/query-keys";
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
  return useWorkspaceQuery({
    queryKey: customFieldKeys.definitionList(workspace?.id ?? "", entityType),
    queryFn: (wsId) => fetchCustomFieldDefinitions(wsId, entityType),
  });
};

export const useCreateCustomFieldDefinition = () =>
  useWorkspaceMutation<CreateCustomFieldDefinitionRequest>({
    mutationFn: (wsId, payload) => createCustomFieldDefinition(wsId, payload),
    keys: [customFieldKeys.all],
  });

export const useUpdateCustomFieldDefinition = () =>
  useWorkspaceMutation<{ id: string; payload: UpdateCustomFieldDefinitionRequest }>({
    mutationFn: (wsId, { id, payload }) => updateCustomFieldDefinition(wsId, id, payload),
    keys: [customFieldKeys.all],
  });

export const useDeleteCustomFieldDefinition = () =>
  useWorkspaceMutation<string>({
    mutationFn: (wsId, id) => deleteCustomFieldDefinition(wsId, id),
    keys: [customFieldKeys.all],
  });

export const useCustomFieldValues = (entityType: string, entityId: string) => {
  const { workspace } = useWorkspace();
  return useWorkspaceQuery({
    queryKey: customFieldKeys.values(workspace?.id ?? "", entityType, entityId),
    queryFn: (wsId) => fetchCustomFieldValues(wsId, entityType, entityId),
    enabled: !!entityType && !!entityId,
  });
};

export const useSetCustomFieldValues = (entityType: string, entityId: string) => {
  const { workspace } = useWorkspace();
  return useWorkspaceMutation<SetCustomFieldValueRequest[]>({
    mutationFn: (wsId, values) => setCustomFieldValues(wsId, entityType, entityId, values),
    keys: [
      customFieldKeys.values(workspace?.id ?? "", entityType, entityId),
      employeeKeys.all(workspace?.id),
      departmentKeys.all(workspace?.id),
    ],
  });
};
