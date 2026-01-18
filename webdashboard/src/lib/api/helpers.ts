import { API_BASE_URL } from "./config";

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

  // Debug Headers (only in development)
  if (import.meta.env.DEV) {
    const isEnabled = localStorage.getItem("debug_enabled") === "true";
    if (isEnabled) {
      const impId = localStorage.getItem("debug_impersonate_id");
      if (impId) {
        headers["X-Debug-Impersonate-Id"] = impId;
      }
      const mockDate = localStorage.getItem("debug_mock_date");
      if (mockDate) {
        headers["X-Debug-Mock-Date"] = mockDate;
      }
    }
  }

  return headers;
}

/**
 * Wrapper for fetch to handle 401 and refresh token
 */
export async function apiFetch(
  url: string,
  options: RequestInit = {},
): Promise<Response> {
  const controller = new AbortController();
  const id = setTimeout(() => controller.abort(), 10000);

  const headers = { ...getHeaders(), ...options.headers };
  
  if (options.body instanceof FormData) {
    delete (headers as any)["Content-Type"];
  }

  // If url already starts with http/https or the API_BASE_URL, don't prepend API_BASE_URL
  let fullUrl = url;
  if (!url.startsWith("http") && !url.startsWith(API_BASE_URL)) {
    const path = url.startsWith("/") ? url : `/${url}`;
    fullUrl = `${API_BASE_URL}${path}`;
  }

  try {
    const response = await fetch(fullUrl, { 
      ...options, 
      headers,
      signal: controller.signal
    });
    clearTimeout(id);
    
    if (response.status === 401) {
        // Handle token refresh logic here if needed, or just return response
        // For now, let the caller handle 401 or global interceptor
    }

    return response;
  } catch (error) {
    clearTimeout(id);
    throw error;
  }
}

export async function fetchHealth(): Promise<{ status: string }> {
  const response = await apiFetch(`${API_BASE_URL}/health`);
  return response.json();
}
