// Low-level API fetch helpers (token refresh, unauthorized handler, timeout).
// Split out from helpers.ts to keep the helpers file under max-lines.

import { logError } from "@/lib/logger";
import { ROUTES } from "@/lib/routes";

import { API_BASE_URL } from "./config";
import { ApiError } from "./errors";
import {
  clearAuthSession,
  getAccessToken,
  setAccessToken,
} from "./helpers";
import { clearMemorySession, hasAuthSessionHint } from "./token-store";

export const TIMEOUT = {
  /** Standard API calls (10 seconds) */
  DEFAULT: 10000,
  /** Report generation and large file operations (3 minutes) */
  REPORT_OPERATION: 180000,
  /** Health checks and quick operations (5 seconds) */
  QUICK: 5000,
} as const;

let refreshPromise: Promise<boolean> | null = null;
const TOKEN_REFRESH_WINDOW_MS = 60_000;

export function isAccessTokenExpiring(token: string, now = Date.now()): boolean {
  const payload = token.split(".")[1];
  if (!payload) return false;

  try {
    const normalizedPayload = payload.replace(/-/g, "+").replace(/_/g, "/");
    const paddedPayload = normalizedPayload.padEnd(Math.ceil(normalizedPayload.length / 4) * 4, "=");
    const claims = JSON.parse(atob(paddedPayload)) as { exp?: unknown };
    return typeof claims.exp === "number" && Number.isFinite(claims.exp)
      && claims.exp * 1000 <= now + TOKEN_REFRESH_WINDOW_MS;
  } catch {
    return false;
  }
}

export function handleSessionExpired(expectedUnauthenticated = false): never {
  if (expectedUnauthenticated) {
    // No client session exists. Avoid refresh/logout requests and expected 401 noise.
    clearMemorySession();
  } else {
    logError("Session expired (401). Redirecting to login...", { component: "apiFetch" });
    clearAuthSession();
  }
  if (window.location.pathname !== ROUTES.LOGIN) {
    window.location.href = ROUTES.LOGIN;
  }
  throw new Error("Session expired");
}

export async function forceRefreshToken(): Promise<boolean> {
  if (refreshPromise) {
    return refreshPromise;
  }

  refreshPromise = (async () => {
    try {
      // The refresh token is HttpOnly, so send a cookie-only request.
      // Omitting both body and Content-Type lets the backend use its cookie
      // fallback without attempting to deserialize a fake JSON payload.
      const refreshResponse = await fetch(`${API_BASE_URL}/v1/auth/refresh`, {
        method: "POST",
        credentials: "include",
      });

      if (!refreshResponse.ok) {
        if (refreshResponse.status === 401) return false;
        await ApiError.rejectResponse(refreshResponse);
      }

      const data = (await refreshResponse.json()) as {
        access_token?: unknown;
        refresh_token?: unknown;
      };
      if (typeof data.access_token !== "string" || data.access_token.trim() === "") {
        return false;
      }
      setAccessToken(data.access_token);
      // NOTE: we deliberately IGNORE the returned refresh_token. The new
      // refresh token is rotated inside the HttpOnly cookie by the backend
      // (Set-Cookie header on this response). Storing it in JS-accessible
      // memory would be a security regression.
      return true;
    } catch (e: unknown) {
      if (e instanceof ApiError) throw e;
      logError(e, { component: "TokenRefresh" });
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
  const hadSession = Boolean(getAccessToken()) || hasAuthSessionHint();
  if (!hadSession) {
    handleSessionExpired(true);
  }

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

  handleSessionExpired(!hadSession);
}

export function getFullUrl(url: string): string {
  if (url.startsWith("http") || url.startsWith(API_BASE_URL)) {
    return url;
  }
  const path = url.startsWith("/") ? url : `/${url}`;
  return `${API_BASE_URL}${path}`;
}

