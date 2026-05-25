import { Briefcase, CheckCircle2, Clock, Loader2, XCircle } from "lucide-react";
import React from "react";

import { formatDistanceToNow } from "date-fns";
import { TFunction } from "i18next";

import { Typography } from "@/components/ui/typography";
import { RecentJob } from "@/lib/api/types";
import { cn } from "@/lib/utils";

interface JobsTableProps {
  recentJobs: RecentJob[];
  t: TFunction;
}

export function JobsTable({ recentJobs, t }: JobsTableProps) {
  return (
    <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[2.5rem] overflow-hidden shadow-sm dark:shadow-black/10">
      <div className="p-6 border-b border-slate-100 dark:border-slate-800">
        <Typography className="text-lg font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight" variant="h4">
          {t("monitor.recent_jobs", { defaultValue: "Recent Jobs" })}
        </Typography>
        <Typography className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em] mt-1 opacity-60" variant="p">
          {t("monitor.recent_jobs_subtitle", { defaultValue: "Job metadata from last 24 hours" })}
        </Typography>
      </div>

      {recentJobs.length === 0 ? (
        <div className="p-12 text-center">
          <Briefcase className="mx-auto text-slate-200 dark:text-slate-700 mb-4" size={48} />
          <Typography className="text-sm font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest" variant="p">
            {t("monitor.no_jobs", { defaultValue: "No recent jobs" })}
          </Typography>
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
                <JobRow formatDate={formatDistanceToNow} job={job} key={job.job_id} />
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}

function JobRow({
  formatDate,
  job,
}: {
  job: RecentJob;
  formatDate: (d: Date, o?: { addSuffix?: boolean }) => string;
}) {
  return (
    <tr className="border-t border-slate-50 dark:border-slate-800 hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-colors">
      <td className="px-6 py-4">
        <code
          className="text-xs font-mono text-slate-600 dark:text-slate-300 bg-slate-100 dark:bg-slate-700 px-2 py-1 rounded cursor-help"
          title={job.job_id}
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
            <Loader2 className="animate-spin" size={12} />
          ) : (
            <Clock size={12} />
          )}
          {job.status}
        </span>
      </td>
      <td className="px-6 py-4">
        <span className="text-sm font-medium text-slate-700 dark:text-slate-200">{job.report_type}</span>
      </td>
      <td className="px-6 py-4">
        <span className="text-sm text-slate-700 dark:text-slate-200">
          {job.completed_batches}/{job.total_batches}
          {job.failed_batches > 0 ? <span className="text-rose-500 ml-1">({job.failed_batches} failed)</span> : null}
        </span>
      </td>
      <td className="px-6 py-4">
        <span className="text-sm text-slate-500 dark:text-slate-400">
          {formatDate(new Date(job.created_at), {
            addSuffix: true,
          })}
        </span>
      </td>
      <td className="px-6 py-4 max-w-[200px]">
        {job.error_message ? (
          <span className="text-xs text-rose-500 dark:text-rose-400 truncate block cursor-help" title={job.error_message}>
            {job.error_message}
          </span>
        ) : (
          <span className="text-sm text-slate-400 dark:text-slate-500">—</span>
        )}
      </td>
    </tr>
  );
}
