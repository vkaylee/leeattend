
import type { components } from "@/types/api-generated";

import { submitAttendanceAdjustment, approveAttendanceAdjustment } from "./attendance-core";
import { createBehalfAssignmentApi } from "./behalf-assignment-factory";
import { client } from "./client";
import { ApiError } from "./errors";
import { apiFetch, TIMEOUT } from "./helpers";
import {
  DailyAttendanceReport,
  MonitorAlert,
  MonitorAlertDetail,
  MonitorAlertMutationRequest,
  MonitorAlertsResponse,
  WorkerMonitorStats,
  MonitorJobsResponse,
  MonitorMetric,
  CreateReportExportRequest,
  CreateReportExportResponse,
} from "./types";

// --- Report Export ---

export async function createReportExportJob(
  request: CreateReportExportRequest
): Promise<CreateReportExportResponse> {
  const { data: resData } = await client.POST<CreateReportExportResponse>("/v1/reports/export", {
    body: request,
  });
  return resData;
}

export async function fetchExportJobStatus(
  jobId: string
): Promise<components["schemas"]["JobStatusResponse"]> {
  const { data: resData } = await client.GET<components["schemas"]["JobStatusResponse"]>(
    "/v1/reports/jobs/{job_id}",
    { params: { path: { job_id: jobId } } },
  );
  return resData;
}

export async function cancelExportJob(jobId: string): Promise<void> {
  await client.DELETE("/v1/reports/jobs/{job_id}", {
    params: { path: { job_id: jobId } },
  });
}

export async function downloadExportReport(jobId: string): Promise<Blob> {
  const response = await apiFetch(`/v1/reports/jobs/${encodeURIComponent(jobId)}/download`, {
    timeout: TIMEOUT.REPORT_OPERATION,
  });
  if (!response.ok) await ApiError.rejectResponse(response);
  return response.blob();
}

// --- Worker Monitoring (Global Infrastructure) ---

const unknownMonitorMetric = (unit = "unknown"): MonitorMetric => ({
  formula: "source unavailable",
  observed_at: new Date(0).toISOString(),
  source: "unavailable",
  time_window: {
    duration_seconds: 0,
    end: new Date(0).toISOString(),
    start: new Date(0).toISOString(),
  },
  unit,
  unknown: {
    code: "MONITOR_UNAVAILABLE",
    reason: "Monitoring source unavailable",
  },
  value: null,
});

const unavailableDependency = (limitation: string) => ({
  availability: null,
  latency_ms: null,
  limitation,
  observed_at: new Date(0).toISOString(),
  pool_utilization_percent: null,
  source: "unavailable",
  status: "unavailable" as const,
});

