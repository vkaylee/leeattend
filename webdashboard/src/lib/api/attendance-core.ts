import { client } from "./client";
import type {
  TimeEntry,
  AttendanceAdjustmentRequest,
  RawAttendanceLog,
  PaginatedResponse,
} from "./types";

export async function punchIn(
  data: { employment_id: string; entry_type?: "Work" | "Break" | "Overtime"; note?: string },
): Promise<TimeEntry> {
  const { data: resData } = await client.POST<TimeEntry>("/api/v1/time/punch-in", {
    body: data,
  });
  return resData;
}

export async function punchOut(
  data: { employee_id: string; note?: string },
): Promise<TimeEntry> {
  const { data: resData } = await client.POST<TimeEntry>("/api/v1/time/punch-out/{employee_id}", {
    params: {
      path: { employee_id: data.employee_id },
    },
    body: {
      note: data.note,
    },
  });
  return resData;
}

export async function submitAttendanceAdjustment(
  data: {
    employment_id: string;
    time_entry_id?: string;
    requested_punch_time: string;
    reason?: string;
    attachment_file_ids?: string[];
  }
): Promise<AttendanceAdjustmentRequest> {
  const { data: resData } = await client.POST<AttendanceAdjustmentRequest>("/api/v1/attendance/adjustments", {
    body: data,
  });
  return resData;
}

export async function fetchAttendanceAdjustments(
  params?: {
    status?: string;
    employee_id?: string;
    page?: number;
    limit?: number;
  }
): Promise<PaginatedResponse<AttendanceAdjustmentRequest>> {
  const { data: resData } = await client.GET<PaginatedResponse<AttendanceAdjustmentRequest>>("/api/v1/attendance/adjustments", {
    params: {
      query: {
        status: params?.status as "pending" | "approved" | "rejected" | null | undefined,
        employee_id: params?.employee_id === "all" ? undefined : params?.employee_id,
        page: params?.page,
        limit: params?.limit,
      },
    },
  });
  return resData;
}

export async function approveAttendanceAdjustment(
  requestId: string,
  data: { status: "pending" | "approved" | "rejected" }
): Promise<void> {
  await client.PUT("/api/v1/attendance/adjustments/{id}/approve", {
    params: { path: { id: requestId } },
    body: data,
  });
}

export async function fetchRawAttendanceLogs(params: {
  employee_id?: string;
  from?: string;
  to?: string;
  page?: number;
  limit?: number;
}): Promise<{ data: RawAttendanceLog[]; total: number; page: number; limit: number }> {
  const { data: resData } = await client.GET<{ data: RawAttendanceLog[]; total: number; page: number; limit: number }>(
    "/api/v1/attendance/raw-logs",
    {
      params: {
        query: {
          employee_id: params.employee_id ?? undefined,
          from: params.from ?? undefined,
          to: params.to ?? undefined,
          page: params.page ?? undefined,
          limit: params.limit ?? undefined,
        },
      },
    },
  );
  return resData;
}
