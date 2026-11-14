import type {
  AttendanceDailySummary,
  PaginatedResponse,
} from "./types";

import { client } from "./client";
import { ApiError } from "./errors";
import { apiFetch } from "./helpers";

// --- Overtime Approval System ---

export async function approveOt(
  employeeId: string,
  data: { date: string; approved_minutes: number; manager_note?: string; status: "approved" | "rejected" }
): Promise<void> {
  await client.POST("/api/v1/attendance/approve-ot/{employee_id}", {
    params: { path: { employee_id: employeeId } },
    body: data,
  });
}

export async function fetchAttendanceSummaries(
  start: string,
  end: string,
  employeeId?: string
): Promise<AttendanceDailySummary[]> {
  const { data: resData } = await client.GET<PaginatedResponse<AttendanceDailySummary> | AttendanceDailySummary[]>(
    "/api/v1/attendance/summaries",
    {
      params: {
        query: { start_date: start, end_date: end, employee_id: employeeId },
      },
    },
  );
  if (Array.isArray(resData)) {
    return resData;
  }
  return resData && "data" in resData && Array.isArray(resData.data) ? resData.data : [];
}

export async function fetchAttendanceSummariesPaginated(
  start: string,
  end: string,
  options?: {
    employeeId?: string;
    page?: number;
    limit?: number;
    otStatus?: string;
  }
): Promise<PaginatedResponse<AttendanceDailySummary>> {
  const { data: resData } = await client.GET<PaginatedResponse<AttendanceDailySummary>>("/api/v1/attendance/summaries", {
    params: {
      query: {
        start_date: start,
        end_date: end,
        employee_id: options?.employeeId,
        page: options?.page,
        limit: options?.limit,
        ot_status: options?.otStatus,
      },
    },
  });
  return resData;
}

export async function enterPaperOt(
  employeeId: string,
  data: {
    date: string;
    claimed_minutes: number;
    reason: string;
    paper_form_photo_url?: string;
  }
): Promise<AttendanceDailySummary> {
  const { data: resData } = await client.POST<AttendanceDailySummary>(
    "/api/v1/attendance/paper-forms/overtime/{employee_id}",
    {
      params: { path: { employee_id: employeeId } },
      body: data,
    },
  );
  return resData;
}

export async function bulkUploadPaperOt(
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

  const res = await apiFetch("/api/v1/attendance/paper-forms/overtime/bulk", {
    method: "POST",
    body: formData,
  });
  if (!res.ok) await ApiError.rejectResponse(res);
  return (await res.json()) as {
    total_rows: number;
    success_count: number;
    failed_count: number;
    errors: { line_number: number; employment_id: string; error_message: string }[];
  };
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
  const { data: resData } = await client.GET<StagingLogsResponse>("/api/v1/attendance/staging-logs", {
    params: {
      query: { page, limit },
    },
  });
  return resData;
}

export async function resolveStagingLog(
  logId: string,
  employeeId: string
): Promise<void> {
  await client.POST("/api/v1/attendance/staging-logs/{id}/resolve", {
    params: { path: { id: logId } },
    body: { employee_id: employeeId },
  });
}

export async function dismissStagingLog(logId: string): Promise<void> {
  await client.DELETE("/api/v1/attendance/staging-logs/{id}", {
    params: { path: { id: logId } },
  });
}
