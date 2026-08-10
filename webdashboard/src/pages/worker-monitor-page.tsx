import {
  AlertTriangle,
  Loader2,
} from "lucide-react";
import React, { useMemo } from "react";
import { useTranslation } from "react-i18next";

import JobStatusDistributionChart from "@/components/features/monitor/job-status-distribution-chart";
import QueuePressureChart from "@/components/features/monitor/queue-pressure-chart";
import WorkerUtilizationGauge from "@/components/features/monitor/worker-utilization-gauge";
import { Typography } from "@/components/ui/typography";
import { useWorkerUtilization, useQueuePressure, useJobStatusDistribution } from "@/hooks/use-monitor-charts";
import { useMonitorStream } from "@/hooks/use-monitor-stream";
import { cn } from "@/lib/utils";
import { useWorkerMonitor, MONITOR_REFETCH_INTERVAL_S } from "@/hooks/use-worker-monitor";

import { MonitorHeader } from "./monitor/header";
import { JobsTable } from "./monitor/jobs-table";
import { MonitorStatCards } from "./monitor/stat-cards";
import { WorkersTable } from "./monitor/workers-table";

export default function WorkerMonitorPage() {
  const { t } = useTranslation();
  const stream = useMonitorStream();
  const { stats, workers, recentJobs, isLoading, isFetching, error, refetch } = useWorkerMonitor(stream.isFresh);
  const quality = stats?.quality;
  const streamIsLive = stream.status === "live";
  const streamStatusLabel = stream.status === "live"
    ? t("monitor.live_updates", { seconds: MONITOR_REFETCH_INTERVAL_S })
    : stream.status === "stale"
      ? t("monitor.stale_updates", { defaultValue: "Live stream is stale; polling fallback is active" })
      : stream.status === "error"
        ? t("monitor.stream_error", { defaultValue: "Live stream unavailable; polling fallback is active" })
        : stream.status === "degraded"
          ? t("monitor.degraded_updates", { defaultValue: "Degraded monitoring data; polling fallback is active" })
          : t("monitor.connecting_updates", { defaultValue: "Connecting to live monitoring…" });
  const utilization = useWorkerUtilization(stats);
  const queuePressure = useQueuePressure(stats);
  const jobDistribution = useJobStatusDistribution(recentJobs);

  const jobTypeStats = useMemo(() => stats?.job_type_stats ?? [], [stats?.job_type_stats]);

  const uniqueTypes = useMemo(() => {
    return jobTypeStats.map((s) => s.task_type).sort();
  }, [jobTypeStats]);

  return (
    <div className="space-y-10 animate-in fade-in duration-700">
      <MonitorHeader isFetching={isFetching} onRefresh={() => refetch()} t={t} />

      {error ? (
        <div className="flex items-center gap-3 px-5 py-4 rounded-2xl bg-rose-50 dark:bg-rose-950 border border-rose-200 dark:border-rose-800 text-rose-700 dark:text-rose-300 text-sm font-medium">
          <AlertTriangle className="shrink-0" size={18} />
          {t("monitor.error_loading", { defaultValue: "Failed to load monitoring data. Retrying..." })}
        </div>
      ) : null}

      {quality && quality.data_quality !== "complete" ? (
        <div className="flex items-start gap-3 rounded-2xl border border-amber-200 bg-amber-50 px-5 py-4 text-sm font-medium text-amber-800 dark:border-amber-800 dark:bg-amber-950/40 dark:text-amber-200">
          <AlertTriangle aria-hidden="true" className="mt-0.5 shrink-0" size={18} />
          <div>
            <div>{t("monitor.partial_data", { defaultValue: "Some monitoring sources are unavailable. Values marked unknown are not zero." })}</div>
            <div className="mt-1 text-xs font-normal opacity-80">
              {quality.unknown_fields.length > 0 ? quality.unknown_fields.join(", ") : quality.data_quality}
            </div>
          </div>
        </div>
      ) : null}

      <MonitorStatCards isLoading={isLoading} stats={stats} t={t} />

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <WorkerUtilizationGauge data={utilization} isLoading={isLoading} />
        <QueuePressureChart data={queuePressure} isLoading={isLoading} />
        <JobStatusDistributionChart data={jobDistribution} isLoading={isLoading} />
      </div>

      <div
        aria-live="polite"
        className="flex items-center gap-3 px-2"
        role="status"
      >
        <div className="relative flex h-3 w-3" aria-hidden="true">
          <span aria-hidden="true" className={cn("absolute inline-flex h-full w-full rounded-full opacity-75", streamIsLive ? "animate-ping bg-emerald-400" : "bg-amber-400")} />
          <span aria-hidden="true" className={cn("relative inline-flex rounded-full h-3 w-3", streamIsLive ? "bg-emerald-500" : "bg-amber-500")} />
        </div>
        <span className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
          {streamStatusLabel}
        </span>
        {isFetching ? <Loader2 aria-hidden="true" className="text-slate-400 dark:text-slate-500 animate-spin" size={14} /> : null}
      </div>


      <WorkersTable t={t} workers={workers} />

      {jobTypeStats.length > 0 ? (
        <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-2xl sm:rounded-[2.5rem] p-5 sm:p-6 shadow-sm dark:shadow-black/10">
          <Typography className="text-lg font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight mb-4" variant="h4">
            {t("monitor.jobs_by_type", { defaultValue: "Breakdown by Type" })}
          </Typography>
          <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 xl:grid-cols-4 gap-4">
            {jobTypeStats.map((item) => {
              const displayType = t(`monitor.task_types.${item.task_type}`, { defaultValue: item.task_type });
              return (
                <div
                  className="bg-slate-50 dark:bg-slate-800/40 border border-slate-100/50 dark:border-slate-800/50 rounded-3xl p-5 flex flex-col justify-between hover:shadow-md transition-all duration-300"
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
                    {item.completed > 0 ? (
                      <span className="text-emerald-600 dark:text-emerald-400 flex items-center gap-1">
                        <span className="h-1.5 w-1.5 rounded-full bg-emerald-500" />
                        {item.completed}
                      </span>
                    ) : null}
                    {item.failed > 0 ? (
                      <span className="text-rose-600 dark:text-rose-400 flex items-center gap-1">
                        <span className="h-1.5 w-1.5 rounded-full bg-rose-500 animate-pulse" />
                        {item.failed}
                      </span>
                    ) : null}
                    {item.processing > 0 ? (
                      <span className="text-amber-600 dark:text-amber-400 flex items-center gap-1">
                        <span className="h-1.5 w-1.5 rounded-full bg-amber-500 animate-ping" />
                        {item.processing}
                      </span>
                    ) : null}
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      ) : null}

      <JobsTable t={t} uniqueTypes={uniqueTypes} />
    </div>
  );
}
