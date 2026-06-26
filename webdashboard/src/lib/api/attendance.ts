import { client } from "./client";
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
  CreateReportExportRequest,
  CreateReportExportResponse,
  WorkerMonitorStats, AttendanceDailySummary, MonitorJobsResponse
} from "./types";

export async function punchIn(
  wsId: string,
  data: { employment_id: string; entry_type?: string; note?: string },
): Promise<TimeEntry> {
  const { data: resData } = await client.POST("/workspaces/{workspace_id}/time/punch-in", {
    params: {
      path: { workspace_id: wsId },
    },
    body: data,
  });
  if (!resData) throw new Error("Punch in failed");
  return resData;
}

export async function punchOut(
  wsId: string,
  data: { employee_id: string; note?: string },
): Promise<TimeEntry> {
  const { data: resData } = await client.POST("/workspaces/{workspace_id}/time/punch-out/{employee_id}", {
    params: {
      path: { workspace_id: wsId, employee_id: data.employee_id },
    },
    body: {
      note: data.note,
    },
  });
  if (!resData) throw new Error("Punch out failed");
  return resData;
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
  return resData;
}

export async function updateTimeEntry(
  wsId: string,
  id: string,
  data: Partial<TimeEntry>,
): Promise<TimeEntry> {
  const { data: resData } = await client.PUT("/v1/workspaces/{ws_id}/attendance/time-tracking/{id}", {
    params: { path: { ws_id: wsId, id } },
    body: data
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export async function deleteTimeEntry(wsId: string, id: string): Promise<void> {
  await client.DELETE("/v1/workspaces/{ws_id}/attendance/time-tracking/{id}", {
    params: { path: { ws_id: wsId, id } }
  });
}

export async function submitAttendanceAdjustment(
  wsId: string,
  data: {
    employment_id: string;
    time_entry_id?: string;
    requested_punch_time: string;
    reason?: string;
    attachment_file_ids?: string[];
  }
): Promise<AttendanceAdjustmentRequest> {
  const { data: resData } = await client.POST("/v1/workspaces/{ws_id}/attendance/adjustments", {
    params: { path: { ws_id: wsId } },
    body: data as any
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export async function fetchWorkspaceAttendanceAdjustments(
  wsId: string,
  params?: {
    status?: string;
    employee_id?: string;
    page?: number;
    limit?: number;
  }
): Promise<PaginatedResponse<AttendanceAdjustmentRequest>> {
  const { data: resData } = await client.GET("/v1/workspaces/{ws_id}/attendance/adjustments", {
    params: {
      path: { ws_id: wsId },
      query: {
        status: params?.status,
        employee_id: params?.employee_id === "all" ? undefined : params?.employee_id,
        page: params?.page,
        limit: params?.limit,
      } as any,
    }
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export async function fetchMyAttendanceAdjustments(
  employeeId: string
): Promise<AttendanceAdjustmentRequest[]> {
  const { data: resData } = await client.GET("/v1/employees/{id}/attendance/adjustments", {
    params: { path: { id: employeeId } }
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export async function approveAttendanceAdjustment(wsId: string, requestId: string, data: { status: "pending" | "approved" | "rejected" }): Promise<void> {
  await client.PUT("/v1/workspaces/{ws_id}/attendance/adjustments/{id}/approve", {
    params: { path: { ws_id: wsId, id: requestId } },
    body: data
  });
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
        employee_id: params.employee_id ?? undefined,
        from: params.from ?? undefined,
        to: params.to ?? undefined,
        page: params.page ?? undefined,
        limit: params.limit ?? undefined,
      },
    },
  });
  if (!resData) throw new Error("No response data");
  return resData as unknown as { data: TimeEntry[]; total: number; page: number; limit: number };
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
        employee_id: params.employee_id ?? undefined,
        from: params.from ?? undefined,
        to: params.to ?? undefined,
        page: params.page ?? undefined,
        limit: params.limit ?? undefined,
      },
    },
  });
  if (!resData) throw new Error("No response data");
  return resData as unknown as { data: RawAttendanceLog[]; total: number; page: number; limit: number };
}

export interface FetchAttendanceReportParams {
  departmentId?: string | null;
  employeeIds?: string[];
  employeeStatus?: string;
  employerId?: string | null;
  employmentId?: string | null;
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
    type,
    page,
    limit,
    sort,
    employeeIds,
    departmentId,
    employeeStatus,
    employerId,
    employmentId,
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
      },
    },
  });

  if (!resData) throw new Error("No response data");
  return resData;
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
      body: formData as unknown as never,
    });
    if (!resData) throw new Error("No response data");
    return resData as { job_id: string };
  } else {
    const { data: resData } = await client.POST("/v1/workspaces/{ws_id}/attendance/upload", {
      params: { path: { ws_id: wsId } },
      body: formData as unknown as never,
    });
    if (!resData) throw new Error("No response data");
    return resData as { job_id: string };
  }
}

