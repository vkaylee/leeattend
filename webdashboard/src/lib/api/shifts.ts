import type { components } from "@/types/api-generated";

import { logError } from "@/lib/logger";

import type { Shift, EmployeeSchedule, AssignmentHistory, ShiftRoster, CreateShiftRequest, UpdateShiftRequest, ShiftVersionHistoryItem } from "./types";

import { client } from "./client";
import { ApiError, type AppErrorResponse, ComplianceConflictError, parseComplianceConflict } from "./errors";
import { apiFetch, TIMEOUT } from "./helpers";

interface PaginatedResponse<T> {
  data: T[];
  limit: number;
  page: number;
  total: number;
}

export async function fetchShifts(): Promise<Shift[]> {
  const { data } = await client.GET<PaginatedResponse<Shift>>("/v1/shifts");
  return data.data;
}

export async function createShift(data: CreateShiftRequest): Promise<Shift> {
  const { data: result } = await client.POST<Shift>("/v1/shifts", {
    body: data,
  });
  return result;
}

export async function updateShift(
  shiftId: string,
  data: UpdateShiftRequest
): Promise<Shift> {
  const { data: result } = await client.PUT<Shift>("/v1/shifts/{shift_id}", {
    params: {
      path: { shift_id: shiftId },
    },
    body: data,
  });
  return result;
}

export async function assignShift(
  data: { employment_id: string; shift_id: string; start_date: string; end_date?: string | null; allow_warnings?: boolean; override_reason?: string }
): Promise<void> {
  await client.POST("/v1/shifts/assign", {
    body: data,
  });
}

export const fetchSchedules = async (
  startDate: string,
  endDate: string,
  employmentIds?: string[]
): Promise<EmployeeSchedule[]> => {
  const { data } = await client.GET<PaginatedResponse<EmployeeSchedule>>("/v1/schedules", {
    params: {
      query: {
        start_date: startDate,
        end_date: endDate,
        employment_ids: employmentIds?.join(",") ?? undefined,
        limit: employmentIds && employmentIds.length > 0 ? 1000 : undefined,
      },
    },
  });
  return data.data;
};

export const fetchAssignmentHistory = async (): Promise<AssignmentHistory[]> => {
  const { data } = await client.GET<AssignmentHistory[]>("/v1/assignment-history");
  return data;
};

export const fetchEmployeeRosters = async (
  employeeId: string,
  startDate?: string,
  endDate?: string
): Promise<ShiftRoster[]> => {
  const { data } = await client.GET<PaginatedResponse<ShiftRoster>>("/v1/employees/{id}/rosters", {
    params: {
      path: { id: employeeId },
      query: {
        start_date: startDate ?? undefined,
        end_date: endDate ?? undefined,
      },
    },
  });
  return data.data;
};

export async function upsertRoster(
  employeeId: string,
  data: Omit<components["schemas"]["CreateRosterRequest"], 'employment_id'>
): Promise<ShiftRoster> {
  const body: components["schemas"]["CreateRosterRequest"] = { ...data, employment_id: employeeId };
  // Only include override fields when explicitly set so they don't pollute normal requests.
  if (data.allow_warnings !== undefined) body.allow_warnings = data.allow_warnings;
  if (data.override_reason !== undefined) body.override_reason = data.override_reason;
  const { data: result } = await client.POST<ShiftRoster>("/v1/employees/{id}/rosters", {
    params: {
      path: { id: employeeId },
    },
    body,
  });
  return result;
}

export async function bulkUpsertRoster(
  employeeId: string,
  items: (Omit<Partial<ShiftRoster>, "date"> & { date: string })[],
  opts?: { allow_warnings?: boolean; override_reason?: string },
): Promise<void> {
  // Bypass the client here: bulk roster writes trigger per-day attendance
  // reprocessing on the backend and can exceed the default 10s timeout for large
  // selections. Use apiFetch directly with the long report-operation timeout.
  const body: { items: typeof items; allow_warnings?: boolean; override_reason?: string } = { items };
  if (opts?.allow_warnings !== undefined) body.allow_warnings = opts.allow_warnings;
  if (opts?.override_reason !== undefined) body.override_reason = opts.override_reason;

  const response = await apiFetch(
    `/v1/employees/${encodeURIComponent(employeeId)}/rosters/bulk`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
      timeout: TIMEOUT.REPORT_OPERATION,
    },
  );
  if (!response.ok) {
    // Detect compliance conflict (409 with retry.violations) so callers can
    // surface the warning modal instead of a generic error toast.
    let parsedBody: unknown;
    try {
      parsedBody = JSON.parse(await response.clone().text());
    } catch (e: unknown) {
      // Ignore parse failures — fall through to rejectResponse
      logError(e, { component: 'bulkUpsertRoster.parseBody' });
    }
    const retry = parseComplianceConflict(parsedBody);
    if (retry) {
      throw new ComplianceConflictError(
        response.status,
        response.statusText,
        parsedBody as AppErrorResponse | undefined,
        retry,
      );
    }
    await ApiError.rejectResponse(response);
  }
}

export async function deleteRoster(employeeId: string, date: string): Promise<void> {
  await client.DELETE("/v1/employees/{id}/rosters/{date}", {
    params: {
      path: { id: employeeId, date },
    },
  });
}

/** Fetch version history for a shift template. */
export async function getShiftVersions(
  shiftId: string,
): Promise<ShiftVersionHistoryItem[]> {
  const { data } = await client.GET<ShiftVersionHistoryItem[]>("/v1/shifts/{shift_id}/versions", {
    params: {
      path: { shift_id: shiftId },
    },
  });
  return data;
}
