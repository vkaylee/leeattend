import { unwrapDataArray } from "@/types/api-overrides";

import { client } from "./client";
import { ApiError } from "./errors";
import { apiFetch } from "./helpers";
import { Workspace, WorkspaceMember, MemberRolesResponse, MemberExtraRole, MemberExtraPermission, PermissionCatalogItem, WorkspaceRole } from "./types";

export async function fetchWorkspaces(): Promise<Workspace[]> {
  const { data: resData } = await client.GET("/workspaces");
  return resData;
}

export async function createWorkspace(data: { name: string; database_url?: string }): Promise<Workspace> {
  const { data: resData } = await client.POST("/workspaces", {
    body: data,
  });
  return resData;
}

export async function fetchWorkspace(wsId: string): Promise<Workspace> {
  const { data: resData } = await client.GET("/workspaces/{ws_id}", {
    params: {
      path: { ws_id: wsId },
    },
  });
  return resData;
}

export async function updateWorkspace(
  wsId: string,
  data: Partial<Workspace>,
): Promise<Workspace> {
  const { data: resData } = await client.PATCH("/workspaces/{ws_id}", {
    params: {
      path: { ws_id: wsId },
    },
    body: data,
  });
  return resData;
}

export async function fetchWorkspaceMembers(wsId: string): Promise<WorkspaceMember[]> {
  const { data } = await client.GET("/v1/workspaces/{ws_id}/members", { params: { path: { ws_id: wsId } } });
  return unwrapDataArray<WorkspaceMember>(data);
}


export async function addWorkspaceMember(wsId: string, email: string, roleId: string): Promise<void> {
  await client.POST("/v1/workspaces/{ws_id}/members", { params: { path: { ws_id: wsId } }, body: { email, role_id: roleId } });
}

export async function removeWorkspaceMember(wsId: string, userId: string): Promise<void> {
  await client.DELETE("/v1/workspaces/{ws_id}/members/{user_id}", { params: { path: { ws_id: wsId, user_id: userId } } });
}

export async function updateWorkspaceMemberRole(wsId: string, userId: string, roleId: string): Promise<void> {
  await client.PATCH("/v1/workspaces/{ws_id}/members/{user_id}", { params: { path: { ws_id: wsId, user_id: userId } }, body: { role_id: roleId } });
}

export async function fetchMemberRoles(wsId: string, userId: string): Promise<MemberRolesResponse> {
  const { data } = await client.GET("/v1/workspaces/{ws_id}/members/{user_id}/roles", { params: { path: { ws_id: wsId, user_id: userId } } });
  return data;
}

export async function assignExtraRole(wsId: string, userId: string, roleId: string, expiresAt: string): Promise<MemberExtraRole> {
  const { data } = await client.POST("/v1/workspaces/{ws_id}/members/{user_id}/roles", { params: { path: { ws_id: wsId, user_id: userId } }, body: { role_id: roleId, expires_at: expiresAt } });
  return data;
}

export async function revokeExtraRole(wsId: string, userId: string, roleId: string): Promise<void> {
  await client.DELETE("/v1/workspaces/{ws_id}/members/{user_id}/roles/{role_id}", { params: { path: { ws_id: wsId, user_id: userId, role_id: roleId } } });
}

export async function fetchWorkspaceRoles(wsId: string): Promise<WorkspaceRole[]> {
  const { data } = await client.GET("/v1/workspaces/{ws_id}/roles", {
    params: {
      path: {
        ws_id: wsId,
      }
    }
  });
  return data.data;
}

export async function clearWorkspaceData(wsId: string, group: string): Promise<void> {
  const response = await apiFetch(`/v1/dev/workspaces/${wsId}/clear?group=${group}`, { method: "DELETE" });
  if (!response.ok) await ApiError.rejectResponse(response);
}

// --- Extra Permissions ---

export async function fetchMemberPermissions(wsId: string, userId: string): Promise<MemberExtraPermission[]> {
  const { data } = await client.GET("/v1/workspaces/{ws_id}/members/{user_id}/permissions", { params: { path: { ws_id: wsId, user_id: userId } } });
  return data;
}

export async function assignExtraPermission(wsId: string, userId: string, permission: string, expiresAt: string): Promise<MemberExtraPermission> {
  const { data } = await client.POST("/v1/workspaces/{ws_id}/members/{user_id}/permissions", { params: { path: { ws_id: wsId, user_id: userId } }, body: { permission, expires_at: expiresAt } });
  return data;
}

export async function revokeExtraPermission(wsId: string, userId: string, permId: string): Promise<void> {
  await client.DELETE("/v1/workspaces/{ws_id}/members/{user_id}/permissions/{perm_id}", { params: { path: { ws_id: wsId, user_id: userId, perm_id: permId } } });
}

export async function fetchPermissionCatalog(
  wsId: string,
): Promise<PermissionCatalogItem[]> {
  const { data: resData } = await client.GET("/v1/workspaces/{ws_id}/permissions/catalog", {
    params: { path: { ws_id: wsId } }
  });
  return resData;
}

export async function fetchAvailablePermissions(
  wsId: string,
): Promise<string[]> {
  const { data: resData } = await client.GET("/v1/workspaces/{ws_id}/permissions/available", {
    params: { path: { ws_id: wsId } }
  });
  return resData;
}

// --- Roles & Matrix ---

export async function createWorkspaceRole(wsId: string, data: { name: string; description?: string; color: string }): Promise<WorkspaceRole> {
  const { data: resData } = await client.POST("/v1/workspaces/{ws_id}/roles", {
    params: { path: { ws_id: wsId } },
    body: data
  });
  return resData;
}

export async function updateWorkspaceRole(wsId: string, roleId: string, data: { name?: string; description?: string; color?: string }): Promise<WorkspaceRole> {
  const { data: resData } = await client.PUT("/v1/workspaces/{ws_id}/roles/{role_id}", {
    params: { path: { ws_id: wsId, role_id: roleId } },
    body: data
  });
  return resData;
}

export async function deleteWorkspaceRole(wsId: string, roleId: string): Promise<void> {
  await client.DELETE("/v1/workspaces/{ws_id}/roles/{role_id}", {
    params: { path: { ws_id: wsId, role_id: roleId } }
  });
}

export async function fetchRolePermissions(wsId: string, roleId: string): Promise<string[]> {
  const { data: resData } = await client.GET("/v1/workspaces/{ws_id}/roles/{role_id}/permissions", {
    params: { path: { ws_id: wsId, role_id: roleId } }
  });
  return resData;
}

export async function updateRolePermissions(wsId: string, roleId: string, permissions: string[]): Promise<void> {
  await client.PUT("/v1/workspaces/{ws_id}/roles/{role_id}/permissions", {
    params: { path: { ws_id: wsId, role_id: roleId } },
    body: { permissions }
  });
}
