/* eslint-disable complexity, max-lines, @typescript-eslint/no-unnecessary-condition, @typescript-eslint/no-confusing-void-expression, perfectionist/sort-jsx-props, perfectionist/sort-imports, import-x/no-named-as-default-member -- Monitor page composes the operational dashboard and alert controls. */
import {
  AlertTriangle,
  CheckCircle2,
  CircleHelp,
  Loader2,
} from "lucide-react";
import React, { useMemo } from "react";
import { useTranslation } from "react-i18next";

import { TFunction } from "i18next";

import type { MonitorStreamStatus } from "@/hooks/use-monitor-stream";
import type { WorkerMonitorStats } from "@/lib/api/types-monitoring";

import JobStatusDistributionChart from "@/components/features/monitor/job-status-distribution-chart";
import QueuePressureChart from "@/components/features/monitor/queue-pressure-chart";
import WorkerUtilizationGauge from "@/components/features/monitor/worker-utilization-gauge";
import { Typography } from "@/components/ui/typography";
import Button from "@/components/ui/button";
import { useWorkerUtilization, useQueuePressure, useJobStatusDistribution } from "@/hooks/use-monitor-charts";
import { useMonitorStream } from "@/hooks/use-monitor-stream";
import {
  MONITOR_REFETCH_INTERVAL_S,
  useMonitorAlert,
  useMonitorAlertMutations,
  useMonitorAlerts,
  useWorkerMonitor,
} from "@/hooks/use-worker-monitor";
import { cn } from "@/lib/utils";

import { DependencyHealthSection } from "./monitor/dependency-health-section";
import { MonitorHeader } from "./monitor/header";
import { JobsTable } from "./monitor/jobs-table";
import { MonitorStatCards } from "./monitor/stat-cards";
import { WorkersTable } from "./monitor/workers-table";

function localizeMonitorValue(t: TFunction, namespace: string, value: string): string {
  const normalized = value.toLowerCase().replace(/[^a-z0-9]+/g, "_");
  const knownDefaults: Record<string, string> = {
    "alert_event.alert_created": "Alert created",
    "alert_event.alert_updated": "Alert updated",
    "alert_event.alert_acknowledged": "Alert acknowledged",
    "alert_event.alert_resolved": "Alert resolved",
    "alert_severity.critical": "Critical",
    "alert_severity.high": "High",
    "alert_severity.medium": "Medium",
    "alert_severity.low": "Low",
    "alert_source.redis": "Redis",
    "alert_source.postgres": "PostgreSQL",
    "alert_status.active": "Active",
    "alert_status.acknowledged": "Acknowledged",
    "alert_status.resolved": "Resolved",
  };
  const key = `${namespace}.${normalized}`;
  return t(`monitor.${key}`, {
    defaultValue: knownDefaults[key] ?? t("monitor.unknown_status", { defaultValue: "Status unavailable" }),
  });
}

