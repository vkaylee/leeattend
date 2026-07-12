import { components } from "@/types/api-generated";

import { client, fetch, fire } from "./client";

// Re-exported for downstream consumers that imported these from here.
export type SystemRoleAssignment = components["schemas"]["SystemRoleAssignment"];
export type SystemRoleInfo = components["schemas"]["SystemRoleInfo"];
export type AssignSystemRoleRequest = components["schemas"]["AssignSystemRoleRequest"];
export type SystemDbCluster = components["schemas"]["SystemDbCluster"];
export type CreateDbClusterRequestPayload = components["schemas"]["CreateDbClusterRequest"];

export async function fetchSystemRoles(): Promise<SystemRoleInfo[]> {
  const { data: resData } = await client.GET("/system/roles");
  return resData;
}

export async function fetchUserSystemRoles(userId: string): Promise<SystemRoleAssignment[]> {
  // Schema advertises `string[]`; runtime payload is the richer assignment list.
  const { data: resData } = await fetch<SystemRoleAssignment[]>("GET", "/system/users/{user_id}/roles", {
    params: { path: { user_id: userId } },
  });
  return resData;
}

export async function assignSystemRole(userId: string, request: AssignSystemRoleRequest): Promise<void> {
  await client.POST("/system/users/{user_id}/roles", {
    params: { path: { user_id: userId } },
    body: request,
  });
}

export async function revokeSystemRole(userId: string, roleName: string): Promise<void> {
  await client.DELETE("/system/users/{user_id}/roles/{role_name}", {
    params: { path: { user_id: userId, role_name: roleName } }
  });
}

export async function fetchSystemHealth(): Promise<Record<string, unknown>> {
  const { data: resData } = await client.GET("/system/health");
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
  const { data: resData } = await fetch<SystemTenant[]>("GET", "/system/tenants", {});
  return resData;
}

export async function fetchStaleTenants(): Promise<SystemTenant[]> {
  const { data: resData } = await fetch<SystemTenant[]>("GET", "/system/tenants/stale", {});
  return resData;
}

export async function createSystemTenant(request: CreateTenantRequestPayload): Promise<void> {
  await fire("POST", "/system/tenants", { body: request });
}

export async function deleteSystemTenant(tenantId: string): Promise<void> {
  await fire("DELETE", "/system/tenants/{tenant_id}", {
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
  const { data: resData } = await fetch<TenantConnectionStats>("GET", "/system/tenants/{tenant_id}/stats", {
    params: { path: { tenant_id: tenantId } },
  });
  return resData;
}

export async function updateTenantDatabase(tenantId: string, customDbUrl: string): Promise<void> {
  await fire("PUT", "/system/tenants/{tenant_id}/database", {
    params: { path: { tenant_id: tenantId } },
    body: { database_url: customDbUrl },
  });
}

export async function fetchSystemDbClusters(): Promise<SystemDbCluster[]> {
  const { data: resData } = await client.GET("/system/db-clusters");
  return resData;
}

export async function createSystemDbCluster(request: CreateDbClusterRequestPayload): Promise<SystemDbCluster> {
  const { data: resData } = await client.POST("/system/db-clusters", {
    body: request,
  });
  return resData;
}

export async function deleteSystemDbCluster(id: string): Promise<void> {
  await client.DELETE("/system/db-clusters/{id}", {
    params: { path: { id } }
  });
}

export async function updateSystemDbClusterStatus(id: string, isActive: boolean): Promise<SystemDbCluster> {
  const { data: resData } = await client.PATCH("/system/db-clusters/{id}/status", {
    params: { path: { id } },
    body: { is_active: isActive },
  });
  return resData;
}
