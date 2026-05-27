import { API_BASE_URL } from "./config";
import { apiFetch } from "./helpers";
import { apiClient } from "./client";

export interface SystemRoleAssignment {
  assigned_at: string;
  assigned_by?: string;
  role_name: string;
  user_id: string;
}

export interface SystemRoleInfo {
  assignees: SystemRoleAssignment[];
  description: string;
  id: string;
  name: string;
}

export interface AssignSystemRoleRequest {
  role_name: string;
}

export async function fetchSystemRoles(): Promise<SystemRoleInfo[]> {
  const { data: resData } = await apiClient.get<SystemRoleInfo[]>(`/system/roles`);
    return resData;
}

export async function fetchUserSystemRoles(userId: string): Promise<SystemRoleAssignment[]> {
  const { data: resData } = await apiClient.get<SystemRoleAssignment[]>(`/system/users/${userId}/roles`);
    return resData;
}

export async function assignSystemRole(userId: string, request: AssignSystemRoleRequest): Promise<void> {
  await apiClient.post(`/system/users/${userId}/roles`, request);
}

export async function revokeSystemRole(userId: string, roleName: string): Promise<void> {
  await apiClient.delete(`/system/users/${userId}/roles/${roleName}`);
}

 
export async function fetchSystemHealth(): Promise<Record<string, unknown>> {
  const { data: resData } = await apiClient.get<Record<string, unknown>>(`/system/health`);
    return resData;
}

export interface SystemTenant {
  created_at: string;
  id: string;
  pool_size: number;
  updated_at: string;
}

export interface CreateTenantRequestPayload {
  create_db: boolean;
  db_name: string;
  pool_size?: number;
}

export async function fetchSystemTenants(): Promise<SystemTenant[]> {
  const { data: resData } = await apiClient.get<SystemTenant[]>(`/system/tenants`);
    return resData;
}

export async function fetchStaleTenants(): Promise<SystemTenant[]> {
  const { data: resData } = await apiClient.get<SystemTenant[]>(`/system/tenants/stale`);
    return resData;
}

export async function createSystemTenant(request: CreateTenantRequestPayload): Promise<void> {
  await apiClient.post(`/system/tenants`, request);
}

export async function deleteSystemTenant(tenantId: string): Promise<void> {
  await apiClient.delete(`/system/tenants/${tenantId}`);
}

export interface TenantConfig {
  db_database?: string;
  default_mode?: string;
  users?: {
    db_user?: string;
    mode_type?: string;
    pool_size?: number;
  }[];
}

export interface TenantConnectionStats {
  config: TenantConfig & {
    data?: TenantConfig;
    tenant?: TenantConfig;
  };
  connection: {
    host: string;
    port: string;
    masked_db_url?: string | null;
  };
}

export async function fetchSystemTenantStats(tenantId: string): Promise<TenantConnectionStats> {
  const { data: resData } = await apiClient.get<TenantConnectionStats>(`/system/tenants/${tenantId}/stats`);
    return resData;
}

export async function updateTenantDatabase(tenantId: string, customDbUrl: string): Promise<void> {
  await apiClient.put(`/system/tenants/${tenantId}/database`, { custom_db_url: customDbUrl });
}

export interface SystemDbCluster {
  active_tenants: number;
  admin_database_url: string;
  created_at: string;
  id: string;
  is_active: boolean;
  max_tenants: number;
  name: string;
  region: string;
  updated_at: string;
}

export interface CreateDbClusterRequestPayload {
  admin_database_url: string;
  is_active: boolean;
  max_tenants: number;
  name: string;
  region: string;
}

export async function fetchSystemDbClusters(): Promise<SystemDbCluster[]> {
  const { data: resData } = await apiClient.get<SystemDbCluster[]>(`/system/db-clusters`);
    return resData;
}

export async function createSystemDbCluster(request: CreateDbClusterRequestPayload): Promise<SystemDbCluster> {
  const { data: resData } = await apiClient.post<SystemDbCluster>(`/system/db-clusters`, request);
    return resData;
}

export async function deleteSystemDbCluster(id: string): Promise<void> {
  await apiClient.delete(`/system/db-clusters/${id}`);
}

export async function updateSystemDbClusterStatus(id: string, isActive: boolean): Promise<SystemDbCluster> {
  const { data: resData } = await apiClient.patch<SystemDbCluster>(`/system/db-clusters/${id}/status`, { is_active: isActive });
    return resData;
}
