// @ts-nocheck
import { API_BASE_URL } from "./config";
import { apiFetch, getCurrentWorkspaceId, TIMEOUT } from "./helpers";
import {
  TimeEntry,
  AttendanceAdjustmentRequest,
  RawAttendanceLog,
  AttendanceImportJob,
  AttendanceSummary,
  DailyAttendanceReport,
  PaginatedResponse,
  AttendanceReportSort,
  ReportExportJob,
  CreateReportExportRequest,
  CreateReportExportResponse,
  WorkerMonitorStats, AttendanceDailySummary 
} from "./types";
import { client } from "./client";

export async function punchIn(
  wsId: string,
  data: { employee_id: string; entry_type?: string; note?: string },
): Promise<TimeEntry> {
  const { data: resData } = await client.POST("/workspaces/{workspace_id}/time/punch-in", {
    params: {
      path: { workspace_id: wsId },
    },
    body: data,
  });
  if (!resData) throw new Error("No response data");
  return resData as TimeEntry;
}

export async function punchOut(
  wsId: string,
  employeeId: string,
  data: { note?: string },
): Promise<TimeEntry> {
  const { data: resData } = await client.POST("/workspaces/{workspace_id}/time/punch-out/{employee_id}", {
    params: {
      path: { workspace_id: wsId, employee_id: employeeId },
    },
    body: data,
  });
  if (!resData) throw new Error("No response data");
  return resData as TimeEntry;
}

export async function fetchTimeHistory(
  wsId: string,
  employeeId: string,
  start: string,
  end: string,
): Promise<TimeEntry[]> {
  const { data: resData } = await client.GET("/workspaces/{workspace_id}/time/history/{employee_id}", {
    params: {
      path: { workspace_id: wsId, employee_id: employeeId },
      query: { start, end },
    },
  });
  if (!resData) throw new Error("No response data");
  return resData as TimeEntry[];
}

export async function updateTimeEntry(
  wsId: string,
  id: string,
  data: Partial<TimeEntry>,
): Promise<TimeEntry> {
  const { data: resData } = await client.PUT("/v1/workspaces/{ws_id}/attendance/time-tracking/{id}", {
            params: { path: { ws_id: wsId, id } },
          body: data });
    return resData!;
}

export async function deleteTimeEntry(wsId: string, id: string): Promise<void> {
  await client.DELETE("/v1/workspaces/{ws_id}/attendance/time-tracking/{id}", {
            params: { path: { ws_id: wsId, id } }
      });
}

export async function submitAttendanceAdjustment(
  wsId: string,
  data: {
    employee_id: string;
    time_entry_id?: string;
    requested_punch_in_time: string;
    requested_punch_out_time?: string;
    is_break_overtime?: boolean;
    reason?: string;
  }
): Promise<AttendanceAdjustmentRequest> {
  const { data: resData } = await client.POST("/v1/workspaces/{ws_id}/attendance/adjustments", {
            params: { path: { ws_id: wsId } },
          body: { is_break_overtime: false, ...data } });
    return resData!;
}

export async function fetchWorkspaceAttendanceAdjustments(
  wsId: string,
  status?: string
): Promise<AttendanceAdjustmentRequest[]> {
  const { data: resData } = await client.GET("/v1/workspaces/{ws_id}/attendance/adjustments", {
    params: {
      path: { ws_id: wsId },
      query: { status }
    }
  });
  return resData!;
}

export async function fetchMyAttendanceAdjustments(
  employeeId: string
): Promise<AttendanceAdjustmentRequest[]> {
  const { data: resData } = await client.GET("/v1/employees/{id}/attendance/adjustments", {
            params: { path: { id: employeeId } }
      });
    return resData!;
}

export async function approveAttendanceAdjustment(wsId: string, requestId: string, data: { status: "pending" | "approved" | "rejected" }): Promise<void> {
  await client.PUT("/v1/workspaces/{ws_id}/attendance/adjustments/{id}/approve", {
            params: { path: { ws_id: wsId, id: requestId } },
          body: data });
}

export async function fetchAttendanceLogs(params: {
  employee_id?: string;
  from?: string;
  to?: string;
  page?: number;
  limit?: number;
}): Promise<{ data: TimeEntry[]; total: number; page: number; limit: number }> {
  const wsId = getCurrentWorkspaceId();
  if (!wsId) throw new Error("Workspace not selected");

  const { data: resData } = await client.GET("/workspaces/{workspace_id}/attendance/logs", {
    params: {
      path: { workspace_id: wsId },
      query: {
        employee_id: params.employee_id || undefined,
        from: params.from || undefined,
        to: params.to || undefined,
        page: params.page || undefined,
        limit: params.limit || undefined,
      },
    },
  });
  if (!resData) throw new Error("No response data");
  return resData as { data: TimeEntry[]; total: number; page: number; limit: number };
}

