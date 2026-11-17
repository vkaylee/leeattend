import type { TFunction } from "i18next";
import type { LucideIcon } from "lucide-react";

import {
  Activity,
  ArrowDown,
  ArrowUp,
  CheckCircle2,
  Coffee,
  Server,
  Zap,
} from "lucide-react";
import React from "react";

import type { MonitorMetric, WorkerMonitorStats } from "@/lib/api/types";

import { Typography } from "@/components/ui/typography";
import { cn } from "@/lib/utils";

interface MonitorStatCardsProps {
  isLoading: boolean;
  stats: WorkerMonitorStats | null | undefined;
  t: TFunction;
}

type Metric = Pick<MonitorMetric, "unit" | "unknown" | "value"> & {
  time_window: Pick<MonitorMetric["time_window"], "duration_seconds">;
};

type QueueThreshold = WorkerMonitorStats["thresholds"]["queue_pressure"];
type UtilizationThreshold = WorkerMonitorStats["thresholds"]["worker_utilization"];

interface ThresholdTextOptions {
  t: TFunction;
  thresholdUnavailable: string;
}

interface QueueThresholdTextOptions extends ThresholdTextOptions {
  level: "high" | "low";
  threshold: QueueThreshold | undefined;
}

interface UtilizationThresholdTextOptions extends ThresholdTextOptions {
  threshold: UtilizationThreshold | undefined;
}

interface MetricCardOptions {
  icon: LucideIcon;
  isLoading: boolean;
  label: string;
  metric?: Metric;
  t: TFunction;
  threshold: string;
  tone: string;
  unit: string;
  value: number | null;
  windowLabel?: string;
}

function formatWindow(metric: Metric | undefined, t: TFunction): string {
  const duration = metric?.time_window.duration_seconds;
  if (!duration) return t("monitor.snapshot_window", { defaultValue: "Current snapshot" });
  if (duration >= 3600) {
    return t("monitor.hours_window", { defaultValue: "Last {{hours}}h", hours: Math.round(duration / 3600) });
  }
  return t("monitor.minutes_window", { defaultValue: "Last {{minutes}}m", minutes: Math.max(1, Math.round(duration / 60)) });
}

function formatCalendarDayWindow(t: TFunction): string {
  return t("monitor.today_window", { defaultValue: "Today" });
}

function metricValue(metric: Metric | undefined, fallback: number | null | undefined): number | null {
  return metric ? metric.value ?? null : fallback ?? null;
}

function metricUnit(metric: Metric | undefined, fallback: string): string {
  return metric?.unit ?? fallback;
}

function getUtilization(activeWorkers: number | null | undefined, totalWorkers: number | null | undefined): number | null {
  if (activeWorkers == null || totalWorkers == null || totalWorkers <= 0) return null;
  return Math.round((activeWorkers / totalWorkers) * 100);
}

function formatQueueThreshold({ level, t, threshold, thresholdUnavailable }: QueueThresholdTextOptions): string {
  if (!threshold || threshold.source === "unavailable") return thresholdUnavailable;
  return level === "high"
    ? t("monitor.queue_high_threshold_value", { defaultValue: "High at {{value}} jobs", value: threshold.high_at })
    : t("monitor.queue_medium_threshold_value", { defaultValue: "Medium at {{value}} jobs", value: threshold.medium_at });
}

function formatUtilizationThreshold({ t, threshold, thresholdUnavailable }: UtilizationThresholdTextOptions): string {
  if (!threshold || threshold.source === "unavailable") return thresholdUnavailable;
  return t("monitor.percent_threshold", {
    defaultValue: "Warn {{warning}}% · critical {{critical}}%",
    warning: threshold.warning_at_percent,
    critical: threshold.critical_at_percent,
  });
}

interface MetricCardValueProps {
  isLoading: boolean;
  t: TFunction;
  unavailable: boolean;
  unit: string;
  value: number | null;
}

