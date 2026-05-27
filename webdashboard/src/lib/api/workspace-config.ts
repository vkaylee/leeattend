import { apiClient } from './client';

export type ScopeType = 'workspace' | 'department' | 'employee';

export interface ConfigValue {
  Bool?: boolean;
  Decimal?: number;
  Int?: number;
  Json?: any;
  Text?: string;
}

export interface WorkspaceConfigEntry {
  config_key: string;
  created_at: string;
  created_by?: string;
  effective_from: string;
  effective_to?: string;
  id: string;
  scope_id?: string;
  scope_type: ScopeType;
  updated_at: string;
  value_bool?: boolean;
  value_decimal?: number;
  value_int?: number;
  value_json?: any;
  value_text?: string;
  workspace_id: string;
}

export interface CreateConfigEntryRequest {
  config_key: string;
  effective_from: string;
  effective_to?: string;
  scope_id?: string;
  scope_type: ScopeType;
  value: ConfigValue;
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
