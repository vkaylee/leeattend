import type {
  CreateCustomFieldDefinitionRequest,
  SetCustomFieldValueRequest,
  UpdateCustomFieldDefinitionRequest,
} from "@/lib/api/types";

import { useOrganizationQuery, useOrganizationMutation } from "@/hooks/common/use-organization-query";
import {
  createCustomFieldDefinition,
  deleteCustomFieldDefinition,
  fetchCustomFieldDefinitions,
  fetchCustomFieldValues,
  setCustomFieldValues,
  updateCustomFieldDefinition,
} from "@/lib/api/custom-fields";
import { departmentKeys, employeeKeys } from "@/lib/query-keys";

export const customFieldKeys = {
  all: ["custom-fields"] as const,
  definitions: () => [...customFieldKeys.all, "definitions"] as const,
  definitionList: (entityType?: string) => [...customFieldKeys.definitions(), { entityType }] as const,
  values: (entityType: string, entityId: string) => [...customFieldKeys.all, "values", entityType, entityId] as const,
};

export const useCustomFieldDefinitions = (entityType?: string) => {
  return useOrganizationQuery({
    queryKey: customFieldKeys.definitionList(entityType),
    queryFn: () => fetchCustomFieldDefinitions(entityType),
  });
};

export const useCreateCustomFieldDefinition = () =>
  useOrganizationMutation<CreateCustomFieldDefinitionRequest>({
    mutationFn: (payload) => createCustomFieldDefinition(payload),
    keys: [customFieldKeys.all],
  });

export const useUpdateCustomFieldDefinition = () =>
  useOrganizationMutation<{ id: string; payload: UpdateCustomFieldDefinitionRequest }>({
    mutationFn: ({ id, payload }) => updateCustomFieldDefinition(id, payload),
    keys: [customFieldKeys.all],
  });

export const useDeleteCustomFieldDefinition = () =>
  useOrganizationMutation<string>({
    mutationFn: (id) => deleteCustomFieldDefinition(id),
    keys: [customFieldKeys.all],
  });

export const useCustomFieldValues = (entityType: string, entityId: string) => {
  return useOrganizationQuery({
    queryKey: customFieldKeys.values(entityType, entityId),
    queryFn: () => fetchCustomFieldValues(entityType, entityId),
    enabled: !!entityType && !!entityId,
  });
};

export const useSetCustomFieldValues = (entityType: string, entityId: string) => {
  return useOrganizationMutation<SetCustomFieldValueRequest[]>({
    mutationFn: (values) => setCustomFieldValues(entityType, entityId, values),
    keys: [customFieldKeys.values(entityType, entityId), employeeKeys.all(), departmentKeys.all()],
  });
};
