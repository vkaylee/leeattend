import { Briefcase } from "lucide-react";
import React, { useMemo, useState } from "react";

import { formatDistanceToNow } from "date-fns";
import { TFunction } from "i18next";

import ResponsiveTable from "@/components/ui/responsive-table";
import { Typography } from "@/components/ui/typography";
import { useDebounce } from "@/hooks/use-debounce";
import { useInfiniteScroll } from "@/hooks/use-infinite-scroll";
import { useWorkerJobs } from "@/hooks/use-worker-monitor";
import { RecentJob } from "@/lib/api/types";
import { cn } from "@/lib/utils";

import { getStatusStyle, JobDetailsDrawer } from "./job-details-drawer";

interface JobsTableProps {
  t: TFunction;
  uniqueTypes: string[];
}

function JobMobileCard({ job, t, onClick }: {
  formatDate: (d: Date, o?: { addSuffix?: boolean }) => string;
  job: RecentJob;
  onClick: () => void;
  t: TFunction;
}) {
  const taskTypeKey = job.task_type ?? job.report_type;
  const displayType = taskTypeKey
    ? t(`monitor.task_types.${taskTypeKey}`, { defaultValue: taskTypeKey })
    : "—";

  const { className: statusClassName, icon: statusIcon } = getStatusStyle(job.status);

  return (
    <div
      className="cursor-pointer rounded-2xl border border-slate-100 bg-white p-5 shadow-sm transition-all active:scale-[0.99] dark:border-slate-800 dark:bg-slate-900"
      onClick={onClick}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          onClick();
        }
      }}
      role="button"
      tabIndex={0}
    >
      <div className="flex items-start justify-between gap-3">
        <code className="rounded bg-slate-100 px-2 py-1 font-mono text-xs text-slate-600 dark:bg-slate-700 dark:text-slate-300">
          {job.job_id.slice(0, 8)}…
        </code>
        <span className={cn("inline-flex items-center gap-1.5 rounded-full px-2.5 py-0.5 text-[9px] font-bold uppercase tracking-wider", statusClassName)}>
          {statusIcon}
          {job.status}
        </span>
      </div>

      <div className="mt-3 grid grid-cols-3 gap-2 border-t border-slate-100 pt-3 dark:border-slate-800">
        <div>
          <div className="text-[9px] font-bold uppercase tracking-wider text-slate-400">Type</div>
          <div className="mt-0.5 text-xs font-medium text-slate-700 dark:text-slate-200">{displayType}</div>
        </div>
        <div>
          <div className="text-[9px] font-bold uppercase tracking-wider text-slate-400">Batches</div>
          <div className="mt-0.5 text-xs text-slate-700 dark:text-slate-200">
            {job.completed_batches}/{job.total_batches}
            {job.failed_batches > 0 ? <span className="text-rose-500 ml-1">({job.failed_batches})</span> : null}
          </div>
        </div>
        <div>
          <div className="text-[9px] font-bold uppercase tracking-wider text-slate-400">Created</div>
          <div className="mt-0.5 text-xs text-slate-500 dark:text-slate-400">
            {formatDistanceToNow(new Date(job.created_at), { addSuffix: true })}
          </div>
        </div>
      </div>

      {job.error_message ? (
        <div className="mt-2 truncate rounded-lg border border-rose-100 bg-rose-50 px-3 py-1.5 text-[10px] font-medium text-rose-600 dark:border-rose-800 dark:bg-rose-950/30 dark:text-rose-400">
          {job.error_message}
        </div>
      ) : null}
    </div>
  );
}

function SkeletonRow() {
  return (
    <tr className="border-t border-slate-50 dark:border-slate-800">
      <td className="px-6 py-4"><div className="h-4 w-32 bg-slate-100 dark:bg-slate-800 rounded animate-pulse" /></td>
      <td className="px-6 py-4"><div className="h-6 w-20 bg-slate-100 dark:bg-slate-800 rounded-full animate-pulse" /></td>
      <td className="px-6 py-4"><div className="h-4 w-24 bg-slate-100 dark:bg-slate-800 rounded animate-pulse" /></td>
      <td className="px-6 py-4"><div className="h-4 w-16 bg-slate-100 dark:bg-slate-800 rounded animate-pulse" /></td>
      <td className="px-6 py-4"><div className="h-4 w-28 bg-slate-100 dark:bg-slate-800 rounded animate-pulse" /></td>
      <td className="px-6 py-4"><div className="h-4 w-40 bg-slate-100 dark:bg-slate-800 rounded animate-pulse" /></td>
    </tr>
  );
}

