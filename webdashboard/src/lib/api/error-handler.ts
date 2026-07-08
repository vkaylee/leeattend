/**
 * Centralized API error → user-friendly message mapper.
 *
 * Maps an unknown error (typically thrown from `openapi-fetch` via `customFetch`)
 * to a translated toast message key plus an optional severity so the global
 * QueryClient `onError` handler can surface consistent feedback.
 *
 * Consumed by the `onError` handler attached to the global QueryClient in
 * `src/app.tsx`. Individual mutations may still override with `.onError`.
 */

import { ApiError } from "./errors";

export type ErrorSeverity = "error" | "warning";

export interface ParsedApiError {
  /** i18n key (without namespace prefix) — see `common.errors.*`. */
  messageKey: string;
  /** Original error (for logs / Sentry). */
  original: unknown;
  /** Toast severity. */
  severity: ErrorSeverity;
}

/**
 * Translate an unknown error into a `ParsedApiError`.
 *
 * Pure function — no i18n call here so it stays import-safe from `app.tsx`
 * (which would create a circular dep with `src/i18n/config.ts`).
 */
export function parseApiError(error: unknown): ParsedApiError {
  // 1. Structured ApiError thrown by openapi-fetch wrapper
  if (error instanceof ApiError) {
    if (error.isUnauthorized) {
      return { messageKey: "errors.unauthorized", severity: "warning", original: error };
    }
    if (error.isNotFound) {
      return { messageKey: "errors.not_found", severity: "warning", original: error };
    }
    if (error.isValidation) {
      return { messageKey: "errors.validation", severity: "warning", original: error };
    }
    // 403 Forbidden
    if (error.status === 403) {
      return { messageKey: "errors.forbidden", severity: "warning", original: error };
    }
    // Server-side error (5xx)
    if (error.status >= 500) {
      return { messageKey: "errors.server", severity: "error", original: error };
    }
    // Other ApiError — use its message if present
    if (error.errorBody?.message) {
      return { messageKey: "", severity: "error", original: error };
    }
    return { messageKey: "errors.server", severity: "error", original: error };
  }

  // 2. Network / fetch failure (offline, CORS, DNS)
  if (error instanceof TypeError && error.message === "Failed to fetch") {
    return { messageKey: "errors.network", severity: "error", original: error };
  }

  // 3. String error (rare)
  if (typeof error === "string") {
    return { messageKey: "", severity: "error", original: error };
  }

  // 4. Generic fallback
  return { messageKey: "errors.unknown", severity: "error", original: error };
}
