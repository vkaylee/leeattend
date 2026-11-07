import { Activity, CircleOff, Clock3, Coffee, Cpu, LucideIcon, Server, Zap } from "lucide-react";
import React from "react";

import { formatDistanceToNow } from "date-fns";
import { TFunction } from "i18next";

import ResponsiveTable from "@/components/ui/responsive-table";
import TableEmptyRow from "@/components/ui/table-empty-row";
import { Typography } from "@/components/ui/typography";
import { WorkerHeartbeat } from "@/lib/api/types";
import { cn } from "@/lib/utils";

interface WorkersTableProps {
  isUnavailable?: boolean;
  t: TFunction;
  workers: WorkerHeartbeat[];
}

type WorkerLiveness = NonNullable<WorkerHeartbeat["liveness"]>;

const LIVENESS_STYLES: Record<WorkerLiveness, { icon: LucideIcon; className: string }> = {
  live: {
    icon: Activity,
    className: "bg-emerald-50 text-emerald-600 dark:bg-emerald-950 dark:text-emerald-300",
  },
  stale: {
    icon: Clock3,
    className: "bg-amber-50 text-amber-600 dark:bg-amber-950 dark:text-amber-300",
  },
  offline: {
    icon: CircleOff,
    className: "bg-rose-50 text-rose-600 dark:bg-rose-950 dark:text-rose-300",
  },
};

function livenessLabel(t: TFunction, liveness: WorkerLiveness | undefined): string {
  return liveness
    ? t(`monitor.worker_liveness.${liveness}`, { defaultValue: t("monitor.unknown_status", { defaultValue: "Status unavailable" }) })
    : t("monitor.metric_unavailable", { defaultValue: "Unavailable" });
}

function workerStatusLabel(t: TFunction, status: string): string {
  const normalized = status.toLowerCase();
  const defaults: Record<string, string> = {
    idle: "Idle",
    processing: "Processing",
  };
  return t(`monitor.worker_status.${normalized}`, { defaultValue: defaults[normalized] ?? t("monitor.unknown_status", { defaultValue: "Status unavailable" }) });
}

function heartbeatAgeLabel(t: TFunction, age: number | null | undefined): string {
  return age === null || age === undefined
    ? "—"
    : t("monitor.heartbeat_age", { defaultValue: `${age}s since heartbeat`, seconds: age });
}

function LivenessBadge({ liveness, t }: { liveness: WorkerLiveness | undefined; t: TFunction }) {
  if (!liveness) {
    return (
      <span className="inline-flex items-center gap-1.5 rounded-full bg-slate-100 px-2.5 py-0.5 text-[9px] font-bold uppercase tracking-wider text-slate-500 dark:bg-slate-700 dark:text-slate-400">
        <CircleOff aria-hidden="true" size={10} />
        {livenessLabel(t, liveness)}
      </span>
    );
  }

  const style = LIVENESS_STYLES[liveness];
  const Icon = style.icon;
  const label = livenessLabel(t, liveness);
  return (
    <span
      aria-label={t("monitor.worker_liveness_label", { defaultValue: `Worker liveness: ${label}`, status: label })}
      className={cn(
        "inline-flex items-center gap-1.5 rounded-full px-2.5 py-0.5 text-[9px] font-bold uppercase tracking-wider",
        style.className
      )}
    >
      <Icon aria-hidden="true" size={10} />
      {label}
    </span>
  );
}

