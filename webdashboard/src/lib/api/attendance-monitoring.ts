import { submitAttendanceAdjustment, approveAttendanceAdjustment } from "./attendance-core";
import { createBehalfAssignmentApi } from "./behalf-assignment-factory";
import { Nullable } from "@/types/api-overrides";
import { client } from "./client";
import { ApiError } from "./errors";
import { apiFetch, TIMEOUT } from "./helpers";
import {
  DailyAttendanceReport,
  WorkerMonitorStats,
  MonitorJobsResponse,
  CreateReportExportRequest,
  CreateReportExportResponse,
} from "./types";

// --- Report Export ---

export async function createReportExportJob(
  wsId: string,
  request: CreateReportExportRequest
): Promise<CreateReportExportResponse> {
  const { data: resData } = await client.POST("/workspaces/{ws_id}/reports/export", {
    params: { path: { ws_id: wsId } },
    body: request
  });
  return resData;
}

export async function fetchExportJobStatus(
  wsId: string,
  jobId: string
) {
  const { data: resData } = await client.GET("/workspaces/{ws_id}/reports/jobs/{job_id}", {
    params: { path: { ws_id: wsId, job_id: jobId } }
  });
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
  if (!response.ok) await ApiError.rejectResponse(response);
  return response.blob();
}

// --- Worker Monitoring (Global Infrastructure) ---

export async function fetchMonitorOverview(): Promise<WorkerMonitorStats> {
  const { data: resData } = await client.GET("/monitor/overview", {});
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
  return (resData as Nullable<MonitorJobsResponse>) ?? { jobs: [], next_cursor: null };
}

// ── OT Claim Behalf Assignments (factory-generated CRUD) ────────────────────

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

const otClaimBehalfApi = createBehalfAssignmentApi<OtClaimBehalfAssignment>({
  basePath: "/v1/workspaces/{ws_id}/attendance/claim-ot/behalf-assignments",
});

export const fetchBehalfAssignments = (wsId: string) => otClaimBehalfApi.list(wsId);
export const createBehalfAssignment = (wsId: string, data: Parameters<typeof otClaimBehalfApi.create>[1]) => otClaimBehalfApi.create(wsId, data);
export const deleteBehalfAssignment = (wsId: string, id: string) => otClaimBehalfApi.delete(wsId, id);

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

// ── Attendance Adjustment Behalf Assignments (factory-generated CRUD) ──────

const adjustmentBehalfApi = createBehalfAssignmentApi<OtClaimBehalfAssignment>({
  basePath: "/v1/workspaces/{ws_id}/attendance/adjustments/behalf-assignments",
});

export const fetchAdjustmentBehalfAssignments = (wsId: string) => adjustmentBehalfApi.list(wsId);
export const createAdjustmentBehalfAssignment = (wsId: string, data: Parameters<typeof adjustmentBehalfApi.create>[1]) => adjustmentBehalfApi.create(wsId, data);
export const deleteAdjustmentBehalfAssignment = (wsId: string, id: string) => adjustmentBehalfApi.delete(wsId, id);
