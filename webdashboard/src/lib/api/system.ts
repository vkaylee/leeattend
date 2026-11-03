import type { components } from "@/types/api-generated";

import { client, fetch } from "./client";

export type SystemRoleAssignment = components["schemas"]["SystemRoleAssignment"];
export type SystemRoleInfo = components["schemas"]["SystemRoleInfo"];
export type AssignSystemRoleRequest = components["schemas"]["AssignSystemRoleRequest"];

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

export async function fetchSystemHealth(): Promise<unknown> {
  const { data: resData } = await client.GET("/system/health");
  return resData;
}

