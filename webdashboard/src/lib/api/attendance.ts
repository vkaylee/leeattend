import { API_BASE_URL } from "./config";
import { apiFetch, getCurrentWorkspaceId, TIMEOUT } from "./helpers";
import {
  TimeEntry,
  AttendanceAdjustmentRequest,
  RawAttendanceLog,
  AttendanceImportJob,
  _AttendanceSummary,
  DailyAttendanceReport,
  _PaginatedResponse,
  AttendanceReportSort,
  ReportExportJob,
  CreateReportExportRequest,
  CreateReportExportResponse,
  WorkerMonitorStats, AttendanceDailySummary 
} from "./types";

export async function punchIn(
  wsId: string,
  data: { employee_id: string; entry_type?: string; note?: string },
): Promise<TimeEntry> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/time/punch-in`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to punch in");
  return response.json();
}

export async function punchOut(
  wsId: string,
  employeeId: string,
  data: { note?: string },
): Promise<TimeEntry> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/time/punch-out/${employeeId}`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to punch out");
  return response.json();
}

export async function fetchTimeHistory(
  wsId: string,
  employeeId: string,
  start: string,
  end: string,
): Promise<TimeEntry[]> {
  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${wsId}/time/history/${employeeId}?start=${start}&end=${end}`,
  );
  if (!response.ok) throw new Error("Failed to fetch time history");
  return response.json();
}

export async function updateTimeEntry(
  wsId: string,
  id: string,
  data: Partial<TimeEntry>,
): Promise<TimeEntry> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/time/entries/${id}`, {
    method: "PUT",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to update time entry");
  return response.json();
}

export async function deleteTimeEntry(wsId: string, id: string): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/time/entries/${id}`, {
    method: "DELETE",
  });
  if (!response.ok) throw new Error("Failed to delete time entry");
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
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/attendance/adjustments`, {
    method: "POST",
    body: JSON.stringify({ is_break_overtime: false, ...data }),
  });
  if (!response.ok) throw new Error("Failed to submit attendance adjustment");
  return response.json();
}

export async function fetchWorkspaceAttendanceAdjustments(
  wsId: string,
  status?: string
): Promise<AttendanceAdjustmentRequest[]> {
  const url = status
    ? `${API_BASE_URL}/workspaces/${wsId}/attendance/adjustments?status=${status}`
    : `${API_BASE_URL}/workspaces/${wsId}/attendance/adjustments`;
  const response = await apiFetch(url);
  if (!response.ok) throw new Error("Failed to fetch attendance adjustments");
  return response.json();
}

export async function fetchMyAttendanceAdjustments(
  employeeId: string
): Promise<AttendanceAdjustmentRequest[]> {
  const response = await apiFetch(`${API_BASE_URL}/employees/${employeeId}/attendance/adjustments`);
  if (!response.ok) throw new Error("Failed to fetch my attendance adjustments");
  return response.json();
}

