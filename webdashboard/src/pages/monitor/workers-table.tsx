import { Coffee, Cpu, Server, Zap } from "lucide-react";
import React from "react";

import { formatDistanceToNow } from "date-fns";
import { TFunction } from "i18next";

import ResponsiveTable from "@/components/ui/responsive-table";
import { Typography } from "@/components/ui/typography";
import { WorkerHeartbeat } from "@/lib/api/types";
import { cn } from "@/lib/utils";

interface WorkersTableProps {
  t: TFunction;
  workers: WorkerHeartbeat[];
}

function WorkerMobileCard({ worker, formatDate }: {
  formatDate: (d: Date, o?: { addSuffix?: boolean }) => string;
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
            {worker.status === "PROCESSING" ? <Zap size={10} /> : <Coffee size={10} />}
            {worker.status}
          </span>
        </div>
        <div className="mt-2 flex items-center gap-2 text-sm font-medium text-slate-700 dark:text-slate-200">
          <span>{worker.host}</span>
        </div>
        <div className="mt-3 grid grid-cols-2 gap-2 border-t border-slate-100 pt-3 dark:border-slate-800">
          <div>
            <div className="text-[9px] font-bold uppercase tracking-wider text-slate-400">Task</div>
            <div className="mt-0.5 text-xs font-medium text-slate-700 dark:text-slate-200">
              {worker.current_task_type ?? "—"}
            </div>
          </div>
          <div>
            <div className="text-[9px] font-bold uppercase tracking-wider text-slate-400">Last Seen</div>
            <div className="mt-0.5 text-xs text-slate-500 dark:text-slate-400">
              {formatDate(new Date(worker.last_seen_at), { addSuffix: true })}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

export function WorkersTable({ t, workers }: WorkersTableProps) {
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
          title: t("monitor.no_workers"),
        }}
        getCardKey={(w) => w.id}
        renderCard={(worker) => (
          <WorkerMobileCard formatDate={formatDistanceToNow} worker={worker} />
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
                    {t("monitor.current_task")}
                  </th>
                  <th className="px-6 py-4 text-left text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400">
                    {t("monitor.last_seen")}
                  </th>
                </tr>
              </thead>
              <tbody>
                {workers.map((worker) => (
                  <tr
                    className="border-t border-slate-50 transition-colors hover:bg-slate-50 dark:border-slate-800 dark:hover:bg-slate-800/50"
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
                        {worker.status === "PROCESSING" ? <Zap size={12} /> : <Coffee size={12} />}
                        {worker.status}
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
