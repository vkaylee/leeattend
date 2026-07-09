import {
  getFullUrl,
  handleUnauthorized,
  TIMEOUT,
} from "./api-fetch-helpers";
import { API_BASE_URL } from "./config";
import {
  clearMemorySession,
  getAccessToken as storeGetAccessToken,
  getCurrentUserId as storeGetCurrentUserId,
  getCurrentWorkspaceId as storeGetCurrentWorkspaceId,
  setAccessToken as storeSetAccessToken,
  setCurrentUserId as storeSetCurrentUserId,
  setCurrentWorkspaceId as storeSetCurrentWorkspaceId,
} from "./token-store";

export { TIMEOUT, forceRefreshToken } from "./api-fetch-helpers";

// Access token + context-ID accessors read from the in-memory store
// (NOT localStorage — see token-store.ts).
export const getAccessToken = storeGetAccessToken;
export const getCurrentWorkspaceId = storeGetCurrentWorkspaceId;
export const getCurrentUserId = storeGetCurrentUserId;

// setAccessToken / setCurrentWorkspaceId / setCurrentUserId are re-exported
// from token-store so existing callers keep working unchanged.
export const setAccessToken = storeSetAccessToken;
export const setCurrentWorkspaceId = storeSetCurrentWorkspaceId;
export const setCurrentUserId = storeSetCurrentUserId;

// NOTE: getRefreshToken / setRefreshToken are INTENTIONALLY removed.
// The refresh token is persisted exclusively as an HttpOnly cookie set by the
// backend (`apiserver/src/handlers/auth.rs::create_auth_cookies`). JS can no
// longer read it — by design. The refresh endpoint reads the cookie directly,
// so the frontend never needs to touch the refresh token.

export const clearAuthSession = () => {
  // Wipe in-memory credentials immediately (synchronous, before navigation).
  clearMemorySession();

  // Ask the backend to expire the HttpOnly cookies. Best-effort: if this call
  // fails (network down, tab closed) the cookies still expire naturally via
  // Max-Age. We intentionally do NOT `await` — logout must be fire-and-forget
  // so the user isn't blocked on a slow/flaky network call.
  fetch(`${API_BASE_URL}/v1/auth/logout`, {
    method: "POST",
    credentials: "include",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({}),
  }).catch(() => { /* silent — cookies expire via Max-Age server-side */ });
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

export async function fetchHealth(): Promise<{ status: string }> {
  const response = await apiFetch(`/v1/health`);
  return response.json() as Promise<{ status: string }>;
}
