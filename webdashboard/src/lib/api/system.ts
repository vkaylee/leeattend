import { API_BASE_URL } from "./config";
import { apiFetch } from "./helpers";

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
  const response = await apiFetch(`${API_BASE_URL}/system/roles`);
  if (!response.ok) throw new Error("Failed to fetch system roles");
  return response.json() as Promise<SystemRoleInfo[]>;
}

export async function fetchUserSystemRoles(userId: string): Promise<SystemRoleAssignment[]> {
  const response = await apiFetch(`${API_BASE_URL}/system/users/${userId}/roles`);
  if (!response.ok) throw new Error("Failed to fetch user system roles");
  return response.json() as Promise<SystemRoleAssignment[]>;
}

export async function assignSystemRole(userId: string, request: AssignSystemRoleRequest): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/system/users/${userId}/roles`, {
    method: "POST",
    body: JSON.stringify(request),
  });
  if (!response.ok) throw new Error("Failed to assign system role");
}

export async function revokeSystemRole(userId: string, roleName: string): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/system/users/${userId}/roles/${roleName}`, {
    method: "DELETE",
  });
  if (!response.ok) throw new Error("Failed to revoke system role");
}

 
export async function fetchSystemHealth(): Promise<Record<string, unknown>> {
  const response = await apiFetch(`${API_BASE_URL}/system/health`);
  if (!response.ok) throw new Error("Failed to fetch system health");
  return response.json() as Promise<Record<string, unknown>>;
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
  const response = await apiFetch(`${API_BASE_URL}/system/tenants`);
  if (!response.ok) throw new Error("Failed to fetch system tenants");
  return response.json() as Promise<SystemTenant[]>;
}

export async function fetchStaleTenants(): Promise<SystemTenant[]> {
  const response = await apiFetch(`${API_BASE_URL}/system/tenants/stale`);
  if (!response.ok) throw new Error("Failed to fetch stale tenants");
  return response.json() as Promise<SystemTenant[]>;
}

export async function createSystemTenant(request: CreateTenantRequestPayload): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/system/tenants`, {
    method: "POST",
    body: JSON.stringify(request),
  });
  if (!response.ok) {
    const errText = await response.text();
    throw new Error(errText || "Failed to create system tenant");
  }
}

export async function deleteSystemTenant(tenantId: string): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/system/tenants/${tenantId}`, {
    method: "DELETE",
  });
  if (!response.ok) throw new Error("Failed to delete system tenant");
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
  const response = await apiFetch(`${API_BASE_URL}/system/tenants/${tenantId}/stats`);
  if (!response.ok) throw new Error("Failed to fetch tenant stats");
  return response.json() as Promise<TenantConnectionStats>;
}

export async function updateTenantDatabase(tenantId: string, customDbUrl: string): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/system/tenants/${tenantId}/database`, {
    method: "PUT",
    body: JSON.stringify({ custom_db_url: customDbUrl }),
  });
  if (!response.ok) {
    const errText = await response.text();
    throw new Error(errText || "Failed to update tenant database");
  }
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
  const response = await apiFetch(`${API_BASE_URL}/system/db-clusters`);
  if (!response.ok) throw new Error("Failed to fetch system DB clusters");
  return response.json() as Promise<SystemDbCluster[]>;
}

export async function createSystemDbCluster(request: CreateDbClusterRequestPayload): Promise<SystemDbCluster> {
  const response = await apiFetch(`${API_BASE_URL}/system/db-clusters`, {
    method: "POST",
    body: JSON.stringify(request),
  });
  if (!response.ok) {
    const errText = await response.text();
    throw new Error(errText || "Failed to create DB cluster");
  }
  return response.json() as Promise<SystemDbCluster>;
}

export async function deleteSystemDbCluster(id: string): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/system/db-clusters/${id}`, {
    method: "DELETE",
  });
  if (!response.ok) throw new Error("Failed to delete DB cluster");
}

export async function updateSystemDbClusterStatus(id: string, isActive: boolean): Promise<SystemDbCluster> {
  const response = await apiFetch(`${API_BASE_URL}/system/db-clusters/${id}/status`, {
    method: "PATCH",
    body: JSON.stringify({ is_active: isActive }),
  });
  if (!response.ok) {
    const errText = await response.text();
    throw new Error(errText || "Failed to update DB cluster status");
  }
  return response.json() as Promise<SystemDbCluster>;
}
