const API_BASE_URL = "/api";
const WS_BASE_URL = `ws://${window.location.host}/ws`;

export interface DeviceStatus {
  device_id: string;
  status: "online" | "offline";
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

export interface UpdateUserRequest {
  full_name?: string;
  avatar_url?: string;
  bio?: string;
}

export interface ChangePasswordRequest {
  old_password: string;
  new_password: string;
}

export interface WorkspaceMember {
  workspace_id: string;
  user_id: string;
  role: "owner" | "admin" | "member";
  email: string;
  full_name?: string;
  avatar_url?: string;
}

// --- Auth Management ---

export function getAccessToken(): string | null {
  return localStorage.getItem("access_token");
}

export function setAccessToken(token: string) {
  localStorage.setItem("access_token", token);
}

export function getRefreshToken(): string | null {
  return localStorage.getItem("refresh_token");
}

export function setRefreshToken(token: string) {
  localStorage.setItem("refresh_token", token);
}

export function getCurrentWorkspaceId(): string | null {
  return localStorage.getItem("active_workspace_id");
}

export function setCurrentWorkspaceId(id: string) {
  localStorage.setItem("active_workspace_id", id);
}

export function getCurrentUserId(): string | null {
  return localStorage.getItem("active_user_id");
}

export const setCurrentUserId = (id: string) => {
  localStorage.setItem("active_user_id", id);
};

export const clearAuthSession = () => {
  localStorage.removeItem("active_user_id");
  localStorage.removeItem("active_workspace_id");
  localStorage.removeItem("access_token");
  localStorage.removeItem("refresh_token");
};

function getHeaders() {
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
  };
  const wsId = getCurrentWorkspaceId();
  if (wsId) {
    headers["X-Workspace-Id"] = wsId;
  }
  const token = getAccessToken();
  if (token) {
    headers["Authorization"] = `Bearer ${token}`;
  } else {
    // Fallback for dev mode
    const userId = getCurrentUserId();
    if (userId) headers["X-User-Id"] = userId;
  }
  return headers;
}

/**
 * Wrapper for fetch to handle 401 and refresh token
 */
async function apiFetch(
  url: string,
  options: RequestInit = {},
): Promise<Response> {
  const headers = { ...getHeaders(), ...options.headers };
  const response = await fetch(url, { ...options, headers });

  if (response.status === 401 && getRefreshToken()) {
    console.warn("Access token expired, attempting refresh...");
    try {
      const refreshRes = await fetch(`${API_BASE_URL}/auth/refresh`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ refresh_token: getRefreshToken() }),
      });

      if (refreshRes.ok) {
        const data = await refreshRes.json();
        setAccessToken(data.access_token);
        setRefreshToken(data.refresh_token);

        // Retry original request with new token
        const retryHeaders = { ...getHeaders(), ...options.headers };
        return fetch(url, { ...options, headers: retryHeaders });
      } else {
        console.error("Refresh token invalid, logging out");
        clearAuthSession();
        window.location.href = "/login";
      }
    } catch (e) {
      clearAuthSession();
      window.location.href = "/login";
    }
  }

  return response;
}

export async function fetchAgents(): Promise<AgentStatus[]> {
  const response = await apiFetch(`${API_BASE_URL}/agents`);
  if (!response.ok) throw new Error("Failed to fetch agents");
  return response.json();
}

export async function fetchDevices(): Promise<Device[]> {
  const response = await apiFetch(`${API_BASE_URL}/devices`);
  if (!response.ok) throw new Error("Failed to fetch devices");
  return response.json();
}

export async function createDevice(
  device: Partial<Device>,
): Promise<{ id: string }> {
  const response = await apiFetch(`${API_BASE_URL}/devices`, {
    method: "POST",
    body: JSON.stringify(device),
  });
  if (!response.ok) throw new Error("Failed to create device");
  return response.json();
}

export async function deleteDevice(id: string): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/devices/${id}`, {
    method: "DELETE",
  });
  if (!response.ok) throw new Error("Failed to delete device");
}

export async function fetchAuthorizedAgents(): Promise<AuthorizedAgent[]> {
  const response = await apiFetch(`${API_BASE_URL}/authorized-agents`);
  if (!response.ok) throw new Error("Failed to fetch authorized agents");
  return response.json();
}

export async function createAuthorizedAgent(
  agent: Partial<AuthorizedAgent>,
): Promise<{ id: string }> {
  const response = await apiFetch(`${API_BASE_URL}/authorized-agents`, {
    method: "POST",
    body: JSON.stringify(agent),
  });
  if (!response.ok) throw new Error("Failed to create authorized agent");
  return response.json();
}

export async function updateAgentTrust(
  id: string,
  is_trusted: boolean,
): Promise<void> {
  const response = await apiFetch(
    `${API_BASE_URL}/authorized-agents/${id}/trust`,
    {
      method: "PUT",
      body: JSON.stringify({ is_trusted }),
    },
  );
  if (!response.ok) throw new Error("Failed to update agent trust");
}

export async function updateAuthorizedAgent(
  id: string,
  agent: Partial<AuthorizedAgent>,
): Promise<AuthorizedAgent> {
  const response = await apiFetch(`${API_BASE_URL}/authorized-agents/${id}`, {
    method: "PATCH",
    body: JSON.stringify(agent),
  });
  if (!response.ok) throw new Error("Failed to update authorized agent");
  return response.json();
}

export async function deleteAuthorizedAgent(id: string): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/authorized-agents/${id}`, {
    method: "DELETE",
  });
  if (!response.ok) throw new Error("Failed to delete authorized agent");
}

// --- User Profile ---

export async function fetchMe(): Promise<User> {
  const response = await apiFetch(`${API_BASE_URL}/me`);
  if (!response.ok) throw new Error("Failed to fetch profile");
  return response.json();
}

export async function updateMe(data: Partial<User>): Promise<User> {
  const response = await apiFetch(`${API_BASE_URL}/me`, {
    method: "PATCH",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to update profile");
  return response.json();
}

export async function changePassword(
  data: ChangePasswordRequest,
): Promise<string> {
  const response = await apiFetch(`${API_BASE_URL}/auth/change-password`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) {
    const error = await response.text();
    throw new Error(error || "Failed to change password");
  }
  return response.text();
}

export async function fetchUsers(): Promise<User[]> {
  const response = await apiFetch(`${API_BASE_URL}/users`);
  if (!response.ok) throw new Error("Failed to fetch users");
  return response.json();
}

// --- Workspace Management ---

export async function fetchWorkspaces(): Promise<Workspace[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces`);
  if (!response.ok) throw new Error("Failed to fetch workspaces");
  return response.json();
}

export async function createWorkspace(name: string): Promise<Workspace> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces`, {
    method: "POST",
    body: JSON.stringify({ name }),
  });
  if (!response.ok) throw new Error("Failed to create workspace");
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

export interface RegisterRequest {
  email: string;
  password: string;
  full_name: string;
}

export async function register(data: RegisterRequest): Promise<AuthResponse> {
  const response = await fetch(`${API_BASE_URL}/auth/register`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(data),
  });
  if (!response.ok) {
    const error = await response.text();
    throw new Error(error || "Registration failed");
  }
  return response.json();
}

export async function fetchDevSuperUser(): Promise<{
  user_id: string;
  workspace_id: string;
  access_token?: string;
  refresh_token?: string;
}> {
  const response = await fetch(`${API_BASE_URL}/dev/super-user`);
  if (!response.ok) throw new Error("Failed to fetch dev super user");
  return response.json();
}
