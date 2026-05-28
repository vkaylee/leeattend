import { Workspace, WorkspaceMember, MemberRolesResponse, MemberExtraRole, MemberExtraPermission, WorkspaceRole } from "./types";
import { apiClient, client } from "./client";

export async function fetchWorkspaces(): Promise<Workspace[]> {
  const { data: resData } = await client.GET("/v1/workspaces");
  if (!resData) throw new Error("No response data");
  return resData as any[];
}

export async function createWorkspace(data: { name: string; database_url?: string }): Promise<Workspace> {
  const { data: resData } = await client.POST("/v1/workspaces", {
    body: data as any,
  });
  if (!resData) throw new Error("No response data");
  return resData as any;
}

export async function fetchWorkspace(wsId: string): Promise<Workspace> {
  const { data: resData } = await client.GET("/v1/workspaces/{ws_id}", {
    params: {
      path: { ws_id: wsId },
    },
  });
  if (!resData) throw new Error("No response data");
  return resData as any;
}

export async function updateWorkspace(
  wsId: string,
  data: Partial<Workspace>,
): Promise<Workspace> {
  const { data: resData } = await client.PATCH("/v1/workspaces/{ws_id}", {
    params: {
      path: { ws_id: wsId },
    },
    body: data as any,
  });
  if (!resData) throw new Error("No response data");
  return resData as any;
}

export async function fetchWorkspaceMembers(
  wsId: string,
): Promise<WorkspaceMember[]> {
  const { data: resData } = await apiClient.get<WorkspaceMember[]>(`/v1/workspaces/${wsId}/members`);
    return resData;
}

export async function addWorkspaceMember(
  wsId: string,
  email: string,
  roleId: string,
): Promise<void> {
  await apiClient.post(`/v1/workspaces/${wsId}/members`, { email, role_id: roleId });
}

export async function removeWorkspaceMember(
  wsId: string,
  userId: string,
): Promise<void> {
  await apiClient.delete(`/v1/workspaces/${wsId}/members/${userId}`);
}

export async function updateWorkspaceMemberRole(
  wsId: string,
  userId: string,
  roleId: string,
): Promise<void> {
  await apiClient.patch(`/v1/workspaces/${wsId}/members/${userId}`, { role_id: roleId });
}

export async function fetchMemberRoles(
  wsId: string,
  userId: string,
): Promise<MemberRolesResponse> {
  const { data: resData } = await apiClient.get<MemberRolesResponse>(`/v1/workspaces/${wsId}/members/${userId}/roles`);
    return resData;
}

export async function assignExtraRole(
  wsId: string,
  userId: string,
  roleId: string,
  expiresAt: string,
): Promise<MemberExtraRole> {
  const { data: resData } = await apiClient.post<MemberExtraRole>(`/v1/workspaces/${wsId}/members/${userId}/roles`, { role_id: roleId, expires_at: expiresAt });
    return resData;
}

export async function revokeExtraRole(
  wsId: string,
  userId: string,
  roleId: string,
): Promise<void> {
  await apiClient.delete(`/v1/workspaces/${wsId}/members/${userId}/roles/${roleId}`);
}

export async function fetchWorkspaceRoles(wsId: string): Promise<WorkspaceRole[]> {
  const { data: resData } = await apiClient.get<WorkspaceRole[]>(`/v1/workspaces/${wsId}/roles`);
    return resData;
}

export async function clearWorkspaceData(wsId: string, group: string): Promise<void> {
  await apiClient.post(`/v1/dev/workspaces/${wsId}/clear?group=${group}`, {});
}

// --- Extra Permissions ---

export async function fetchMemberPermissions(
  wsId: string,
  userId: string,
): Promise<MemberExtraPermission[]> {
  const { data: resData } = await apiClient.get<MemberExtraPermission[]>(`/v1/workspaces/${wsId}/members/${userId}/permissions`);
    return resData;
}

export async function assignExtraPermission(
  wsId: string,
  userId: string,
  permission: string,
  expiresAt: string,
): Promise<MemberExtraPermission> {
  const { data: resData } = await apiClient.post<MemberExtraPermission>(`/v1/workspaces/${wsId}/members/${userId}/permissions`, { permission, expires_at: expiresAt });
    return resData;
}

export async function revokeExtraPermission(
  wsId: string,
  userId: string,
  permId: string,
): Promise<void> {
  await apiClient.delete(`/v1/workspaces/${wsId}/members/${userId}/permissions/${permId}`);
}

export async function fetchAvailablePermissions(
  wsId: string,
): Promise<string[]> {
  const { data: resData } = await apiClient.get<string[]>(`/v1/workspaces/${wsId}/permissions/available`);
    return resData;
}

// --- Roles & Matrix ---

export async function createWorkspaceRole(wsId: string, data: { name: string; description?: string; color: string }): Promise<WorkspaceRole> {
  const { data: resData } = await apiClient.post<WorkspaceRole>(`/v1/workspaces/${wsId}/roles`, data);
    return resData;
}

export async function updateWorkspaceRole(wsId: string, roleId: string, data: { name?: string; description?: string; color?: string }): Promise<WorkspaceRole> {
  const { data: resData } = await apiClient.put<WorkspaceRole>(`/v1/workspaces/${wsId}/roles/${roleId}`, data);
    return resData;
}

export async function deleteWorkspaceRole(wsId: string, roleId: string): Promise<void> {
  await apiClient.delete(`/v1/workspaces/${wsId}/roles/${roleId}`);
}

export async function fetchRolePermissions(wsId: string, roleId: string): Promise<string[]> {
  const { data: resData } = await apiClient.get<string[]>(`/v1/workspaces/${wsId}/roles/${roleId}/permissions`);
    return resData;
}

export async function updateRolePermissions(wsId: string, roleId: string, permissions: string[]): Promise<void> {
  await apiClient.put(`/v1/workspaces/${wsId}/roles/${roleId}/permissions`, permissions);
}