function MonitorAlertsSection({ t }: { t: TFunction }) {
  const [selectedAlertId, setSelectedAlertId] = React.useState<string | null>(null);
  const [reason, setReason] = React.useState("");
  const alertsQuery = useMonitorAlerts({ status: "active", limit: 20 });
  const detailQuery = useMonitorAlert(selectedAlertId);
  const { acknowledge, resolve } = useMonitorAlertMutations();
  const selectedAlert = detailQuery.data;
  const isMutating = acknowledge.isPending || resolve.isPending;
  const reasonIsValid = reason.trim().length > 0 && reason.trim().length <= 500;
  const submit = (action: "acknowledge" | "resolve") => {
    if (!selectedAlertId || !reasonIsValid || isMutating) return;
    const variables = { alertId: selectedAlertId, reason: reason.trim() };
    if (action === "resolve") resolve.mutate(variables, { onSuccess: () => setReason("") });
    else acknowledge.mutate(variables, { onSuccess: () => setReason("") });
  };

  return (
    <section className="rounded-2xl border border-amber-200 bg-white p-5 shadow-sm dark:border-amber-900/70 dark:bg-slate-900" aria-label={t("monitor.alerts", { defaultValue: "Monitor alerts" })}>
      <div className="flex items-center justify-between gap-4">
        <div>
          <Typography className="text-[10px] font-bold uppercase tracking-widest text-amber-600 dark:text-amber-300" variant="overline">{t("monitor.alerts", { defaultValue: "Monitor alerts" })}</Typography>
          <Typography className="mt-1 text-sm text-slate-500 dark:text-slate-400" variant="p">{t("monitor.alerts_description", { defaultValue: "Review active platform alerts and record operator actions." })}</Typography>
        </div>
        <AlertTriangle aria-hidden="true" className="text-amber-500" size={22} />
      </div>
      {alertsQuery.isLoading ? <div aria-busy="true" className="mt-5 grid gap-3 md:grid-cols-2"><div className="h-28 motion-safe:animate-pulse rounded-xl bg-slate-100 dark:bg-slate-800" /><div className="h-28 motion-safe:animate-pulse rounded-xl bg-slate-100 dark:bg-slate-800" /></div> : null}
      {alertsQuery.isError ? <div className="mt-5 flex items-center justify-between gap-3 text-sm text-rose-600" role="alert"><span>{t("monitor.alerts_error", { defaultValue: "Unable to load alerts." })}</span><Button onClick={() => void alertsQuery.refetch()} size="sm" variant="outline">{t("common.retry", { defaultValue: "Retry" })}</Button></div> : null}
      {!alertsQuery.isLoading && !alertsQuery.isError && alertsQuery.data?.data.length === 0 ? <div className="mt-5 flex items-center gap-2 text-sm text-emerald-600"><CheckCircle2 size={16} />{t("monitor.no_active_alerts", { defaultValue: "No active alerts." })}</div> : null}
      <div className="mt-5 grid gap-3 md:grid-cols-2">
        {alertsQuery.data?.data.map((alert) => (
          <Button
            aria-pressed={selectedAlertId === alert.id}
            className={cn(
              "h-auto min-h-28 w-full min-w-0 flex-col items-stretch justify-start rounded-xl border p-4 text-left normal-case tracking-normal motion-safe:transition-colors active:scale-100",
              selectedAlertId === alert.id
                ? "border-primary-500 bg-primary-50/60 dark:bg-primary-950/30"
                : "border-slate-200 bg-transparent text-inherit shadow-none hover:border-primary-300 hover:bg-transparent dark:border-slate-700",
            )}
            key={alert.id}
            onClick={() => setSelectedAlertId(alert.id)}
            variant="ghost"
          >
            <div className="flex min-w-0 items-center justify-between gap-3"><span className="min-w-0 break-words font-bold text-slate-800 dark:text-slate-100">{alert.title}</span><span className="shrink-0 text-[10px] font-bold uppercase tracking-widest text-amber-600">{localizeMonitorValue(t, "alert_severity", alert.severity)}</span></div>
            <div className="mt-2 line-clamp-2 text-sm text-slate-500 dark:text-slate-400">{alert.summary}</div>
          </Button>
        ))}
      </div>
      {selectedAlertId ? (
        <div className="mt-5 rounded-xl border border-slate-200 p-4 dark:border-slate-700">
          {detailQuery.isLoading ? <div aria-busy="true" className="h-24 motion-safe:animate-pulse rounded-lg bg-slate-100 dark:bg-slate-800" /> : null}
          {detailQuery.isError ? <div className="flex items-center justify-between gap-3 text-sm text-rose-600" role="alert"><span>{t("monitor.alert_detail_error", { defaultValue: "Unable to load alert details." })}</span><Button onClick={() => void detailQuery.refetch()} size="sm" variant="outline">{t("common.retry", { defaultValue: "Retry" })}</Button></div> : null}
          {selectedAlert ? <>
            <div className="text-sm text-slate-500 dark:text-slate-400">{localizeMonitorValue(t, "alert_source", selectedAlert.source)} · {localizeMonitorValue(t, "alert_status", selectedAlert.status)}</div>
            {selectedAlert.events.length > 0 ? <div className="mt-3 space-y-2" aria-label={t("monitor.alert_events", { defaultValue: "Alert events" })}>{selectedAlert.events.map((event) => <div className="rounded-lg bg-slate-50 p-2 text-xs dark:bg-slate-800" key={event.id}><div className="font-semibold">{localizeMonitorValue(t, "alert_event", event.event_type)} · {localizeMonitorValue(t, "alert_severity", event.severity)}</div><div className="text-slate-500 dark:text-slate-400">{event.summary}</div></div>)}</div> : null}
            <textarea aria-label={t("monitor.alert_reason", { defaultValue: "Operator reason" })} className="mt-3 min-h-20 w-full rounded-xl border border-slate-200 bg-transparent p-3 text-sm outline-none focus:border-primary-500 focus-visible:ring-2 focus-visible:ring-primary-500 focus-visible:ring-offset-2 dark:border-slate-700" maxLength={500} onChange={(event) => setReason(event.target.value)} placeholder={t("monitor.alert_reason_placeholder", { defaultValue: "Describe the action taken…" })} value={reason} />
            <div className="mt-3 flex flex-wrap justify-end gap-2">
              <Button disabled={!reasonIsValid || isMutating} loading={acknowledge.isPending} onClick={() => submit("acknowledge")} size="sm" variant="outline">{t("monitor.acknowledge", { defaultValue: "Acknowledge" })}</Button>
              <Button disabled={!reasonIsValid || isMutating} loading={resolve.isPending} onClick={() => submit("resolve")} size="sm" variant="destructive">{t("monitor.resolve", { defaultValue: "Resolve" })}</Button>
            </div>
            {acknowledge.isError || resolve.isError ? <div className="mt-2 text-right text-xs text-rose-600" role="alert">{t("monitor.alert_action_error", { defaultValue: "Action failed. Please retry." })}</div> : null}
          </> : null}
        </div>
      ) : null}
    </section>
  );
}

