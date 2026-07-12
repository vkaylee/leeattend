import { Briefcase } from "lucide-react";
import React, { useMemo, useState } from "react";

import { formatDistanceToNow } from "date-fns";
import { TFunction } from "i18next";

import InfiniteScrollSentinel from "@/components/common/infinite-scroll-sentinel";
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

interface FilterBarProps {
  searchTerm: string;
  setSearchTerm: (s: string) => void;
  setStatusFilter: (s: string) => void;
  setTypeFilter: (s: string) => void;
  statusFilter: string;
  t: TFunction;
  typeFilter: string;
  uniqueTypes: string[];
}

function FilterBar({
  searchTerm,
  setSearchTerm,
  statusFilter,
  setStatusFilter,
  t,
  typeFilter,
  setTypeFilter,
  uniqueTypes,
}: FilterBarProps) {
  return (
    <div className="p-6 border-b border-slate-100 dark:border-slate-800 space-y-4">
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <Typography className="text-lg font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tight" variant="h4">
            {t("monitor.recent_jobs", { defaultValue: "Recent Jobs" })}
          </Typography>
          <Typography className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em] mt-1 opacity-60" variant="p">
            {t("monitor.recent_jobs_subtitle", { defaultValue: "Job metadata from last 24 hours" })}
          </Typography>
        </div>
      </div>

      <div className="flex flex-col md:flex-row gap-3 pt-2">
        <div className="flex-1">
          <input
            className="w-full bg-slate-50 dark:bg-slate-800/40 border border-slate-100 dark:border-slate-800/80 rounded-2xl px-4 py-2.5 text-sm text-slate-700 dark:text-slate-200 placeholder:text-slate-400 dark:placeholder:text-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500/30 transition-all"
            onChange={(e) => {
              setSearchTerm(e.target.value);
            }}
            placeholder={t("monitor.filter_search_placeholder", { defaultValue: "Search Job ID or error message..." })}
            type="text"
            value={searchTerm}
          />
        </div>
        <div className="w-full md:w-48">
          <select
            className="w-full bg-slate-50 dark:bg-slate-800/40 border border-slate-100 dark:border-slate-800/80 rounded-2xl px-4 py-2.5 text-sm text-slate-700 dark:text-slate-200 focus:outline-none focus:ring-2 focus:ring-blue-500/30 transition-all cursor-pointer"
            onChange={(e) => {
              setStatusFilter(e.target.value);
            }}
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
            className="w-full bg-slate-50 dark:bg-slate-800/40 border border-slate-100 dark:border-slate-800/80 rounded-2xl px-4 py-2.5 text-sm text-slate-700 dark:text-slate-200 focus:outline-none focus:ring-2 focus:ring-blue-500/30 transition-all cursor-pointer"
            onChange={(e) => {
              setTypeFilter(e.target.value);
            }}
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
  );
}

function InfiniteScrollFooter({
  loadedCount,
  t,
}: {
  loadedCount: number;
  t: TFunction;
}) {
  return (
    <div className="px-6 py-4 bg-slate-50/50 dark:bg-slate-800/20 border-t border-slate-100 dark:border-slate-800/80 flex items-center justify-between text-xs text-slate-500 dark:text-slate-400">
      <span>
        {t("monitor.pagination_loaded", {
          defaultValue: "Loaded {{count}} jobs",
          count: loadedCount,
        })}
      </span>
    </div>
  );
}

interface JobsTableListProps {
  hasNextPage: boolean;
  isFetchingNextPage: boolean;
  loadMoreRef: React.RefObject<HTMLTableRowElement | null>;
  setSelectedJob: (job: RecentJob) => void;
  t: TFunction;
  visibleJobs: RecentJob[];
}

