/**
 * Lấy địa chỉ API Base từ biến môi trường.
 * Mặc định là http://localhost:3000 nếu không được cấu hình.
 */
export const API_BASE_URL = (import.meta.env.VITE_API_URL || 'http://localhost:3000').replace(/\/$/, '');

/**
 * Tự động tạo URL WebSocket dựa trên API_BASE_URL
 */
export const SOCKET_URL = API_BASE_URL.replace(/^http/, 'ws') + '/ws';

export interface DeviceStatus {
  device_id: string;
  status: 'online' | 'offline';
  last_sync?: string;
  error?: string;
}

export interface AgentStatus {
  agent_id: string;
  last_seen: string;
  is_online: boolean;
  devices: DeviceStatus[];
}

export interface Device {
  id: string;
  agent_id: string;
  name: string;
  ip: string;
  port: number;
  is_enabled: boolean;
  last_status: string;
  created_at: string;
  workspace_id: string;
}

export interface AuthorizedAgent {
  id: string;
  name: string;
  token: string;
  is_trusted: boolean;
  last_ip?: string;
  created_at: string;
  is_active: boolean;
  workspace_id: string;
}

export interface User {
  id: string;
  email: string;
  created_at: string;
  full_name?: string;
  avatar_url?: string;
  bio?: string;
}

export interface Workspace {
  id: string;
  name: string;
  created_at: string;
}

export interface WorkspaceMember {
  workspace_id: string;
  user_id: string;
  role: 'owner' | 'admin' | 'member';
}

// Utility to get active workspace and user ID
export function getCurrentWorkspaceId(): string | null {
  return localStorage.getItem('active_workspace_id');
}

export function setCurrentWorkspaceId(id: string) {
  localStorage.setItem('active_workspace_id', id);
}

export function getCurrentUserId(): string | null {
  return localStorage.getItem('active_user_id');
}

export function setCurrentUserId(id: string) {
  localStorage.setItem('active_user_id', id);
}

function getHeaders() {
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
  };
  const wsId = getCurrentWorkspaceId();
  if (wsId) {
    headers['X-Workspace-Id'] = wsId;
  }
  const userId = getCurrentUserId();
  if (userId) {
    headers['X-User-Id'] = userId;
  }
  return headers;
}

export async function fetchAgents(): Promise<AgentStatus[]> {
  const response = await fetch(`${API_BASE_URL}/agents`, { headers: getHeaders() });
  if (!response.ok) throw new Error('Failed to fetch agents');
  return response.json();
}

export async function fetchDevices(): Promise<Device[]> {
  const response = await fetch(`${API_BASE_URL}/devices`, { headers: getHeaders() });
  if (!response.ok) throw new Error('Failed to fetch devices');
  return response.json();
}

export async function createDevice(device: Partial<Device>): Promise<{ id: string }> {
  const response = await fetch(`${API_BASE_URL}/devices`, {
    method: 'POST',
    headers: getHeaders(),
    body: JSON.stringify(device),
  });
  if (!response.ok) throw new Error('Failed to create device');
  return response.json();
}

export async function deleteDevice(id: string): Promise<void> {
  const response = await fetch(`${API_BASE_URL}/devices/${id}`, {
    method: 'DELETE',
    headers: getHeaders(),
  });
  if (!response.ok) throw new Error('Failed to delete device');
}

export async function fetchAuthorizedAgents(): Promise<AuthorizedAgent[]> {
  const response = await fetch(`${API_BASE_URL}/authorized-agents`, { headers: getHeaders() });
  if (!response.ok) throw new Error('Failed to fetch authorized agents');
  return response.json();
}

export async function createAuthorizedAgent(agent: Partial<AuthorizedAgent>): Promise<{ id: string }> {
  const response = await fetch(`${API_BASE_URL}/authorized-agents`, {
    method: 'POST',
    headers: getHeaders(),
    body: JSON.stringify(agent),
  });
  if (!response.ok) throw new Error('Failed to create authorized agent');
  return response.json();
}

export async function updateAgentTrust(id: string, is_trusted: boolean): Promise<void> {
  const response = await fetch(`${API_BASE_URL}/authorized-agents/${id}/trust`, {
    method: 'PUT',
    headers: getHeaders(),
    body: JSON.stringify({ is_trusted }),
  });
  if (!response.ok) throw new Error('Failed to update agent trust');
}

