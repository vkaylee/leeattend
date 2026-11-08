import { client } from "./client";
import type {
  BulkCreateWorkSegmentsRequest,
  CreateShiftIntervalRequest,
  CreateWorkSegmentRequest,
  ShiftDefaultInterval,
  UpdateShiftIntervalRequest,
  UpdateWorkSegmentRequest,
  WorkSegment,
} from "./types";

export interface WorkSegmentListResponse {
  segments: WorkSegment[];
  total_count: number;
}

export async function fetchWorkSegments(
  params?: {
    employment_ids?: string[];
    start_date?: string;
    end_date?: string;
    segment_type?: string;
    segment_status?: string;
    source?: string;
    page?: number;
    limit?: number;
  }
): Promise<WorkSegment[]> {
  const queryParams: Record<string, unknown> = {
    ...params,
    employment_ids: params?.employment_ids && params.employment_ids.length > 0 ? params.employment_ids.join(",") : undefined,
  };

  const { data: resData } = await client.GET<WorkSegmentListResponse>("/v1/work-segments", {
    params: {
      query: queryParams,
    },
  });
  return resData.segments;
}

export async function createWorkSegment(
  body: CreateWorkSegmentRequest
): Promise<WorkSegment> {
  const { data: resData } = await client.POST<WorkSegment>("/v1/work-segments", {
    body,
  });
  return resData;
}

export async function getWorkSegment(
  segmentId: string
): Promise<WorkSegment> {
  const { data: resData } = await client.GET<WorkSegment>(
    "/v1/work-segments/{segment_id}",
    { params: { path: { segment_id: segmentId } } },
  );
  return resData;
}

export async function updateWorkSegment(
  segmentId: string,
  body: UpdateWorkSegmentRequest
): Promise<WorkSegment> {
  const { data: resData } = await client.PUT<WorkSegment>(
    "/v1/work-segments/{segment_id}",
    {
      params: { path: { segment_id: segmentId } },
      body,
    },
  );
  return resData;
}

export async function deleteWorkSegment(
  segmentId: string
): Promise<void> {
  await client.DELETE("/v1/work-segments/{segment_id}", {
    params: { path: { segment_id: segmentId } },
  });
}

export async function bulkCreateWorkSegments(
  body: BulkCreateWorkSegmentsRequest
): Promise<WorkSegment[]> {
  const { data: resData } = await client.POST<WorkSegment[]>("/v1/work-segments/bulk", {
    body,
  });
  return resData;
}

export async function scheduleWorkSegments(
  params: { start_date: string; end_date: string; employment_ids?: string[] }
): Promise<{ scheduled: number; skipped: number }> {
  const queryParams: Record<string, unknown> = {
    ...params,
    employment_ids: params.employment_ids && params.employment_ids.length > 0 ? params.employment_ids.join(",") : undefined,
  };
  const { data: resData } = await client.GET<{ scheduled: number; skipped: number }>("/v1/work-segments/schedule", {
    params: {
      query: queryParams,
    },
  });
  return resData;
}

// ── Shift Intervals ────────────────────────────────────────────────────────

export async function fetchShiftIntervals(
  shiftId: string
): Promise<ShiftDefaultInterval[]> {
  const { data: resData } = await client.GET<ShiftDefaultInterval[]>("/v1/shifts/{shift_id}/intervals", {
    params: { path: { shift_id: shiftId } },
  });
  return resData;
}

export async function createShiftInterval(
  shiftId: string,
  body: CreateShiftIntervalRequest
): Promise<ShiftDefaultInterval> {
  const { data: resData } = await client.POST<ShiftDefaultInterval>(
    "/v1/shifts/{shift_id}/intervals",
    {
      params: { path: { shift_id: shiftId } },
      body,
    },
  );
  return resData;
}

export async function updateShiftInterval(
  shiftId: string,
  intervalId: string,
  body: UpdateShiftIntervalRequest
): Promise<ShiftDefaultInterval> {
  const { data: resData } = await client.PUT<ShiftDefaultInterval>(
    "/v1/shifts/{shift_id}/intervals/{interval_id}",
    {
      params: { path: { shift_id: shiftId, interval_id: intervalId } },
      body,
    },
  );
  return resData;
}

export async function deleteShiftInterval(
  shiftId: string,
  intervalId: string
): Promise<void> {
  await client.DELETE(
    "/v1/shifts/{shift_id}/intervals/{interval_id}",
    { params: { path: { shift_id: shiftId, interval_id: intervalId } } },
  );
}

export async function reorderShiftIntervals(
  shiftId: string,
  intervalIds: string[]
): Promise<void> {
  await client.POST("/v1/shifts/{shift_id}/intervals/reorder", {
    params: { path: { shift_id: shiftId } },
    body: { interval_ids: intervalIds },
  });
}
