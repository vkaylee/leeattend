import { API_BASE_URL } from "@/lib/api/config";
import {
  getAccessToken,
  getRefreshToken,
  setAccessToken,
  setRefreshToken,
} from "@/lib/api/helpers";

/**
 * Build WebSocket URL for report streaming
 * Uses wss: for HTTPS, ws: for HTTP
 * Includes JWT token as query param (required by ws_auth_middleware)
 */
export function buildWebSocketUrl(jobId: string): string {
  const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
  const host = window.location.host;
  const token = getAccessToken();
  const params = token ? `?token=${encodeURIComponent(token)}` : "";
  return `${protocol}//${host}/api/ws/reports/${jobId}${params}`;
}

/**
 * Calculate exponential backoff delay
 * Returns delays: 1s, 2s, 4s for retries 0, 1, 2
 */
export function getBackoffDelay(retryCount: number): number {
  return Math.pow(2, retryCount) * 1000;
}

/**
 * Attempt to refresh JWT token (same logic as apiFetch 401 handler)
 * Returns true if refresh succeeded, false otherwise
 */
export async function tryRefreshToken(): Promise<boolean> {
  const refreshToken = getRefreshToken();
  if (!refreshToken) return false;

  try {
    const response = await fetch(`${API_BASE_URL}/auth/refresh`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ refresh_token: refreshToken }),
    });

    if (response.ok) {
      const data = await response.json() as { access_token: string; refresh_token: string };
      setAccessToken(data.access_token);
      setRefreshToken(data.refresh_token);
      
      return true;
    }
  } catch (e) {
    console.error("[ReportStream] Token refresh failed", e);
  }

  return false;
}
