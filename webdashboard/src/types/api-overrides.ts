/**
 * Explicit nullable annotations for known API response shape mismatches
 * where the OpenAPI-generated types in api-generated.ts don't match
 * actual backend behavior.
 *
 * This file augments (not replaces) the existing Omit<>/& override pattern
 * used in domain-specific API files. It targets the common case of
 * paginated response envelope mismatches and nullable responses.
 *
 * @see lib/api/types.ts for domain-specific overrides
 * @see lib/api/api-generated-reexports.ts for 1:1 generated type re-exports
 */

/**
 * Safely extract a data array from API responses that may be:
 * - null/undefined (empty response)
 * - a flat array (schema says T[])
 * - a paginated response (schema says { data: T[], total, page, limit })
 */
export function unwrapDataArray<T>(response: unknown): T[] {
  if (response === null || response === undefined) return [];
  if (Array.isArray(response)) return response as T[];
  const wrapped = response as { data?: T[] };
  return wrapped.data ?? [];
}

/**
 * Nullable response wrapper — use when the generated type says `T`
 * but the backend may return null (e.g., 204 No Content, missing resource).
 */
export type Nullable<T> = T | null | undefined;

/**
 * Widen a generated-type field to include null/undefined.
 * Use when the OpenAPI schema declares a field as non-null, but the backend
 * may omit or null it at runtime.  The double-cast through `unknown` breaks
 * the type chain so that both tsc and ESLint's no-unnecessary-condition rule
 * treat the result as genuinely nullable.
 *
 * @example
 *   // Before: config.endpoint_url ?? ""  // ESLint flags ?? as unnecessary
 *   // After:  maybeNull(config.endpoint_url) ?? ""
 */
export function maybeNull<T>(value: T): T | null | undefined {
  return value;
}