export async function approveAttendanceAdjustment(
  wsId: string,
  requestId: string,
  data: { status: string }
): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/attendance/adjustments/${requestId}/approve`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to approve attendance adjustment");
}

export async function fetchAttendanceLogs(params: {
  employee_id?: string;
  from?: string;
  to?: string;
  page?: number;
  limit?: number;
}): Promise<{ data: TimeEntry[]; total: number; page: number; limit: number }> {
  const searchParams = new URLSearchParams();
  if (params.employee_id) searchParams.append("employee_id", params.employee_id);
  if (params.from) searchParams.append("from", params.from);
  if (params.to) searchParams.append("to", params.to);
  if (params.page) searchParams.append("page", params.page.toString());
  if (params.limit) searchParams.append("limit", params.limit.toString());

  const wsId = getCurrentWorkspaceId();
  const url = wsId
    ? `${API_BASE_URL}/workspaces/${wsId}/attendance/logs?${searchParams.toString()}`
    : `${API_BASE_URL}/attendance/logs?${searchParams.toString()}`;

  const response = await apiFetch(url);
  if (!response.ok) throw new Error("Failed to fetch attendance logs");
  return response.json();
}

export async function fetchRawAttendanceLogs(params: {
  employee_id?: string;
  from?: string;
  to?: string;
  page?: number;
  limit?: number;
}): Promise<{ data: RawAttendanceLog[]; total: number; page: number; limit: number }> {
  const searchParams = new URLSearchParams();
  if (params.employee_id) searchParams.append("employee_id", params.employee_id);
  if (params.from) searchParams.append("from", params.from);
  if (params.to) searchParams.append("to", params.to);
  if (params.page) searchParams.append("page", params.page.toString());
  if (params.limit) searchParams.append("limit", params.limit.toString());

  const wsId = getCurrentWorkspaceId();
  if (!wsId) throw new Error("Workspace not selected");

  const url = `${API_BASE_URL}/workspaces/${wsId}/attendance/raw-logs?${searchParams.toString()}`;

  const response = await apiFetch(url);
  if (!response.ok) throw new Error("Failed to fetch raw attendance logs");
  return response.json();
}

export async function fetchAttendanceReport(
  wsId: string,
  start: string,
  end: string,
  type: "summary" | "daily" = "summary",
  page?: number,
  limit?: number,
  sort?: AttendanceReportSort,
  employeeIds?: string[],
  departmentId?: string | null
): Promise<any> {
  const params = new URLSearchParams({
    start_date: start,
    end_date: end,
    type,
  });
  if (page) params.append("page", page.toString());
  if (limit) params.append("limit", limit.toString());
  if (sort) params.append("sort", sort);
  if (employeeIds && employeeIds.length > 0) {
    params.append("employee_ids", employeeIds.join(","));
  }
  if (departmentId) {
    params.append("department_id", departmentId);
  }

  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${wsId}/reports/attendance?${params.toString()}`,
  );
  if (!response.ok) throw new Error("Failed to fetch attendance report");
  return response.json();
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

  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}${endpoint}`, {
    method: "POST",
    body: formData,
  });
  if (!response.ok) throw new Error("Failed to upload attendance");
  return response.json();
}

export async function fetchLatestImportJob(
  wsId: string,
  targetType?: string
): Promise<AttendanceImportJob | null> {
  const url = targetType
    ? `${API_BASE_URL}/workspaces/${wsId}/attendance/import-jobs/latest?target_type=${targetType}`
    : `${API_BASE_URL}/workspaces/${wsId}/attendance/import-jobs/latest`;
  const response = await apiFetch(url);
  if (!response.ok) throw new Error("Failed to fetch latest import job");
  return response.json();
}

export async function fetchImportJobStatus(
  wsId: string,
  jobId: string
): Promise<AttendanceImportJob> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/attendance/import-jobs/${jobId}`);
  if (!response.ok) throw new Error("Failed to fetch import job status");
  return response.json();
}

export async function reprocessAttendance(
  wsId: string,
  employeeId: string,
  startDate: string,
  endDate: string
): Promise<void> {
  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${wsId}/attendance/reprocess/${employeeId}?start_date=${startDate}&end_date=${endDate}`,
    {
      method: "POST",
      timeout: TIMEOUT.REPORT_OPERATION,
    }
  );
  if (!response.ok) throw new Error("Failed to reprocess attendance");
}

export async function reprocessAllAttendance(
  wsId: string,
  startDate: string,
  endDate: string
): Promise<{ message: string; job_id: string }> {
  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${wsId}/attendance/reprocess-all?start_date=${startDate}&end_date=${endDate}`,
    {
      method: "POST",
    }
  );
  if (!response.ok) throw new Error("Failed to queue reprocess all attendance job");
  return response.json();
}

export interface GenericJobProgress {
  total_batches: number;
  completed_batches: number;
  failed_batches: number;
  percentage: number;
}

