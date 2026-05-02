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
  const response = await apiFetch(`${API_BASE_URL ?? ""}/system/roles`);
  if (!response.ok) throw new Error("Failed to fetch system roles");
  return response.json() as Promise<SystemRoleInfo[]>;
}

export async function fetchUserSystemRoles(userId: string): Promise<SystemRoleAssignment[]> {
  const response = await apiFetch(`${API_BASE_URL ?? ""}/system/users/${userId ?? ""}/roles`);
  if (!response.ok) throw new Error("Failed to fetch user system roles");
  return response.json() as Promise<SystemRoleAssignment[]>;
}

export async function assignSystemRole(userId: string, request: AssignSystemRoleRequest): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL ?? ""}/system/users/${userId ?? ""}/roles`, {
    method: "POST",
    body: JSON.stringify(request),
  });
  if (!response.ok) throw new Error("Failed to assign system role");
}

export async function revokeSystemRole(userId: string, roleName: string): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL ?? ""}/system/users/${userId ?? ""}/roles/${roleName ?? ""}`, {
    method: "DELETE",
  });
  if (!response.ok) throw new Error("Failed to revoke system role");
}

 
export async function fetchSystemHealth(): Promise<Record<string, unknown>> {
  const response = await apiFetch(`${API_BASE_URL ?? ""}/system/health`);
  if (!response.ok) throw new Error("Failed to fetch system health");
  return response.json() as Promise<Record<string, unknown>>;
}