export async function fetchRawAttendanceLogs(params: {
  employee_id?: string;
  from?: string;
  to?: string;
  page?: number;
  limit?: number;
}): Promise<{ data: RawAttendanceLog[]; total: number; page: number; limit: number }> {
  const wsId = getCurrentWorkspaceId();
  if (!wsId) throw new Error("Workspace not selected");

  const { data: resData } = await client.GET("/workspaces/{workspace_id}/attendance/raw-logs", {
    params: {
      path: { workspace_id: wsId },
      query: {
        employee_id: params.employee_id || undefined,
        from: params.from || undefined,
        to: params.to || undefined,
        page: params.page || undefined,
        limit: params.limit || undefined,
      },
    },
  });
  if (!resData) throw new Error("No response data");
  return resData as { data: RawAttendanceLog[]; total: number; page: number; limit: number };
}

export interface FetchAttendanceReportParams {
  departmentId?: string | null;
  employeeIds?: string[];
  end: string;
  limit?: number;
  page?: number;
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
    type = "summary",
    page,
    limit,
    sort,
    employeeIds,
    departmentId,
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
        employee_ids: employeeIds?.join(",") || undefined,
        department_id: departmentId ?? undefined,
      },
    },
  });

  if (!resData) throw new Error("No response data");
  return resData as unknown as PaginatedResponse<AttendanceSummary | DailyAttendanceReport>;
}

export async function uploadAttendance(
  wsId: string,
  file: File,
  endpoint: string
): Promise<{ job_id: string }> {
  const formData = new FormData();
  formData.append("file", file);

  if (endpoint === "/employees/upload") {
    const { data: resData } = await client.POST("/v1/workspaces/{ws_id}/employees/upload", {
      params: { path: { ws_id: wsId } },
      body: formData as any as any,
    });
    return resData as any;
  } else {
    const { data: resData } = await client.POST("/v1/workspaces/{ws_id}/attendance/upload", {
      params: { path: { ws_id: wsId } },
      body: formData as any as any,
    });
    return resData as any;
  }
}

export async function fetchLatestImportJob(
  wsId: string,
  targetType?: "log" | "adjustment" | "employee" | "leave_import"
) {
  const { data: resData } = await client.GET("/v1/workspaces/{ws_id}/attendance/import-jobs/latest", {
    params: { path: { ws_id: wsId }, query: { target_type: targetType } }
  });
  return resData!;
}

export async function fetchImportJobStatus(
  wsId: string,
  jobId: string
): Promise<AttendanceImportJob> {
  const { data: resData } = await client.GET("/v1/workspaces/{ws_id}/attendance/import-jobs/{id}", {
            params: { path: { ws_id: wsId, id: jobId } }
      });
    return resData!;
}

export async function reprocessAttendance(
  wsId: string,
  employeeId: string,
  startDate: string,
  endDate: string
): Promise<void> {
  await client.POST("/v1/attendance/reprocess/{employee_id}", {
    params: { path: { employee_id: employeeId }, query: { start_date: startDate, end_date: endDate } },
    body: {}
  });
}

export async function reprocessAllAttendance(
  wsId: string,
  startDate: string,
  endDate: string
): Promise<{ message: string; job_id: string }> {
  const { data: resData } = await client.POST("/v1/attendance/reprocess/all", {
    params: { query: { start_date: startDate, end_date: endDate } },
    body: {}
  });
  return resData as any;
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
}

export async function fetchGenericJobStatus(
  wsId: string,
  jobId: string
): Promise<GenericJobStatus> {
  const { data: resData } = await client.GET("/workspaces/{ws_id}/jobs/{job_id}", {
            params: { path: { ws_id: wsId, id: jobId } }
      });
    return resData!;
}

// --- Admin Confirm Attendance ---

/**
 * Admin shortcut: confirm attendance for a row in one action via the Confirm Modal.
 * 
 * - If an `existingRequestId` is provided (meaning the employee already submitted an adjustment),
 *   this function will bypass the creation step and directly approve that pending request.
 * - Otherwise, it submits a new adjustment using the times provided by the admin in the modal,
 *   and then immediately approves it in a subsequent API call.
 * 
 * @param wsId - The ID of the current workspace
 * @param row - The DailyAttendanceReport record being confirmed
 * @param existingRequestId - Optional ID of a pending adjustment request to approve
 * @param reason - The reason text entered by the admin in the confirmation modal
 */
