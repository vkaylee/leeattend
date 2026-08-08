export interface ReportExportJobProgress {
  completed_batches: number;
  failed_batches: number;
  percentage: number;
  total_batches: number;
}

export interface WorkerMonitorStats {
  active_workers: number;
  completed_today: number;
  failed_today: number;
  idle_workers: number;
  job_type_stats: JobTypeStats[];
  pending_high: number;
  pending_low: number;
  pending_normal?: number;
  pending_tasks: number;
  recent_jobs: RecentJob[];
  total_workers: number;
  workers: import("@/lib/api/types").WorkerHeartbeat[];
}

export interface JobTypeStats {
  completed: number;
  failed: number;
  pending: number;
  processing: number;
  task_type: string;
  total: number;
}

export interface MonitorJobsResponse {
  jobs: RecentJob[];
  next_cursor?: number | null;
}

export interface RecentJob {
  completed_at?: string | null;
  completed_batches: number;
  created_at: string;
  error_summary?: string | null;
  failed_batches: number;
  job_id: string;
  report_type?: string | null;
  status: string;
  task_type?: string | null;
  total_batches: number;
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