export async function fetchMonitorOverview(): Promise<WorkerMonitorStats> {
  const { data: resData } = await client.GET<WorkerMonitorStats>("/v1/monitor/overview", {});
  // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- SafeClient types say non-null but API may return null
  return resData ?? {
    active_workers: null,
    completed_today: null,
    durable_jobs: { error_code: "MONITOR_UNAVAILABLE", events_truncated: false, jobs: null, limit: 20, next_cursor_token: null, status: "unavailable" },
    dependencies: {
      cpu: unavailableDependency("No trustworthy CPU metric source is configured."),
      database_pool: unavailableDependency("Database pool status is unavailable."),
      disk: unavailableDependency("No trustworthy disk metric source is configured."),
      memory: unavailableDependency("No trustworthy memory metric source is configured."),
      outbox_sweeper: unavailableDependency("No persisted sweeper heartbeat or authoritative backlog source is available."),
      postgres: unavailableDependency("PostgreSQL health source is unavailable."),
      redis: unavailableDependency("Redis health source is unavailable."),
      worker_process: unavailableDependency("Worker heartbeat source is unavailable."),
    },
    failed_today: null,
    oldest_queued_job: null,
    oldest_running_job: null,
    idle_workers: null,
    job_type_stats: [],
    observed_at: new Date(0).toISOString(),
    pending_high: null,
    pending_low: null,
    pending_normal: null,
    pending_tasks: null,
    queue_health: {
      dead_letter_depth: unknownMonitorMetric("jobs"),
      depth_high: unknownMonitorMetric("jobs"),
      depth_low: unknownMonitorMetric("jobs"),
      depth_normal: unknownMonitorMetric("jobs"),
      depth_total: unknownMonitorMetric("jobs"),
      enqueue_rate_per_second: unknownMonitorMetric("jobs/second"),
      estimated_drain_seconds: unknownMonitorMetric("seconds"),
      growth_rate_per_second: unknownMonitorMetric("jobs/second"),
      oldest_queued_age_seconds: unknownMonitorMetric("seconds"),
      processing_rate_per_second: unknownMonitorMetric("jobs/second"),
      queue_wait_seconds_p50: unknownMonitorMetric("seconds"),
      queue_wait_seconds_p95: unknownMonitorMetric("seconds"),
    },
    reliability: {
      consecutive_failures: unknownMonitorMetric("jobs"),
      dead_letter_count: unknownMonitorMetric("jobs"),
      execution_seconds_p50: unknownMonitorMetric("seconds"),
      execution_seconds_p95: unknownMonitorMetric("seconds"),
      execution_seconds_p99: unknownMonitorMetric("seconds"),
      failure_rate: unknownMonitorMetric("ratio"),
      lost_count: unknownMonitorMetric("jobs"),
      retryable_count: unknownMonitorMetric("jobs"),
      success_rate: unknownMonitorMetric("ratio"),
      timeout_count: unknownMonitorMetric("jobs"),
    },
    task_type_reliability: [],
    thresholds: {
      live_after_seconds: 15,
      queue_pressure: { critical_at: 16, high_at: 6, medium_at: 1, source: "unavailable" },
      source: "unavailable",
      stale_after_seconds: 60,
      worker_utilization: { critical_at_percent: 80, scale_down_below_percent: 30, source: "unavailable", warning_at_percent: 50 },
    },
    quality: {
      data_quality: "unavailable",
      durable_db: { error_code: "MONITOR_UNAVAILABLE", status: "unavailable" },
      freshness: "unknown",
      redis: { error_code: "MONITOR_UNAVAILABLE", status: "unavailable" },
      snapshot_sequence: "unknown",
      unknown_fields: [
        "active_workers",
        "completed_today",
        "durable_jobs",
        "failed_today",
        "idle_workers",
        "job_type_stats",
        "pending_high",
        "pending_low",
        "pending_normal",
        "pending_tasks",
        "recent_jobs",
        "total_workers",
        "workers",
        "queue_health",
        "reliability",
        "oldest_queued_job",
        "oldest_running_job",
        "task_type_reliability",
        "thresholds",
      ],
      worker_heartbeats: { error_code: "MONITOR_UNAVAILABLE", status: "unavailable" },
    },
    recent_jobs: [],
    snapshot_age_seconds: Number.MAX_SAFE_INTEGER,
    total_workers: null,
    workers: [],
  };
}

export interface FetchMonitorJobsParams {
  cursor?: number | null;
  durable_cursor?: string | null;
  durable_limit?: number;
  include_durable?: boolean;
  include_legacy?: boolean;
  limit?: number;
  search?: string;
  state?: string;
  status?: string;
  task_type?: string;
}

export interface FetchMonitorAlertsParams {
  cursor?: string | null;
  limit?: number;
  severity?: string;
  status?: string;
}

export async function fetchMonitorAlerts(
  params: FetchMonitorAlertsParams = {},
): Promise<MonitorAlertsResponse> {
  const { data } = await client.GET<MonitorAlertsResponse>("/v1/monitor/alerts", {
    params: {
      query: {
        cursor: params.cursor ?? undefined,
        limit: params.limit,
        severity: params.severity,
        status: params.status,
      },
    },
  });
  return data;
}

export async function fetchMonitorAlert(alertId: string): Promise<MonitorAlertDetail> {
  const { data } = await client.GET<MonitorAlertDetail>("/v1/monitor/alerts/{alert_id}", {
    params: { path: { alert_id: alertId } },
  });
  return data;
}

export async function acknowledgeMonitorAlert(
  alertId: string,
  request: MonitorAlertMutationRequest,
): Promise<MonitorAlert> {
  const { data } = await client.POST<MonitorAlert>("/v1/monitor/alerts/{alert_id}/acknowledge", {
    params: { path: { alert_id: alertId } },
    body: request,
  });
  return data;
}

