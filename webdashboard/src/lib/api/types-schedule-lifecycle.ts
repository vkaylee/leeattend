import type { EmployeeSchedule, EffectiveSchedule } from "./types-employment";

export type ScheduleLifecycleState =
  | "draft"
  | "published"
  | "superseded"
  | "explicit_off"
  | "missing"
  | "closed"
  | "legacy_unverified"
  | "conflict";

export interface SchedulePreviewRequest {
  approval_instance_id?: string | null;
  employment_ids: string[];
  end_date: string;
  expected_revisions: Record<string, number>;
  mode: "office";
  reason: string;
  retroactive: boolean;
  start_date: string;
  timezone: string;
}

export interface SchedulePublishRequest extends Omit<SchedulePreviewRequest, "mode"> {
  preview_hash: string;
  preview_id: string;
}

export interface ScheduleReplanBreak {
  end: string;
  start: string;
}

export interface ScheduleReplanInterval {
  breaks: ScheduleReplanBreak[];
  end: string;
  sort_order: number;
  start: string;
}

export interface ScheduleReplanReplacement {
  intervals: ScheduleReplanInterval[];
  shift_version_id: string;
  source: string;
  status: "work";
}

export interface ScheduleReplanPreviewRequest {
  expected_revision: number;
  reason: string;
  replacement: ScheduleReplanReplacement;
}

export interface ScheduleReplanRequest extends ScheduleReplanPreviewRequest {
  confirm_diff_hash: string;
  preview_hash: string;
  preview_id: string;
}

export interface SchedulePreviewResponse {
  data: unknown[];
  expires_at: string;
  preview_hash: string;
  preview_id: string;
  provisional: boolean;
  resolver_version: string;
}

export interface SchedulePublicationResponse {
  affected_days: number;
  batch_id: string;
  published_at: string;
  revision_ids: string[];
  status: string;
}

export interface ScheduleReplanPreviewResponse {
  diff: unknown;
  diff_hash: string;
  expires_at: string;
  original_revision_id: string;
  preview_hash: string;
  preview_id: string;
  replacement: unknown;
}

export interface ScheduleReplanResponse {
  actual_time: string;
  batch_id: string;
  diff: unknown;
  original_revision_id: string;
  reason: string;
  replacement_revision_id: string;
  retroactive: boolean;
  revision_number: number;
}

export interface ScheduleHistoryCursor {
  business_date: string;
  employment_id: string;
  revision_id: string;
  revision_number: number;
}

export interface ScheduleHistoryEntry {
  business_date: string;
  created_at: string;
  employer_id: string;
  employment_id: string;
  previous_revision_id?: string | null;
  publication_batch_id?: string | null;
  published_actor_name?: string | null;
  published_actor_type: string;
  published_at?: string | null;
  published_by?: string | null;
  reason: string;
  resolver_version: string;
  revision_id: string;
  revision_number: number;
  source: string;
  source_snapshot: Record<string, unknown>;
  state: ScheduleLifecycleState;
  timezone: string;
}

export interface ScheduleHistoryResponse {
  data: ScheduleHistoryEntry[];
  next_cursor?: string | null;
}

export interface ScheduleEvidenceSegment {
  breaks_snapshot: unknown;
  business_date: string;
  employment_id: string;
  id: string;
  schedule_revision_id?: string | null;
  scheduled_end_at?: string | null;
  scheduled_start_at?: string | null;
  selected_shift_version_id?: string | null;
  sort_order: number;
}

export interface ScheduleEvidenceClosureDay {
  business_date: string;
  closure_id: string;
  employment_id: string;
  id: string;
  period_id: string;
  schedule_revision_id: string;
  schedule_state: ScheduleLifecycleState;
}

export interface ScheduleEvidenceAuditAction {
  action: string;
  actor_id?: string | null;
  actor_name?: string | null;
  actor_type?: string | null;
  created_at: string;
  id: string;
}

export interface ScheduleEvidenceEvent extends ScheduleEvidenceAuditAction {
  type: string;
}

export interface ScheduleEvidenceDiffLink {
  previous_revision_id?: string | null;
  revision_id: string;
}

export interface ScheduleEvidenceResponse {
  audit_actions: ScheduleEvidenceAuditAction[];
  business_date: string;
  closure_days: ScheduleEvidenceClosureDay[];
  diff_chain: ScheduleEvidenceDiffLink[];
  employment_id: string;
  events: ScheduleEvidenceEvent[];
  revisions: ScheduleHistoryEntry[];
  segments: ScheduleEvidenceSegment[];
}

export interface LifecycleSchedule extends EffectiveSchedule {
  closed?: boolean;
}

export interface LifecycleEmployeeSchedule extends Omit<EmployeeSchedule, "schedules"> {
  schedules: LifecycleSchedule[];
}