export interface GenericJobStatus {
  job_id: string;
  status: "pending" | "processing" | "completed" | "failed";
  progress: GenericJobProgress;
  error?: string;
  created_at: string;
  completed_at?: string;
}

export async function fetchGenericJobStatus(
  wsId: string,
  jobId: string
): Promise<GenericJobStatus> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/jobs/${jobId}`);
  if (!response.ok) throw new Error("Failed to fetch job status");
  return response.json();
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
    const punchIn = row.actual_punch_in || row.scheduled_start || "00:00:00";
    const punchOut = row.actual_punch_out || row.scheduled_end || undefined;

    const adjustment = await submitAttendanceAdjustment(wsId, {
      employee_id: row.employee_id,
      requested_punch_in_time: toDatetime(row.date, punchIn),
      requested_punch_out_time: punchOut
        ? toDatetime(row.date, punchOut)
        : undefined,
      reason: reason || "Admin confirmation",
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
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/attendance/claim-ot/${employeeId}`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to claim OT");
}

export async function approveOt(
  wsId: string,
  employeeId: string,
  data: { date: string; approved_minutes: number; manager_note?: string; status: "approved" | "rejected" }
): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/attendance/approve-ot/${employeeId}`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to approve OT");
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

  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/attendance/summaries?${params.toString()}`);
  if (!response.ok) throw new Error("Failed to fetch attendance summaries");
  return response.json();
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
  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${wsId}/attendance/enter-paper-ot/${employeeId}`,
    {
      method: "POST",
      body: JSON.stringify(data),
    }
  );
  if (!response.ok) {
    const err = await response.text();
    throw new Error(err || "Failed to enter paper OT");
  }
  return response.json();
}

export async function bulkUploadPaperOt(
  wsId: string,
  file: File
): Promise<{
  total_rows: number;
  success_count: number;
  failed_count: number;
  errors: Array<{
    line_number: number;
    employee_id: string;
    error_message: string;
  }>;
}> {
  const formData = new FormData();
  formData.append("file", file);

  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${wsId}/attendance/bulk-upload-paper-ot`,
    {
      method: "POST",
      body: formData,
    }
  );

  if (!response.ok) {
    const err = await response.text();
    throw new Error(err || "Failed to upload CSV file");
  }

  return response.json();
}

// --- Staging Logs (Unknown Users) ---

export interface StagingLog {
  id: string;
  workspace_id: string;
  device_id: string | null;
  raw_user_id: string;
  punch_time: string;
  raw_data: Record<string, unknown> | null;
  status: string;
  created_at: string;
}

export interface StagingLogsResponse {
  data: StagingLog[];
  total: number;
  page: number;
  limit: number;
}

export async function fetchStagingLogs(
  wsId: string,
  page: number = 1,
  limit: number = 50
): Promise<StagingLogsResponse> {
  const params = new URLSearchParams({
    page: page.toString(),
    limit: limit.toString(),
  });

  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${wsId}/attendance/staging-logs?${params.toString()}`
  );
  if (!response.ok) throw new Error("Failed to fetch staging logs");
  return response.json();
}

export async function resolveStagingLog(
  wsId: string,
  logId: string,
  employeeId: string
): Promise<void> {
  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${wsId}/attendance/staging-logs/${logId}/resolve`,
    {
      method: "POST",
      body: JSON.stringify({ employee_id: employeeId }),
    }
  );
  if (!response.ok) throw new Error("Failed to resolve staging log");
}

export async function dismissStagingLog(
  wsId: string,
  logId: string
): Promise<void> {
  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${wsId}/attendance/staging-logs/${logId}`,
    {
      method: "DELETE",
    }
  );
  if (!response.ok) throw new Error("Failed to dismiss staging log");
}

// --- Report Export ---

export async function createReportExportJob(
  wsId: string,
  request: CreateReportExportRequest
): Promise<CreateReportExportResponse> {
  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${wsId}/reports/export`,
    {
      method: "POST",
      body: JSON.stringify(request),
      timeout: TIMEOUT.REPORT_OPERATION, // 3 minutes for report generation
    }
  );
  if (!response.ok) {
    const err = await response.text();
    throw new Error(err || "Failed to start report export");
  }
  return response.json();
}