// eslint-disable-next-line max-lines-per-function -- table component with search, pagination, status filters, modals
export function JobsTable({ uniqueTypes, t }: JobsTableProps) {
  const [selectedJob, setSelectedJob] = useState<RecentJob | null>(null);
  const [searchTerm, setSearchTerm] = useState("");
  const [statusFilter, setStatusFilter] = useState("all");
  const [typeFilter, setTypeFilter] = useState("all");

  const debouncedSearchTerm = useDebounce(searchTerm, 300);

  const queryParams = useMemo(() => ({
    limit: 20,
    search: debouncedSearchTerm || undefined,
    status: statusFilter === "all" ? undefined : statusFilter,
    task_type: typeFilter === "all" ? undefined : typeFilter,
  }), [debouncedSearchTerm, statusFilter, typeFilter]);

  const {
    data,
    fetchNextPage,
    hasNextPage,
    isFetchingNextPage,
    isLoading,
  } = useWorkerJobs(queryParams);

  const jobs = useMemo(() => {
    return data?.pages.flatMap((page) => page.jobs) ?? [];
  }, [data]);

  const mobileLoadMoreRef = useInfiniteScroll<HTMLDivElement>({
    fetchNextPage: () => { void fetchNextPage(); },
    hasNextPage,
    isFetchingNextPage,
  });

  return (
    <>
      <div className="overflow-hidden rounded-[2.5rem] border border-slate-100 bg-white shadow-sm dark:border-slate-800 dark:bg-slate-900 dark:shadow-black/10">
        {/* Filter Bar */}
        <div className="space-y-4 border-b border-slate-100 p-6 dark:border-slate-800">
          <div>
            <Typography className="text-lg font-bold uppercase tracking-tight text-slate-700 dark:text-slate-200" variant="h4">
              {t("monitor.recent_jobs", { defaultValue: "Recent Jobs" })}
            </Typography>
            <Typography className="mt-1 text-[10px] font-bold uppercase tracking-[0.2em] text-slate-500 opacity-60 dark:text-slate-400" variant="p">
              {t("monitor.recent_jobs_subtitle", { defaultValue: "Job metadata from last 24 hours" })}
            </Typography>
          </div>

          <div className="flex flex-col gap-3 md:flex-row pt-2">
            <div className="flex-1">
              <input
                className="w-full rounded-2xl border border-slate-100 bg-slate-50 px-4 py-2.5 text-sm text-slate-700 placeholder:text-slate-400 transition-all focus:outline-none focus:ring-2 focus:ring-blue-500/30 dark:border-slate-800/80 dark:bg-slate-800/40 dark:text-slate-200 dark:placeholder:text-slate-500"
                onChange={(e) => { setSearchTerm(e.target.value); }}
                placeholder={t("monitor.filter_search_placeholder", { defaultValue: "Search Job ID or error message..." })}
                type="text"
                value={searchTerm}
              />
            </div>
            <div className="w-full md:w-48">
              <select
                className="w-full cursor-pointer rounded-2xl border border-slate-100 bg-slate-50 px-4 py-2.5 text-sm text-slate-700 transition-all focus:outline-none focus:ring-2 focus:ring-blue-500/30 dark:border-slate-800/80 dark:bg-slate-800/40 dark:text-slate-200"
                onChange={(e) => { setStatusFilter(e.target.value); }}
                value={statusFilter}
              >
                <option value="all">{t("monitor.filter_status_all", { defaultValue: "All Statuses" })}</option>
                <option value="completed">{t("monitor.status_completed", { defaultValue: "Completed" })}</option>
                <option value="failed">{t("monitor.status_failed", { defaultValue: "Failed" })}</option>
                <option value="processing">{t("monitor.status_processing", { defaultValue: "Processing" })}</option>
                <option value="pending">{t("monitor.status_pending", { defaultValue: "Pending" })}</option>
              </select>
            </div>
            <div className="w-full md:w-56">
              <select
                className="w-full cursor-pointer rounded-2xl border border-slate-100 bg-slate-50 px-4 py-2.5 text-sm text-slate-700 transition-all focus:outline-none focus:ring-2 focus:ring-blue-500/30 dark:border-slate-800/80 dark:bg-slate-800/40 dark:text-slate-200"
                onChange={(e) => { setTypeFilter(e.target.value); }}
                value={typeFilter}
              >
                <option value="all">{t("monitor.filter_type_all", { defaultValue: "All Types" })}</option>
                {uniqueTypes.map((type) => (
                  <option key={type} value={type}>
                    {t(`monitor.task_types.${type}`, { defaultValue: type })}
                  </option>
                ))}
              </select>
            </div>
          </div>
        </div>

        <ResponsiveTable<RecentJob>
          data={jobs}
          emptyState={{
            icon: <Briefcase className="h-10 w-10" />,
            title: t("monitor.no_matching_jobs", { defaultValue: "No jobs match your search filters" }),
          }}
          getCardKey={(job) => job.job_id}
          isLoading={isLoading}
          mobileFooter={hasNextPage ? (
            <div className="flex min-h-10 items-center justify-center" ref={mobileLoadMoreRef}>
              {isFetchingNextPage ? (
                <span className="text-[10px] font-bold uppercase tracking-widest text-slate-400">
                  {t("common.loading")}
                </span>
              ) : null}
            </div>
          ) : undefined}
          renderCard={(job) => (
            <JobMobileCard
              formatDate={formatDistanceToNow}
              job={job}
              onClick={() => { setSelectedJob(job); }}
              t={t}
            />
          )}
          renderDesktopTable={() => (
            <div className="overflow-x-auto">
              <table className="w-full">
                <thead>
                  <tr className="bg-slate-50 dark:bg-slate-800/50">
                    <th className="px-6 py-4 text-left text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400">
                      {t("monitor.job_id", { defaultValue: "Job ID" })}
                    </th>
                    <th className="px-6 py-4 text-left text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400">
                      {t("monitor.status")}
                    </th>
                    <th className="px-6 py-4 text-left text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400">
                      {t("monitor.job_type", { defaultValue: "Type" })}
                    </th>
                    <th className="px-6 py-4 text-left text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400">
                      {t("monitor.batches", { defaultValue: "Batches" })}
                    </th>
                    <th className="px-6 py-4 text-left text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400">
                      {t("monitor.created_at", { defaultValue: "Created" })}
                    </th>
                    <th className="px-6 py-4 text-left text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400">
                      {t("monitor.error", { defaultValue: "Error" })}
                    </th>
                  </tr>
                </thead>
                <tbody>
                  {isLoading ? (
                    Array.from({ length: 5 }).map((_, i) => <SkeletonRow key={`skel-${i}`} />)
                  ) : (
                    jobs.map((job) => {
                      const taskTypeKey = job.task_type ?? job.report_type;
                      const displayType = taskTypeKey
                        ? t(`monitor.task_types.${taskTypeKey}`, { defaultValue: taskTypeKey })
                        : "—";

                      const { className: statusClassName, icon: statusIcon } = getStatusStyle(job.status);

                      return (
                        <tr
                          className="cursor-pointer border-t border-slate-50 transition-colors hover:bg-slate-50 focus:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-primary-500 dark:border-slate-800 dark:hover:bg-slate-800/50"
                          key={job.job_id}
                          onClick={() => { setSelectedJob(job); }}
                          onKeyDown={(e) => {
                            if (e.key === "Enter" || e.key === " ") {
                              e.preventDefault();
                              setSelectedJob(job);
                            }
                          }}
                          tabIndex={0}
                        >
                          <td className="px-6 py-4">
                            <code className="font-mono text-xs text-slate-600 dark:text-slate-400">
                              {job.job_id.slice(0, 8)}…
                            </code>
                          </td>
                          <td className="px-6 py-4">
                            <span className={cn("inline-flex items-center gap-2 rounded-full px-3 py-1 text-xs font-bold uppercase tracking-wider", statusClassName)}>
                              {statusIcon}
                              {job.status}
                            </span>
                          </td>
                          <td className="px-6 py-4">
                            <span className="text-sm font-medium text-slate-700 dark:text-slate-200">{displayType}</span>
                          </td>
                          <td className="px-6 py-4">
                            <span className="text-sm text-slate-700 dark:text-slate-200">
                              {job.completed_batches}/{job.total_batches}
                              {job.failed_batches > 0 ? <span className="ml-1 text-rose-500">({job.failed_batches} failed)</span> : null}
                            </span>
                          </td>
                          <td className="px-6 py-4">
                            <span className="text-sm text-slate-500 dark:text-slate-400">
                              {formatDistanceToNow(new Date(job.created_at), { addSuffix: true })}
                            </span>
                          </td>
                          <td className="max-w-[200px] px-6 py-4">
                            {job.error_message ? (
                              <span className="block cursor-help truncate text-xs text-rose-500 dark:text-rose-400" title={job.error_message}>
                                {job.error_message}
                              </span>
                            ) : (
                              <span className="text-sm text-slate-400 dark:text-slate-500">—</span>
                            )}
                          </td>
                        </tr>
                      );
                    })
                  )}
                </tbody>
              </table>
            </div>
          )}
        />

        {!isLoading && jobs.length > 0 ? (
          <div className="flex items-center justify-between border-t border-slate-100 bg-slate-50/50 px-6 py-4 text-xs text-slate-500 dark:border-slate-800/80 dark:bg-slate-800/20 dark:text-slate-400">
            <span>
              {t("monitor.pagination_loaded", {
                defaultValue: "Loaded {{count}} jobs",
                count: jobs.length,
              })}
            </span>
          </div>
        ) : null}
      </div>

      <JobDetailsDrawer
        isOpen={selectedJob !== null}
        job={selectedJob}
        onClose={() => { setSelectedJob(null); }}
        t={t}
      />
    </>
  );
}