function MetricCardValue({ isLoading, t, unavailable, unit, value }: MetricCardValueProps) {
  if (isLoading) return <span className="text-2xl font-black tabular-nums tracking-tight text-slate-800 dark:text-slate-100">...</span>;
  if (unavailable) {
    return <span className="text-2xl font-black tabular-nums tracking-tight text-slate-800 dark:text-slate-100">{t("monitor.metric_unavailable", { defaultValue: "Unavailable" })}</span>;
  }
  return (
    <>
      <span className="text-2xl font-black tabular-nums tracking-tight text-slate-800 dark:text-slate-100">{value}</span>
      <span className="text-xs font-semibold text-slate-500 dark:text-slate-400">{unit}</span>
    </>
  );
}

interface MetricCardDetailsProps {
  metric: Metric | undefined;
  t: TFunction;
  threshold: string;
  windowLabel: string | undefined;
}

function MetricCardDetails({ metric, t, threshold, windowLabel }: MetricCardDetailsProps) {
  return (
    <dl className="mt-3 space-y-1 border-t border-slate-100 pt-3 text-[11px] dark:border-slate-800">
      <div className="flex justify-between gap-2"><dt className="text-slate-500 dark:text-slate-400">{t("monitor.time_window", { defaultValue: "Window" })}</dt><dd className="text-right font-medium text-slate-700 dark:text-slate-300">{windowLabel ?? formatWindow(metric, t)}</dd></div>
      <div className="flex justify-between gap-2"><dt className="text-slate-500 dark:text-slate-400">{t("monitor.threshold", { defaultValue: "Threshold" })}</dt><dd className="text-right font-medium text-slate-700 dark:text-slate-300">{threshold}</dd></div>
      <div className="flex justify-between gap-2"><dt className="text-slate-500 dark:text-slate-400">{t("monitor.trend", { defaultValue: "Trend" })}</dt><dd className="text-right font-medium text-slate-500 dark:text-slate-400">{t("monitor.trend_unavailable", { defaultValue: "Unavailable" })}</dd></div>
    </dl>
  );
}

function MetricCard(options: MetricCardOptions) {
  const { icon: Icon, isLoading, label, metric, t, threshold, tone, unit, value, windowLabel } = options;
  const unavailable = metric?.unknown != null || value === null;

  return (
    <article className="group relative overflow-hidden rounded-2xl border border-slate-200/80 bg-white p-4 shadow-sm transition-shadow duration-200 hover:shadow-md dark:border-slate-800 dark:bg-slate-900 motion-reduce:transition-none">
      <div className={cn("mb-3 flex h-9 w-9 items-center justify-center rounded-xl border", tone)}>
        <Icon aria-hidden="true" size={17} />
      </div>
      <div className="text-[10px] font-bold uppercase tracking-[0.16em] text-slate-500 dark:text-slate-400">{label}</div>
      <div className="mt-1 flex items-baseline gap-1.5"><MetricCardValue isLoading={isLoading} t={t} unavailable={unavailable} unit={unit} value={value} /></div>
      <MetricCardDetails metric={metric} t={t} threshold={threshold} windowLabel={windowLabel} />
      {metric?.unknown ? <Typography className="mt-2 text-xs text-amber-700 dark:text-amber-300" variant="helper">{t("monitor.unknown_metric_explanation", { defaultValue: "The source did not provide a trustworthy value." })}</Typography> : null}
    </article>
  );
}

interface MonitorStatSectionOptions {
  isLoading: boolean;
  stats: WorkerMonitorStats | null | undefined;
  t: TFunction;
  thresholdUnavailable: string;
}

interface WorkerValues {
  activeWorkers: number | null;
  idleWorkers: number | null;
  totalWorkers: number | null;
  utilization: number | null;
}

function getWorkerValues(stats: WorkerMonitorStats | null | undefined): WorkerValues {
  const activeWorkers = stats?.active_workers ?? null;
  const totalWorkers = stats?.total_workers ?? null;
  return {
    activeWorkers,
    idleWorkers: stats?.idle_workers ?? null,
    totalWorkers,
    utilization: getUtilization(activeWorkers, totalWorkers),
  };
}

