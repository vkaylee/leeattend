import { client } from "./client";
import { ApiError } from "./errors";
import { apiFetch } from "./helpers";
import type {
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
}

export async function fetchAttendanceReport(
  params: FetchAttendanceReportParams
): Promise<PaginatedResponse<AttendanceSummary | DailyAttendanceReport>> {
  const {
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

  const { data: resData } = await client.GET<PaginatedResponse<AttendanceSummary | DailyAttendanceReport>>(
    "/v1/reports/attendance",
    {
      params: {
        query: {
          start_date: start,
          end_date: end,
          type: type,
          page: page,
          limit: limit,
          sort: sort,
          employee_ids: employeeIds && employeeIds.length > 0 ? employeeIds.join(",") : undefined,
          department_id: departmentId ?? undefined,
          employee_status: employeeStatus,
          employer_id: employerId ?? undefined,
          employment_id: employmentId ?? undefined,
          shift_type: shiftType && shiftType.length > 0 ? shiftType.join(",") : undefined,
        },
      },
    },
  );

  return resData;
}

export async function uploadAttendance(
  file: File,
  endpoint: string
): Promise<{ job_id: string }> {
  const formData = new FormData();
  formData.append("file", file);

  const url = endpoint === "/employees/upload"
    ? "/v1/employees/upload"
    : endpoint === "/attendance/logs/upload"
      ? "/v1/attendance/logs/upload"
      : "/v1/attendance/upload";

  const res = await apiFetch(url, {
    method: "POST",
    body: formData,
  });
  if (!res.ok) await ApiError.rejectResponse(res);
  return (await res.json()) as { job_id: string };
}

export async function fetchLatestImportJob(
  targetType?: "log" | "adjustment" | "employee" | "leave_import"
): Promise<AttendanceImportJob | null> {
  const { data: resData } = await client.GET<AttendanceImportJob | null>(
    "/v1/attendance/import-jobs/latest",
    {
      params: {
        query: targetType ? { target_type: targetType } : undefined,
      },
    },
  );
  return resData;
}

export async function fetchImportJobStatus(jobId: string): Promise<AttendanceImportJob> {
  const { data: resData } = await client.GET<AttendanceImportJob>(
    "/v1/attendance/import-jobs/{id}",
    {
      params: {
        path: { id: jobId },
      },
    },
  );
  return resData;
}

export async function reprocessAttendance(
  employeeId: string,
  startDate: string,
  endDate: string
): Promise<void> {
  await client.POST("/v1/attendance/reprocess/{employee_id}", {
    params: {
      path: { employee_id: employeeId },
      query: { start_date: startDate, end_date: endDate },
    },
  });
}

export async function reprocessAllAttendance(
  startDate: string,
  endDate: string
): Promise<{ message: string; job_id: string }> {
  const { data: resData } = await client.POST<{ message: string; job_id: string }>("/v1/attendance/reprocess-all", {
    params: { query: { start_date: startDate, end_date: endDate } },
  });
  return resData;
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
}

export type RecordingIssue =
  | "missing_punch_in"
  | "active_overtime"
  | "punch_out_pending"
  | "missing_punch_out";

export type NoShowReportRow = DailyAttendanceReport & {
  recording_issue: RecordingIssue;
  expected_end_at?: string;
};

export async function fetchNoShowReport(
  params: FetchNoShowReportParams,
): Promise<PaginatedResponse<NoShowReportRow>> {
  const { date, page, limit, employeeIds, shiftType } = params;
  const { data: resData } = await client.GET<PaginatedResponse<NoShowReportRow>>(
    "/v1/reports/no-show",
    {
      params: {
        query: {
          date,
          page,
          limit,
          employee_ids: employeeIds && employeeIds.length > 0 ? employeeIds.join(",") : undefined,
          shift_type: shiftType && shiftType.length > 0 ? shiftType.join(",") : undefined,
        },
      },
    },
  );
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
  task_type: string;
}

export async function checkReprocessStatus(jobId: string): Promise<GenericJobStatus> {
  const { data: resData } = await client.GET<GenericJobStatus>("/v1/reports/jobs/{job_id}", {
    params: { path: { job_id: jobId } },
  });
  return resData;
}

export interface AttendanceStats {
  early_out_count: number;
  late_in_count: number;
  missing_punch_count: number;
  present_count: number;
  total_count: number;
}
