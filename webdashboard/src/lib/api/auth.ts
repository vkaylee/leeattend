import { client } from "./client";
import { apiFetch, setAccessToken, setCurrentUserId, clearAuthSession, TIMEOUT } from "./helpers";
import { AuthResponse, RegisterRequest, ChangePasswordRequest, User } from "./types";

export async function login(username: string, password: string): Promise<AuthResponse> {
  const { data } = await client.POST("/auth/login", {
    body: { email: username, password },
  });
  if (!data) throw new Error("Login failed");
  // access_token is held in-memory (see token-store.ts). The corresponding
  // refresh_token is persisted by the backend inside an HttpOnly cookie —
  // the frontend never sees it.
  setAccessToken(data.access_token);
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
  const { error } = await client.POST("/auth/change-password", {
    body: data,
  });
  if (error as unknown) throw new Error("Failed to change password");
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
  return data as unknown as {
    permissions: string[];
    roles: string[];
    managed_department_ids: string[] | null;
  };
}

/**
 * Request a password reset link. The backend always returns 200 OK
 * to prevent email enumeration, even if the email is not registered.
 */
export async function forgotPassword(email: string): Promise<void> {
  const response = await apiFetch("/v1/auth/forgot-password", {
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
  const response = await apiFetch("/v1/auth/reset-password", {
    method: "POST",
    body: JSON.stringify({ token, new_password: newPassword }),
    timeout: TIMEOUT.DEFAULT,
  });
  if (!response.ok) {
    const errorBody = (await response.json().catch(() => null)) as { message?: string } | null;
    throw new Error(
      errorBody?.message ?? "Failed to reset password",
    );
  }
  const data = (await response.json()) as { message?: string };
  return data.message ?? "Password reset successfully";
}
