import { API_BASE_URL } from "./config";
import { apiFetch, setAccessToken, setRefreshToken, setCurrentUserId, clearAuthSession } from "./helpers";
import { AuthResponse, RegisterRequest, ChangePasswordRequest, User } from "./types";

export async function login(username: string, password: string): Promise<AuthResponse> {
  const response = await apiFetch(`${API_BASE_URL}/auth/login`, {
    method: "POST",
    body: JSON.stringify({ email: username, password }),
  });
  if (!response.ok) {
    let errorMessage = "Login failed";
    try {
      const errorData = (await response.json()) as { error?: { message?: string } };
      if (errorData.error?.message) {
        errorMessage = errorData.error.message;
      }
    } catch {
      // Ignore JSON parse errors and use default message
    }
    throw new Error(errorMessage);
  }
  const data = (await response.json()) as AuthResponse;
  setAccessToken(data.access_token);
  setRefreshToken(data.refresh_token);
  setCurrentUserId(data.user.id);
  return data;
}

export async function logout(): Promise<void> {
  await apiFetch(`${API_BASE_URL}/auth/logout`, { method: "POST" });
  clearAuthSession();
}

export async function register(data: RegisterRequest): Promise<AuthResponse> {
  const response = await apiFetch(`${API_BASE_URL}/auth/register`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) {
    const error = await response.text();
    throw new Error(error || "Registration failed");
  }
  return response.json() as Promise<AuthResponse>;
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

export async function fetchMe(): Promise<User> {
  const response = await apiFetch(`${API_BASE_URL}/me`);
  if (!response.ok) throw new Error("Failed to fetch profile");
  return response.json() as Promise<User>;
}

export async function updateMe(data: Partial<User>): Promise<User> {
  const response = await apiFetch(`${API_BASE_URL}/me`, {
    method: "PATCH",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to update profile");
  return response.json() as Promise<User>;
}

export async function fetchUsers(): Promise<User[]> {
  const response = await apiFetch(`${API_BASE_URL}/users`);
  if (!response.ok) throw new Error("Failed to fetch users");
  return response.json() as Promise<User[]>;
}

export async function fetchDevSuperUser(): Promise<{
  user_id: string;
  workspace_id: string;
  access_token?: string;
  refresh_token?: string;
}> {
  const response = await apiFetch(`${API_BASE_URL}/dev/super-user`);
  if (!response.ok) throw new Error("Failed to fetch dev super user");
  return response.json() as Promise<{ user_id: string; workspace_id: string; access_token?: string; refresh_token?: string; }>;
}

export async function fetchMyPermissions(): Promise<{
  permissions: string[];
  roles: string[];
  managed_department_ids: string[] | null;
}> {
  const response = await apiFetch(`${API_BASE_URL}/me/permissions`);
  if (!response.ok) throw new Error("Failed to fetch my permissions");
  return response.json() as Promise<{ permissions: string[]; roles: string[]; managed_department_ids: string[] | null; }>;
}