function getCompletedAnnouncement(stats: WorkerMonitorStats | null | undefined, t: TFunction): string {
  if (stats?.completed_today !== null && stats?.completed_today !== undefined) {
    return `${t("monitor.completed_today")}: ${stats.completed_today}`;
  }
  return t("monitor.metric_unavailable", { defaultValue: "Metric unavailable" });
}

interface FleetSectionOptions {
  activeWorkers: number | null;
  idleWorkers: number | null;
  isLoading: boolean;
  t: TFunction;
  thresholdUnavailable: string;
  totalWorkers: number | null;
  utilization: number | null;
  utilizationThresholdText: string;
}

function FleetSection({ activeWorkers, idleWorkers, isLoading, t, thresholdUnavailable, totalWorkers, utilization, utilizationThresholdText }: FleetSectionOptions) {
  return (
    <section aria-labelledby="monitor-fleet-heading">
      <div className="mb-3 flex items-end justify-between gap-3">
        <div>
          <Typography className="text-lg font-bold tracking-tight text-slate-800 dark:text-slate-100" id="monitor-fleet-heading" variant="h4">{t("monitor.worker_fleet", { defaultValue: "Worker fleet & capacity" })}</Typography>
          <Typography className="mt-1 text-xs text-slate-500 dark:text-slate-400" variant="helper">{t("monitor.worker_fleet_description", { defaultValue: "Capacity signals for deciding whether to scale, investigate, or hold." })}</Typography>
        </div>
      </div>
      <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-4">
        <MetricCard icon={Zap} isLoading={isLoading} label={t("monitor.active_workers")} metric={{ unit: "workers", time_window: { duration_seconds: 0 }, value: activeWorkers }} t={t} threshold={thresholdUnavailable} tone="border-emerald-200 bg-emerald-50 text-emerald-700 dark:border-emerald-900 dark:bg-emerald-950/40 dark:text-emerald-300" unit="workers" value={activeWorkers} />
        <MetricCard icon={Coffee} isLoading={isLoading} label={t("monitor.idle_workers")} metric={{ unit: "workers", time_window: { duration_seconds: 0 }, value: idleWorkers }} t={t} threshold={thresholdUnavailable} tone="border-slate-200 bg-slate-50 text-slate-700 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-300" unit="workers" value={idleWorkers} />
        <MetricCard icon={Server} isLoading={isLoading} label={t("monitor.total_workers")} metric={{ unit: "workers", time_window: { duration_seconds: 0 }, value: totalWorkers }} t={t} threshold={thresholdUnavailable} tone="border-primary-200 bg-primary-50 text-primary-700 dark:border-primary-900 dark:bg-primary-950/40 dark:text-primary-300" unit="workers" value={totalWorkers} />
        <MetricCard icon={Activity} isLoading={isLoading} label={t("monitor.utilization", { defaultValue: "Worker utilization" })} metric={{ unit: "%", time_window: { duration_seconds: 0 }, value: utilization }} t={t} threshold={utilizationThresholdText} tone="border-amber-200 bg-amber-50 text-amber-700 dark:border-amber-900 dark:bg-amber-950/40 dark:text-amber-300" unit="%" value={utilization} />
      </div>
    </section>
  );
}