export async function fetchLatestImportJob(
  wsId: string,
  targetType?: "log" | "adjustment" | "employee" | "leave_import"
) {
  const { data: resData } = await client.GET("/v1/workspaces/{ws_id}/attendance/import-jobs/latest", {
    params: { path: { ws_id: wsId }, query: { target_type: targetType } }
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export async function fetchImportJobStatus(
  wsId: string,
  jobId: string
): Promise<AttendanceImportJob> {
  const { data: resData } = await client.GET("/v1/workspaces/{ws_id}/attendance/import-jobs/{id}", {
    params: { path: { ws_id: wsId, id: jobId } }
  });
  if (!resData) throw new Error("No response data");
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
  if (!resData) throw new Error("No response data");
  return resData as { message: string; job_id: string };
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
  if (!resData) throw new Error("No response data");
  return resData as GenericJobStatus;
}

export interface AttendanceStats {
  early_out_count: number;
  late_in_count: number;
  missing_punch_count: number;
  present_count: number;
  total_count: number;
}

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
  if (!resData) throw new Error("No response data");
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
  if (!resData) throw new Error("No response data");
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

  const { data: resData } = await client.POST("/v1/workspaces/{ws_id}/attendance/paper-forms/overtime/bulk", {
    params: { path: { ws_id: wsId } },
    body: formData as unknown as never
  });
  if (!resData) throw new Error("No response data");
  return resData;
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
      query: { page, limit } as unknown as never
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
    body: { employee_id: employeeId }
  });
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
    body: request
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export async function fetchExportJobStatus(
  wsId: string,
  jobId: string
) {
  const { data: resData } = await client.GET("/workspaces/{ws_id}/reports/jobs/{job_id}", {
    params: { path: { ws_id: wsId, job_id: jobId } }
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export async function cancelExportJob(
  wsId: string,
  jobId: string
): Promise<void> {
  await client.DELETE("/workspaces/{ws_id}/reports/jobs/{job_id}", {
    params: { path: { ws_id: wsId, job_id: jobId } }
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
  const { data: resData } = await client.GET("/monitor/overview", {});
  if (!resData) throw new Error("No response data");
  return resData;
}

export interface FetchMonitorJobsParams {
  cursor?: number | null;
  limit?: number;
  search?: string;
  status?: string;
  task_type?: string;
}

export async function fetchMonitorJobs(params: FetchMonitorJobsParams): Promise<MonitorJobsResponse> {
  const { data: resData } = await client.GET("/monitor/jobs", {
    params: {
      query: {
        cursor: params.cursor ?? undefined,
        limit: params.limit,
        search: params.search ?? undefined,
        status: params.status ?? undefined,
        task_type: params.task_type ?? undefined,
      },
    },
  });
  return resData ?? { jobs: [], next_cursor: null };
}

export interface AttendanceRules {
  early_arrival_strategy?: "allow" | "block" | "clamp" | null;
  grace_period_minutes?: number | null;
  rounding_policy: string;
  work_unit_minutes: number;
}

export interface AttendancePolicyVersion {
  created_at: string;
  created_by?: string | null;
  early_arrival_strategy: "allow" | "block" | "clamp";
  grace_period_minutes: number;
  id: string;
  rounding_policy: string;
  valid_from: string;
  valid_to?: string | null;
  work_unit_minutes: number;
}

export interface UpdateAttendanceRulesRequest {
  early_arrival_strategy?: "allow" | "block" | "clamp";
  grace_period_minutes?: number;
  rounding_policy?: string;
  valid_from?: string;
  work_unit_minutes?: number;
}

export async function fetchDefaultAttendanceRules(wsId: string): Promise<AttendanceRules> {
  const { data: resData } = await client.GET("/v1/workspaces/{ws_id}/attendance/rules", {
    params: { path: { ws_id: wsId } }
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export async function fetchAttendancePolicyVersions(wsId: string): Promise<AttendancePolicyVersion[]> {
  const { data: resData } = await client.GET("/v1/workspaces/{ws_id}/attendance/rules/versions", {
    params: { path: { ws_id: wsId } }
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export interface PolicyAssignmentResponse {
  id: string;
  rules: AttendanceRules;
  target_id: string;
  target_type: string;
  valid_from: string;
  valid_to?: string | null;
}

export interface UpsertPolicyAssignmentRequest {
  early_arrival_strategy: "allow" | "block" | "clamp";
  grace_period_minutes?: number | null;
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
  if (!resData) throw new Error("No response data");
  return resData;
}

export async function upsertPolicyAssignment(
  wsId: string,
  data: UpsertPolicyAssignmentRequest
): Promise<PolicyAssignmentResponse> {
  const { data: resData } = await client.POST("/v1/workspaces/{ws_id}/attendance/policies/assignments", {
    params: { path: { ws_id: wsId } },
    body: data
  });
  if (!resData) throw new Error("No response data");
  return resData;
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
    body: data
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export interface OtClaimBehalfAssignment {
  assignee_user_id: string;
  assignment_type: string;
  created_at: string;
  id: string;
  target_department_id?: string | null;
  target_employee_id?: string | null;
  updated_at: string;
  workspace_id: string;
}

export async function fetchBehalfAssignments(
  wsId: string
): Promise<OtClaimBehalfAssignment[]> {
  const { data: resData } = await client.GET("/v1/workspaces/{ws_id}/attendance/claim-ot/behalf-assignments", {
    params: { path: { ws_id: wsId } }
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export async function createBehalfAssignment(
  wsId: string,
  data: {
    assignee_user_id: string;
    target_department_id?: string | null;
    target_employee_id?: string | null;
  }
): Promise<OtClaimBehalfAssignment> {
  const { data: resData } = await client.POST("/v1/workspaces/{ws_id}/attendance/claim-ot/behalf-assignments", {
    params: { path: { ws_id: wsId } },
    body: data
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export async function deleteBehalfAssignment(wsId: string, id: string): Promise<void> {
  await client.DELETE("/v1/workspaces/{ws_id}/attendance/claim-ot/behalf-assignments/{id}", {
    params: { path: { ws_id: wsId, id } }
  });
}

export async function claimOt(
  wsId: string,
  employeeId: string,
  data: {
    date: string;
    claimed_minutes: number;
    reason?: string | null;
  }
): Promise<void> {
  await client.POST("/v1/workspaces/{ws_id}/attendance/claim-ot/{employee_id}", {
    params: { path: { ws_id: wsId, employee_id: employeeId } },
    body: data
  });
}

export async function adminConfirmAttendance(
  wsId: string,
  row: DailyAttendanceReport,
  existingRequestId?: string,
  reason?: string,
): Promise<void> {
  let requestId = existingRequestId;

  if (!requestId) {
    const toDatetime = (dateStr: string, timeStr: string) => {
      if (timeStr.includes("T") && timeStr.includes("Z")) {
        return timeStr;
      }
      const [h, m, s] = timeStr.substring(0, 8).split(":").map(Number);
      const [Y, M, D] = dateStr.split("-").map(Number);
      return new Date(Y, M - 1, D, h, m, s || 0).toISOString();
    };

    const punchIn = row.actual_punch_in ?? row.scheduled_start ?? "00:00:00";
    const punchOut = row.actual_punch_out ?? row.scheduled_end ?? undefined;

    const adjustment = await submitAttendanceAdjustment(wsId, {
      employment_id: row.employment_id,
      requested_punch_time: toDatetime(row.date, punchIn),
      reason: reason ?? "Admin confirmation",
    });
    requestId = adjustment.id;

    if (punchOut) {
      const adjustmentOut = await submitAttendanceAdjustment(wsId, {
        employment_id: row.employment_id,
        requested_punch_time: toDatetime(row.date, punchOut),
        reason: reason ?? "Admin confirmation",
      });
      await approveAttendanceAdjustment(wsId, adjustmentOut.id, { status: "approved" });
    }
  }

  await approveAttendanceAdjustment(wsId, requestId, { status: "approved" });
}

// ── Attendance Adjustment Behalf Assignments ─────────────────────

export async function fetchAdjustmentBehalfAssignments(
  wsId: string
): Promise<OtClaimBehalfAssignment[]> {
  const { data: resData } = await client.GET("/v1/workspaces/{ws_id}/attendance/adjustments/behalf-assignments", {
    params: { path: { ws_id: wsId } }
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export async function createAdjustmentBehalfAssignment(
  wsId: string,
  data: {
    assignee_user_id: string;
    target_department_id?: string | null;
    target_employee_id?: string | null;
  }
): Promise<OtClaimBehalfAssignment> {
  const { data: resData } = await client.POST("/v1/workspaces/{ws_id}/attendance/adjustments/behalf-assignments", {
    params: { path: { ws_id: wsId } },
    body: data
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export async function deleteAdjustmentBehalfAssignment(wsId: string, id: string): Promise<void> {
  await client.DELETE("/v1/workspaces/{ws_id}/attendance/adjustments/behalf-assignments/{id}", {
    params: { path: { ws_id: wsId, id } }
  });
}
