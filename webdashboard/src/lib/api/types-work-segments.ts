// ── Work Segments ──────────────────────────────────────────────────────────

/** Metadata written by the generation pipeline (auto-scheduling/recalculation run). */
export interface WorkSegmentGenerationMetadata {
  generated_at?: string;
  generator?: 'auto_scheduler' | 'recalculation' | 'roster_sync' | 'pattern_match';
  rule_version?: string;
  source_run_id?: string;
}

/** Structured detail about the source that produced this segment. */
export interface WorkSegmentSourceDetail {
  adjusted_by?: string;
  adjustment_reason?: string;
  pattern_id?: string;
  roster_id?: string;
  rule_id?: string;
  template_id?: string;
}

/** Override config for break behavior on a specific day/segment. */
export interface BreakConfigOverride {
  break_duration_minutes?: number;
  break_end?: string;
  break_is_overtime?: boolean;
  break_start?: string;
  enabled: boolean;
}

/** Override config for overtime behavior on a specific day/segment. */
export interface OtConfigOverride {
  enabled: boolean;
  max_minutes?: number;
  multiplier?: number;
  threshold_hours?: number;
}

export interface WorkSegment {
  break_config_override?: BreakConfigOverride | null;
  compensation_rate: number;
  created_at: string;
  date: string;
  employment_id: string;
  gap_compensation_before: 'unpaid' | 'standby' | 'break';
  gap_compensation_rate_before: number;
  generation_metadata: WorkSegmentGenerationMetadata;
  id: string;
  interval_id?: string | null;
  notes?: string | null;
  ot_config_override?: OtConfigOverride | null;
  scheduled_end: string;
  scheduled_start: string;
  segment_status: 'scheduled' | 'in_progress' | 'completed' | 'cancelled';
  segment_type: 'work' | 'standby' | 'on_call' | 'training' | 'travel';
  sort_order: number;
  source: 'roster' | 'pattern' | 'rule' | 'fixed' | 'manual' | 'auto';
  source_detail: WorkSegmentSourceDetail;
  template_id: string;
  updated_at: string;
  workspace_id: string;
}

export interface CreateWorkSegmentRequest {
  break_config_override?: BreakConfigOverride | null;
  compensation_rate?: number;
  date: string;
  employment_id: string;
  gap_compensation_before?: string;
  gap_compensation_rate_before?: number;
  interval_id?: string | null;
  notes?: string | null;
  ot_config_override?: OtConfigOverride | null;
  scheduled_end: string;
  scheduled_start: string;
  segment_status?: string;
  segment_type?: string;
  sort_order?: number;
  source?: string;
  source_detail?: WorkSegmentSourceDetail;
  template_id: string;
}

export interface UpdateWorkSegmentRequest {
  break_config_override?: BreakConfigOverride | null;
  compensation_rate?: number | null;
  gap_compensation_before?: string | null;
  gap_compensation_rate_before?: number | null;
  interval_id?: string | null;
  notes?: string | null;
  ot_config_override?: OtConfigOverride | null;
  scheduled_end?: string | null;
  scheduled_start?: string | null;
  segment_status?: string | null;
  segment_type?: string | null;
  sort_order?: number | null;
  source?: string | null;
  source_detail?: WorkSegmentSourceDetail | null;
  template_id?: string | null;
}

export interface BulkCreateWorkSegmentsRequest {
  replace_existing?: boolean;
  segments: CreateWorkSegmentRequest[];
}

// ── Shift Default Intervals ─────────────────────────────────────────────────

export interface ShiftDefaultInterval {
  allow_early_ot: boolean;
  break_duration_minutes: number;
  break_end?: string | null;
  break_is_overtime: boolean;
  break_start?: string | null;
  created_at: string;
  early_punch_in_grace_period_minutes: number;
  end_time: string;
  id: string;
  interval_name?: string | null;
  is_break_auto_deduct: boolean;
  is_night_shift: boolean;
  max_early_ot_minutes?: number | null;
  ot_aggregation_mode: string;
  shift_id: string;
  sort_order: number;
  start_time: string;
  updated_at: string;
}

export interface CreateShiftIntervalRequest {
  allow_early_ot?: boolean;
  break_duration_minutes?: number;
  break_end?: string | null;
  break_is_overtime?: boolean;
  break_start?: string | null;
  early_punch_in_grace_period_minutes?: number;
  end_time: string;
  interval_name?: string | null;
  is_break_auto_deduct?: boolean;
  is_night_shift?: boolean;
  max_early_ot_minutes?: number | null;
  ot_aggregation_mode?: string;
  sort_order?: number;
  start_time: string;
}

export interface UpdateShiftIntervalRequest {
  allow_early_ot?: boolean;
  break_duration_minutes?: number;
  break_end?: string | null;
  break_is_overtime?: boolean;
  break_start?: string | null;
  early_punch_in_grace_period_minutes?: number;
  end_time?: string | null;
  interval_name?: string | null;
  is_break_auto_deduct?: boolean;
  is_night_shift?: boolean;
  max_early_ot_minutes?: number | null;
  ot_aggregation_mode?: string;
  sort_order?: number | null;
  start_time?: string | null;
}
