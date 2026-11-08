export interface ReportExportJobProgress {
  completed_batches: number;
  failed_batches: number;
  percentage: number;
  total_batches: number;
}

export type MonitorInfraJobState =
  | "dispatching"
  | "queued"
  | "processing"
  | "completed"
  | "failed"
  | "retryable"
  | "dead_letter"
  | "lost";

export interface MonitorInfraJobEvent {
  attempt_count: number;
  created_at: string;
  error_code?: string | null;
  error_summary?: string | null;
  event_id: string;
  event_type: string;
  from_state?: MonitorInfraJobState | string | null;
  job_id: string;
  request_id?: string | null;
  to_state: MonitorInfraJobState | string;
  trace_id?: string | null;
  worker_id?: string | null;
}

export interface MonitorInfraJobRecord {
  attempt_count: number;
  created_at: string;
  error_code?: string | null;
  error_summary?: string | null;
  events: MonitorInfraJobEvent[];
  finished_at?: string | null;
  job_id: string;
  parent_job_id?: string | null;
  priority: string;
  queued_at?: string | null;
  request_id?: string | null;
  started_at?: string | null;
  state: MonitorInfraJobState | string;
  task_type: string;
  trace_id?: string | null;
  updated_at: string;
  worker?: string | null;
}

export interface MonitorDurableJobsSection {
  error_code?: string | null;
  events_truncated: boolean;
  jobs?: MonitorInfraJobRecord[] | null;
  limit?: number;
  next_cursor_token?: string | null;
  status: "available" | "degraded" | "unavailable" | string;
}

export interface MonitorJob {
  completed_at?: string | null;
  completed_batches: number;
  created_at: string;
  error?: string | null;
  error_summary?: string | null;
  failed_batches: number;
  job_id: string;
  report_type?: string | null;
  status: string;
  task_type?: string | null;
  total_batches: number;
}

export type RecentJob = MonitorJob;

export interface JobTypeStats {
  count: number;
  task_type: string;
}

export interface WorkerHeartbeat {
  current_task_type?: string | null;
  heartbeat_age_seconds?: number;
  host: string;
  id: string;
  last_seen_at: string;
  liveness?: "live" | "stale" | "offline" | string;
  status: "IDLE" | "PROCESSING" | string;
}

export interface MonitorMetricWindow {
  duration_seconds: number;
  end: string;
  start: string;
}

export interface MonitorMetricUnknown {
  code: string;
  reason: string;
}

export interface MonitorMetric {
  formula: string;
  observed_at: string;
  source: string;
  time_window: MonitorMetricWindow;
  unit: string;
  unknown?: MonitorMetricUnknown | null;
  value?: number | null;
}

export interface MonitorQueueHealth {
  dead_letter_depth?: MonitorMetric;
  depth_high: MonitorMetric;
  depth_low: MonitorMetric;
  depth_normal: MonitorMetric;
  depth_total: MonitorMetric;
  enqueue_rate_per_second?: MonitorMetric;
  estimated_drain_seconds?: MonitorMetric;
  growth_rate_per_second?: MonitorMetric;
  oldest_queued_age_seconds?: MonitorMetric;
  processing_rate_per_second?: MonitorMetric;
  queue_wait_seconds_p50?: MonitorMetric;
  queue_wait_seconds_p95?: MonitorMetric;
}

export interface MonitorReliability {
  consecutive_failures: MonitorMetric;
  dead_letter_count?: MonitorMetric;
  execution_seconds_p50?: MonitorMetric;
  execution_seconds_p95?: MonitorMetric;
  execution_seconds_p99?: MonitorMetric;
  failure_rate: MonitorMetric;
  lost_count?: MonitorMetric;
  retryable_count?: MonitorMetric;
  success_rate: MonitorMetric;
  timeout_count?: MonitorMetric;
}

export interface MonitorQueueThresholds {
  critical_at: number;
  high_at: number;
  medium_at: number;
  source: string;
}

