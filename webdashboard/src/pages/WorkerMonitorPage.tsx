import { formatDistanceToNow } from "date-fns";
import {
  Activity,
  Server,
  CheckCircle2,
  XCircle,
  Clock,
  Cpu,
  RefreshCw,
  Loader2,
  Zap,
  Coffee,
  AlertTriangle,
  Briefcase,
  ArrowUp,
  ArrowDown,
} from "lucide-react";
import { useTranslation } from "react-i18next";

import JobStatusDistributionChart from "@/components/features/monitor/JobStatusDistributionChart";
import QueuePressureChart from "@/components/features/monitor/QueuePressureChart";
import WorkerUtilizationGauge from "@/components/features/monitor/WorkerUtilizationGauge";

import { useWorkerUtilization, useQueuePressure, useJobStatusDistribution } from "@/hooks/use-monitor-charts";
import { useMonitorStream } from "@/hooks/use-monitor-stream";
import { useWorkerMonitor, MONITOR_REFETCH_INTERVAL_S } from "@/hooks/use-worker-monitor";
import { cn } from "@/lib/utils";


export default function WorkerMonitorPage() {
  const { t } = useTranslation();
  const { stats, workers, recentJobs, isLoading, isFetching, error, refetch } = useWorkerMonitor();
  const { connected: sseConnected } = useMonitorStream();
  const utilization = useWorkerUtilization(stats);
  const queuePressure = useQueuePressure(stats);
  const jobDistribution = useJobStatusDistribution(recentJobs);

  const statCards = [
    {
      label: t("monitor.pending_high"),
      value: stats?.pending_high ?? 0,
      icon: ArrowUp,
      color: "bg-orange-50 dark:bg-orange-950 text-orange-600 dark:text-orange-300 border-orange-100 dark:border-orange-800",
      bgIcon: "text-orange-100",
    },
    {
      label: t("monitor.pending_low"),
      value: stats?.pending_low ?? 0,
      icon: ArrowDown,
      color: "bg-amber-50 dark:bg-amber-950 text-amber-600 dark:text-amber-300 border-amber-100 dark:border-amber-800",
      bgIcon: "text-amber-100",
    },
    {
      label: t("monitor.active_workers"),
      value: stats?.active_workers ?? 0,
      icon: Zap,
      color: "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800",
      bgIcon: "text-emerald-100",
    },
    {
      label: t("monitor.idle_workers"),
      value: stats?.idle_workers ?? 0,
      icon: Coffee,
      color: "bg-slate-50 dark:bg-slate-800 text-slate-600 dark:text-slate-300 border-slate-100 dark:border-slate-700",
      bgIcon: "text-slate-100",
    },
    {
      label: t("monitor.completed_today"),
      value: stats?.completed_today ?? 0,
      icon: CheckCircle2,
      color: "bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-400 border-primary-100 dark:border-primary-800",
      bgIcon: "text-primary-100",
    },
    {
      label: t("monitor.failed_today"),
      value: stats?.failed_today ?? 0,
      icon: XCircle,
      color: "bg-rose-50 dark:bg-rose-950 text-rose-600 dark:text-rose-300 border-rose-100 dark:border-rose-800",
      bgIcon: "text-rose-100",
    },
    {
      label: t("monitor.total_workers"),
      value: stats?.total_workers ?? 0,
      icon: Server,
      color: "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800",
      bgIcon: "text-emerald-100 dark:text-emerald-800",
    },
  ];

  return (
    <div className="space-y-10 animate-in fade-in duration-700">
      {/* Header */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-8 px-2">
        <div className="flex items-center gap-5">
          <div className="w-14 h-14 rounded-2xl bg-primary-50 dark:bg-primary-950 border border-primary-100 dark:border-primary-800 flex items-center justify-center">
            <Activity size={28} className="text-primary-600 dark:text-primary-400" />
          </div>
          <div>
            <h3 className="text-4xl font-bold tracking-tighter text-slate-700 dark:text-slate-200 uppercase leading-none">
              {t("monitor.title")}
            </h3>
            <p className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] mt-2 opacity-60">
              {t("monitor.subtitle")}
            </p>
          </div>
        </div>

        <button
          onClick={() => refetch()}
          disabled={isFetching}
          className="flex items-center gap-3 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-200 px-6 py-3 rounded-2xl hover:bg-slate-50 dark:hover:bg-slate-700 transition-all font-bold text-xs uppercase tracking-widest shadow-lg dark:shadow-black/20 active:scale-95 disabled:opacity-50"
        >
          <RefreshCw size={16} className={cn(isFetching && "animate-spin")} />
          {t("common.refresh")}
        </button>
      </div>

      {/* Error Banner */}
      {error && (
        <div className="flex items-center gap-3 px-5 py-4 rounded-2xl bg-rose-50 dark:bg-rose-950 border border-rose-200 dark:border-rose-800 text-rose-700 dark:text-rose-300 text-sm font-medium">
          <AlertTriangle size={18} className="shrink-0" />
          {t("monitor.error_loading", { defaultValue: "Failed to load monitoring data. Retrying..." })}
        </div>
      )}

      {/* Stats Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-7 gap-6">
        {statCards.map((stat, i) => (
          <div
            key={i}
            className="bg-white dark:bg-slate-900 p-6 rounded-[2rem] border border-slate-100 dark:border-slate-800 shadow-sm dark:shadow-black/10 hover:shadow-xl dark:hover:shadow-black/20 transition-all group relative overflow-hidden"
          >
            <div
              className={cn(
                "w-12 h-12 rounded-xl flex items-center justify-center mb-4 border",
                stat.color
              )}
            >
              <stat.icon size={22} />
            </div>
            <p className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em] mb-1 opacity-70">
              {stat.label}
            </p>
            <p className="text-3xl font-bold text-slate-700 dark:text-slate-200 tracking-tighter">
              {isLoading ? "..." : stat.value}
            </p>
            <stat.icon
              className={cn(
                "absolute -right-4 -bottom-4 w-24 h-24 opacity-10 -rotate-12 group-hover:rotate-0 transition-transform duration-500",
                stat.bgIcon
              )}
            />
          </div>
        ))}
      </div>

      {/* Scaling Decision Charts */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <WorkerUtilizationGauge data={utilization} isLoading={isLoading} />
        <QueuePressureChart data={queuePressure} isLoading={isLoading} />
        <JobStatusDistributionChart data={jobDistribution} isLoading={isLoading} />
      </div>

      {/* Live Indicator */}
      <div className="flex items-center gap-3 px-2">
        <div className="relative flex h-3 w-3">
          <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
          <span className="relative inline-flex rounded-full h-3 w-3 bg-emerald-50 dark:bg-emerald-950"></span>
        </div>
        <span className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
          {t("monitor.live_updates", { seconds: MONITOR_REFETCH_INTERVAL_S })}
        </span>
        {isFetching && (
          <Loader2 size={14} className="text-slate-400 dark:text-slate-500 animate-spin" />
        )}
      </div>

      {/* Workers Table */}
      <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[2.5rem] overflow-hidden shadow-sm dark:shadow-black/10">
        <div className="p-6 border-b border-slate-100 dark:border-slate-800">
          <h4 className="text-lg font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight">
            {t("monitor.worker_list")}
          </h4>
          <p className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em] mt-1 opacity-60">
            {t("monitor.worker_list_subtitle")}
          </p>
        </div>

        {workers.length === 0 ? (
          <div className="p-12 text-center">
            <Server size={48} className="mx-auto text-slate-200 dark:text-slate-700 mb-4" />
            <p className="text-sm font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest">
              {t("monitor.no_workers")}
            </p>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="bg-slate-50 dark:bg-slate-800/50">
                  <th className="text-left px-6 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
                    {t("monitor.worker_id")}
                  </th>
                  <th className="text-left px-6 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
                    {t("monitor.host")}
                  </th>
                  <th className="text-left px-6 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
                    {t("monitor.status")}
                  </th>
                  <th className="text-left px-6 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
                    {t("monitor.current_task")}
                  </th>
                  <th className="text-left px-6 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
                    {t("monitor.last_seen")}
                  </th>
                </tr>
              </thead>
              <tbody>
                {workers.map((worker) => (
                  <tr
                    key={worker.id}
                    className="border-t border-slate-50 dark:border-slate-800 hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-colors"
                  >
                    <td className="px-6 py-4">
                      <code
                        title={worker.id}
                        className="text-xs font-mono text-slate-600 dark:text-slate-300 bg-slate-100 dark:bg-slate-700 px-2 py-1 rounded cursor-help"
                      >
                        {worker.id.slice(0, 8)}...
                      </code>
                    </td>
                    <td className="px-6 py-4">
                      <div className="flex items-center gap-2">
                        <Cpu size={14} className="text-slate-400 dark:text-slate-500" />
                        <span className="text-sm font-medium text-slate-700 dark:text-slate-200">
                          {worker.host}
                        </span>
                      </div>
                    </td>
                    <td className="px-6 py-4">
                      <span
                        className={cn(
                          "inline-flex items-center gap-2 px-3 py-1 rounded-full text-xs font-bold uppercase tracking-wider",
                          worker.status === "PROCESSING"
                            ? "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300"
                            : "bg-slate-100 dark:bg-slate-700 text-slate-500 dark:text-slate-400"
                        )}
                      >
                        {worker.status === "PROCESSING" ? (
                          <Zap size={12} />
                        ) : (
                          <Coffee size={12} />
                        )}
                        {worker.status}
                      </span>
                    </td>
                    <td className="px-6 py-4">
                      {worker.current_task_type ? (
                        <span className="text-sm font-medium text-slate-700 dark:text-slate-200">
                          {worker.current_task_type}
                        </span>
                      ) : (
                        <span className="text-sm text-slate-400 dark:text-slate-500">—</span>
                      )}
                    </td>
                    <td className="px-6 py-4">
                      <span className="text-sm text-slate-500 dark:text-slate-400">
                        {formatDistanceToNow(new Date(worker.last_seen_at), {
                          addSuffix: true,
                        })}
                      </span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {/* Recent Jobs Table */}
      <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[2.5rem] overflow-hidden shadow-sm dark:shadow-black/10">
        <div className="p-6 border-b border-slate-100 dark:border-slate-800">
          <h4 className="text-lg font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight">
            {t("monitor.recent_jobs", { defaultValue: "Recent Jobs" })}
          </h4>
          <p className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em] mt-1 opacity-60">
            {t("monitor.recent_jobs_subtitle", { defaultValue: "Job metadata from last 24 hours" })}
          </p>
        </div>

        {recentJobs.length === 0 ? (
          <div className="p-12 text-center">
            <Briefcase size={48} className="mx-auto text-slate-200 dark:text-slate-700 mb-4" />
            <p className="text-sm font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest">
              {t("monitor.no_jobs", { defaultValue: "No recent jobs" })}
            </p>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="bg-slate-50 dark:bg-slate-800/50">
                  <th className="text-left px-6 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
                    {t("monitor.job_id", { defaultValue: "Job ID" })}
                  </th>
                  <th className="text-left px-6 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
                    {t("monitor.status")}
                  </th>
                  <th className="text-left px-6 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
                    {t("monitor.job_type", { defaultValue: "Type" })}
                  </th>
                  <th className="text-left px-6 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
                    {t("monitor.batches", { defaultValue: "Batches" })}
                  </th>
                  <th className="text-left px-6 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
                    {t("monitor.created_at", { defaultValue: "Created" })}
                  </th>
                  <th className="text-left px-6 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
                    {t("monitor.error", { defaultValue: "Error" })}
                  </th>
                </tr>
              </thead>
              <tbody>
                {recentJobs.map((job) => (
                  <tr
                    key={job.job_id}
                    className="border-t border-slate-50 dark:border-slate-800 hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-colors"
                  >
                    <td className="px-6 py-4">
                      <code
                        title={job.job_id}
                        className="text-xs font-mono text-slate-600 dark:text-slate-300 bg-slate-100 dark:bg-slate-700 px-2 py-1 rounded cursor-help"
                      >
                        {job.job_id.slice(0, 8)}...
                      </code>
                    </td>
                    <td className="px-6 py-4">
                      <span
                        className={cn(
                          "inline-flex items-center gap-2 px-3 py-1 rounded-full text-xs font-bold uppercase tracking-wider",
                          job.status === "completed"
                            ? "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300"
                            : job.status === "failed"
                              ? "bg-rose-50 dark:bg-rose-950 text-rose-600 dark:text-rose-300"
                              : job.status === "processing"
                                ? "bg-amber-50 dark:bg-amber-950 text-amber-600 dark:text-amber-300"
                                : "bg-slate-100 dark:bg-slate-700 text-slate-500 dark:text-slate-400"
                        )}
                      >
                        {job.status === "completed" ? (
                          <CheckCircle2 size={12} />
                        ) : job.status === "failed" ? (
                          <XCircle size={12} />
                        ) : job.status === "processing" ? (
                          <Loader2 size={12} className="animate-spin" />
                        ) : (
                          <Clock size={12} />
                        )}
                        {job.status}
                      </span>
                    </td>
                    <td className="px-6 py-4">
                      <span className="text-sm font-medium text-slate-700 dark:text-slate-200">
                        {job.report_type}
                      </span>
                    </td>
                    <td className="px-6 py-4">
                      <span className="text-sm text-slate-700 dark:text-slate-200">
                        {job.completed_batches}/{job.total_batches}
                        {job.failed_batches > 0 && (
                          <span className="text-rose-500 ml-1">
                            ({job.failed_batches} failed)
                          </span>
                        )}
                      </span>
                    </td>
                    <td className="px-6 py-4">
                      <span className="text-sm text-slate-500 dark:text-slate-400">
                        {formatDistanceToNow(new Date(job.created_at), {
                          addSuffix: true,
                        })}
                      </span>
                    </td>
                    <td className="px-6 py-4 max-w-[200px]">
                      {job.error_message ? (
                        <span
                          title={job.error_message}
                          className="text-xs text-rose-500 dark:text-rose-400 truncate block cursor-help"
                        >
                          {job.error_message}
                        </span>
                      ) : (
                        <span className="text-sm text-slate-400 dark:text-slate-500">—</span>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
}
