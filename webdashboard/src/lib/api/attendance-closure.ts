import { client } from "./client";

export type AttendancePeriodState = "open" | "closed" | "reopened" | string;

export interface AttendancePeriodStatus {
  closure_id?: string | null;
  closure_number?: number;
  current_closure_id?: string | null;
  day_count?: number;
  payroll_period_found?: boolean;
  payroll_status?: string;
  period_id?: string | null;
  snapshot_hash?: string;
  status: AttendancePeriodState;
  version: number;
}

export interface AttendanceClosureMutationResponse extends AttendancePeriodStatus {
  day_count?: number;
  prior_closure_id?: string | null;
  snapshot_hash?: string;
}

export interface AttendanceClosureRequest {
  reason: string;
  version: number;
}

const periodPath = "/api/v1/attendance/periods/{year}/{month}";

export async function fetchAttendancePeriodStatus(
  year: number,
  month: number,
  employerId: string,
): Promise<AttendancePeriodStatus> {
  const { data, response } = await client.GET<AttendancePeriodStatus | null>(periodPath, {
    params: { path: { year, month }, query: { employer_id: employerId } },
  });
  if (response.status === 204 || data === null) {
    return { status: "open", version: 1, payroll_period_found: false };
  }
  return data;
}

export async function closeAttendancePeriod(
  year: number,
  month: number,
  employerId: string,
  request: AttendanceClosureRequest,
): Promise<AttendanceClosureMutationResponse> {
  const { data } = await client.POST<AttendanceClosureMutationResponse>(`${periodPath}/close`, {
    params: { path: { year, month }, query: { employer_id: employerId } },
    body: request,
  });
  return data;
}

export async function reopenAttendancePeriod(
  year: number,
  month: number,
  employerId: string,
  request: AttendanceClosureRequest,
): Promise<AttendanceClosureMutationResponse> {
  const { data } = await client.POST<AttendanceClosureMutationResponse>(`${periodPath}/reopen`, {
    params: { path: { year, month }, query: { employer_id: employerId } },
    body: request,
  });
  return data;
}
