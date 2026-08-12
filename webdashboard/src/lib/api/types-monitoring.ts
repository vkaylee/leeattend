export interface ReportExportJobProgress {
  completed_batches: number;
  failed_batches: number;
  percentage: number;
  total_batches: number;
}

import type { components } from "@/types/api-generated";

export type WorkerMonitorStats = components["schemas"]["MonitorOperationalSnapshot"];
export type JobTypeStats = components["schemas"]["JobTypeStats"];
export type MonitorJobsResponse = components["schemas"]["MonitorJobsResponse"];
export type RecentJob = components["schemas"]["MonitorJob"];
export type MonitorInfraJobState = components["schemas"]["MonitorInfraJobState"];
export type MonitorInfraJobEvent = components["schemas"]["MonitorInfraJobEvent"];
export type MonitorInfraJobRecord = components["schemas"]["MonitorInfraJobRecord"];
export type MonitorDurableJobsSection = components["schemas"]["MonitorDurableJobsSection"];
export type MonitorQuality = components["schemas"]["MonitorQuality"];
export type MonitorMetric = components["schemas"]["MonitorMetric"];
export type MonitorQueueHealth = components["schemas"]["MonitorQueueHealth"];
export type MonitorReliability = components["schemas"]["MonitorReliability"];
export type MonitorOperationalThresholds = components["schemas"]["MonitorOperationalThresholds"];
export type MonitorDependencyHealth = components["schemas"]["MonitorDependencyHealth"];
export type MonitorDependencyHealthSet = components["schemas"]["MonitorDependencyHealthSet"];


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
