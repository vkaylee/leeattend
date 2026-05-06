// VITE_API_URL: Full API base URL including /api prefix
// Examples:
//   Dev container:   http://apiserver:3000/api  (only used for Vite proxy target)
//   Standalone EXE:  http://127.0.0.1:3003/api  (frontend calls directly)
//   Tauri App:       http://localhost:3000/api   (frontend calls directly)
const BASE_URL_FROM_ENV: string | undefined = (import.meta.env.VITE_API_URL as string | undefined)?.replace(/\/$/, "");

// Dev mode: Vite proxy handles /api → apiserver, so frontend just uses "/api"
// Production/Tauri: No proxy, use VITE_API_URL directly (must include /api)
export const API_BASE_URL: string = import.meta.env.DEV
  ? "/api"
  : (BASE_URL_FROM_ENV ?? "/api");

// WebSocket URL derived from API URL
const getWsBaseUrl = (): string => {
  if (typeof window === "undefined") {
    return "ws://localhost:3000/api/ws";
  }

  const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
  const host = window.location.host;

  if (import.meta.env.DEV) {
    return `${protocol}//${host}/api/ws`;
  }

  if (BASE_URL_FROM_ENV) {
    return `${BASE_URL_FROM_ENV.replace(/^http/, "ws")}/ws`;
  }

  return `${protocol}//${host}/api/ws`;
};

export const WS_BASE_URL: string = getWsBaseUrl();
