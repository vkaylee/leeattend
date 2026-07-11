import {
  type BulkCreateWorkSegmentsRequest,
  type CreateShiftIntervalRequest,
  type CreateWorkSegmentRequest,
  type ShiftDefaultInterval,
  type UpdateShiftIntervalRequest,
  type UpdateWorkSegmentRequest,
  type WorkSegment,
} from "./types";
import { fetch, fire } from "./client";

// /workspaces/{workspace_id}/work-segments* and shifts/{shift_id}/intervals*
// paths are not yet registered in the generated OpenAPI schema. They route
// through the typed `fetchUntyped` wrapper so call-site bodies stay typed and
// the data shape is asserted locally. When these paths land in openapi.raw.json
// and the schema is regenerated, migrate to the typed `client.METHOD(...)`.

// ── Work Segments ──────────────────────────────────────────────────────────

export async function fetchWorkSegments(
  wsId: string,
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
  const { data: resData } = await fetch<{ data: WorkSegment[] }>("GET", "/workspaces/{workspace_id}/work-segments", {
    params: {
      path: { workspace_id: wsId },
      query: params,
    },
  });
  if (!resData) throw new Error("No response data");
  return resData.data;
}

export async function createWorkSegment(
  wsId: string,
  body: CreateWorkSegmentRequest
): Promise<WorkSegment> {
  const { data: resData } = await fetch<WorkSegment>("POST", "/workspaces/{workspace_id}/work-segments", {
    params: { path: { workspace_id: wsId } },
    body,
  });
  return resData;
}

export async function getWorkSegment(
  wsId: string,
  segmentId: string
): Promise<WorkSegment> {
  const { data: resData } = await fetch<WorkSegment>("GET",
    "/workspaces/{workspace_id}/work-segments/{segment_id}",
    { params: { path: { workspace_id: wsId, segment_id: segmentId } } },
  );
  return resData;
}

export async function updateWorkSegment(
  wsId: string,
  segmentId: string,
  body: UpdateWorkSegmentRequest
): Promise<WorkSegment> {
  const { data: resData } = await fetch<WorkSegment>("PUT",
    "/workspaces/{workspace_id}/work-segments/{segment_id}",
    {
      params: { path: { workspace_id: wsId, segment_id: segmentId } },
      body,
    },
  );
  return resData;
}

export async function deleteWorkSegment(
  wsId: string,
  segmentId: string
): Promise<void> {
  await fire("DELETE", "/workspaces/{workspace_id}/work-segments/{segment_id}", {
    params: { path: { workspace_id: wsId, segment_id: segmentId } },
  });
}

export async function bulkCreateWorkSegments(
  wsId: string,
  body: BulkCreateWorkSegmentsRequest
): Promise<WorkSegment[]> {
  const { data: resData } = await fetch<WorkSegment[]>("POST", "/workspaces/{workspace_id}/work-segments/bulk", {
    params: { path: { workspace_id: wsId } },
    body,
  });
  return resData;
}

export async function scheduleWorkSegments(
  wsId: string,
  params: { start_date: string; end_date: string; employment_ids?: string[] }
): Promise<{ scheduled: number; skipped: number }> {
  const { data: resData } = await fetch<{ scheduled: number; skipped: number }>("GET", "/workspaces/{workspace_id}/work-segments/schedule", {
    params: {
      path: { workspace_id: wsId },
      query: params,
    },
  });
  return resData;
}

// ── Shift Intervals ─────────────────────────────────────────────────────────

export async function fetchShiftIntervals(
  wsId: string,
  shiftId: string
): Promise<ShiftDefaultInterval[]> {
  const { data: resData } = await fetch<ShiftDefaultInterval[]>("GET", "/workspaces/{workspace_id}/shifts/{shift_id}/intervals", {
    params: { path: { workspace_id: wsId, shift_id: shiftId } },
  });
  return resData;
}

export async function createShiftInterval(
  wsId: string,
  shiftId: string,
  body: CreateShiftIntervalRequest
): Promise<ShiftDefaultInterval> {
  const { data: resData } = await fetch<ShiftDefaultInterval>("POST",
    "/workspaces/{workspace_id}/shifts/{shift_id}/intervals",
    {
      params: { path: { workspace_id: wsId, shift_id: shiftId } },
      body,
    },
  );
  return resData;
}

export async function updateShiftInterval(
  wsId: string,
  shiftId: string,
  intervalId: string,
  body: UpdateShiftIntervalRequest
): Promise<ShiftDefaultInterval> {
  const { data: resData } = await fetch<ShiftDefaultInterval>("PUT",
    "/workspaces/{workspace_id}/shifts/{shift_id}/intervals/{interval_id}",
    {
      params: { path: { workspace_id: wsId, shift_id: shiftId, interval_id: intervalId } },
      body,
    },
  );
  return resData;
}

export async function deleteShiftInterval(
  wsId: string,
  shiftId: string,
  intervalId: string
): Promise<void> {
  await fire("DELETE",
    "/workspaces/{workspace_id}/shifts/{shift_id}/intervals/{interval_id}",
    { params: { path: { workspace_id: wsId, shift_id: shiftId, interval_id: intervalId } } },
  );
}

export async function reorderShiftIntervals(
  wsId: string,
  shiftId: string,
  intervalIds: string[]
): Promise<void> {
  await fire("POST", "/workspaces/{workspace_id}/shifts/{shift_id}/intervals/reorder", {
    params: { path: { workspace_id: wsId, shift_id: shiftId } },
    body: { interval_ids: intervalIds },
  });
}
