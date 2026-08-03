import { client, fetch, fire } from "./client";
import {
  type BulkCreateWorkSegmentsRequest,
  type CreateShiftIntervalRequest,
  type CreateWorkSegmentRequest,
  type ShiftDefaultInterval,
  type UpdateShiftIntervalRequest,
  type UpdateWorkSegmentRequest,
  type WorkSegment,
} from "./types";

// ── Work Segments (typed client) ───────────────────────────────────────────
//
// These paths are registered in the Rust ApiDoc (utoipa) and present in the
// generated OpenAPI schema.  Migrated from legacy `fetch<T>()` to typed
// `client.METHOD(...)` for compile-time path/param/response validation.
//
// NOTE: /work-segments/schedule and /shifts/{shift_id}/intervals* are NOT
// yet backed by backend handlers — see .ai/ADRs/work-segments-intervals-openapi-gap.md.

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
  const { data: resData } = await client.GET("/workspaces/{workspace_id}/work-segments", {
    params: {
      path: { workspace_id: wsId },
      // The OpenAPI schema models employment_ids as string | null (comma-separated),
      // but the call-site parameter is string[] for ergonomics.  The legacy path
      // also sent an array via openapi-fetch serialization.
      query: params as Record<string, unknown>,
    },
  });
  return resData.segments as WorkSegment[];
}

export async function createWorkSegment(
  wsId: string,
  body: CreateWorkSegmentRequest
): Promise<WorkSegment> {
  const { data: resData } = await client.POST("/workspaces/{workspace_id}/work-segments", {
    params: { path: { workspace_id: wsId } },
    body,
  });
  return resData as WorkSegment;
}

export async function getWorkSegment(
  wsId: string,
  segmentId: string
): Promise<WorkSegment> {
  const { data: resData } = await client.GET(
    "/workspaces/{workspace_id}/work-segments/{segment_id}",
    { params: { path: { workspace_id: wsId, segment_id: segmentId } } },
  );
  return resData as WorkSegment;
}

export async function updateWorkSegment(
  wsId: string,
  segmentId: string,
  body: UpdateWorkSegmentRequest
): Promise<WorkSegment> {
  const { data: resData } = await client.PUT(
    "/workspaces/{workspace_id}/work-segments/{segment_id}",
    {
      params: { path: { workspace_id: wsId, segment_id: segmentId } },
      body,
    },
  );
  return resData as WorkSegment;
}

export async function deleteWorkSegment(
  wsId: string,
  segmentId: string
): Promise<void> {
  await client.DELETE("/workspaces/{workspace_id}/work-segments/{segment_id}", {
    params: { path: { workspace_id: wsId, segment_id: segmentId } },
  });
}

export async function bulkCreateWorkSegments(
  wsId: string,
  body: BulkCreateWorkSegmentsRequest
): Promise<WorkSegment[]> {
  const { data: resData } = await client.POST("/workspaces/{workspace_id}/work-segments/bulk", {
    params: { path: { workspace_id: wsId } },
    body,
  });
  return resData as WorkSegment[];
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

// ── Shift Intervals (legacy escape hatch) ──────────────────────────────────
// No backend routes/handlers exist yet for /shifts/{shift_id}/intervals*.
// These stay on legacy `fetch`/`fire` until backend endpoints are built.
// See .ai/ADRs/work-segments-intervals-openapi-gap.md.

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