export async function resolveMonitorAlert(
  alertId: string,
  request: MonitorAlertMutationRequest,
): Promise<MonitorAlert> {
  const { data } = await client.POST<MonitorAlert>("/v1/monitor/alerts/{alert_id}/resolve", {
    params: { path: { alert_id: alertId } },
    body: request,
  });
  return data;
}

export async function fetchMonitorJobs(params: FetchMonitorJobsParams): Promise<MonitorJobsResponse> {
  const { data: resData } = await client.GET<MonitorJobsResponse>("/v1/monitor/jobs", {
    params: {
      query: {
        cursor: params.cursor ?? undefined,
        durable_cursor: params.durable_cursor ?? undefined,
        durable_limit: params.durable_limit,
        include_durable: params.include_durable,
        include_legacy: params.include_legacy,
        limit: params.limit,
        search: params.search ?? undefined,
        state: params.state ?? undefined,
        status: params.status ?? undefined,
        task_type: params.task_type ?? undefined,
      },
    },
  });
  // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- SafeClient types say non-null but API may return null
  return resData ?? {
    durable_jobs: { error_code: "MONITOR_UNAVAILABLE", events_truncated: false, jobs: null, limit: params.durable_limit ?? 20, next_cursor_token: null, status: "unavailable" },
    jobs: [],
    next_cursor: null,
    quality: {
      data_quality: "unavailable",
      durable_db: { error_code: "MONITOR_UNAVAILABLE", status: "unavailable" },
      freshness: "unknown",
      redis: { error_code: "MONITOR_UNAVAILABLE", status: "unavailable" },
      snapshot_sequence: "unknown",
      unknown_fields: ["jobs", "durable_jobs"],
      worker_heartbeats: { error_code: "MONITOR_UNAVAILABLE", status: "unavailable" },
    },
  };
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
  workspace_id?: string;
}

const otClaimBehalfApi = createBehalfAssignmentApi<OtClaimBehalfAssignment>({
  basePath: "/v1/attendance/claim-ot/behalf-assignments",
});

export const fetchBehalfAssignments = (): Promise<OtClaimBehalfAssignment[]> =>
  otClaimBehalfApi.list();
export const createBehalfAssignment = (
  data: Parameters<typeof otClaimBehalfApi.create>[0],
): Promise<OtClaimBehalfAssignment> => otClaimBehalfApi.create(data);
export const deleteBehalfAssignment = (id: string): Promise<void> =>
  otClaimBehalfApi.delete(id);

export async function claimOt(
  employeeId: string,
  data: {
    date: string;
    claimed_minutes: number;
    reason?: string | null;
  }
): Promise<void> {
  await client.POST("/v1/attendance/claim-ot/{employee_id}", {
    params: { path: { employee_id: employeeId } },
    body: data
  });
}

export async function adminConfirmAttendance(
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

    const adjustment = await submitAttendanceAdjustment({
      employment_id: row.employment_id,
      requested_punch_time: toDatetime(row.date, punchIn),
      reason: reason ?? "Admin confirmation",
    });
    requestId = adjustment.id;

    if (punchOut) {
      const adjustmentOut = await submitAttendanceAdjustment({
        employment_id: row.employment_id,
        requested_punch_time: toDatetime(row.date, punchOut),
        reason: reason ?? "Admin confirmation",
      });
      await approveAttendanceAdjustment(adjustmentOut.id, { status: "approved" });
    }
  }

  await approveAttendanceAdjustment(requestId, { status: "approved" });
}

// ── Attendance Adjustment Behalf Assignments (factory-generated CRUD) ──────

const adjustmentBehalfApi = createBehalfAssignmentApi<OtClaimBehalfAssignment>({
  basePath: "/v1/attendance/adjustments/behalf-assignments",
});

export const fetchAdjustmentBehalfAssignments = (): Promise<OtClaimBehalfAssignment[]> =>
  adjustmentBehalfApi.list();
export const createAdjustmentBehalfAssignment = (
  data: Parameters<typeof adjustmentBehalfApi.create>[0],
): Promise<OtClaimBehalfAssignment> => adjustmentBehalfApi.create(data);
export const deleteAdjustmentBehalfAssignment = (id: string): Promise<void> =>
  adjustmentBehalfApi.delete(id);