export async function adminConfirmAttendance(
  wsId: string,
  row: DailyAttendanceReport,
  existingRequestId?: string,
  reason?: string,
): Promise<void> {
  let requestId = existingRequestId;

  if (!requestId) {
    // Helper: Safely convert any time string (either full ISO or raw HH:mm:ss) into a UTC ISO string.
    // This is crucial because `actual_punch_in` from the modal might be a full ISO string (Z-suffixed),
    // but fallbacks like `scheduled_start` are just raw time strings.
    const toDatetime = (date: string, time: string) => {
      // If it's already a complete UTC ISO string (has 'T' and 'Z'), use it directly
      if (time.includes('T') && time.includes('Z')) {
        return time;
      }

      // Otherwise, it's a raw local time like "08:00:00" from scheduled_start or fallback.
      // We parse it as a local time on the given date, then convert to UTC automatically 
      // via `toISOString()` to avoid AM/PM timezone wrap-around bugs.
      const [h, m, s] = time.substring(0, 8).split(':').map(Number);
      const [Y, M, D] = date.split('-').map(Number);
      return new Date(Y, M - 1, D, h, m, s || 0).toISOString();
    };

    // Use actual punch times if available; otherwise use scheduled times or midnight fallback
    const punchIn = (row.actual_punch_in ?? row.scheduled_start) ?? "00:00:00";
    const punchOut = (row.actual_punch_out ?? row.scheduled_end) ?? undefined;

    const adjustment = await submitAttendanceAdjustment(wsId, {
      employee_id: row.employee_id,
      requested_punch_in_time: toDatetime(row.date, punchIn),
      requested_punch_out_time: punchOut
        ? toDatetime(row.date, punchOut)
        : undefined,
      reason: reason ?? "Admin confirmation",
    });
    requestId = adjustment.id;
  }

  await approveAttendanceAdjustment(wsId, requestId, { status: "approved" });
}

// --- OT Claim & Approval ---


export async function claimOt(
  wsId: string,
  employeeId: string,
  data: { date: string; claimed_minutes: number; reason?: string }
): Promise<void> {
  await client.POST("/v1/workspaces/{ws_id}/attendance/claim-ot/{employee_id}", {
            params: { path: { ws_id: wsId, id: employeeId } },
          body: data });
}

export async function approveOt(
  wsId: string,
  employeeId: string,
  data: { date: string; approved_minutes: number; manager_note?: string; status: "approved" | "rejected" }
): Promise<void> {
  await client.POST("/v1/workspaces/{ws_id}/attendance/approve-ot/{employee_id}", {
            params: { path: { ws_id: wsId, id: employeeId } },
          body: data });
}

export async function fetchAttendanceSummaries(
  wsId: string,
  start: string,
  end: string,
  employeeId?: string
): Promise<AttendanceDailySummary[]> {
  const { data: resData } = await client.GET("/v1/workspaces/{ws_id}/attendance/overtime/summaries" as any, {
    params: {
      path: { ws_id: wsId },
      query: { start_date: start, end_date: end, employee_id: employeeId }
    }
  });
  if (!resData) throw new Error("No response data");
  return resData as unknown as AttendanceDailySummary[];
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
            params: { path: { ws_id: wsId, id: employeeId } },
          body: data });
    return resData!;
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
    employee_id: string;
    error_message: string;
  }[];
}> {
  const formData = new FormData();
  formData.append("file", file);

  const { data: resData } = await client.POST("/v1/workspaces/{ws_id}/attendance/paper-forms/overtime/bulk", {
            params: { path: { ws_id: wsId } },
          body: formData as any });
    return resData!;
}

// --- Staging Logs (Unknown Users) ---

export interface StagingLog {
  created_at: string;
  device_id: string | null;
  id: string;
  punch_time: string;
  raw_data: Record<string, unknown> | null;
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
  wsId: string,
  page = 1,
  limit = 50
): Promise<StagingLogsResponse> {
  const { data: resData } = await client.GET("/v1/workspaces/{ws_id}/attendance/staging-logs", {
    params: {
      path: { ws_id: wsId },
      query: { page, limit }
    }
  });
  if (!resData) throw new Error("No response data");
  return resData as unknown as StagingLogsResponse;
}

export async function resolveStagingLog(
  wsId: string,
  logId: string,
  employeeId: string
): Promise<void> {
  await client.POST("/v1/workspaces/{ws_id}/attendance/staging-logs/{id}/resolve", {
            params: { path: { ws_id: wsId, id: logId } },
          body: { employee_id: employeeId } });
}

