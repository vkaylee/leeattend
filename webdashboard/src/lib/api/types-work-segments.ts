// ── Work Segments ──────────────────────────────────────────────────────────

export interface WorkSegment {
  break_config_override?: Record<string, unknown> | null;
  compensation_rate: number;
  created_at: string;
  date: string;
  employment_id: string;
  gap_compensation_before: 'unpaid' | 'standby' | 'break';
  gap_compensation_rate_before: number;
  generation_metadata: Record<string, unknown>;
  id: string;
  interval_id?: string | null;
  notes?: string | null;
  ot_config_override?: Record<string, unknown> | null;
  scheduled_end: string;
  scheduled_start: string;
  segment_status: 'scheduled' | 'in_progress' | 'completed' | 'cancelled';
  segment_type: 'work' | 'standby' | 'on_call' | 'training' | 'travel';
  shift_id: string;
  sort_order: number;
  source: 'roster' | 'pattern' | 'rule' | 'fixed' | 'manual' | 'auto';
  source_detail: Record<string, unknown>;
  updated_at: string;
  workspace_id: string;
}

export interface CreateWorkSegmentRequest {
  break_config_override?: Record<string, unknown> | null;
  compensation_rate?: number;
  date: string;
  employment_id: string;
  gap_compensation_before?: string;
  gap_compensation_rate_before?: number;
  interval_id?: string | null;
  notes?: string | null;
  ot_config_override?: Record<string, unknown> | null;
  scheduled_end: string;
  scheduled_start: string;
  segment_status?: string;
  segment_type?: string;
  shift_id: string;
  sort_order?: number;
  source?: string;
  source_detail?: Record<string, unknown>;
}

export interface UpdateWorkSegmentRequest {
  break_config_override?: Record<string, unknown> | null;
  compensation_rate?: number | null;
  gap_compensation_before?: string | null;
  gap_compensation_rate_before?: number | null;
  interval_id?: string | null;
  notes?: string | null;
  ot_config_override?: Record<string, unknown> | null;
  scheduled_end?: string | null;
  scheduled_start?: string | null;
  segment_status?: string | null;
  segment_type?: string | null;
  shift_id?: string | null;
  sort_order?: number | null;
  source?: string | null;
  source_detail?: Record<string, unknown> | null;
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
