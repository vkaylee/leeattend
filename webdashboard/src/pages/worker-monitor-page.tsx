import {
  AlertTriangle,
  Loader2,
} from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import JobStatusDistributionChart from "@/components/features/monitor/job-status-distribution-chart";
import QueuePressureChart from "@/components/features/monitor/queue-pressure-chart";
import WorkerUtilizationGauge from "@/components/features/monitor/worker-utilization-gauge";
import { useWorkerUtilization, useQueuePressure, useJobStatusDistribution } from "@/hooks/use-monitor-charts";
import { useMonitorStream } from "@/hooks/use-monitor-stream";
import { useWorkerMonitor, MONITOR_REFETCH_INTERVAL_S } from "@/hooks/use-worker-monitor";

import { MonitorHeader } from "./monitor/header";
import { JobsTable } from "./monitor/jobs-table";
import { MonitorStatCards } from "./monitor/stat-cards";
import { WorkersTable } from "./monitor/workers-table";

export default function WorkerMonitorPage() {
  const { t } = useTranslation();
  const { stats, workers, recentJobs, isLoading, isFetching, error, refetch } = useWorkerMonitor();
  const { connected: _sseConnected } = useMonitorStream();
  const utilization = useWorkerUtilization(stats);
  const queuePressure = useQueuePressure(stats);
  const jobDistribution = useJobStatusDistribution(recentJobs);

  return (
    <div className="space-y-10 animate-in fade-in duration-700">
      <MonitorHeader isFetching={isFetching} onRefresh={() => refetch()} t={t} />

      {error ? (
        <div className="flex items-center gap-3 px-5 py-4 rounded-2xl bg-rose-50 dark:bg-rose-950 border border-rose-200 dark:border-rose-800 text-rose-700 dark:text-rose-300 text-sm font-medium">
          <AlertTriangle className="shrink-0" size={18} />
          {t("monitor.error_loading", { defaultValue: "Failed to load monitoring data. Retrying..." })}
        </div>
      ) : null}

      <MonitorStatCards isLoading={isLoading} stats={stats} t={t} />

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <WorkerUtilizationGauge data={utilization} isLoading={isLoading} />
        <QueuePressureChart data={queuePressure} isLoading={isLoading} />
        <JobStatusDistributionChart data={jobDistribution} isLoading={isLoading} />
      </div>

      <div className="flex items-center gap-3 px-2">
        <div className="relative flex h-3 w-3">
          <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
          <span className="relative inline-flex rounded-full h-3 w-3 bg-emerald-50 dark:bg-emerald-950"></span>
        </div>
        <span className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
          {t("monitor.live_updates", { seconds: MONITOR_REFETCH_INTERVAL_S })}
        </span>
        {isFetching ? <Loader2 className="text-slate-400 dark:text-slate-500 animate-spin" size={14} /> : null}
      </div>

      <WorkersTable t={t} workers={workers} />
      <JobsTable recentJobs={recentJobs} t={t} />
    </div>
  );
}