export async function dismissStagingLog(
  wsId: string,
  logId: string
): Promise<void> {
  await client.DELETE("/v1/workspaces/{ws_id}/attendance/staging-logs/{id}", {
            params: { path: { ws_id: wsId, id: logId } }
      });
}

// --- Report Export ---

export async function createReportExportJob(
  wsId: string,
  request: CreateReportExportRequest
): Promise<CreateReportExportResponse> {
  const { data: resData } = await client.POST("/workspaces/{ws_id}/reports/export", {
            params: { path: { ws_id: wsId } },
          body: request });
    return resData!;
}

export async function fetchExportJobStatus(
  wsId: string,
  jobId: string
) {
  const { data: resData } = await client.GET("/workspaces/{ws_id}/reports/jobs/{job_id}", {
            params: { path: { ws_id: wsId, id: jobId } }
      });
    return resData!;
}

export async function cancelExportJob(
  wsId: string,
  jobId: string
): Promise<void> {
  await client.DELETE("/workspaces/{ws_id}/reports/jobs/{job_id}", {
            params: { path: { ws_id: wsId, id: jobId } }
      });
}

export async function downloadExportReport(
  wsId: string,
  jobId: string
): Promise<Blob> {
  const response = await apiFetch(`/v1/workspaces/${wsId}/reports/jobs/${jobId}/download`, {
    timeout: TIMEOUT.REPORT_OPERATION,
  });
  if (!response.ok) throw new Error("Failed to download report");
  return response.blob();
}

// --- Worker Monitoring (Global Infrastructure) ---

export async function fetchMonitorOverview(): Promise<WorkerMonitorStats> {
  const { data: resData } = await client.GET(`/v1/monitor/overview`, {});
    return resData!;
}

export interface AttendanceRules {
  early_arrival_strategy: "allow" | "block" | "clamp";
  rounding_policy: string;
  work_unit_minutes: number;
}

export interface AttendancePolicyVersion {
  created_at: string;
  created_by: string | null;
  early_arrival_strategy: "allow" | "block" | "clamp";
  id: string;
  rounding_policy: string;
  valid_from: string;
  valid_to: string | null;
  work_unit_minutes: number;
}

export interface UpdateAttendanceRulesRequest {
  early_arrival_strategy?: "allow" | "block" | "clamp";
  rounding_policy?: string;
  valid_from?: string;
  work_unit_minutes?: number;
}

export async function fetchDefaultAttendanceRules(wsId: string): Promise<AttendanceRules> {
  const { data: resData } = await client.GET("/v1/workspaces/{ws_id}/attendance/rules", {
            params: { path: { ws_id: wsId } }
      });
    return resData!;
}

export async function fetchAttendancePolicyVersions(wsId: string): Promise<AttendancePolicyVersion[]> {
  const { data: resData } = await client.GET("/v1/workspaces/{ws_id}/attendance/rules/versions", {
            params: { path: { ws_id: wsId } }
      });
    return resData!;
}

export interface PolicyAssignmentResponse {
  id: string;
  rules: AttendanceRules;
  target_id: string;
  target_type: "USER" | "GROUP";
  valid_from: string;
  valid_to: string | null;
}

export interface UpsertPolicyAssignmentRequest {
  early_arrival_strategy: "allow" | "block" | "clamp";
  rounding_policy: string;
  target_id: string;
  target_type: "USER" | "GROUP";
  valid_from?: string;
  work_unit_minutes: number;
}

export async function fetchPolicyAssignments(wsId: string): Promise<PolicyAssignmentResponse[]> {
  const { data: resData } = await client.GET("/v1/workspaces/{ws_id}/attendance/policies/assignments", {
            params: { path: { ws_id: wsId } }
      });
    return resData!;
}

export async function upsertPolicyAssignment(
  wsId: string,
  data: UpsertPolicyAssignmentRequest
): Promise<PolicyAssignmentResponse> {
  const { data: resData } = await client.POST("/v1/workspaces/{ws_id}/attendance/policies/assignments", {
            params: { path: { ws_id: wsId } },
          body: data });
    return resData!;
}

export async function deletePolicyAssignment(wsId: string, assignmentId: string): Promise<void> {
  await client.DELETE("/v1/workspaces/{ws_id}/attendance/policies/assignments/{assignment_id}", {
            params: { path: { ws_id: wsId, assignment_id: assignmentId } }
      });
}

export async function updateDefaultAttendanceRules(
  wsId: string,
  data: UpdateAttendanceRulesRequest
): Promise<AttendancePolicyVersion> {
  const { data: resData } = await client.PATCH("/v1/workspaces/{ws_id}/attendance/rules", {
            params: { path: { ws_id: wsId } },
          body: data });
    return resData!;
}
