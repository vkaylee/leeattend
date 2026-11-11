import type { BehalfAssignment, CreateBehalfAssignmentRequest } from "./types";

import { ApiError } from "./errors";
/**
 * behalf-assignment-factory — generic CRUD factory for behalf-assignment endpoints.
 *
 * Four domain modules (OT claim, attendance adjustment, leave, OT request)
 * duplicated the same list/create/delete pattern.  This factory produces a
 * typed { list, create, delete } triple from a concrete collection path,
 * eliminating the duplication.
 *
 * Uses {@link apiFetch} directly (not the typed {@link client}) because the
 * config-driven dynamic URL is not a compile-time literal path.  All callers
 * mock `apiFetch` in tests, so this choice is transparent to consumers.
 */
import { apiFetch } from "./helpers";

// ── Config ────────────────────────────────────────────────────────────────────

export interface BehalfAssignmentConfig {
  /**
   * Concrete collection path for the behalf-assignment endpoint — the
   * organization is implied by the shared contract and never appears in the URL.
   *
   * @example "/api/v1/leave/behalf-assignments"
   */
  basePath: string;

  /**
   * When `true`, the list endpoint wraps its payload in `{ data: […] }`.
   * Defaults to `false` (most endpoints return the array directly).
   *
   * This flag exists because the OT-request behalf endpoint wraps the
   * response while the other domains do not — a backend inconsistency
   * tracked for future normalization.
   */
  dataWrapper?: boolean;
}

// ── Return type ───────────────────────────────────────────────────────────────

export interface BehalfAssignmentApi<T = BehalfAssignment> {
  /** Create a new behalf assignment. */
  create(payload: CreateBehalfAssignmentRequest): Promise<T>;

  /** Delete a behalf assignment by its ID. */
  delete(id: string): Promise<void>;

  /** Fetch all behalf assignments. */
  list(): Promise<T[]>;
}

// ── Factory ───────────────────────────────────────────────────────────────────

/**
 * Create a typed CRUD triple for a behalf-assignment endpoint.
 *
 * @typeParam T - The assignment record type returned by the endpoint.
 *                Defaults to the shared {@link BehalfAssignment} schema type.
 *                Override for domain-specific types (e.g. `OtClaimBehalfAssignment`).
 */
export function createBehalfAssignmentApi<T = BehalfAssignment>(
  config: BehalfAssignmentConfig,
): BehalfAssignmentApi<T> {
  const { basePath, dataWrapper = false } = config;

  /** Collection URL + `/{id}` for single-resource operations. */
  const itemUrl = (id: string): string => `${basePath}/${encodeURIComponent(id)}`;

  return {
    list: async (): Promise<T[]> => {
      const response = await apiFetch(basePath);
      if (!response.ok) await ApiError.rejectResponse(response);
      const json = (await response.json()) as T[] | { data: T[] };
      if (dataWrapper) {
        return (json as { data: T[] }).data;
      }
      return json as T[];
    },

    create: async (payload: CreateBehalfAssignmentRequest): Promise<T> => {
      const response = await apiFetch(basePath, {
        method: "POST",
        body: JSON.stringify(payload),
      });
      if (!response.ok) await ApiError.rejectResponse(response);
      return (await response.json()) as T;
    },

    delete: async (id: string): Promise<void> => {
      const response = await apiFetch(itemUrl(id), {
        method: "DELETE",
      });
      if (!response.ok) await ApiError.rejectResponse(response);
    },
  };
}
