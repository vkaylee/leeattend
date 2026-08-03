import type { BehalfAssignment, CreateBehalfAssignmentRequest } from "./types";

import { ApiError } from "./errors";
/**
 * behalf-assignment-factory — generic CRUD factory for behalf-assignment endpoints.
 *
 * Four domain modules (OT claim, attendance adjustment, leave, OT request)
 * duplicated the same list/create/delete pattern.  This factory produces a
 * typed { list, create, delete } triple from a base-path config, eliminating
 * the duplication.
 *
 * Uses {@link apiFetch} directly (not the typed {@link client}) because the
 * OpenAPI-generated client requires path *literals* — a config-driven dynamic
 * string is incompatible with the strict path union type.  All callers mock
 * `apiFetch` in tests, so this choice is transparent to consumers.
 */
import { apiFetch } from "./helpers";

// ── Config ────────────────────────────────────────────────────────────────────

export interface BehalfAssignmentConfig {
  /**
   * Base path template for the behalf-assignment collection endpoint.
   * Must include a `{ws_id}` placeholder that will be substituted at call time.
   *
   * @example "/v1/workspaces/{ws_id}/leave/behalf-assignments"
   */
  basePath: string;

  /**
   * When `true`, the list endpoint wraps its payload in `{ data: […] }`.
   * Defaults to `false` (most endpoints return the array directly).
   *
   * This flag exists because the OT-request behalf endpoint wraps the
   * response while the other three domains do not — a backend inconsistency
   * tracked for future normalization.
   */
  dataWrapper?: boolean;
}

// ── Return type ───────────────────────────────────────────────────────────────

export interface BehalfAssignmentApi<T = BehalfAssignment> {
  /** Create a new behalf assignment. */
  create(wsId: string, payload: CreateBehalfAssignmentRequest): Promise<T>;

  /** Delete a behalf assignment by its ID. */
  delete(wsId: string, id: string): Promise<void>;

  /** Fetch all behalf assignments for the given workspace. */
  list(wsId: string): Promise<T[]>;
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

  /** Substitute `{ws_id}` → the concrete workspace ID. */
  const collectionUrl = (wsId: string): string =>
    basePath.replace("{ws_id}", wsId);

  /** Collection URL + `/{id}` for single-resource operations. */
  const itemUrl = (wsId: string, id: string): string =>
    `${collectionUrl(wsId)}/${id}`;

  return {
    list: async (wsId: string): Promise<T[]> => {
      const response = await apiFetch(collectionUrl(wsId));
      if (!response.ok) await ApiError.rejectResponse(response);
      const json = (await response.json()) as T[] | { data: T[] };
      if (dataWrapper) {
        return (json as { data: T[] }).data;
      }
      return json as T[];
    },

    create: async (
      wsId: string,
      payload: CreateBehalfAssignmentRequest,
    ): Promise<T> => {
      const response = await apiFetch(collectionUrl(wsId), {
        method: "POST",
        body: JSON.stringify(payload),
      });
      if (!response.ok) await ApiError.rejectResponse(response);
      return (await response.json()) as T;
    },

    delete: async (wsId: string, id: string): Promise<void> => {
      const response = await apiFetch(itemUrl(wsId, id), {
        method: "DELETE",
      });
      if (!response.ok) await ApiError.rejectResponse(response);
    },
  };
}
