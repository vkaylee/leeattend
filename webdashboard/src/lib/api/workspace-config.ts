// @ts-nocheck
import { client } from './client';
import { WorkspaceConfigEntry, CreateConfigEntryRequest } from './types';
export type { WorkspaceConfigEntry, CreateConfigEntryRequest };

export type ScopeType = 'workspace' | 'department' | 'employee';

export interface ConfigValue {
  Bool?: boolean;
  Decimal?: number;
  Int?: number;
  Json?: any;
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
    return (resData) || null;
  },

  upsertConfig: async (workspaceId: string, payload: CreateConfigEntryRequest): Promise<WorkspaceConfigEntry> => {
    const { data: resData } = await client.PUT("/v1/workspaces/{workspace_id}/configs", {
      params: {
        path: { workspace_id: workspaceId },
      },
      body: payload,
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

