import type { components } from "@/types/api-generated";

import type { WorkspaceConfigEntry, CreateConfigEntryRequest } from './types';

import { client } from './client';

export type ScopeType = 'workspace' | 'department' | 'employee';

export interface ConfigValue {
  Bool?: boolean;
  Decimal?: number;
  Int?: number;
  Json?: unknown;
  Text?: string;
}

export interface UpdateConfigEntryRequest {
  effective_to?: string;
}

export const workspaceConfigApi = {
  listConfigs: async (workspaceId: string, prefix?: string): Promise<WorkspaceConfigEntry[]> => {
    const { data: resData } = await client.GET("/v1/workspaces/{workspace_id}/configs", {
      params: {
        path: { workspace_id: workspaceId },
        query: prefix ? { prefix } : undefined,
      },
    });
    return resData;
  },

  getConfig: async (workspaceId: string, key: string, date?: string): Promise<components["schemas"]["ResolvedConfig"] | null> => {
    const { data: resData } = await client.GET("/v1/workspaces/{workspace_id}/configs/{key}", {
      params: {
        path: { workspace_id: workspaceId, key },
        query: date ? { date } : undefined,
      },
    });
  // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
    return resData ?? null;
  },

  // eslint-disable-next-line complexity
  upsertConfig: async (workspaceId: string, payload: CreateConfigEntryRequest): Promise<WorkspaceConfigEntry> => {
    let value: ConfigValue;
    if (payload.value_bool !== undefined && payload.value_bool !== null) {
      value = { Bool: payload.value_bool };
    } else if (payload.value_decimal !== undefined && payload.value_decimal !== null) {
      value = { Decimal: payload.value_decimal };
    } else if (payload.value_int !== undefined && payload.value_int !== null) {
      value = { Int: payload.value_int };
    } else if (payload.value_json !== undefined && payload.value_json !== null) {
      value = { Json: payload.value_json };
    } else if (payload.value_text !== undefined && payload.value_text !== null) {
      value = { Text: payload.value_text };
    } else {
      value = { Text: "" };
    }

    const { data: resData } = await client.PUT("/v1/workspaces/{workspace_id}/configs", {
      params: {
        path: { workspace_id: workspaceId },
      },
      body: {
        config_key: payload.config_key,
        effective_from: payload.effective_from,
        effective_to: payload.effective_to,
        scope_id: payload.scope_id,
        scope_type: payload.scope_type,
        value: value as components["schemas"]["ConfigValue"],
      },
    });
    return resData;
  },

  deleteConfig: async (workspaceId: string, id: string): Promise<void> => {
    await client.DELETE("/v1/workspaces/{workspace_id}/configs/{id}", {
      params: {
        path: { workspace_id: workspaceId, id },
      },
    });
  },
};