function WorkerMobileCard({ t, worker, formatDate }: {
  formatDate: (d: Date, o?: { addSuffix?: boolean }) => string;
  t: TFunction;
  worker: WorkerHeartbeat;
}) {
  return (
    <div className="flex items-start gap-4 rounded-2xl border border-slate-100 bg-white p-5 shadow-sm dark:border-slate-800 dark:bg-slate-900">
      <div className="flex h-11 w-11 shrink-0 items-center justify-center rounded-xl border border-slate-100 bg-slate-50 text-slate-400 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-500">
        <Cpu size={18} />
      </div>
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2">
          <code className="rounded bg-slate-100 px-2 py-0.5 font-mono text-xs text-slate-600 dark:bg-slate-700 dark:text-slate-300">
            {worker.id.slice(0, 8)}...
          </code>
          <span
            className={cn(
              "inline-flex items-center gap-1.5 rounded-full px-2.5 py-0.5 text-[9px] font-bold uppercase tracking-wider",
              worker.status === "PROCESSING"
                ? "bg-emerald-50 text-emerald-600 dark:bg-emerald-950 dark:text-emerald-300"
                : "bg-slate-100 text-slate-500 dark:bg-slate-700 dark:text-slate-400"
            )}
          >
            {worker.status === "PROCESSING" ? <Zap aria-hidden="true" size={10} /> : <Coffee aria-hidden="true" size={10} />}
            {workerStatusLabel(t, worker.status)}
          </span>
          <LivenessBadge liveness={worker.liveness} t={t} />
        </div>
        <div className="mt-2 flex items-center gap-2 text-sm font-medium text-slate-700 dark:text-slate-200">
          <span>{worker.host}</span>
        </div>
        <div className="mt-3 grid grid-cols-2 gap-2 border-t border-slate-100 pt-3 dark:border-slate-800">
          <div>
            <div className="text-[9px] font-bold uppercase tracking-wider text-slate-400">{t("monitor.current_task", { defaultValue: "Task" })}</div>
            <div className="mt-0.5 text-xs font-medium text-slate-700 dark:text-slate-200">
              {worker.current_task_type ?? "—"}
            </div>
          </div>
          <div>
            <div className="text-[9px] font-bold uppercase tracking-wider text-slate-400">{t("monitor.last_seen", { defaultValue: "Last seen" })}</div>
            <div className="mt-0.5 text-xs text-slate-500 dark:text-slate-400">
              {formatDate(new Date(worker.last_seen_at), { addSuffix: true })}
              <span className="block text-[10px] text-slate-400 dark:text-slate-500">
                {heartbeatAgeLabel(t, worker.heartbeat_age_seconds)}
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

export function WorkersTable({ isUnavailable = false, t, workers }: WorkersTableProps) {
  return (
    <div className="overflow-hidden rounded-2xl border border-slate-100 bg-white shadow-sm dark:border-slate-800 dark:bg-slate-900 sm:rounded-[2.5rem] dark:shadow-black/10">
      <div className="border-b border-slate-100 p-6 dark:border-slate-800">
        <Typography className="text-lg font-bold uppercase tracking-tight text-slate-700 dark:text-slate-200" variant="h4">
          {t("monitor.worker_list")}
        </Typography>
        <Typography className="mt-1 text-[10px] font-bold uppercase tracking-[0.2em] text-slate-500 opacity-60 dark:text-slate-400" variant="p">
          {t("monitor.worker_list_subtitle")}
        </Typography>
      </div>

      <ResponsiveTable<WorkerHeartbeat>
        data={workers}
        emptyState={{
          icon: <Server className="h-10 w-10" />,
          title: isUnavailable
            ? t("monitor.metric_unavailable", { defaultValue: "Worker heartbeat source unavailable" })
            : t("monitor.no_workers"),
        }}
        getCardKey={(w) => w.id}
        renderCard={(worker) => (
          <WorkerMobileCard formatDate={formatDistanceToNow} t={t} worker={worker} />
        )}
        renderDesktopTable={() => (
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="bg-slate-50 dark:bg-slate-800/50">
                  <th className="px-6 py-4 text-left text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400">
                    {t("monitor.worker_id")}
                  </th>
                  <th className="px-6 py-4 text-left text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400">
                    {t("monitor.host")}
                  </th>
                  <th className="px-6 py-4 text-left text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400">
                    {t("monitor.status")}
                  </th>
                  <th className="px-6 py-4 text-left text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400">
                    {t("monitor.liveness", { defaultValue: "Liveness" })}
                  </th>
                  <th className="px-6 py-4 text-left text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400">
                    {t("monitor.current_task")}
                  </th>
                  <th className="px-6 py-4 text-left text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400">
                    {t("monitor.last_seen")}
                  </th>
                </tr>
              </thead>
              <tbody>
                {workers.length === 0 ? (
                  <TableEmptyRow
                    colSpan={6}
                    description={t("monitor.no_workers_description", { defaultValue: "Không có worker nào đang kết nối tới cụm dịch vụ." })}
                    title={t("monitor.no_workers", { defaultValue: "Chưa có tiến trình worker nào hoạt động" })}
                  />
                ) : (
                  workers.map((worker) => (
                    <tr
                      className="border-t border-slate-50 motion-safe:transition-colors hover:bg-slate-50 dark:border-slate-800 dark:hover:bg-slate-800/50"
                      key={worker.id}
                    >
                    <td className="px-6 py-4">
                      <code
                        className="cursor-help rounded bg-slate-100 px-2 py-1 font-mono text-xs text-slate-600 dark:bg-slate-700 dark:text-slate-300"
                        title={worker.id}
                      >
                        {worker.id.slice(0, 8)}...
                      </code>
                    </td>
                    <td className="px-6 py-4">
                      <div className="flex items-center gap-2">
                        <Cpu className="text-slate-400 dark:text-slate-500" size={14} />
                        <span className="text-sm font-medium text-slate-700 dark:text-slate-200">{worker.host}</span>
                      </div>
                    </td>
                    <td className="px-6 py-4">
                      <span
                        className={cn(
                          "inline-flex items-center gap-2 rounded-full px-3 py-1 text-xs font-bold uppercase tracking-wider",
                          worker.status === "PROCESSING"
                            ? "bg-emerald-50 text-emerald-600 dark:bg-emerald-950 dark:text-emerald-300"
                            : "bg-slate-100 text-slate-500 dark:bg-slate-700 dark:text-slate-400"
                        )}
                      >
                        {worker.status === "PROCESSING" ? <Zap aria-hidden="true" size={12} /> : <Coffee aria-hidden="true" size={12} />}
                        {workerStatusLabel(t, worker.status)}
                      </span>
                    </td>
                    <td className="px-6 py-4">
                      <LivenessBadge liveness={worker.liveness} t={t} />
                      <span className="mt-1 block text-xs text-slate-500 dark:text-slate-400">
                        {heartbeatAgeLabel(t, worker.heartbeat_age_seconds)}
                      </span>
                    </td>
                    <td className="px-6 py-4">
                      {worker.current_task_type ? (
                        <span className="text-sm font-medium text-slate-700 dark:text-slate-200">{worker.current_task_type}</span>
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
      />
    </div>
  );
}
