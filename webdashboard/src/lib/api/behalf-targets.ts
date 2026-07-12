/**
 * Fetch target employees that the current user can act on behalf of.
 *
 * Uses the new `/behalf-assignments/my-targets` API endpoint that resolves
 * both direct employee assignments and department-level assignments into
 * a concrete list of Employee records.
 */

import type { Employee } from "./types";

import { ApiError } from "./errors";
import { apiFetch } from "./helpers";

export type BehalfDomain =
  | "leave"
  | "attendance_adjustment"
  | "overtime_request";

const DOMAIN_PATH_MAP: Record<BehalfDomain, string> = {
  leave: "leave/behalf-assignments/my-targets",
  attendance_adjustment: "attendance/adjustments/behalf-assignments/my-targets",
  overtime_request: "overtime/requests/behalf-assignments/my-targets",
};

export async function fetchMyBehalfTargets(
  wsId: string,
  domain: BehalfDomain,
): Promise<Employee[]> {
  const path = DOMAIN_PATH_MAP[domain];
  const response = await apiFetch(
    `/v1/workspaces/${wsId}/${path}`,
  );

  if (!response.ok) await ApiError.rejectResponse(response);

  return response.json() as Promise<Employee[]>;
}
