import { API_BASE_URL } from "./config";
import { apiFetch } from "./helpers";
import { Workspace, WorkspaceMember, MemberRolesResponse, MemberExtraRole, MemberExtraPermission, WorkspaceRole } from "./types";

export async function fetchWorkspaces(): Promise<Workspace[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces`);
  if (!response.ok) throw new Error("Failed to fetch workspaces");
  return response.json() as Promise<Workspace[]>;
}

export async function createWorkspace(data: { name: string } | string): Promise<Workspace> {
  const payload = typeof data === "string" ? { name: data } : data;
  const response = await apiFetch(`${API_BASE_URL}/workspaces`, {
    method: "POST",
    body: JSON.stringify(payload),
  });
  if (!response.ok) throw new Error("Failed to create workspace");
  return response.json() as Promise<Workspace>;
}

export async function fetchWorkspace(wsId: string): Promise<Workspace> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}`);
  if (!response.ok) throw new Error("Failed to fetch workspace");
  return response.json() as Promise<Workspace>;
}

export async function updateWorkspace(
  wsId: string,
  data: Partial<Workspace>,
): Promise<Workspace> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}`, {
    method: "PATCH",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to update workspace");
  return response.json() as Promise<Workspace>;
}

export async function fetchWorkspaceMembers(
  wsId: string,
): Promise<WorkspaceMember[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/members`);
  if (!response.ok) throw new Error("Failed to fetch members");
  return response.json() as Promise<WorkspaceMember[]>;
}

export async function addWorkspaceMember(
  wsId: string,
  email: string,
  roleId: string,
): Promise<void> {
  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${wsId}/members`,
    {
      method: "POST",
      body: JSON.stringify({ email, role_id: roleId }),
    },
  );
  if (!response.ok) throw new Error("Failed to add member");
}

export async function removeWorkspaceMember(
  wsId: string,
  userId: string,
): Promise<void> {
  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${wsId}/members/${userId}`,
    {
      method: "DELETE",
    },
  );
  if (!response.ok) {
    const error = await response.text();
    throw new Error(error || "Failed to remove member");
  }
}

export async function updateWorkspaceMemberRole(
  wsId: string,
  userId: string,
  roleId: string,
): Promise<void> {
  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${wsId}/members/${userId}`,
    {
      method: "PATCH",
      body: JSON.stringify({ role_id: roleId }),
    },
  );
  if (!response.ok) {
    const error = await response.text();
    throw new Error(error || "Failed to update member role");
  }
}

export async function fetchMemberRoles(
  wsId: string,
  userId: string,
): Promise<MemberRolesResponse> {
  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${wsId}/members/${userId}/roles`,
  );
  if (!response.ok) throw new Error("Failed to fetch member roles");
  return response.json() as Promise<MemberRolesResponse>;
}

export async function assignExtraRole(
  wsId: string,
  userId: string,
  roleId: string,
  expiresAt: string,
): Promise<MemberExtraRole> {
  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${wsId}/members/${userId}/roles`,
    {
      method: "POST",
      body: JSON.stringify({ role_id: roleId, expires_at: expiresAt }),
    },
  );
  if (!response.ok) throw new Error("Failed to assign role");
  return response.json() as Promise<MemberExtraRole>;
}

export async function revokeExtraRole(
  wsId: string,
  userId: string,
  roleId: string,
): Promise<void> {
  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${wsId}/members/${userId}/roles/${roleId}`,
    {
      method: "DELETE",
    },
  );
  if (!response.ok) throw new Error("Failed to revoke role");
}

export async function fetchWorkspaceRoles(wsId: string): Promise<WorkspaceRole[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/roles`);
  if (!response.ok) throw new Error("Failed to fetch workspace roles");
  return response.json() as Promise<WorkspaceRole[]>;
}

export async function clearWorkspaceData(wsId: string, group: string): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/dev/workspaces/${wsId}/clear?group=${group}`, {
    method: "POST",
  });
  if (!response.ok) {
    const error = await response.text();
    throw new Error(error || "Failed to clear workspace data");
  }
}

// --- Extra Permissions ---

export async function fetchMemberPermissions(
  wsId: string,
  userId: string,
): Promise<MemberExtraPermission[]> {
  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${wsId}/members/${userId}/permissions`,
  );
  if (!response.ok) throw new Error("Failed to fetch member permissions");
  return response.json() as Promise<MemberExtraPermission[]>;
}

export async function assignExtraPermission(
  wsId: string,
  userId: string,
  permission: string,
  expiresAt: string,
): Promise<MemberExtraPermission> {
  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${wsId}/members/${userId}/permissions`,
    {
      method: "POST",
      body: JSON.stringify({ permission, expires_at: expiresAt }),
    },
  );
  if (!response.ok) {
    const error = await response.text();
    throw new Error(error || "Failed to assign permission");
  }
  return response.json() as Promise<MemberExtraPermission>;
}

export async function revokeExtraPermission(
  wsId: string,
  userId: string,
  permId: string,
): Promise<void> {
  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${wsId}/members/${userId}/permissions/${permId}`,
    {
      method: "DELETE",
    },
  );
  if (!response.ok) throw new Error("Failed to revoke permission");
}

export async function fetchAvailablePermissions(
  wsId: string,
): Promise<string[]> {
  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${wsId}/permissions/available`,
  );
  if (!response.ok) throw new Error("Failed to fetch available permissions");
  return response.json() as Promise<string[]>;
}

// --- Roles & Matrix ---

export async function createWorkspaceRole(wsId: string, data: { name: string; description?: string; color: string }): Promise<WorkspaceRole> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/roles`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) {
    const error = await response.text();
    throw new Error(error || "Failed to create role");
  }
  return response.json() as Promise<WorkspaceRole>;
}

export async function updateWorkspaceRole(wsId: string, roleId: string, data: { name?: string; description?: string; color?: string }): Promise<WorkspaceRole> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/roles/${roleId}`, {
    method: "PUT",
    body: JSON.stringify(data),
  });
  if (!response.ok) {
    const error = await response.text();
    throw new Error(error || "Failed to update role");
  }
  return response.json() as Promise<WorkspaceRole>;
}

export async function deleteWorkspaceRole(wsId: string, roleId: string): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/roles/${roleId}`, {
    method: "DELETE",
  });
  if (!response.ok) {
    const error = await response.text();
    throw new Error(error || "Failed to delete role");
  }
}

export async function fetchRolePermissions(wsId: string, roleId: string): Promise<string[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/roles/${roleId}/permissions`);
  if (!response.ok) throw new Error("Failed to fetch role permissions");
  return response.json() as Promise<string[]>;
}

export async function updateRolePermissions(wsId: string, roleId: string, permissions: string[]): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/roles/${roleId}/permissions`, {
    method: "PUT",
    body: JSON.stringify(permissions),
  });
  if (!response.ok) {
    const error = await response.text();
    throw new Error(error || "Failed to update role permissions");
  }
}
