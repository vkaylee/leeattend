import { client } from "./client";
import { ApiError } from "./errors";
import { apiFetch } from "./helpers";
import {
  AttendanceImportJob,
  AttendanceSummary,
  DailyAttendanceReport,
  PaginatedResponse,
  AttendanceReportSort,
} from "./types";

export interface FetchAttendanceReportParams {
  departmentId?: string | null;
  employeeIds?: string[];
  employeeStatus?: string;
  employerId?: string | null;
  employmentId?: string | null;
  end: string;
  limit?: number;
  page?: number;
  shiftType?: string[];
  sort?: AttendanceReportSort;
  start: string;
  type?: "summary" | "daily";
  wsId: string;
}

export async function fetchAttendanceReport(
  params: FetchAttendanceReportParams
): Promise<PaginatedResponse<AttendanceSummary | DailyAttendanceReport>> {
  const {
    wsId,
    start,
    end,
    type,
    page,
    limit,
    sort,
    employeeIds,
    departmentId,
    employeeStatus,
    employerId,
    employmentId,
    shiftType,
  } = params;

  const { data: resData } = await client.GET("/v1/workspaces/{ws_id}/reports/attendance", {
    params: {
      path: { ws_id: wsId },
      query: {
        start_date: start,
        end_date: end,
        type: type,
        page: page,
        limit: limit,
        sort: sort,
        employee_ids: employeeIds?.join(",") ?? undefined,
        department_id: departmentId ?? undefined,
        employee_status: employeeStatus,
        employer_id: employerId ?? undefined,
        employment_id: employmentId ?? undefined,
        shift_type: shiftType?.join(",") ?? undefined,
      },
    },
  });

  return resData;
}

export async function uploadAttendance(
  wsId: string,
  file: File,
  endpoint: string
): Promise<{ job_id: string }> {
  const formData = new FormData();
  formData.append("file", file);

  const url = endpoint === "/employees/upload"
    ? `/v1/workspaces/${wsId}/employees/upload`
    : `/v1/workspaces/${wsId}/attendance/upload`;

  const res = await apiFetch(url, {
    method: "POST",
    body: formData,
  });
  if (!res.ok) await ApiError.rejectResponse(res);
  return res.json();
}

export async function fetchLatestImportJob(
  wsId: string,
  targetType?: "log" | "adjustment" | "employee" | "leave_import"
) {
  const { data: resData } = await client.GET("/v1/workspaces/{ws_id}/attendance/import-jobs/latest", {
    params: { path: { ws_id: wsId }, query: { target_type: targetType } }
  });
  return resData;
}

export async function fetchImportJobStatus(
  wsId: string,
  jobId: string
): Promise<AttendanceImportJob> {
  const { data: resData } = await client.GET("/v1/workspaces/{ws_id}/attendance/import-jobs/{id}", {
    params: { path: { ws_id: wsId, id: jobId } }
  });
  return resData;
}

export async function reprocessAttendance(
  wsId: string,
  employeeId: string,
  startDate: string,
  endDate: string
): Promise<void> {
  await client.POST("/workspaces/{workspace_id}/attendance/reprocess/{employee_id}", {
    params: { path: { workspace_id: wsId, employee_id: employeeId }, query: { start_date: startDate, end_date: endDate } }
  });
}

export async function reprocessAllAttendance(
  wsId: string,
  startDate: string,
  endDate: string
): Promise<{ message: string; job_id: string }> {
  const { data: resData } = await client.POST("/workspaces/{workspace_id}/attendance/reprocess-all", {
    params: { path: { workspace_id: wsId }, query: { start_date: startDate, end_date: endDate } }
  });
  return resData as { message: string; job_id: string };
}

// ── No-show timeline report ──────────────────────────────────────────

export interface FetchNoShowReportParams {
  /** Calendar date to query (yyyy-MM-dd). Only shifts overlapping this day
   *  that have already started are included. */
  date: string;
  employeeIds?: string[];
  limit?: number;
  page?: number;
  shiftType?: string[];
  wsId: string;
}

/** No-show report: employees on a shift overlapping `date` who have not
 *  punched in yet (status `absent`), filtered to only shifts that have
 *  *already started*. Backed by `GET /v1/workspaces/{ws_id}/reports/no-show`.
 *
 *  NOTE: uses the untyped `apiFetch` because the generated OpenAPI routes are
 *  synced from the backend via `leedevkit manage sync:api`. Until that runs
 *  after the new endpoint compiles, `client.GET` has no typed path for this
 *  route. Replace with a typed `client.GET("/v1/workspaces/{ws_id}/…")` once
 *  the schema is regenerated. */
export async function fetchNoShowReport(
  params: FetchNoShowReportParams,
): Promise<PaginatedResponse<DailyAttendanceReport>> {
  const { wsId, date, page, limit, employeeIds, shiftType } = params;
  const search = new URLSearchParams();
  search.set("date", date);
  if (page != null) search.set("page", String(page));
  if (limit != null) search.set("limit", String(limit));
  if (employeeIds && employeeIds.length > 0) search.set("employee_ids", employeeIds.join(","));
  if (shiftType && shiftType.length > 0) search.set("shift_type", shiftType.join(","));

  const url = `/v1/workspaces/${wsId}/reports/no-show?${search.toString()}`;
  const res = await apiFetch(url);
  if (!res.ok) await ApiError.rejectResponse(res);
  return (await res.json()) as PaginatedResponse<DailyAttendanceReport>;
}

export interface GenericJobProgress {
  completed_batches: number;
  failed_batches: number;
  percentage: number;
  total_batches: number;
}

export interface GenericJobStatus {
  completed_at?: string;
  created_at: string;
  error?: string;
  job_id: string;
  progress: GenericJobProgress;
  status: "pending" | "processing" | "completed" | "failed";
  task_type: string;
}

export async function checkReprocessStatus(wsId: string, jobId: string): Promise<GenericJobStatus> {
  const { data: resData } = await client.GET("/workspaces/{ws_id}/reports/jobs/{job_id}", {
    params: { path: { ws_id: wsId, job_id: jobId } }
  });
  return resData as GenericJobStatus;
}

export interface AttendanceStats {
  early_out_count: number;
  late_in_count: number;
  missing_punch_count: number;
  present_count: number;
  total_count: number;
}
