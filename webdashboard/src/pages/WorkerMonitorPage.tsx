import { useTranslation } from "react-i18next";
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
} from "lucide-react";
import { useWorkerMonitor } from "@/hooks/use-worker-monitor";
import { cn } from "@/lib/utils";
import { formatDistanceToNow } from "date-fns";

export default function WorkerMonitorPage() {
  const { t } = useTranslation();
  const { stats, workers, isLoading, isFetching, refetch } = useWorkerMonitor();

  const statCards = [
    {
      label: t("monitor.pending_tasks"),
      value: stats?.pending_tasks ?? 0,
      icon: Clock,
      color: "bg-amber-50 text-amber-600 border-amber-100",
      bgIcon: "text-amber-100",
    },
    {
      label: t("monitor.active_workers"),
      value: stats?.active_workers ?? 0,
      icon: Zap,
      color: "bg-emerald-50 text-emerald-600 border-emerald-100",
      bgIcon: "text-emerald-100",
    },
    {
      label: t("monitor.idle_workers"),
      value: stats?.idle_workers ?? 0,
      icon: Coffee,
      color: "bg-slate-50 text-slate-600 border-slate-100",
      bgIcon: "text-slate-100",
    },
    {
      label: t("monitor.completed_today"),
      value: stats?.completed_today ?? 0,
      icon: CheckCircle2,
      color: "bg-primary-50 text-primary-600 border-primary-100",
      bgIcon: "text-primary-100",
    },
    {
      label: t("monitor.failed_today"),
      value: stats?.failed_today ?? 0,
      icon: XCircle,
      color: "bg-rose-50 text-rose-600 border-rose-100",
      bgIcon: "text-rose-100",
    },
    {
      label: t("monitor.total_workers"),
      value: stats?.total_workers ?? 0,
      icon: Server,
      color: "bg-violet-50 text-violet-600 border-violet-100",
      bgIcon: "text-violet-100",
    },
  ];

  return (
    <div className="space-y-10 animate-in fade-in duration-700">
      {/* Header */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-8 px-2">
        <div className="flex items-center gap-5">
          <div className="w-14 h-14 rounded-2xl bg-primary-50 border border-primary-100 flex items-center justify-center">
            <Activity size={28} className="text-primary-600" />
          </div>
          <div>
            <h3 className="text-4xl font-bold tracking-tighter text-slate-700 uppercase leading-none">
              {t("monitor.title")}
            </h3>
            <p className="text-[10px] font-bold text-slate-500 uppercase tracking-[0.25em] mt-2 opacity-60">
              {t("monitor.subtitle")}
            </p>
          </div>
        </div>

        <button
          onClick={() => refetch()}
          disabled={isFetching}
          className="flex items-center gap-3 bg-white border border-slate-200 text-slate-700 px-6 py-3 rounded-2xl hover:bg-slate-50 transition-all font-bold text-xs uppercase tracking-widest shadow-lg active:scale-95 disabled:opacity-50"
        >
          <RefreshCw size={16} className={cn(isFetching && "animate-spin")} />
          {t("common.refresh")}
        </button>
      </div>

      {/* Stats Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-6 gap-6">
        {statCards.map((stat, i) => (
          <div
            key={i}
            className="bg-white p-6 rounded-[2rem] border border-slate-100 shadow-sm hover:shadow-xl transition-all group relative overflow-hidden"
          >
            <div
              className={cn(
                "w-12 h-12 rounded-xl flex items-center justify-center mb-4 border",
                stat.color
              )}
            >
              <stat.icon size={22} />
            </div>
            <p className="text-[9px] font-bold text-slate-500 uppercase tracking-[0.2em] mb-1 opacity-70">
              {stat.label}
            </p>
            <p className="text-3xl font-bold text-slate-700 tracking-tighter">
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

      {/* Live Indicator */}
      <div className="flex items-center gap-3 px-2">
        <div className="relative flex h-3 w-3">
          <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
          <span className="relative inline-flex rounded-full h-3 w-3 bg-emerald-500"></span>
        </div>
        <span className="text-xs font-bold text-slate-500 uppercase tracking-widest">
          {t("monitor.live_updates")}
        </span>
        {isFetching && (
          <Loader2 size={14} className="text-slate-400 animate-spin" />
        )}
      </div>

      {/* Workers Table */}
      <div className="bg-white border border-slate-100 rounded-[2.5rem] overflow-hidden shadow-sm">
        <div className="p-6 border-b border-slate-100">
          <h4 className="text-lg font-bold text-slate-700 uppercase tracking-tight">
            {t("monitor.worker_list")}
          </h4>
          <p className="text-[10px] font-bold text-slate-500 uppercase tracking-[0.2em] mt-1 opacity-60">
            {t("monitor.worker_list_subtitle")}
          </p>
        </div>

        {workers.length === 0 ? (
          <div className="p-12 text-center">
            <Server size={48} className="mx-auto text-slate-200 mb-4" />
            <p className="text-sm font-bold text-slate-400 uppercase tracking-widest">
              {t("monitor.no_workers")}
            </p>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="bg-slate-50/50">
                  <th className="text-left px-6 py-4 text-[10px] font-bold text-slate-500 uppercase tracking-widest">
                    {t("monitor.worker_id")}
                  </th>
                  <th className="text-left px-6 py-4 text-[10px] font-bold text-slate-500 uppercase tracking-widest">
                    {t("monitor.host")}
                  </th>
                  <th className="text-left px-6 py-4 text-[10px] font-bold text-slate-500 uppercase tracking-widest">
                    {t("monitor.status")}
                  </th>
                  <th className="text-left px-6 py-4 text-[10px] font-bold text-slate-500 uppercase tracking-widest">
                    {t("monitor.current_task")}
                  </th>
                  <th className="text-left px-6 py-4 text-[10px] font-bold text-slate-500 uppercase tracking-widest">
                    {t("monitor.last_seen")}
                  </th>
                </tr>
              </thead>
              <tbody>
                {workers.map((worker) => (
                  <tr
                    key={worker.id}
                    className="border-t border-slate-50 hover:bg-slate-50/50 transition-colors"
                  >
                    <td className="px-6 py-4">
                      <code className="text-xs font-mono text-slate-600 bg-slate-100 px-2 py-1 rounded">
                        {worker.id.slice(0, 8)}...
                      </code>
                    </td>
                    <td className="px-6 py-4">
                      <div className="flex items-center gap-2">
                        <Cpu size={14} className="text-slate-400" />
                        <span className="text-sm font-medium text-slate-700">
                          {worker.host}
                        </span>
                      </div>
                    </td>
                    <td className="px-6 py-4">
                      <span
                        className={cn(
                          "inline-flex items-center gap-2 px-3 py-1 rounded-full text-xs font-bold uppercase tracking-wider",
                          worker.status === "PROCESSING"
                            ? "bg-emerald-50 text-emerald-600"
                            : "bg-slate-100 text-slate-500"
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
                        <span className="text-sm font-medium text-slate-700">
                          {worker.current_task_type}
                        </span>
                      ) : (
                        <span className="text-sm text-slate-400">—</span>
                      )}
                    </td>
                    <td className="px-6 py-4">
                      <span className="text-sm text-slate-500">
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
    </div>
  );
}
