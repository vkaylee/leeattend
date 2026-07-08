// Low-level API fetch helpers (token refresh, unauthorized handler, timeout).
// Split out from helpers.ts to keep the helpers file under max-lines.

import { ROUTES } from "@/lib/routes";

import { API_BASE_URL } from "./config";
import {
  clearAuthSession,
  getAccessToken,
  getRefreshToken,
  setAccessToken,
  setRefreshToken,
} from "./helpers";

export const TIMEOUT = {
  /** Standard API calls (10 seconds) */
  DEFAULT: 10000,
  /** Report generation and large file operations (3 minutes) */
  REPORT_OPERATION: 180000,
  /** Health checks and quick operations (5 seconds) */
  QUICK: 5000,
} as const;

let refreshPromise: Promise<boolean> | null = null;

export async function forceRefreshToken(): Promise<boolean> {
  if (refreshPromise) {
    return refreshPromise;
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
        const data = (await refreshResponse.json()) as {
          access_token: string;
          refresh_token: string;
        };
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

interface UnauthorizedParams {
  controller: AbortController;
  fetchOptions: RequestInit;
  fullUrl: string;
  headers: Headers;
  timeoutId: number | ReturnType<typeof setTimeout>;
}

export async function handleUnauthorized({
  fullUrl,
  fetchOptions,
  headers,
  controller,
  timeoutId,
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
      signal: controller.signal,
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

export function getFullUrl(url: string): string {
  if (url.startsWith("http") || url.startsWith(API_BASE_URL)) {
    return url;
  }
  const path = url.startsWith("/") ? url : `/${url}`;
  return `${API_BASE_URL}${path}`;
}
