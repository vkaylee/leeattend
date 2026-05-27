import { apiClient } from './client';

export type ScopeType = 'workspace' | 'department' | 'employee';

export interface ConfigValue {
  Int?: number;
  Decimal?: number;
  Text?: string;
  Bool?: boolean;
  Json?: any;
}

export interface WorkspaceConfigEntry {
  id: string;
  workspace_id: string;
  scope_type: ScopeType;
  scope_id?: string;
  config_key: string;
  value_int?: number;
  value_decimal?: number;
  value_text?: string;
  value_bool?: boolean;
  value_json?: any;
  effective_from: string;
  effective_to?: string;
  created_by?: string;
  created_at: string;
  updated_at: string;
}

export interface CreateConfigEntryRequest {
  scope_type: ScopeType;
  scope_id?: string;
  config_key: string;
  value: ConfigValue;
  effective_from: string;
  effective_to?: string;
}

export interface UpdateConfigEntryRequest {
  effective_to?: string;
}

export const workspaceConfigApi = {
  listConfigs: async (workspaceId: string, prefix?: string): Promise<WorkspaceConfigEntry[]> => {
    const params = new URLSearchParams();
    if (prefix) params.append('prefix', prefix);
    
    const response = await apiClient.get(`/api/workspaces/${workspaceId}/configs?${params.toString()}`);
    return response.data;
  },

  getConfig: async (workspaceId: string, key: string, date?: string): Promise<WorkspaceConfigEntry | null> => {
    const params = new URLSearchParams();
    if (date) params.append('date', date);
    
    const response = await apiClient.get(`/api/workspaces/${workspaceId}/configs/${key}?${params.toString()}`);
    return response.data;
  },

  upsertConfig: async (workspaceId: string, payload: CreateConfigEntryRequest): Promise<WorkspaceConfigEntry> => {
    const response = await apiClient.put(`/api/workspaces/${workspaceId}/configs`, payload);
    return response.data;
  },

  deleteConfig: async (workspaceId: string, id: string): Promise<void> => {
    await apiClient.delete(`/api/workspaces/${workspaceId}/configs/${id}`);
  },
};
