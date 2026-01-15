import { API_BASE_URL } from "./config";
import { apiFetch } from "./helpers";
import { Workspace, WorkspaceMember, MemberRolesResponse, MemberExtraRole } from "./types";

export async function fetchWorkspaces(): Promise<Workspace[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces`);
  if (!response.ok) throw new Error("Failed to fetch workspaces");
  return response.json();
}

export async function createWorkspace(data: { name: string } | string): Promise<Workspace> {
  const payload = typeof data === "string" ? { name: data } : data;
  const response = await apiFetch(`${API_BASE_URL}/workspaces`, {
    method: "POST",
    body: JSON.stringify(payload),
  });
  if (!response.ok) throw new Error("Failed to create workspace");
  return response.json();
}

export async function fetchWorkspace(wsId: string): Promise<Workspace> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}`);
  if (!response.ok) throw new Error("Failed to fetch workspace");
  return response.json();
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
  return response.json();
}

export async function fetchWorkspaceMembers(
  wsId: string,
): Promise<WorkspaceMember[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/members`);
  if (!response.ok) throw new Error("Failed to fetch members");
  return response.json();
}

export async function addWorkspaceMember(
  wsId: string,
  email: string,
  role: string,
): Promise<void> {
  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${wsId}/members`,
    {
      method: "POST",
      body: JSON.stringify({ email, role }),
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
  role: string,
): Promise<void> {
  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${wsId}/members/${userId}`,
    {
      method: "PATCH",
      body: JSON.stringify({ role }),
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
  return response.json();
}

export async function assignExtraRole(
  wsId: string,
  userId: string,
  role: string,
  expiresAt: string,
): Promise<MemberExtraRole> {
  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${wsId}/members/${userId}/roles`,
    {
      method: "POST",
      body: JSON.stringify({ role, expires_at: expiresAt }),
    },
  );
  if (!response.ok) throw new Error("Failed to assign role");
  return response.json();
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

export async function clearWorkspaceData(wsId: string, group: string): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/dev/workspaces/${wsId}/clear?group=${group}`, {
    method: "POST",
  });
  if (!response.ok) {
    const error = await response.text();
    throw new Error(error || "Failed to clear workspace data");
  }
}
