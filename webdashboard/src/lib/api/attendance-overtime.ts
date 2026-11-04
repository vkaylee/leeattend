import { client } from "./client";
import { ApiError } from "./errors";
import { apiFetch } from "./helpers";
import {
  AttendanceDailySummary,
  PaginatedResponse,
} from "./types";

// --- Overtime Approval System ---

export async function approveOt(
  wsId: string,
  employeeId: string,
  data: { date: string; approved_minutes: number; manager_note?: string; status: "approved" | "rejected" }
): Promise<void> {
  await client.POST("/v1/workspaces/{ws_id}/attendance/approve-ot/{employee_id}", {
    params: { path: { ws_id: wsId, employee_id: employeeId } },
    body: data
  });
}

export async function fetchAttendanceSummaries(
  wsId: string,
  start: string,
  end: string,
  employeeId?: string
): Promise<AttendanceDailySummary[]> {
  const { data: resData } = await client.GET("/v1/workspaces/{ws_id}/attendance/summaries", {
    params: {
      path: { ws_id: wsId },
      query: { start_date: start, end_date: end, employee_id: employeeId }
    }
  });
  return Array.isArray(resData) ? resData : (resData as { data?: AttendanceDailySummary[] }).data ?? [];
}

export async function fetchAttendanceSummariesPaginated(
  wsId: string,
  start: string,
  end: string,
  options?: {
    employeeId?: string;
    page?: number;
    limit?: number;
    otStatus?: string;
  }
): Promise<PaginatedResponse<AttendanceDailySummary>> {
  const { data: resData } = await client.GET("/v1/workspaces/{ws_id}/attendance/summaries", {
    params: {
      path: { ws_id: wsId },
      query: {
        start_date: start,
        end_date: end,
        employee_id: options?.employeeId,
        page: options?.page,
        limit: options?.limit,
        ot_status: options?.otStatus,
      }
    }
  });
  return resData;
}

export async function enterPaperOt(
  wsId: string,
  employeeId: string,
  data: {
    date: string;
    claimed_minutes: number;
    reason: string;
    paper_form_photo_url?: string;
  }
): Promise<AttendanceDailySummary> {
  const { data: resData } = await client.POST("/v1/workspaces/{ws_id}/attendance/paper-forms/overtime/{employee_id}", {
    params: { path: { ws_id: wsId, employee_id: employeeId } },
    body: data
  });
  return resData;
}

export async function bulkUploadPaperOt(
  wsId: string,
  file: File
): Promise<{
  total_rows: number;
  success_count: number;
  failed_count: number;
  errors: {
    line_number: number;
    employment_id: string;
    error_message: string;
  }[];
}> {
  const formData = new FormData();
  formData.append("file", file);

  const res = await apiFetch(`/v1/workspaces/${wsId}/attendance/paper-forms/overtime/bulk`, {
    method: "POST",
    body: formData,
  });
  if (!res.ok) await ApiError.rejectResponse(res);
  return res.json() as Promise<{
    total_rows: number;
    success_count: number;
    failed_count: number;
    errors: { line_number: number; employment_id: string; error_message: string }[];
  }>;
}

// --- Staging Logs (Unknown Users) ---

/** Provider metadata captured at punch time. */
export interface StagingLogRawData {
  verification_score?: number;
}

export interface StagingLog {
  created_at: string;
  id: string;
  punch_time: string;
  raw_data: StagingLogRawData | null;
  raw_user_id: string;
  status: string;
  workspace_id: string;
}

export interface StagingLogsResponse {
  data: StagingLog[];
  limit: number;
  page: number;
  total: number;
}

export async function fetchStagingLogs(
  page = 1,
  limit = 50
): Promise<StagingLogsResponse> {
  const res = await apiFetch(`/v1/attendance/staging-logs?page=${page}&limit=${limit}`);
  if (!res.ok) await ApiError.rejectResponse(res);
  return res.json() as Promise<StagingLogsResponse>;
}

export async function resolveStagingLog(
  logId: string,
  employeeId: string
): Promise<void> {
  const res = await apiFetch(`/v1/attendance/staging-logs/${encodeURIComponent(logId)}/resolve`, {
    method: "POST",
    body: JSON.stringify({ employee_id: employeeId }),
  });
  if (!res.ok) await ApiError.rejectResponse(res);
}

export async function dismissStagingLog(logId: string): Promise<void> {
  const res = await apiFetch(`/v1/attendance/staging-logs/${encodeURIComponent(logId)}`, {
    method: "DELETE",
  });
  if (!res.ok) await ApiError.rejectResponse(res);
}
