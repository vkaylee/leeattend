import type { TFunction } from "i18next";
import React from "react";
import {
  Activity,
  ArrowDown,
  ArrowUp,
  CheckCircle2,
  Coffee,
  Server,
  Zap,
} from "lucide-react";
import type { LucideIcon } from "lucide-react";

import { Typography } from "@/components/ui/typography";
import type { MonitorMetric, WorkerMonitorStats } from "@/lib/api/types";
import { cn } from "@/lib/utils";

interface MonitorStatCardsProps {
  isLoading: boolean;
  stats: WorkerMonitorStats | null | undefined;
  t: TFunction;
}

type Metric = Pick<MonitorMetric, "unit" | "unknown" | "value"> & {
  time_window: Pick<MonitorMetric["time_window"], "duration_seconds">;
};

interface MetricCardConfig {
  icon: LucideIcon;
  label: string;
  metric?: Metric;
  threshold: string;
  tone: string;
  unit: string;
  value: number | null;
  windowLabel?: string;
}

interface MetricCardProps extends MetricCardConfig {
  isLoading: boolean;
  t: TFunction;
}

interface MetricGridProps {
  cards: MetricCardConfig[];
  columnsClassName: string;
  isLoading: boolean;
  t: TFunction;
}

function formatWindow(metric: Metric | undefined, t: TFunction): string {
  const duration = metric?.time_window.duration_seconds;
  if (!duration) return t("monitor.snapshot_window", { defaultValue: "Current snapshot" });
  if (duration >= 3600) {
    return t("monitor.hours_window", {
      defaultValue: "Last {{hours}}h",
      hours: Math.round(duration / 3600),
    });
  }
  return t("monitor.minutes_window", {
    defaultValue: "Last {{minutes}}m",
    minutes: Math.max(1, Math.round(duration / 60)),
  });
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

function snapshotMetric(value: number | null | undefined, unit: string): Metric {
  return {
    unit,
    time_window: { duration_seconds: 0 },
    value: value ?? null,
  };
}

function MetricCard({
  icon: Icon,
  isLoading,
  label,
  metric,
  threshold,
  tone,
  t,
  value,
  unit,
  windowLabel,
}: MetricCardProps) {
  const unavailable = metric?.unknown != null || value === null;

  return (
    <article className="group relative overflow-hidden rounded-2xl border border-slate-200/80 bg-white p-4 shadow-sm transition-shadow duration-200 hover:shadow-md dark:border-slate-800 dark:bg-slate-900 motion-reduce:transition-none">
      <div className={cn("mb-3 flex h-9 w-9 items-center justify-center rounded-xl border", tone)}>
        <Icon aria-hidden="true" size={17} />
      </div>
      <div className="text-[10px] font-bold uppercase tracking-[0.16em] text-slate-500 dark:text-slate-400">{label}</div>
      <div className="mt-1 flex items-baseline gap-1.5">
        <span className="text-2xl font-black tabular-nums tracking-tight text-slate-800 dark:text-slate-100">
          {isLoading ? "..." : unavailable ? t("monitor.metric_unavailable", { defaultValue: "Unavailable" }) : value}
        </span>
        {!isLoading && !unavailable ? <span className="text-xs font-semibold text-slate-500 dark:text-slate-400">{unit}</span> : null}
      </div>
      <dl className="mt-3 space-y-1 border-t border-slate-100 pt-3 text-[11px] dark:border-slate-800">
        <div className="flex justify-between gap-2"><dt className="text-slate-500 dark:text-slate-400">{t("monitor.time_window", { defaultValue: "Window" })}</dt><dd className="text-right font-medium text-slate-700 dark:text-slate-300">{windowLabel ?? formatWindow(metric, t)}</dd></div>
        <div className="flex justify-between gap-2"><dt className="text-slate-500 dark:text-slate-400">{t("monitor.threshold", { defaultValue: "Threshold" })}</dt><dd className="text-right font-medium text-slate-700 dark:text-slate-300">{threshold}</dd></div>
        <div className="flex justify-between gap-2"><dt className="text-slate-500 dark:text-slate-400">{t("monitor.trend", { defaultValue: "Trend" })}</dt><dd className="text-right font-medium text-slate-500 dark:text-slate-400">{t("monitor.trend_unavailable", { defaultValue: "Unavailable" })}</dd></div>
      </dl>
      {metric?.unknown ? <p className="mt-2 text-xs text-amber-700 dark:text-amber-300">{t("monitor.unknown_metric_explanation", { defaultValue: "The source did not provide a trustworthy value." })}</p> : null}
    </article>
  );
}

function MetricGrid({ cards, columnsClassName, isLoading, t }: MetricGridProps) {
  return (
    <div className={cn("grid grid-cols-1 gap-3", columnsClassName)}>
      {cards.map((card) => <MetricCard {...card} isLoading={isLoading} key={card.label} t={t} />)}
    </div>
  );
}

function formatUtilizationThreshold(
  threshold: WorkerMonitorStats["thresholds"]["worker_utilization"] | undefined,
  unavailableText: string,
  t: TFunction,
): string {
  if (!threshold || threshold.source === "unavailable") return unavailableText;
  return t("monitor.percent_threshold", {
    defaultValue: "Warn {{warning}}% · critical {{critical}}%",
    warning: threshold.warning_at_percent,
    critical: threshold.critical_at_percent,
  });
}

function formatQueueThreshold(
  level: "high" | "low",
  threshold: WorkerMonitorStats["thresholds"]["queue_pressure"] | undefined,
  unavailableText: string,
  t: TFunction,
): string {
  if (!threshold || threshold.source === "unavailable") return unavailableText;
  return level === "high"
    ? t("monitor.queue_high_threshold_value", { defaultValue: "High at {{value}} jobs", value: threshold.high_at })
    : t("monitor.queue_medium_threshold_value", { defaultValue: "Medium at {{value}} jobs", value: threshold.medium_at });
}

function buildFleetCards(
  stats: WorkerMonitorStats | null | undefined,
  utilization: number | null,
  utilizationThreshold: string,
  unavailableText: string,
  t: TFunction,
): MetricCardConfig[] {
  return [
    {
      icon: Zap,
      label: t("monitor.active_workers"),
      metric: snapshotMetric(stats?.active_workers, "workers"),
      threshold: unavailableText,
      tone: "border-emerald-200 bg-emerald-50 text-emerald-700 dark:border-emerald-900 dark:bg-emerald-950/40 dark:text-emerald-300",
      unit: "workers",
      value: stats?.active_workers ?? null,
    },
    {
      icon: Coffee,
      label: t("monitor.idle_workers"),
      metric: snapshotMetric(stats?.idle_workers, "workers"),
      threshold: unavailableText,
      tone: "border-slate-200 bg-slate-50 text-slate-700 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-300",
      unit: "workers",
      value: stats?.idle_workers ?? null,
    },
    {
      icon: Server,
      label: t("monitor.total_workers"),
      metric: snapshotMetric(stats?.total_workers, "workers"),
      threshold: unavailableText,
      tone: "border-primary-200 bg-primary-50 text-primary-700 dark:border-primary-900 dark:bg-primary-950/40 dark:text-primary-300",
      unit: "workers",
      value: stats?.total_workers ?? null,
    },
    {
      icon: Activity,
      label: t("monitor.utilization", { defaultValue: "Worker utilization" }),
      metric: snapshotMetric(utilization, "%"),
      threshold: utilizationThreshold,
      tone: "border-amber-200 bg-amber-50 text-amber-700 dark:border-amber-900 dark:bg-amber-950/40 dark:text-amber-300",
      unit: "%",
      value: utilization,
    },
  ];
}

function buildQueueCards(
  stats: WorkerMonitorStats | null | undefined,
  thresholdFor: (level: "high" | "low") => string,
  t: TFunction,
): MetricCardConfig[] {
  const highMetric = stats?.queue_health.depth_high;
  const lowMetric = stats?.queue_health.depth_low;

  return [
    {
      icon: ArrowUp,
      label: t("monitor.pending_high"),
      metric: highMetric,
      threshold: thresholdFor("high"),
      tone: "border-orange-200 bg-orange-50 text-orange-700 dark:border-orange-900 dark:bg-orange-950/40 dark:text-orange-300",
      unit: metricUnit(highMetric, "jobs"),
      value: metricValue(highMetric, stats?.pending_high),
    },
    {
      icon: ArrowDown,
      label: t("monitor.pending_low"),
      metric: lowMetric,
      threshold: thresholdFor("low"),
      tone: "border-amber-200 bg-amber-50 text-amber-700 dark:border-amber-900 dark:bg-amber-950/40 dark:text-amber-300",
      unit: metricUnit(lowMetric, "jobs"),
      value: metricValue(lowMetric, stats?.pending_low),
    },
  ];
}

function buildThroughputCards(
  stats: WorkerMonitorStats | null | undefined,
  unavailableText: string,
  t: TFunction,
): MetricCardConfig[] {
  const todayWindow = formatCalendarDayWindow(t);

  return [
    {
      icon: CheckCircle2,
      label: t("monitor.completed_today"),
      metric: snapshotMetric(stats?.completed_today, "jobs"),
      threshold: unavailableText,
      tone: "border-emerald-200 bg-emerald-50 text-emerald-700 dark:border-emerald-900 dark:bg-emerald-950/40 dark:text-emerald-300",
      unit: "jobs",
      value: stats?.completed_today ?? null,
      windowLabel: todayWindow,
    },
    {
      icon: Activity,
      label: t("monitor.failed_today"),
      metric: snapshotMetric(stats?.failed_today, "jobs"),
      threshold: unavailableText,
      tone: "border-rose-200 bg-rose-50 text-rose-700 dark:border-rose-900 dark:bg-rose-950/40 dark:text-rose-300",
      unit: "jobs",
      value: stats?.failed_today ?? null,
      windowLabel: todayWindow,
    },
  ];
}

export function MonitorStatCards({ isLoading, stats, t }: MonitorStatCardsProps) {
  const activeWorkers = stats?.active_workers;
  const totalWorkers = stats?.total_workers;
  const utilization = activeWorkers != null && totalWorkers != null && totalWorkers > 0
    ? Math.round((activeWorkers / totalWorkers) * 100)
    : null;
  const thresholdUnavailable = t("monitor.threshold_unavailable", { defaultValue: "Unavailable" });
  const utilizationThreshold = formatUtilizationThreshold(stats?.thresholds.worker_utilization, thresholdUnavailable, t);
  const queueThreshold = stats?.thresholds.queue_pressure;
  const queueThresholdFor = (level: "high" | "low") => formatQueueThreshold(level, queueThreshold, thresholdUnavailable, t);

  return (
    <div className="space-y-5">
      <section aria-labelledby="monitor-fleet-heading">
        <div className="mb-3 flex items-end justify-between gap-3">
          <div>
            <Typography className="text-lg font-bold tracking-tight text-slate-800 dark:text-slate-100" id="monitor-fleet-heading" variant="h4">{t("monitor.worker_fleet", { defaultValue: "Worker fleet & capacity" })}</Typography>
            <p className="mt-1 text-xs text-slate-500 dark:text-slate-400">{t("monitor.worker_fleet_description", { defaultValue: "Capacity signals for deciding whether to scale, investigate, or hold." })}</p>
          </div>
        </div>
        <MetricGrid cards={buildFleetCards(stats, utilization, utilizationThreshold, thresholdUnavailable, t)} columnsClassName="sm:grid-cols-2 xl:grid-cols-4" isLoading={isLoading} t={t} />
      </section>
      <section aria-labelledby="monitor-queue-signals-heading">
        <div className="mb-3 flex items-end justify-between gap-3">
          <div>
            <Typography className="text-lg font-bold tracking-tight text-slate-800 dark:text-slate-100" id="monitor-queue-signals-heading" variant="h4">{t("monitor.queue_signals", { defaultValue: "Queue signals" })}</Typography>
            <p className="mt-1 text-xs text-slate-500 dark:text-slate-400">{t("monitor.queue_signals_description", { defaultValue: "Measured queue counts; zero means empty, unavailable means unmeasured." })}</p>
          </div>
        </div>
        <MetricGrid cards={buildQueueCards(stats, queueThresholdFor, t)} columnsClassName="sm:grid-cols-2" isLoading={isLoading} t={t} />
      </section>
      <section aria-labelledby="monitor-throughput-heading">
        <div className="mb-3">
          <Typography className="text-lg font-bold tracking-tight text-slate-800 dark:text-slate-100" id="monitor-throughput-heading" variant="h4">{t("monitor.job_throughput", { defaultValue: "Job throughput" })}</Typography>
        </div>
        <MetricGrid cards={buildThroughputCards(stats, thresholdUnavailable, t)} columnsClassName="sm:grid-cols-2" isLoading={isLoading} t={t} />
      </section>
      <div aria-live="polite" className="sr-only">{stats?.completed_today !== null && stats?.completed_today !== undefined ? `${t("monitor.completed_today")}: ${stats.completed_today}` : t("monitor.metric_unavailable", { defaultValue: "Metric unavailable" })}</div>
    </div>
  );
}