function QueueSection({ isLoading, stats, t, thresholdUnavailable }: MonitorStatSectionOptions) {
  const queueThreshold = stats?.thresholds?.queue_pressure;
  return (
    <section aria-labelledby="monitor-queue-signals-heading">
      <div className="mb-3 flex items-end justify-between gap-3">
        <div>
          <Typography className="text-lg font-bold tracking-tight text-slate-800 dark:text-slate-100" id="monitor-queue-signals-heading" variant="h4">{t("monitor.queue_signals", { defaultValue: "Queue signals" })}</Typography>
          <Typography className="mt-1 text-xs text-slate-500 dark:text-slate-400" variant="helper">{t("monitor.queue_signals_description", { defaultValue: "Measured queue counts; zero means empty, unavailable means unmeasured." })}</Typography>
        </div>
      </div>
      <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
        <MetricCard icon={ArrowUp} isLoading={isLoading} label={t("monitor.pending_high")} metric={stats?.queue_health.depth_high} t={t} threshold={formatQueueThreshold({ level: "high", t, threshold: queueThreshold, thresholdUnavailable })} tone="border-orange-200 bg-orange-50 text-orange-700 dark:border-orange-900 dark:bg-orange-950/40 dark:text-orange-300" unit={metricUnit(stats?.queue_health.depth_high, "jobs")} value={metricValue(stats?.queue_health.depth_high, stats?.pending_high)} />
        <MetricCard icon={ArrowDown} isLoading={isLoading} label={t("monitor.pending_low")} metric={stats?.queue_health.depth_low} t={t} threshold={formatQueueThreshold({ level: "low", t, threshold: queueThreshold, thresholdUnavailable })} tone="border-amber-200 bg-amber-50 text-amber-700 dark:border-amber-900 dark:bg-amber-950/40 dark:text-amber-300" unit={metricUnit(stats?.queue_health.depth_low, "jobs")} value={metricValue(stats?.queue_health.depth_low, stats?.pending_low)} />
      </div>
    </section>
  );
}

function ThroughputSection({ isLoading, stats, t, thresholdUnavailable }: MonitorStatSectionOptions) {
  return (
    <section aria-labelledby="monitor-throughput-heading">
      <div className="mb-3">
        <Typography className="text-lg font-bold tracking-tight text-slate-800 dark:text-slate-100" id="monitor-throughput-heading" variant="h4">{t("monitor.job_throughput", { defaultValue: "Job throughput" })}</Typography>
      </div>
      <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
        <MetricCard icon={CheckCircle2} isLoading={isLoading} label={t("monitor.completed_today")} metric={{ unit: "jobs", time_window: { duration_seconds: 0 }, value: stats?.completed_today ?? null }} t={t} threshold={thresholdUnavailable} tone="border-emerald-200 bg-emerald-50 text-emerald-700 dark:border-emerald-900 dark:bg-emerald-950/40 dark:text-emerald-300" unit="jobs" value={stats?.completed_today ?? null} windowLabel={formatCalendarDayWindow(t)} />
        <MetricCard icon={Activity} isLoading={isLoading} label={t("monitor.failed_today")} metric={{ unit: "jobs", time_window: { duration_seconds: 0 }, value: stats?.failed_today ?? null }} t={t} threshold={thresholdUnavailable} tone="border-rose-200 bg-rose-50 text-rose-700 dark:border-rose-900 dark:bg-rose-950/40 dark:text-rose-300" unit="jobs" value={stats?.failed_today ?? null} windowLabel={formatCalendarDayWindow(t)} />
      </div>
    </section>
  );
}

export function MonitorStatCards({ isLoading, stats, t }: MonitorStatCardsProps) {
  const thresholdUnavailable = t("monitor.threshold_unavailable", { defaultValue: "Unavailable" });
  const { activeWorkers, idleWorkers, totalWorkers, utilization } = getWorkerValues(stats);
  const utilizationThresholdText = formatUtilizationThreshold({
    t,
    threshold: stats?.thresholds?.worker_utilization,
    thresholdUnavailable,
  });
  const sectionOptions = { isLoading, stats, t, thresholdUnavailable };

  return (
    <div className="space-y-5">
      <FleetSection
        activeWorkers={activeWorkers}
        idleWorkers={idleWorkers}
        isLoading={isLoading}
        t={t}
        thresholdUnavailable={thresholdUnavailable}
        totalWorkers={totalWorkers}
        utilization={utilization}
        utilizationThresholdText={utilizationThresholdText}
      />
      <QueueSection {...sectionOptions} />
      <ThroughputSection {...sectionOptions} />
      <div aria-live="polite" className="sr-only">{getCompletedAnnouncement(stats, t)}</div>
    </div>
  );
}