function formatSnapshotObservedAt(observedAt: string | undefined): string {
  if (!observedAt || !Number.isFinite(Date.parse(observedAt))) return "—";
  return new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "medium",
  }).format(new Date(observedAt));
}

function streamStatusLabel(status: MonitorStreamStatus, t: TFunction): string {
  switch (status) {
    case "live":
      return t("monitor.live_updates", { seconds: MONITOR_REFETCH_INTERVAL_S });
    case "stale":
      return t("monitor.stale_updates", { defaultValue: "Live stream is stale; polling fallback is active" });
    case "error":
      return t("monitor.stream_error", { defaultValue: "Live stream unavailable; polling fallback is active" });
    case "degraded":
      return t("monitor.degraded_updates", { defaultValue: "Degraded monitoring data; polling fallback is active" });
    default:
      return t("monitor.connecting_updates", { defaultValue: "Connecting to live monitoring…" });
  }
}

interface SnapshotMetadataProps {
  stats: WorkerMonitorStats;
  t: TFunction;
}

function SnapshotMetadata({ stats, t }: SnapshotMetadataProps) {
  const quality = stats.quality;
  if (!quality) return null; // API fallback always supplies quality; retain guard for partial snapshots.
  const snapshotIsAvailable = quality.data_quality !== "unavailable";
  const snapshotAgeSeconds = snapshotIsAvailable ? stats.snapshot_age_seconds : null;
  const sourceQuality = [
    [t("monitor.source_redis", { defaultValue: "Redis" }), quality.redis.status],
    [t("monitor.source_workers", { defaultValue: "Worker heartbeats" }), quality.worker_heartbeats.status],
    [t("monitor.source_durable", { defaultValue: "Durable jobs" }), quality.durable_db.status],
  ] as const;

  return (
    <section
      aria-label={t("monitor.snapshot_metadata", { defaultValue: "Snapshot metadata" })}
      className="grid grid-cols-1 gap-3 rounded-2xl border border-slate-200/70 bg-white/70 px-5 py-4 text-xs dark:border-slate-700/70 dark:bg-slate-900/70 sm:grid-cols-3"
    >
      <div>
        <div className="font-bold uppercase tracking-wider text-slate-400 dark:text-slate-500">
          {t("monitor.snapshot_age", { defaultValue: "Snapshot age" })}
        </div>
        <div className="mt-1 font-semibold text-slate-700 dark:text-slate-200">
          {snapshotAgeSeconds === null
            ? "—"
            : t("monitor.seconds_value", { defaultValue: `${snapshotAgeSeconds}s`, seconds: snapshotAgeSeconds })}
        </div>
      </div>
      <div>
        <div className="font-bold uppercase tracking-wider text-slate-400 dark:text-slate-500">
          {t("monitor.observed_at", { defaultValue: "Observed at" })}
        </div>
        <div className="mt-1 font-semibold text-slate-700 dark:text-slate-200">
          {snapshotIsAvailable ? formatSnapshotObservedAt(stats.observed_at) : "—"}
        </div>
      </div>
      <div>
        <div className="font-bold uppercase tracking-wider text-slate-400 dark:text-slate-500">
          {t("monitor.source_quality", { defaultValue: "Source quality" })}
        </div>
        <div className="mt-1 flex flex-wrap gap-x-3 gap-y-1 font-semibold text-slate-700 dark:text-slate-200">
          {sourceQuality.map(([label, status]) => (
            <span key={label}>
              {label}: {t(`monitor.source_status.${status}`, { defaultValue: status })}
            </span>
          ))}
        </div>
      </div>
    </section>
  );
}