export interface MonitorUtilizationThresholds {
  critical_at_percent: number;
  scale_down_below_percent: number;
  source: string;
  warning_at_percent: number;
}

export interface MonitorOperationalThresholds {
  live_after_seconds: number;
  queue_pressure: MonitorQueueThresholds;
  source: string;
  stale_after_seconds: number;
  worker_utilization: MonitorUtilizationThresholds;
}

export interface MonitorDependencyHealth {
  availability?: boolean | null;
  latency_ms?: number | null;
  limitation?: string | null;
  observed_at: string;
  pool_utilization_percent?: number | null;
  source: string;
  status: "available" | "unavailable" | "degraded" | string;
}

export interface MonitorDependencyHealthSet {
  cpu: MonitorDependencyHealth;
  database_pool: MonitorDependencyHealth;
  disk: MonitorDependencyHealth;
  memory: MonitorDependencyHealth;
  outbox_sweeper: MonitorDependencyHealth;
  postgres: MonitorDependencyHealth;
  redis: MonitorDependencyHealth;
  worker_process: MonitorDependencyHealth;
}

export interface MonitorSourceHealth {
  error_code?: string | null;
  status: "available" | "degraded" | "unavailable" | string;
}

export interface MonitorQuality {
  data_quality: "complete" | "partial" | "degraded" | "unavailable" | string;
  durable_db: MonitorSourceHealth;
  freshness: "fresh" | "aging" | "stale" | "unknown" | string;
  redis: MonitorSourceHealth;
  snapshot_sequence: string;
  unknown_fields: string[];
  worker_heartbeats: MonitorSourceHealth;
}

export interface MonitorOldestJob {
  age_seconds?: number;
  job_id: string;
  queued_at?: string | null;
  started_at?: string | null;
  task_type: string;
}

export interface MonitorTaskTypeReliability {
  completed: number;
  dead_letter?: number;
  failed: number;
  lost?: number;
  processing: number;
  queued: number;
  retryable?: number;
  success_rate?: MonitorMetric;
  task_type: string;
}

export interface WorkerMonitorStats {
  active_workers?: number | null;
  completed_today?: number | null;
  dependencies: MonitorDependencyHealthSet;
  durable_jobs: MonitorDurableJobsSection;
  failed_today?: number | null;
  idle_workers?: number | null;
  job_type_stats: JobTypeStats[];
  observed_at: string;
  oldest_queued_job?: MonitorOldestJob | null;
  oldest_running_job?: MonitorOldestJob | null;
  pending_high?: number | null;
  pending_low?: number | null;
  pending_normal?: number | null;
  pending_tasks?: number | null;
  quality: MonitorQuality;
  queue_health: MonitorQueueHealth;
  recent_jobs: RecentJob[];
  reliability?: MonitorReliability | null;
  snapshot_age_seconds: number;
  task_type_reliability?: MonitorTaskTypeReliability[];
  thresholds: MonitorOperationalThresholds;
  total_workers?: number | null;
  workers: WorkerHeartbeat[];
}

export interface MonitorJobsResponse {
  durable_jobs: MonitorDurableJobsSection;
  jobs: RecentJob[];
  next_cursor?: number | null;
  quality: MonitorQuality;
}

export interface ReportExportJob {
  completed_at?: string;
  created_at: string;
  error?: string;
  job_id: string;
  progress: ReportExportJobProgress;
  result_url?: string;
  status: "pending" | "processing" | "completed" | "failed";
}

export interface CreateReportExportRequest {
  employee_ids?: string[];
  end_date: string;
  report_type: "summary" | "detail";
  start_date: string;
}

export interface CreateReportExportResponse {
  estimated_seconds: number;
  job_id: string;
  status: string;
  status_url: string;
  total_batches: number;
}

export interface ProcessResponse {
  logs_deleted: number;
  logs_processed: number;
}