function JobsTableList({
  hasNextPage,
  isFetchingNextPage,
  loadMoreRef,
  setSelectedJob,
  t,
  visibleJobs,
}: JobsTableListProps) {
  return (
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
          {visibleJobs.map((job) => (
            <JobRow
              formatDate={formatDistanceToNow}
              job={job}
              key={job.job_id}
              onClick={() => { setSelectedJob(job); }}
              t={t}
            />
          ))}
          {hasNextPage ? (
            <InfiniteScrollSentinel
              colSpan={6}
              isFetchingNextPage={isFetchingNextPage}
              sentinelRef={loadMoreRef}
            />
          ) : null}
        </tbody>
      </table>
    </div>
  );
}

function JobsTableSkeleton({ t }: { t: TFunction }) {
  return (
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
          {Array.from({ length: 5 }).map((_, i) => (
            <tr className="border-t border-slate-50 dark:border-slate-800" key={i}>
              <td className="px-6 py-4"><div className="h-4 w-32 bg-slate-100 dark:bg-slate-800 rounded animate-pulse" /></td>
              <td className="px-6 py-4"><div className="h-6 w-20 bg-slate-100 dark:bg-slate-800 rounded-full animate-pulse" /></td>
              <td className="px-6 py-4"><div className="h-4 w-24 bg-slate-100 dark:bg-slate-800 rounded animate-pulse" /></td>
              <td className="px-6 py-4"><div className="h-4 w-16 bg-slate-100 dark:bg-slate-800 rounded animate-pulse" /></td>
              <td className="px-6 py-4"><div className="h-4 w-28 bg-slate-100 dark:bg-slate-800 rounded animate-pulse" /></td>
              <td className="px-6 py-4"><div className="h-4 w-40 bg-slate-100 dark:bg-slate-800 rounded animate-pulse" /></td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

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

  const loadMoreRef = useInfiniteScroll<HTMLTableRowElement>({
    fetchNextPage: () => { void fetchNextPage(); },
    hasNextPage,
    isFetchingNextPage,
  });

  return (
    <>
      <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[2.5rem] overflow-hidden shadow-sm dark:shadow-black/10">
        <FilterBar
          searchTerm={searchTerm}
          setSearchTerm={setSearchTerm}
          setStatusFilter={setStatusFilter}
          setTypeFilter={setTypeFilter}
          statusFilter={statusFilter}
          t={t}
          typeFilter={typeFilter}
          uniqueTypes={uniqueTypes}
        />

        {isLoading ? (
          <JobsTableSkeleton t={t} />
        ) : jobs.length === 0 ? (
          <div className="p-12 text-center">
            <Briefcase className="mx-auto text-slate-200 dark:text-slate-700 mb-4" size={48} />
            <Typography className="text-sm font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest" variant="p">
              {t("monitor.no_matching_jobs", { defaultValue: "No jobs match your search filters" })}
            </Typography>
          </div>
        ) : (
          <JobsTableList
            hasNextPage={hasNextPage}
            isFetchingNextPage={isFetchingNextPage}
            loadMoreRef={loadMoreRef}
            setSelectedJob={setSelectedJob}
            t={t}
            visibleJobs={jobs}
          />
        )}

        {!isLoading && jobs.length > 0 && (
          <InfiniteScrollFooter
            loadedCount={jobs.length}
            t={t}
          />
        )}
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

function JobRow({
  formatDate,
  job,
  onClick,
  t,
}: {
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
    <tr
      className="border-t border-slate-50 dark:border-slate-800 hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-colors cursor-pointer focus:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-primary-500"
      onClick={onClick}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          onClick();
        }
      }}
      tabIndex={0}
    >
      <td className="px-6 py-4">
        <code className="text-xs font-mono text-slate-600 dark:text-slate-400">
          {job.job_id.slice(0, 8)}…
        </code>
      </td>
      <td className="px-6 py-4">
        <span
          className={cn(
            "inline-flex items-center gap-2 px-3 py-1 rounded-full text-xs font-bold uppercase tracking-wider",
            statusClassName
          )}
        >
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
