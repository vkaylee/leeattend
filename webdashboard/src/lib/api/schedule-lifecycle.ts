import { client } from "./client";
import type {
  ScheduleEvidenceResponse,
  ScheduleHistoryResponse,
  SchedulePreviewRequest,
  SchedulePreviewResponse,
  SchedulePublishRequest,
  SchedulePublicationResponse,
  ScheduleReplanPreviewRequest,
  ScheduleReplanPreviewResponse,
  ScheduleReplanRequest,
  ScheduleReplanResponse,
} from "./types-schedule-lifecycle";

const schedulePath = "/api/v1/schedules";

export function previewSchedules(body: SchedulePreviewRequest): Promise<{ data: SchedulePreviewResponse }> {
  return client.POST<SchedulePreviewResponse>(`${schedulePath}/preview`, { body });
}

export function publishSchedules(
  body: SchedulePublishRequest,
  idempotencyKey: string,
): Promise<{ data: SchedulePublicationResponse }> {
  return client.POST<SchedulePublicationResponse>(`${schedulePath}/publish`, {
    body,
    headers: { "Idempotency-Key": idempotencyKey },
  });
}

export function previewScheduleReplan(
  employmentId: string,
  businessDate: string,
  body: ScheduleReplanPreviewRequest,
): Promise<{ data: ScheduleReplanPreviewResponse }> {
  return client.POST<ScheduleReplanPreviewResponse>(
    `${schedulePath}/{employment_id}/{business_date}/replan/preview`,
    { params: { path: { employment_id: employmentId, business_date: businessDate } }, body },
  );
}

export function commitScheduleReplan(
  employmentId: string,
  businessDate: string,
  body: ScheduleReplanRequest,
  idempotencyKey: string,
): Promise<{ data: ScheduleReplanResponse }> {
  return client.POST<ScheduleReplanResponse>(
    `${schedulePath}/{employment_id}/{business_date}/replan`,
    {
      params: { path: { employment_id: employmentId, business_date: businessDate } },
      body,
      headers: { "Idempotency-Key": idempotencyKey },
    },
  );
}

export function fetchScheduleHistory(params: {
  employment_id: string;
  start_date?: string;
  end_date?: string;
  cursor?: string;
  limit?: number;
}): Promise<{ data: ScheduleHistoryResponse }> {
  return client.GET<ScheduleHistoryResponse>(`${schedulePath}/history`, { params: { query: params } });
}

export function fetchScheduleEvidence(
  employmentId: string,
  businessDate: string,
): Promise<{ data: ScheduleEvidenceResponse }> {
  return client.GET<ScheduleEvidenceResponse>(
    `${schedulePath}/evidence/{employment_id}/{business_date}`,
    { params: { path: { employment_id: employmentId, business_date: businessDate } } },
  );
}
