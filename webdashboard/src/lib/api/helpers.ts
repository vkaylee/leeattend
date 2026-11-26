import {
  clearAuthSession,
  getFullUrl,
  handleSessionExpired,
  handleUnauthorized,
  isAccessTokenExpiring,
  forceRefreshToken,
  TIMEOUT,
} from "./api-fetch-helpers";
export { clearAuthSession, forceRefreshToken, TIMEOUT } from "./api-fetch-helpers";
import {
  getAccessToken as storeGetAccessToken,
  getCurrentUserId as storeGetCurrentUserId,
  setAccessToken as storeSetAccessToken,
  setCurrentUserId as storeSetCurrentUserId,
} from "./token-store";


// Access token and user context remain in memory; organization context is backend-owned.
export const getAccessToken = storeGetAccessToken;
export const getCurrentUserId = storeGetCurrentUserId;

export const setAccessToken = storeSetAccessToken;
export const setCurrentUserId = storeSetCurrentUserId;

// NOTE: getRefreshToken / setRefreshToken are INTENTIONALLY removed.
// The refresh token is persisted exclusively as an HttpOnly cookie set by the
// backend. JS can no longer read it — by design. The refresh endpoint reads
// the cookie directly, so the frontend never needs to touch the refresh token.

function getHeaders() {
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
  };
  const token = getAccessToken();
  if (token) {
    headers.Authorization = `Bearer ${token}`;
  } else {
    const userId = getCurrentUserId();
    if (userId) headers["X-User-Id"] = userId;
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
const PUBLIC_AUTH_ENDPOINT = /\/auth\/(?:login|register|refresh|logout|forgot-password|reset-password)(?:$|[/?])/;

function isPublicAuthEndpoint(url: string): boolean {
  return PUBLIC_AUTH_ENDPOINT.test(url);
}

async function refreshExpiringToken(token: string | null, isPublicEndpoint: boolean): Promise<void> {
  if (!token || isPublicEndpoint || !isAccessTokenExpiring(token)) return;
  const refreshed = await forceRefreshToken();
  if (!refreshed) handleSessionExpired();
}

function applyHeaderEntries(headers: Headers, entries: Iterable<[string, string]>): void {
  for (const [key, value] of entries) {
    if (value) headers.set(key, value);
  }
}

function createHeaders(customHeaders?: HeadersInit): Headers {
  const headers = new Headers();
  applyHeaderEntries(headers, Object.entries(getHeaders()));
  if (!customHeaders) return headers;

  if (customHeaders instanceof Headers) {
    customHeaders.forEach((value, key) => {
      headers.set(key, value);
    });
  } else if (Array.isArray(customHeaders)) {
    customHeaders.forEach(([key, value]) => {
      headers.set(key, value);
    });
  } else {
    applyHeaderEntries(headers, Object.entries(customHeaders));
  }
  return headers;
}

function prepareFetchOptions(options: ApiFetchOptions, headers: Headers): RequestInit {
  const { timeout: _timeout, ...fetchOptions } = options;
  if (fetchOptions.body instanceof FormData) headers.delete("Content-Type");
  return { ...fetchOptions, headers };
}

export async function apiFetch(
  url: string,
  options: ApiFetchOptions = {},
): Promise<Response> {
  const fullUrl = getFullUrl(url);
  const isPublicEndpoint = isPublicAuthEndpoint(fullUrl);
  await refreshExpiringToken(getAccessToken(), isPublicEndpoint);

  const controller = new AbortController();
  const timeoutMs = options.timeout ?? TIMEOUT.DEFAULT;
  const timeoutId = setTimeout(() => {
    controller.abort();
  }, timeoutMs);
  const headers = createHeaders(options.headers);
  const fetchOptions = prepareFetchOptions(options, headers);

  try {
    const response = await fetch(fullUrl, {
      ...fetchOptions,
      credentials: "include",
      signal: controller.signal,
    });
    clearTimeout(timeoutId);

    if (response.status !== 401 || isPublicEndpoint) return response;
    return await handleUnauthorized({
      fullUrl,
      fetchOptions,
      headers,
      controller,
      timeoutId,
    });
  } catch (error) {
    clearTimeout(timeoutId);
    throw error;
  }
}
