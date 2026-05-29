import { client } from "./client";

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
  const { data: resData } = await client.GET("/system/roles");
  if (!resData) throw new Error("No response data");
  return resData as unknown as SystemRoleInfo[];
}

export async function fetchUserSystemRoles(userId: string): Promise<SystemRoleAssignment[]> {
  const { data: resData } = await client.GET("/system/users/{user_id}/roles", {
    params: { path: { user_id: userId } }
  });
  if (!resData) throw new Error("No response data");
  return resData as unknown as SystemRoleAssignment[];
}

export async function assignSystemRole(userId: string, request: AssignSystemRoleRequest): Promise<void> {
  await client.POST("/system/users/{user_id}/roles", {
    params: { path: { user_id: userId } },
    body: request as never
  });
}

export async function revokeSystemRole(userId: string, roleName: string): Promise<void> {
  await client.DELETE("/system/users/{user_id}/roles/{role_name}", {
    params: { path: { user_id: userId, role_name: roleName } }
  });
}

export async function fetchSystemHealth(): Promise<Record<string, unknown>> {
  const { data: resData } = await client.GET("/system/health");
  if (!resData) throw new Error("No response data");
  return resData as Record<string, unknown>;
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
  const { data: resData } = await client.GET("/system/tenants");
  if (!resData) throw new Error("No response data");
  return resData as unknown as SystemTenant[];
}

export async function fetchStaleTenants(): Promise<SystemTenant[]> {
  const { data: resData } = await client.GET("/system/tenants/stale");
  if (!resData) throw new Error("No response data");
  return resData as unknown as SystemTenant[];
}

export async function createSystemTenant(request: CreateTenantRequestPayload): Promise<void> {
  await client.POST("/system/tenants", {
    body: request as never
  });
}

export async function deleteSystemTenant(tenantId: string): Promise<void> {
  await client.DELETE("/system/tenants/{tenant_id}", {
    params: { path: { tenant_id: tenantId } }
  });
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
  const { data: resData } = await client.GET("/system/tenants/{tenant_id}/stats", {
    params: { path: { tenant_id: tenantId } }
  });
  if (!resData) throw new Error("No response data");
  return resData as unknown as TenantConnectionStats;
}

export async function updateTenantDatabase(tenantId: string, customDbUrl: string): Promise<void> {
  await client.PUT("/system/tenants/{tenant_id}/database", {
    params: { path: { tenant_id: tenantId } },
    body: { database_url: customDbUrl } as never
  });
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
  const { data: resData } = await client.GET("/system/db-clusters");
  if (!resData) throw new Error("No response data");
  return resData!;
}

export async function createSystemDbCluster(request: CreateDbClusterRequestPayload): Promise<SystemDbCluster> {
  const { data: resData } = await client.POST("/system/db-clusters", {
    body: request as never
  });
  if (!resData) throw new Error("No response data");
  return resData!;
}

export async function deleteSystemDbCluster(id: string): Promise<void> {
  await client.DELETE("/system/db-clusters/{id}", {
    params: { path: { id } }
  });
}

export async function updateSystemDbClusterStatus(id: string, isActive: boolean): Promise<SystemDbCluster> {
  const { data: resData } = await client.PATCH("/system/db-clusters/{id}/status", {
    params: { path: { id } },
    body: { is_active: isActive } as never
  });
  if (!resData) throw new Error("No response data");
  return resData!;
}
