import { client } from "./client";
// @ts-nocheck
import { apiFetch, setAccessToken, setRefreshToken, setCurrentUserId, clearAuthSession } from "./helpers";
import { AuthResponse, RegisterRequest, ChangePasswordRequest, User } from "./types";

export async function login(username: string, password: string): Promise<AuthResponse> {
  const { data } = await client.POST("/auth/login", {
    body: { email: username, password },
  });
  if (!data) throw new Error("Login failed");
  setAccessToken(data.access_token);
  setRefreshToken(data.refresh_token);
  setCurrentUserId(data.user.id);
  return data;
}

export async function logout(): Promise<void> {
  await client.POST("/auth/logout", {
    body: { refresh_token: "" },
  });
  clearAuthSession();
}

export async function register(data: RegisterRequest): Promise<AuthResponse> {
  const { data: resData } = await client.POST("/auth/register", {
    body: data,
  });
  if (!resData) throw new Error("Registration failed");
  return resData;
}

export async function changePassword(
  data: ChangePasswordRequest,
): Promise<string> {
  const { data: resData, error } = await client.POST("/auth/change-password", {
    body: data,
  });
  if (error) throw new Error("Failed to change password");
  return "Password changed successfully";
}

export async function fetchMe(): Promise<User> {
  const { data } = await client.GET("/me");
  if (!data) throw new Error("Failed to fetch profile");
  return data;
}

export async function updateMe(data: Partial<User>): Promise<User> {
  const { data: resData } = await client.PATCH("/me", {
    body: data,
  });
  if (!resData) throw new Error("Failed to update profile");
  return resData;
}

export async function fetchUsers(): Promise<User[]> {
  const { data } = await client.GET("/users");
  if (!data) throw new Error("Failed to fetch users");
  return data;
}

export async function fetchDevSuperUser(): Promise<{
  user_id: string;
  workspace_id: string;
  access_token?: string;
  refresh_token?: string;
}> {
  const response = await apiFetch(`/v1/dev/super-user`);
  if (!response.ok) throw new Error("Failed to fetch dev super user");
  return response.json() as Promise<{ user_id: string; workspace_id: string; access_token?: string; refresh_token?: string; }>;
}

export async function fetchMyPermissions(): Promise<{
  permissions: string[];
  roles: string[];
  managed_department_ids: string[] | null;
}> {
  const { data } = await client.GET("/me/permissions");
  if (!data) throw new Error("Failed to fetch my permissions");
  return data;
}
