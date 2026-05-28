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
import { apiClient, client } from "./client";

export async function punchIn(
  wsId: string,
  data: { employee_id: string; entry_type?: string; note?: string },
): Promise<TimeEntry> {
  const { data: resData } = await client.POST("/v1/workspaces/{workspace_id}/time/punch-in", {
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
  const { data: resData } = await client.POST("/v1/workspaces/{workspace_id}/time/punch-out/{employee_id}", {
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
  const { data: resData } = await client.GET("/v1/workspaces/{workspace_id}/time/history/{employee_id}", {
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
  const { data: resData } = await apiClient.put<TimeEntry>(`/v1/workspaces/${wsId}/time/entries/${id}`, data);
    return resData;
}

export async function deleteTimeEntry(wsId: string, id: string): Promise<void> {
  await apiClient.delete(`/v1/workspaces/${wsId}/time/entries/${id}`);
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
  const { data: resData } = await apiClient.post<AttendanceAdjustmentRequest>(`/v1/workspaces/${wsId}/attendance/adjustments`, { is_break_overtime: false, ...data });
    return resData;
}

export async function fetchWorkspaceAttendanceAdjustments(
  wsId: string,
  status?: string
): Promise<AttendanceAdjustmentRequest[]> {
  const url = status
    ? `${API_BASE_URL}/workspaces/${wsId}/attendance/adjustments?status=${status}`
    : `${API_BASE_URL}/workspaces/${wsId}/attendance/adjustments`;
  const { data: resData } = await apiClient.get<AttendanceAdjustmentRequest[]>(url);
    return resData;
}

export async function fetchMyAttendanceAdjustments(
  employeeId: string
): Promise<AttendanceAdjustmentRequest[]> {
  const { data: resData } = await apiClient.get<AttendanceAdjustmentRequest[]>(`/v1/employees/${employeeId}/attendance/adjustments`);
    return resData;
}

export async function approveAttendanceAdjustment(
  wsId: string,
  requestId: string,
  data: { status: string }
): Promise<void> {
  await apiClient.post(`/v1/workspaces/${wsId}/attendance/adjustments/${requestId}/approve`, data);
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

  const { data: resData } = await client.GET("/v1/workspaces/{workspace_id}/attendance/logs", {
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

  const { data: resData } = await client.GET("/v1/workspaces/{workspace_id}/attendance/raw-logs", {
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
  
  const searchParams = new URLSearchParams({
    start_date: start,
    end_date: end,
    type,
  });
  if (page) searchParams.append("page", page.toString());
  if (limit) searchParams.append("limit", limit.toString());
  if (sort) searchParams.append("sort", sort);
  if (employeeIds && employeeIds.length > 0) {
    searchParams.append("employee_ids", employeeIds.join(","));
  }
  if (departmentId) {
    searchParams.append("department_id", departmentId);
  }

  const { data: resData } = await apiClient.get<PaginatedResponse<AttendanceSummary | DailyAttendanceReport>>(`/workspaces/${wsId}/reports/attendance?${searchParams.toString()}`);
    return resData;
}

export async function uploadAttendanceExcel(
  wsId: string,
  file: File
): Promise<{ job_id: string }> {
  return uploadAttendance(wsId, file, "/attendance/upload");
}

export async function uploadEmployees(
  wsId: string,
  file: File
): Promise<{ job_id: string }> {
  return uploadAttendance(wsId, file, "/employees/upload");
}

export async function uploadAttendance(
  wsId: string,
  file: File,
  endpoint: string
): Promise<{ job_id: string }> {
  const formData = new FormData();
  formData.append("file", file);

  const { data: resData } = await apiClient.post<{ job_id: string }>(`/v1/workspaces/${wsId}${endpoint}`, formData);
    return resData;
}

export async function fetchLatestImportJob(
  wsId: string,
  targetType?: string
): Promise<AttendanceImportJob | null> {
  const url = targetType
    ? `${API_BASE_URL}/workspaces/${wsId}/attendance/import-jobs/latest?target_type=${targetType}`
    : `${API_BASE_URL}/workspaces/${wsId}/attendance/import-jobs/latest`;
  const { data: resData } = await apiClient.get<AttendanceImportJob | null>(url);
    return resData;
}

export async function fetchImportJobStatus(
  wsId: string,
  jobId: string
): Promise<AttendanceImportJob> {
  const { data: resData } = await apiClient.get<AttendanceImportJob>(`/v1/workspaces/${wsId}/attendance/import-jobs/${jobId}`);
    return resData;
}

export async function reprocessAttendance(
  wsId: string,
  employeeId: string,
  startDate: string,
  endDate: string
): Promise<void> {
  await apiClient.post(`/v1/workspaces/${wsId}/attendance/reprocess/${employeeId}?start_date=${startDate}&end_date=${endDate}`, {});
}

export async function reprocessAllAttendance(
  wsId: string,
  startDate: string,
  endDate: string
): Promise<{ message: string; job_id: string }> {
  const { data: resData } = await apiClient.post<{ message: string; job_id: string }>(`/v1/workspaces/${wsId}/attendance/reprocess-all?start_date=${startDate}&end_date=${endDate}`, {});
    return resData;
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
  const { data: resData } = await apiClient.get<GenericJobStatus>(`/v1/workspaces/${wsId}/jobs/${jobId}`);
    return resData;
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
  await apiClient.post(`/v1/workspaces/${wsId}/attendance/claim-ot/${employeeId}`, data);
}

export async function approveOt(
  wsId: string,
  employeeId: string,
  data: { date: string; approved_minutes: number; manager_note?: string; status: "approved" | "rejected" }
): Promise<void> {
  await apiClient.post(`/v1/workspaces/${wsId}/attendance/approve-ot/${employeeId}`, data);
}

export async function fetchAttendanceSummaries(
  wsId: string,
  start: string,
  end: string,
  employeeId?: string
): Promise<AttendanceDailySummary[]> {
  const params = new URLSearchParams({
    start_date: start,
    end_date: end,
  });
  if (employeeId) params.append("employee_id", employeeId);

  const { data: resData } = await apiClient.get<AttendanceDailySummary[]>(`/v1/workspaces/${wsId}/attendance/summaries?${params.toString()}`);
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
  const { data: resData } = await apiClient.post<AttendanceDailySummary>(`/v1/workspaces/${wsId}/attendance/enter-paper-ot/${employeeId}`, data);
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
    employee_id: string;
    error_message: string;
  }[];
}> {
  const formData = new FormData();
  formData.append("file", file);

  const { data: resData } = await apiClient.post<{
        total_rows: number;
        success_count: number;
        failed_count: number;
        errors: {
          line_number: number;
          employee_id: string;
          error_message: string;
        }[];
      }>(`/v1/workspaces/${wsId}/attendance/bulk-upload-paper-ot`, formData);
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
  const params = new URLSearchParams({
    page: page.toString(),
    limit: limit.toString(),
  });

  const { data: resData } = await apiClient.get<StagingLogsResponse>(`/v1/workspaces/${wsId}/attendance/staging-logs?${params.toString()}`);
    return resData;
}

export async function resolveStagingLog(
  wsId: string,
  logId: string,
  employeeId: string
): Promise<void> {
  await apiClient.post(`/v1/workspaces/${wsId}/attendance/staging-logs/${logId}/resolve`, { employee_id: employeeId });
}

export async function dismissStagingLog(
  wsId: string,
  logId: string
): Promise<void> {
  await apiClient.delete(`/v1/workspaces/${wsId}/attendance/staging-logs/${logId}`);
}

// --- Report Export ---

export async function createReportExportJob(
  wsId: string,
  request: CreateReportExportRequest
): Promise<CreateReportExportResponse> {
  const { data: resData } = await apiClient.post<CreateReportExportResponse>(`/v1/workspaces/${wsId}/reports/export`, request);
    return resData;
}

export async function fetchExportJobStatus(
  wsId: string,
  jobId: string
): Promise<ReportExportJob> {
  const { data: resData } = await apiClient.get<ReportExportJob>(`/v1/workspaces/${wsId}/reports/jobs/${jobId}`);
    return resData;
}

export async function cancelExportJob(
  wsId: string,
  jobId: string
): Promise<void> {
  await apiClient.delete(`/v1/workspaces/${wsId}/reports/jobs/${jobId}`);
}

export async function downloadExportReport(
  wsId: string,
  jobId: string
): Promise<Blob> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/reports/jobs/${jobId}/download`, {
    timeout: TIMEOUT.REPORT_OPERATION,
  });
  if (!response.ok) throw new Error("Failed to download report");
  return response.blob();
}

// --- Worker Monitoring (Global Infrastructure) ---

export async function fetchMonitorOverview(): Promise<WorkerMonitorStats> {
  const { data: resData } = await apiClient.get<WorkerMonitorStats>(`/v1/monitor/overview`);
    return resData;
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
  const { data: resData } = await apiClient.get<AttendanceRules>(`/v1/workspaces/${wsId}/attendance/policies/default`);
    return resData;
}

export async function fetchAttendancePolicyVersions(wsId: string): Promise<AttendancePolicyVersion[]> {
  const { data: resData } = await apiClient.get<AttendancePolicyVersion[]>(`/v1/workspaces/${wsId}/attendance/policies/default/versions`);
    return resData;
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
  const { data: resData } = await apiClient.get<PolicyAssignmentResponse[]>(`/v1/workspaces/${wsId}/attendance/policies/assignments`);
    return resData;
}

export async function upsertPolicyAssignment(
  wsId: string,
  data: UpsertPolicyAssignmentRequest
): Promise<PolicyAssignmentResponse> {
  const { data: resData } = await apiClient.post<PolicyAssignmentResponse>(`/v1/workspaces/${wsId}/attendance/policies/assignments`, data);
    return resData;
}

export async function deletePolicyAssignment(wsId: string, assignmentId: string): Promise<void> {
  await apiClient.delete(`/v1/workspaces/${wsId}/attendance/policies/assignments/${assignmentId}`);
}

export async function updateDefaultAttendanceRules(
  wsId: string,
  data: UpdateAttendanceRulesRequest
): Promise<AttendancePolicyVersion> {
  const { data: resData } = await apiClient.patch<AttendancePolicyVersion>(`/v1/workspaces/${wsId}/attendance/policies/default`, data);
    return resData;
}