export async function fetchExportJobStatus(
  wsId: string,
  jobId: string
): Promise<ReportExportJob> {
  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${wsId}/reports/jobs/${jobId}`
  );
  if (!response.ok) throw new Error("Failed to fetch export job status");
  return response.json();
}

export async function cancelExportJob(
  wsId: string,
  jobId: string
): Promise<void> {
  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${wsId}/reports/jobs/${jobId}`,
    { method: "DELETE" }
  );
  if (!response.ok) throw new Error("Failed to cancel export job");
}

export async function downloadExportReport(
  wsId: string,
  jobId: string
): Promise<Blob> {
  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${wsId}/reports/jobs/${jobId}/download`,
    {
      timeout: TIMEOUT.REPORT_OPERATION, // 3 minutes for large Excel file download
    }
  );
  if (!response.ok) throw new Error("Failed to download report");
  return response.blob();
}

// --- Worker Monitoring (Global Infrastructure) ---

export async function fetchMonitorOverview(): Promise<WorkerMonitorStats> {
  const response = await apiFetch(`${API_BASE_URL}/monitor/overview`);
  if (!response.ok) throw new Error("Failed to fetch monitor overview");
  return response.json();
}

export interface AttendanceRules {
  work_unit_minutes: number;
  rounding_policy: string;
  early_arrival_strategy: "allow" | "block" | "clamp";
}

export interface AttendancePolicyVersion {
  id: string;
  work_unit_minutes: number;
  rounding_policy: string;
  early_arrival_strategy: "allow" | "block" | "clamp";
  valid_from: string;
  valid_to: string | null;
  created_by: string | null;
  created_at: string;
}

export interface UpdateAttendanceRulesRequest {
  work_unit_minutes?: number;
  rounding_policy?: string;
  early_arrival_strategy?: "allow" | "block" | "clamp";
  valid_from?: string;
}

export async function fetchDefaultAttendanceRules(wsId: string): Promise<AttendanceRules> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/attendance/policies/default`);
  if (!response.ok) throw new Error("Failed to fetch default attendance rules");
  return response.json();
}

export async function fetchAttendancePolicyVersions(wsId: string): Promise<AttendancePolicyVersion[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/attendance/policies/default/versions`);
  if (!response.ok) throw new Error("Failed to fetch attendance policy versions");
  return response.json();
}

export interface PolicyAssignmentResponse {
  id: string;
  target_type: "USER" | "GROUP";
  target_id: string;
  valid_from: string;
  valid_to: string | null;
  rules: AttendanceRules;
}

export interface UpsertPolicyAssignmentRequest {
  target_type: "USER" | "GROUP";
  target_id: string;
  work_unit_minutes: number;
  rounding_policy: string;
  early_arrival_strategy: "allow" | "block" | "clamp";
  valid_from?: string;
}

export async function fetchPolicyAssignments(wsId: string): Promise<PolicyAssignmentResponse[]> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/attendance/policies/assignments`);
  if (!response.ok) throw new Error("Failed to fetch policy assignments");
  return response.json();
}

export async function upsertPolicyAssignment(
  wsId: string,
  data: UpsertPolicyAssignmentRequest
): Promise<PolicyAssignmentResponse> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/attendance/policies/assignments`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to upsert policy assignment");
  return response.json();
}

export async function deletePolicyAssignment(wsId: string, assignmentId: string): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/attendance/policies/assignments/${assignmentId}`, {
    method: "DELETE",
  });
  if (!response.ok) throw new Error("Failed to delete policy assignment");
}

export async function updateDefaultAttendanceRules(
  wsId: string,
  data: UpdateAttendanceRulesRequest
): Promise<AttendancePolicyVersion> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/attendance/policies/default`, {
    method: "PATCH",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to update default attendance rules");
  return response.json();
}