interface StreamStatusIndicatorProps {
  isFetching: boolean;
  status: MonitorStreamStatus;
  t: TFunction;
}

function StreamStatusIndicator({ isFetching, status, t }: StreamStatusIndicatorProps) {
  const isLive = status === "live";
  return (
    <div aria-live="polite" className="flex items-center gap-3 px-2" role="status">
      <div aria-hidden="true" className="relative flex h-3 w-3">
        <span aria-hidden="true" className={cn("absolute inline-flex h-full w-full rounded-full opacity-75", isLive ? "motion-safe:animate-ping bg-emerald-400" : "bg-amber-400")} />
        <span aria-hidden="true" className={cn("relative inline-flex rounded-full h-3 w-3", isLive ? "bg-emerald-500" : "bg-amber-500")} />
      </div>
      <span className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
        {streamStatusLabel(status, t)}
      </span>
      {isFetching ? <Loader2 aria-hidden="true" className="text-slate-400 dark:text-slate-500 motion-safe:animate-spin" size={14} /> : null}
    </div>
  );
}

interface JobTypeStatsSectionProps {
  jobTypeStats: WorkerMonitorStats["job_type_stats"];
  t: TFunction;
}

function JobTypeStatsSection({ jobTypeStats, t }: JobTypeStatsSectionProps) {
  if (jobTypeStats.length === 0) return null;
  return (
    <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-2xl sm:rounded-[2.5rem] p-5 sm:p-6 shadow-sm dark:shadow-black/10">
      <Typography className="text-lg font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight mb-4" variant="h4">
        {t("monitor.jobs_by_type", { defaultValue: "Breakdown by Type" })}
      </Typography>
      <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 xl:grid-cols-4 gap-4">
        {jobTypeStats.map((item) => {
          const displayType = t(`monitor.task_types.${item.task_type}`, { defaultValue: item.task_type });
          return (
            <div
              className="bg-slate-50 dark:bg-slate-800/40 border border-slate-100/50 dark:border-slate-800/50 rounded-3xl p-5 flex flex-col justify-between hover:shadow-md motion-safe:transition-all duration-300"
              key={item.task_type}
            >
              <div>
                <Typography className="text-xs font-extrabold text-slate-400 dark:text-slate-500 uppercase tracking-widest" variant="p">
                  {displayType}
                </Typography>
                <div className="flex items-baseline gap-2 mt-2">
                  <Typography className="text-3xl font-extrabold text-slate-700 dark:text-slate-200" variant="h3">
                    {item.total}
                  </Typography>
                  <Typography className="text-xs text-slate-400" variant="p">
                    {t("monitor.total_pending", { defaultValue: "total runs" })}
                  </Typography>
                </div>
              </div>
              <div className="flex gap-3 mt-4 text-[11px] font-bold uppercase tracking-wider">
                {item.completed > 0 ? <span className="text-emerald-600 dark:text-emerald-400 flex items-center gap-1"><span className="h-1.5 w-1.5 rounded-full bg-emerald-500" />{item.completed}</span> : null}
                {item.failed > 0 ? <span className="text-rose-600 dark:text-rose-400 flex items-center gap-1"><span className="h-1.5 w-1.5 rounded-full bg-rose-500 motion-safe:animate-pulse" />{item.failed}</span> : null}
                {item.processing > 0 ? <span className="text-amber-600 dark:text-amber-400 flex items-center gap-1"><span className="h-1.5 w-1.5 rounded-full bg-amber-500 motion-safe:animate-ping" />{item.processing}</span> : null}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}

interface MonitorMetricCardProps {
  label: string;
  metric: { value?: number | null; unit: string; formula: string; source: string; observed_at: string; time_window: { start: string; end: string; duration_seconds: number }; unknown?: { reason: string } | null };
  t: TFunction;
}

function MonitorMetricCard({ label, metric, t }: MonitorMetricCardProps) {
  const value = metric.value === null || metric.value === undefined
    ? t("monitor.metric_unavailable", { defaultValue: "Unavailable" })
    : `${metric.value} ${metric.unit}`;
  return (
    <div className="rounded-2xl border border-slate-100 bg-slate-50/80 p-4 dark:border-slate-800 dark:bg-slate-800/40">
      <div className="text-[10px] font-bold uppercase tracking-widest text-slate-400 dark:text-slate-500">{label}</div>
      <div className="mt-2 text-xl font-black tabular-nums text-slate-700 dark:text-slate-200">{value}</div>
      {metric.unknown ? <div className="mt-1 text-xs text-amber-700 dark:text-amber-300">{metric.unknown.reason}</div> : null}
      <details className="mt-2 text-[10px] text-slate-400 dark:text-slate-500">
        <summary className="cursor-pointer">{t("monitor.metric_details", { defaultValue: "Metric details" })}</summary>
        <div className="mt-1 space-y-0.5">
          <div>{metric.formula}</div>
          <div>{metric.source}</div>
          <div>{metric.time_window.duration_seconds}s window · {formatSnapshotObservedAt(metric.observed_at)}</div>
        </div>
      </details>
    </div>
  );
}

function OperationalMetricsSections({ stats, t }: { stats: WorkerMonitorStats; t: TFunction }) {
  const queue = stats.queue_health;
  const reliability = stats.reliability;
  const queueMetrics = [
    [t("monitor.queue_depth_total", { defaultValue: "Queue depth" }), queue.depth_total],
    [t("monitor.oldest_queued_age", { defaultValue: "Oldest queued age" }), queue.oldest_queued_age_seconds],
    [t("monitor.enqueue_rate", { defaultValue: "Enqueue rate" }), queue.enqueue_rate_per_second],
    [t("monitor.processing_rate", { defaultValue: "Processing rate" }), queue.processing_rate_per_second],
    [t("monitor.queue_growth_rate", { defaultValue: "Queue growth" }), queue.growth_rate_per_second],
    [t("monitor.estimated_drain", { defaultValue: "Estimated drain" }), queue.estimated_drain_seconds],
    [t("monitor.queue_wait_p50", { defaultValue: "Queue wait p50" }), queue.queue_wait_seconds_p50],
    [t("monitor.dead_letter_depth", { defaultValue: "Dead-letter depth" }), queue.dead_letter_depth],
  ] as const;
  const reliabilityMetrics = [
    [t("monitor.success_rate", { defaultValue: "Success rate" }), reliability.success_rate],
    [t("monitor.failure_rate", { defaultValue: "Failure rate" }), reliability.failure_rate],
    [t("monitor.consecutive_failures", { defaultValue: "Consecutive failures" }), reliability.consecutive_failures],
    [t("monitor.retryable_count", { defaultValue: "Retryable" }), reliability.retryable_count],
    [t("monitor.dead_letter_count", { defaultValue: "Dead letter" }), reliability.dead_letter_count],
    [t("monitor.lost_count", { defaultValue: "Lost" }), reliability.lost_count],
    [t("monitor.timeout_count", { defaultValue: "Timeouts" }), reliability.timeout_count],
    [t("monitor.execution_p95", { defaultValue: "Execution p95" }), reliability.execution_seconds_p95],
  ] as const;
  const oldestJobs: {
    label: string;
    job: WorkerMonitorStats["oldest_queued_job"];
  }[] = [
    {
      label: t("monitor.oldest_queued_job", { defaultValue: "Oldest queued job" }),
      job: stats.oldest_queued_job,
    },
    {
      label: t("monitor.oldest_running_job", { defaultValue: "Oldest running job" }),
      job: stats.oldest_running_job,
    },
  ];
  return (
    <div className="space-y-6">
      <div className="grid grid-cols-1 gap-6 xl:grid-cols-2">
      <section aria-label={t("monitor.queue_health", { defaultValue: "Queue health" })} className="rounded-2xl border border-slate-100 bg-white p-5 shadow-sm dark:border-slate-800 dark:bg-slate-900 sm:p-6">
        <Typography className="mb-4 text-lg font-bold uppercase tracking-tight text-slate-700 dark:text-slate-200" variant="h4">{t("monitor.queue_health", { defaultValue: "Queue health" })}</Typography>
        <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
          {queueMetrics.map(([label, metric]) => <MonitorMetricCard key={label} label={label} metric={metric} t={t} />)}
        </div>
      </section>
      <section aria-label={t("monitor.job_reliability", { defaultValue: "Job reliability" })} className="rounded-2xl border border-slate-100 bg-white p-5 shadow-sm dark:border-slate-800 dark:bg-slate-900 sm:p-6">
        <Typography className="mb-4 text-lg font-bold uppercase tracking-tight text-slate-700 dark:text-slate-200" variant="h4">{t("monitor.job_reliability", { defaultValue: "Job reliability" })}</Typography>
        <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
          {reliabilityMetrics.map(([label, metric]) => <MonitorMetricCard key={label} label={label} metric={metric} t={t} />)}
        </div>
      </section>
      </div>
      <div className="grid grid-cols-1 gap-6 xl:grid-cols-3">
        {oldestJobs.map(({ label, job }) => (
          <div className="rounded-2xl border border-slate-100 bg-white p-5 shadow-sm dark:border-slate-800 dark:bg-slate-900" key={label}>
            <Typography className="text-[10px] font-bold uppercase tracking-widest text-slate-400" variant="h5">{label}</Typography>
            {job ? <div className="mt-3 space-y-1 text-sm text-slate-600 dark:text-slate-300"><div className="font-bold">{job.task_type}</div><div>{job.age_seconds}s · {job.job_id}</div></div> : <div className="mt-3 text-sm text-slate-400">{t("monitor.metric_unavailable", { defaultValue: "Unavailable" })}</div>}
          </div>
        ))}
        <div className="rounded-2xl border border-slate-100 bg-white p-5 shadow-sm dark:border-slate-800 dark:bg-slate-900">
          <Typography className="text-[10px] font-bold uppercase tracking-widest text-slate-400" variant="h5">{t("monitor.task_type_reliability", { defaultValue: "Reliability by task type" })}</Typography>
          <div className="mt-3 space-y-2 text-sm text-slate-600 dark:text-slate-300">
            {stats.task_type_reliability.length === 0 ? <span className="text-slate-400">{t("monitor.metric_unavailable", { defaultValue: "Unavailable" })}</span> : stats.task_type_reliability.slice(0, 5).map((item) => <div className="flex justify-between gap-3" key={item.task_type}><span>{item.task_type}</span><span>{item.success_rate.value === null || item.success_rate.value === undefined ? "—" : `${Math.round(item.success_rate.value * 100)}%`}</span></div>)}
          </div>
        </div>
      </div>
    </div>
  );
}

export default function WorkerMonitorPage() {
  const { t } = useTranslation();
  const stream = useMonitorStream();
  const { stats, workers, recentJobs, isLoading, isFetching, error, refetch } = useWorkerMonitor(stream.isFresh);
  const quality = stats?.quality;
  const utilization = useWorkerUtilization(stats);
  const queuePressure = useQueuePressure(stats);
  const jobDistribution = useJobStatusDistribution(recentJobs);
  const workerHeartbeatsUnavailable = quality?.worker_heartbeats.status === "unavailable";
  const recentJobsUnavailable = quality?.redis.status === "unavailable";
  const jobTypeStats = useMemo(() => stats?.job_type_stats ?? [], [stats?.job_type_stats]);

  const uniqueTypes = useMemo(() => {
    return jobTypeStats.map((s) => s.task_type).sort();
  }, [jobTypeStats]);

  const healthState = error
    ? "error"
    : isLoading && !stats
      ? "loading"
      : !quality
        ? "unknown"
        : quality.data_quality === "complete"
          ? "healthy"
          : "degraded";
  const healthCopy = {
    degraded: ["health_degraded", "health_description"],
    error: ["health_error", "error_loading"],
    healthy: ["health_healthy", "health_description"],
    loading: ["health_loading", "health_loading_description"],
    unknown: ["health_unknown", "health_unknown_description"],
  } as const;
  const [healthLabelKey, healthDescriptionKey] = healthCopy[healthState];
  const healthTone = {
    degraded: "border-amber-200 bg-amber-50 dark:border-amber-900 dark:bg-amber-950/30",
    error: "border-rose-200 bg-rose-50 dark:border-rose-900 dark:bg-rose-950/40",
    healthy: "border-emerald-200 bg-emerald-50 dark:border-emerald-900 dark:bg-emerald-950/30",
    loading: "border-slate-200 bg-slate-50 dark:border-slate-700 dark:bg-slate-800/60",
    unknown: "border-slate-200 bg-slate-50 dark:border-slate-700 dark:bg-slate-800/60",
  } as const;
  const HealthIcon = healthState === "error" ? AlertTriangle : healthState === "healthy" ? CheckCircle2 : healthState === "loading" ? Loader2 : CircleHelp;

  return (
    <main className="space-y-8 motion-reduce:animate-none" aria-labelledby="monitor-page-title">
      <MonitorHeader isFetching={isFetching} onRefresh={() => void refetch()} t={t} />

      <section className={cn("rounded-2xl border p-5 shadow-sm sm:p-6", healthTone[healthState])} aria-live="polite">
        <div className="flex items-start gap-3">
          <HealthIcon aria-hidden="true" className={cn("mt-0.5 shrink-0", healthState === "loading" && "motion-safe:animate-spin", healthState === "error" ? "text-rose-600" : healthState === "healthy" ? "text-emerald-600" : healthState === "degraded" ? "text-amber-600" : "text-slate-500")} size={20} />
          <div>
            <Typography className="text-xl font-black tracking-tight text-slate-900 dark:text-slate-100" variant="h2">{t(`monitor.${healthLabelKey}`, { defaultValue: healthLabelKey })}</Typography>
            <Typography className="mt-1 text-sm text-slate-700 dark:text-slate-300" variant="p">{t(`monitor.${healthDescriptionKey}`, { defaultValue: healthDescriptionKey === "error_loading" ? "Failed to load monitoring data. Retrying..." : "Monitoring state is not yet available." })}</Typography>
          </div>
        </div>
      </section>

      {quality && quality.data_quality !== "complete" ? (
        <div className="flex items-start gap-3 rounded-2xl border border-amber-200 bg-amber-50 px-5 py-4 text-sm font-medium text-amber-800 dark:border-amber-800 dark:bg-amber-950/40 dark:text-amber-200" role="status">
          <AlertTriangle aria-hidden="true" className="mt-0.5 shrink-0" size={18} />
          <div>
            <div>{t("monitor.partial_data", { defaultValue: "Some monitoring sources are unavailable. Unknown values are not zero." })}</div>
            <div className="mt-1 text-xs font-normal opacity-80">{t(`monitor.data_quality.${quality.data_quality}`, { defaultValue: "Some sources are degraded." })}</div>
          </div>
        </div>
      ) : null}

      {stats ? <SnapshotMetadata stats={stats} t={t} /> : null}
      <MonitorAlertsSection t={t} />
      <MonitorStatCards isLoading={isLoading} stats={stats} t={t} />
      {stats ? <OperationalMetricsSections stats={stats} t={t} /> : null}
      {stats ? <DependencyHealthSection dependencies={stats.dependencies} t={t} /> : null}

      <section aria-labelledby="monitor-charts-heading">
        <div className="mb-4"><Typography className="text-lg font-bold tracking-tight text-slate-800 dark:text-slate-100" id="monitor-charts-heading" variant="h4">{t("monitor.decision_charts", { defaultValue: "Decision charts" })}</Typography><Typography className="mt-1 text-xs text-slate-500 dark:text-slate-400" variant="p">{t("monitor.decision_charts_description", { defaultValue: "Capacity, pressure, and outcome views for the next operator decision." })}</Typography></div>
        <div className="grid grid-cols-1 gap-4 lg:grid-cols-3">
          <WorkerUtilizationGauge data={utilization} isLoading={isLoading} />
          <QueuePressureChart data={queuePressure} isLoading={isLoading} />
          <JobStatusDistributionChart data={jobDistribution} isLoading={isLoading} unavailable={recentJobsUnavailable} />
        </div>
      </section>

      <StreamStatusIndicator isFetching={isFetching} status={stream.status} t={t} />
      <WorkersTable isUnavailable={workerHeartbeatsUnavailable} t={t} workers={workers} />
      <JobTypeStatsSection jobTypeStats={jobTypeStats} t={t} />
      <JobsTable t={t} uniqueTypes={uniqueTypes} />
    </main>
  );
}
