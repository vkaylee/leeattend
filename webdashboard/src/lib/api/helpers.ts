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
  fetch(`${API_BASE_URL}/v1/auth/logout`, { 
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
    headers.Authorization = `Bearer ${token}`;
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

  // Set Accept-Language to sync with frontend's active locale
  const lng = localStorage.getItem("i18nextLng");
  if (lng) {
    headers["Accept-Language"] = lng;
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
  const id = setTimeout(() => { controller.abort(); }, timeoutMs);

  const headers = new Headers();
  
  // Add base headers
  const baseHeaders = getHeaders();
  Object.entries(baseHeaders).forEach(([k, v]) => {
    if (v) headers.set(k, v);
  });

  // Add custom headers (overriding base headers)
  if (options.headers) {
    if (options.headers instanceof Headers) {
      options.headers.forEach((v, k) => { headers.set(k, v); });
    } else if (Array.isArray(options.headers)) {
      options.headers.forEach(([k, v]) => { headers.set(k, v); });
    } else {
      Object.entries(options.headers).forEach(([k, v]) => {
        if (v) headers.set(k, v);
      });
    }
  }

  // Remove timeout from options to avoid passing it to fetch
  const { timeout: _timeout, ...fetchOptions } = options;

  if (fetchOptions.body instanceof FormData) {
    headers.delete("Content-Type");
  }

  const fullUrl = getFullUrl(url);

  try {
    const response = await fetch(fullUrl, {
      ...fetchOptions,
      headers,
      credentials: "include",
      signal: controller.signal
    });
    clearTimeout(id);
    
    if (response.status === 401 && !fullUrl.includes("/auth/login")) {
      return await handleUnauthorized({
        fullUrl,
        fetchOptions,
        headers,
        controller,
        timeoutId: id
      });
    }

    return response;
  } catch (error) {
    clearTimeout(id);
    throw error;
  }
}

function getFullUrl(url: string): string {
  if (url.startsWith("http") || url.startsWith(API_BASE_URL)) {
    return url;
  }
  const path = url.startsWith("/") ? url : `/${url}`;
  return `${API_BASE_URL}${path}`;
}

interface UnauthorizedParams {
  controller: AbortController;
  fetchOptions: RequestInit;
  fullUrl: string;
  headers: Headers;
  timeoutId: number | ReturnType<typeof setTimeout>;
}

let refreshPromise: Promise<boolean> | null = null;

export async function forceRefreshToken(): Promise<boolean> {
  if (refreshPromise) {
    return refreshPromise; // Xếp hàng chờ nếu đang có request refresh
  }

  refreshPromise = (async () => {
    const refreshToken = getRefreshToken();
    if (!refreshToken) return false;

    try {
      const refreshResponse = await fetch(`${API_BASE_URL}/v1/auth/refresh`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        credentials: "include",
        body: JSON.stringify({ refresh_token: refreshToken }),
      });

      if (refreshResponse.ok) {
        const data = (await refreshResponse.json()) as { access_token: string; refresh_token: string };
        setAccessToken(data.access_token);
        setRefreshToken(data.refresh_token);
        return true;
      }
    } catch (e) {
      console.error("Token refresh failed", e);
    }
    return false;
  })().finally(() => {
    refreshPromise = null;
  });

  return refreshPromise;
}

async function handleUnauthorized({
  fullUrl,
  fetchOptions,
  headers,
  controller,
  timeoutId
}: UnauthorizedParams): Promise<Response> {
  const success = await forceRefreshToken();
  if (success) {
    const token = getAccessToken();
    const retryHeaders = new Headers(headers);
    if (token) {
      retryHeaders.set("Authorization", `Bearer ${token}`);
    }
    
    const retryResponse = await fetch(fullUrl, {
      ...fetchOptions,
      headers: retryHeaders,
      credentials: "include",
      signal: controller.signal
    });
    clearTimeout(timeoutId);
    return retryResponse;
  }
  
  console.warn("Session expired (401). Redirecting to login...");
  clearAuthSession();
  if (window.location.pathname !== ROUTES.LOGIN) {
    window.location.href = ROUTES.LOGIN;
  }
  throw new Error("Session expired");
}

export async function fetchHealth(): Promise<{ status: string }> {
  const response = await apiFetch(`/v1/health`);
  return response.json() as Promise<{ status: string }>;
}
