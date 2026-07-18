import type { components } from "@/types/api-generated";

import { client } from "./client";

// Re-export generated types with JSONB field overrides (OpenAPI doesn't type JSONB strongly)
export type ConditionTypeDefinition = Omit<
  components["schemas"]["ConditionTypeDefinition"],
  "name_i18n" | "affects_rules"
> & {
  name_i18n: Record<string, string>;
  affects_rules: Record<string, boolean>;
};

export type CreateConditionTypeRequest = components["schemas"]["CreateConditionTypeRequest"] & {
  name_i18n?: Record<string, string>;
  affects_rules?: Record<string, boolean>;
};

export type UpdateConditionTypeRequest = components["schemas"]["UpdateConditionTypeRequest"] & {
  name_i18n?: Record<string, string>;
  affects_rules?: Record<string, boolean>;
};

const buildPath = (wsId: string, id?: string): string =>
  id
    ? `/workspaces/${wsId}/condition-types/${id}`
    : `/workspaces/${wsId}/condition-types`;

export async function fetchConditionTypes(
  wsId: string,
  activeOnly?: boolean,
): Promise<ConditionTypeDefinition[]> {
  const { data } = await client.GET("/workspaces/{workspace_id}/condition-types", {
    params: {
      path: { workspace_id: wsId },
      query: activeOnly !== undefined ? { active_only: activeOnly } : undefined,
    },
  });
  return (data as ConditionTypeDefinition[]) ?? [];
}

export async function fetchConditionType(
  wsId: string,
  id: string,
): Promise<ConditionTypeDefinition> {
  const { data } = await client.GET("/workspaces/{workspace_id}/condition-types/{id}", {
    params: {
      path: { workspace_id: wsId, id },
    },
  });
  return data as ConditionTypeDefinition;
}

export async function createConditionType(
  wsId: string,
  req: CreateConditionTypeRequest,
): Promise<ConditionTypeDefinition> {
  const { data } = await client.POST("/workspaces/{workspace_id}/condition-types", {
    params: {
      path: { workspace_id: wsId },
    },
    body: req,
  });
  return data as ConditionTypeDefinition;
}

export async function updateConditionType(
  wsId: string,
  id: string,
  req: UpdateConditionTypeRequest,
): Promise<ConditionTypeDefinition> {
  const { data } = await client.PATCH("/workspaces/{workspace_id}/condition-types/{id}", {
    params: {
      path: { workspace_id: wsId, id },
    },
    body: req,
  });
  return data as ConditionTypeDefinition;
}

export async function deleteConditionType(wsId: string, id: string): Promise<void> {
  await client.DELETE("/workspaces/{workspace_id}/condition-types/{id}", {
    params: {
      path: { workspace_id: wsId, id },
    },
  });
}
