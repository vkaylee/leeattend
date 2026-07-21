import { Coffee, Cpu, Server, Zap } from "lucide-react";
import React from "react";

import { formatDistanceToNow } from "date-fns";
import { TFunction } from "i18next";

import { Typography } from "@/components/ui/typography";
import { WorkerHeartbeat } from "@/lib/api/types";
import { cn } from "@/lib/utils";

interface WorkersTableProps {
  t: TFunction;
  workers: WorkerHeartbeat[];
}

export function WorkersTable({ t, workers }: WorkersTableProps) {
  return (
    <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-2xl sm:rounded-[2.5rem] overflow-hidden shadow-sm dark:shadow-black/10">
      <div className="p-6 border-b border-slate-100 dark:border-slate-800">
        <Typography className="text-lg font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight" variant="h4">
          {t("monitor.worker_list")}
        </Typography>
        <Typography className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em] mt-1 opacity-60" variant="p">
          {t("monitor.worker_list_subtitle")}
        </Typography>
      </div>

      {workers.length === 0 ? (
        <div className="p-12 text-center">
          <Server className="mx-auto text-slate-200 dark:text-slate-700 mb-4" size={48} />
          <Typography className="text-sm font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest" variant="p">
            {t("monitor.no_workers")}
          </Typography>
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
                <WorkerRow formatDate={formatDistanceToNow} key={worker.id} worker={worker} />
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}

function WorkerRow({
  formatDate,
  worker,
}: {
  worker: WorkerHeartbeat;
  formatDate: (d: Date, o?: { addSuffix?: boolean }) => string;
}) {
  return (
    <tr className="border-t border-slate-50 dark:border-slate-800 hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-colors">
      <td className="px-6 py-4">
        <code
          className="text-xs font-mono text-slate-600 dark:text-slate-300 bg-slate-100 dark:bg-slate-700 px-2 py-1 rounded cursor-help"
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
            "inline-flex items-center gap-2 px-3 py-1 rounded-full text-xs font-bold uppercase tracking-wider",
            worker.status === "PROCESSING"
              ? "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300"
              : "bg-slate-100 dark:bg-slate-700 text-slate-500 dark:text-slate-400"
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
          {formatDate(new Date(worker.last_seen_at), {
            addSuffix: true,
          })}
        </span>
      </td>
    </tr>
  );
}
