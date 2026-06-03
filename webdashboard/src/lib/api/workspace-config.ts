import type { paths } from "@/types/api-generated";

import type { WorkspaceConfigEntry, CreateConfigEntryRequest } from './types';

import { client } from './client';

export type ScopeType = 'workspace' | 'department' | 'employee';

export interface ConfigValue {
  Bool?: boolean;
  Decimal?: number;
  Int?: number;
  Json?: Record<string, unknown> | null;
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
    if (!resData) throw new Error("No response data");
    return resData;
  },

  getConfig: async (workspaceId: string, key: string, date?: string): Promise<WorkspaceConfigEntry | null> => {
    const { data: resData } = await client.GET("/v1/workspaces/{workspace_id}/configs/{key}", {
      params: {
        path: { workspace_id: workspaceId, key },
        query: date ? { date } : undefined,
      },
    });
    return (resData as unknown as WorkspaceConfigEntry | null) ?? null;
  },

  upsertConfig: async (workspaceId: string, payload: CreateConfigEntryRequest): Promise<WorkspaceConfigEntry> => {
    const { data: resData } = await client.PUT("/v1/workspaces/{workspace_id}/configs", {
      params: {
        path: { workspace_id: workspaceId },
      },
      body: payload as unknown as paths["/v1/workspaces/{workspace_id}/configs"]["put"]["requestBody"]["content"]["application/json"],
    });
    if (!resData) throw new Error("No response data");
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

