import { ROUTES } from "@/lib/routes";

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

  // Call backend to clear HttpOnly cookies
  fetch(`${API_BASE_URL}/auth/logout`, { 
    method: "POST", 
    credentials: "include",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({})
  }).catch(console.error);
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
 * Extended RequestInit with custom timeout
 */
export interface ApiFetchOptions extends RequestInit {
  /**
   * Request timeout in milliseconds
   * @default 10000 (10 seconds)
   */
  timeout?: number;
}

/**
 * Timeout constants for different operation types
 */
export const TIMEOUT = {
  /** Standard API calls (10 seconds) */
  DEFAULT: 10000,
  /** Report generation and large file operations (3 minutes) */
  REPORT_OPERATION: 180000,
  /** Health checks and quick operations (5 seconds) */
  QUICK: 5000,
} as const;

/**
 * Wrapper for fetch to handle 401 and refresh token
 *
 * @param url - API endpoint URL
 * @param options - Fetch options with optional timeout (default: 10 seconds)
 */
export async function apiFetch(
  url: string,
  options: ApiFetchOptions = {},
): Promise<Response> {
  const controller = new AbortController();
  // Default timeout: 10 seconds for normal API calls
  const timeoutMs = options.timeout ?? TIMEOUT.DEFAULT;
  const id = setTimeout(() => controller.abort(), timeoutMs);

  const headers = { ...getHeaders(), ...options.headers };

  // Remove timeout from options to avoid passing it to fetch
  const { timeout, ...fetchOptions } = options;

  if (fetchOptions.body instanceof FormData) {
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
      ...fetchOptions,
      headers,
      credentials: "include", // Send HttpOnly cookies
      signal: controller.signal
    });
    clearTimeout(id);
    
    if (response.status === 401 && !fullUrl.includes("/auth/login")) {
      // Attempt to refresh token
      const refreshToken = getRefreshToken();
      if (refreshToken) {
        try {
          const refreshResponse = await fetch(`${API_BASE_URL}/auth/refresh`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            credentials: "include", // Send HttpOnly cookies
            body: JSON.stringify({ refresh_token: refreshToken }),
          });

          if (refreshResponse.ok) {
            const data = await refreshResponse.json();
            setAccessToken(data.access_token);
            setRefreshToken(data.refresh_token);

            // Retry original request with new token
            const retryHeaders = { ...headers, Authorization: `Bearer ${data.access_token}` };
            const retryResponse = await fetch(fullUrl, {
              ...fetchOptions,
              headers: retryHeaders,
              credentials: "include", // Send HttpOnly cookies
              signal: controller.signal
            });
            clearTimeout(id);
            return retryResponse;
          }
        } catch (e) {
          console.error("Token refresh failed", e);
        }
      }
      
      console.warn("Session expired (401). Redirecting to login...");
      clearAuthSession();
      if (window.location.pathname !== ROUTES.LOGIN) {
        window.location.href = ROUTES.LOGIN;
      }
      throw new Error("Session expired");
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
