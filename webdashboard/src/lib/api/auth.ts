import { client, fetch } from "./client";
import { ApiError } from "./errors";
import { apiFetch, setAccessToken, setCurrentUserId, clearAuthSession, TIMEOUT } from "./helpers";
import { loadRuntimeFeatureFlags } from "@/lib/feature-flags";

import { markAuthSession } from "./token-store";
import {
  AuthResponse,
  ChangePasswordRequest,
  RegisterRequest,
  UpdateUserRequest,
  User,
} from "./types";

export async function login(username: string, password: string): Promise<AuthResponse> {
  const { data } = await client.POST<AuthResponse>("/api/v1/auth/login", {
    body: { email: username, password },
  });
  setAccessToken(data.access_token);
  setCurrentUserId(data.user.id);
  markAuthSession();
  void loadRuntimeFeatureFlags();
  return data;
}

export async function logout(): Promise<void> {
  await client.POST("/api/v1/auth/logout", {
    body: { refresh_token: "" },
  });
  clearAuthSession();
}

export async function register(data: RegisterRequest): Promise<AuthResponse> {
  const { data: resData } = await client.POST<AuthResponse>("/api/v1/auth/register", {
    body: data,
  });
  return resData;
}

export async function changePassword(
  data: ChangePasswordRequest,
): Promise<string> {
  await client.POST("/api/v1/auth/change-password", {
    body: data,
  });
  return "Password changed successfully";
}

export async function fetchMe(): Promise<User> {
  const { data } = await client.GET<User>("/api/v1/me");
  return data;
}

export async function updateMe(data: UpdateUserRequest): Promise<User> {
  const { data: resData } = await client.PATCH<User>("/api/v1/me", {
    body: data,
  });
  return resData;
}

export async function fetchUsers(): Promise<User[]> {
  const { data } = await client.GET<User[]>("/api/v1/users");
  return data;
}


export async function fetchMyPermissions(): Promise<{
  permissions: string[];
  roles: string[];
  managed_department_ids: string[] | null;
}> {
  const { data } = await fetch<{
    permissions: string[];
    roles: string[];
    managed_department_ids: string[] | null;
  }>("GET", "/api/v1/me/permissions");
  return data;
}

/**
 * Request a password reset link. The backend always returns 200 OK
 * to prevent email enumeration, even if the email is not registered.
 */
export async function forgotPassword(email: string): Promise<void> {
  const response = await apiFetch("/api/v1/auth/forgot-password", {
    method: "POST",
    body: JSON.stringify({ email }),
    timeout: TIMEOUT.DEFAULT,
  });
  if (!response.ok) {
    const errorBody = (await response.json().catch(() => null)) as { message?: string } | null;
    throw new Error(
      errorBody?.message ?? "Failed to send reset link",
    );
  }
}

/**
 * Reset password using a token received via email.
 * Returns a success message on completion.
 */
export async function resetPassword(
  token: string,
  newPassword: string,
): Promise<string> {
  const response = await apiFetch("/api/v1/auth/reset-password", {
    method: "POST",
    body: JSON.stringify({ token, new_password: newPassword }),
    timeout: TIMEOUT.DEFAULT,
  });
  if (!response.ok) await ApiError.rejectResponse(response);
  const data = (await response.json()) as { message?: string };
  return data.message ?? "Password reset successfully";
}