export async function updateAuthorizedAgent(id: string, agent: Partial<AuthorizedAgent>): Promise<AuthorizedAgent> {
  const response = await fetch(`${API_BASE_URL}/authorized-agents/${id}`, {
    method: 'PATCH',
    headers: getHeaders(),
    body: JSON.stringify(agent),
  });
  if (!response.ok) throw new Error('Failed to update authorized agent');
  return response.json();
}

export async function deleteAuthorizedAgent(id: string): Promise<void> {
  const response = await fetch(`${API_BASE_URL}/authorized-agents/${id}`, {
    method: 'DELETE',
    headers: getHeaders(),
  });
  if (!response.ok) throw new Error('Failed to delete authorized agent');
}

// --- User Profile ---

export async function fetchMe(): Promise<User> {
  const response = await fetch(`${API_BASE_URL}/me`, { headers: getHeaders() });
  if (!response.ok) throw new Error('Failed to fetch profile');
  return response.json();
}

export async function updateMe(data: Partial<User>): Promise<User> {
  const response = await fetch(`${API_BASE_URL}/me`, {
    method: 'PATCH',
    headers: getHeaders(),
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error('Failed to update profile');
  return response.json();
}

export async function fetchUsers(): Promise<User[]> {
  const response = await fetch(`${API_BASE_URL}/users`, { headers: getHeaders() });
  if (!response.ok) throw new Error('Failed to fetch users');
  return response.json();
}

// --- Workspace Management ---

export async function fetchWorkspaces(): Promise<Workspace[]> {
  const response = await fetch(`${API_BASE_URL}/workspaces`, { headers: getHeaders() });
  if (!response.ok) throw new Error('Failed to fetch workspaces');
  return response.json();
}

export async function createWorkspace(name: string): Promise<Workspace> {
  const response = await fetch(`${API_BASE_URL}/workspaces`, {
    method: 'POST',
    headers: getHeaders(),
    body: JSON.stringify({ name }),
  });
  if (!response.ok) throw new Error('Failed to create workspace');
  return response.json();
}

export async function fetchWorkspaceMembers(wsId: string): Promise<WorkspaceMember[]> {
  const response = await fetch(`${API_BASE_URL}/workspaces/${wsId}/members`, { headers: getHeaders() });
  if (!response.ok) throw new Error('Failed to fetch members');
  return response.json();
}

export async function addWorkspaceMember(wsId: string, email: string, role: string): Promise<void> {
  const response = await fetch(`${API_BASE_URL}/workspaces/${wsId}/members`, {
    method: 'POST',
    headers: getHeaders(),
    body: JSON.stringify({ email, role }),
  });
  if (!response.ok) throw new Error('Failed to add member');
}

export interface MemberRolesResponse {
  primary: string;
  extra: MemberExtraRole[];
}

export interface MemberExtraRole {
  id: string;
  workspace_id: string;
  user_id: string;
  role: string;
  assigned_at: string;
  expires_at: string;
}

export async function fetchMemberRoles(wsId: string, userId: string): Promise<MemberRolesResponse> {
  const response = await fetch(`${API_BASE_URL}/workspaces/${wsId}/members/${userId}/roles`, { headers: getHeaders() });
  if (!response.ok) throw new Error('Failed to fetch member roles');
  return response.json();
}

export async function assignExtraRole(wsId: string, userId: string, role: string, expiresAt: string): Promise<MemberExtraRole> {
  const response = await fetch(`${API_BASE_URL}/workspaces/${wsId}/members/${userId}/roles`, {
    method: 'POST',
    headers: getHeaders(),
    body: JSON.stringify({ role, expires_at: expiresAt }),
  });
  if (!response.ok) throw new Error('Failed to assign role');
  return response.json();
}

export async function revokeExtraRole(wsId: string, userId: string, roleId: string): Promise<void> {
  const response = await fetch(`${API_BASE_URL}/workspaces/${wsId}/members/${userId}/roles/${roleId}`, {
    method: 'DELETE',
    headers: getHeaders(),
  });
  if (!response.ok) throw new Error('Failed to revoke role');
}
