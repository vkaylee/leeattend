import { Briefcase, CheckCircle2, Clock, Copy, Loader2, ShieldAlert, XCircle } from "lucide-react";
import React, { useMemo, useState } from "react";

import { formatDistanceToNow } from "date-fns";
import { TFunction } from "i18next";

import InfiniteScrollSentinel from "@/components/common/infinite-scroll-sentinel";
import Button from "@/components/ui/button";
import Sheet from "@/components/ui/sheet";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { useDebounce } from "@/hooks/use-debounce";
import { useInfiniteScroll } from "@/hooks/use-infinite-scroll";
import { useWorkerJobs } from "@/hooks/use-worker-monitor";
import { RecentJob } from "@/lib/api/types";
import { cn } from "@/lib/utils";

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
          {Array.from({ length: 5 }).map((_, idx) => (
            <tr className="border-t border-slate-50 dark:border-slate-800" key={idx}>
              <td className="px-6 py-4">
                <div className="h-4 w-20 bg-slate-100 dark:bg-slate-800 rounded animate-pulse" />
              </td>
              <td className="px-6 py-4">
                <div className="h-6 w-24 bg-slate-100 dark:bg-slate-800 rounded-full animate-pulse" />
              </td>
              <td className="px-6 py-4">
                <div className="h-4 w-32 bg-slate-100 dark:bg-slate-800 rounded animate-pulse" />
              </td>
              <td className="px-6 py-4">
                <div className="h-4 w-12 bg-slate-100 dark:bg-slate-800 rounded animate-pulse" />
              </td>
              <td className="px-6 py-4">
                <div className="h-4 w-24 bg-slate-100 dark:bg-slate-800 rounded animate-pulse" />
              </td>
              <td className="px-6 py-4">
                <div className="h-4 w-28 bg-slate-100 dark:bg-slate-800 rounded animate-pulse" />
              </td>
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

interface StatusStyle {
  className: string;
  icon: React.ReactNode;
}

function getStatusStyle(status: string): StatusStyle {
  switch (status) {
    case "completed":
      return {
        className: "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300",
        icon: <CheckCircle2 size={12} />,
      };
    case "failed":
      return {
        className: "bg-rose-50 dark:bg-rose-950 text-rose-600 dark:text-rose-300",
        icon: <XCircle size={12} />,
      };
    case "processing":
      return {
        className: "bg-amber-50 dark:bg-amber-950 text-amber-600 dark:text-amber-300",
        icon: <Loader2 className="animate-spin" size={12} />,
      };
    default:
      return {
        className: "bg-slate-100 dark:bg-slate-700 text-slate-500 dark:text-slate-400",
        icon: <Clock size={12} />,
      };
  }
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

interface JobDetailsDrawerProps {
  isOpen: boolean;
  job: RecentJob | null;
  onClose: () => void;
  t: TFunction;
}

function JobHeaderCard({
  jobId,
  onCopy,
  t,
}: {
  jobId: string;
  onCopy: (text: string, msg: string) => void;
  t: TFunction;
}) {
  return (
    <div className="bg-slate-50 dark:bg-slate-800/50 rounded-[2rem] p-6 border border-slate-100 dark:border-slate-800 space-y-4">
      <div className="flex items-center justify-between">
        <Typography className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest" variant="p">
          {t("monitor.job_id", { defaultValue: "Job ID" })}
        </Typography>
        <Button
          aria-label={t("monitor.copy_job_id", { defaultValue: "Copy Job ID" })}
          className="p-2 hover:bg-slate-100 dark:hover:bg-slate-700 rounded-lg transition-colors text-slate-500 dark:text-slate-400 h-auto min-w-0"
          onClick={() => {
            onCopy(jobId, t("monitor.job_id_copied", { defaultValue: "Copied Job ID!" }));
          }}
          variant="ghost"
        >
          <Copy size={16} />
        </Button>
      </div>
      <code className="text-sm font-mono text-slate-700 dark:text-slate-300 break-all block bg-white dark:bg-slate-900 p-3 rounded-xl border border-slate-100 dark:border-slate-800">
        {jobId}
      </code>
    </div>
  );
}

function JobStatusCard({
  durationText,
  status,
  t,
}: {
  durationText: string;
  status: string;
  t: TFunction;
}) {
  const { className: statusClassName, icon: statusIcon } = getStatusStyle(status);

  return (
    <div className="grid grid-cols-2 gap-4">
      <div className="bg-slate-50 dark:bg-slate-800/50 rounded-[2rem] p-6 border border-slate-100 dark:border-slate-800 flex flex-col justify-between h-28">
        <Typography className="text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest" variant="p">
          {t("monitor.status", { defaultValue: "Status" })}
        </Typography>
        <div className="mt-2">
          <span
            className={cn(
              "inline-flex items-center gap-2 px-3 py-1.5 rounded-full text-xs font-bold uppercase tracking-wider",
              statusClassName
            )}
          >
            {statusIcon}
            {status}
          </span>
        </div>
      </div>

      <div className="bg-slate-50 dark:bg-slate-800/50 rounded-[2rem] p-6 border border-slate-100 dark:border-slate-800 flex flex-col justify-between h-28">
        <Typography className="text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest" variant="p">
          {t("monitor.duration", { defaultValue: "Duration" })}
        </Typography>
        <Typography className="text-lg font-bold text-slate-700 dark:text-slate-200 mt-2" variant="h4">
          {durationText}
        </Typography>
      </div>
    </div>
  );
}

function JobBatchesCard({
  completed,
  failed,
  t,
  total,
}: {
  completed: number;
  failed: number;
  t: TFunction;
  total: number;
}) {
  return (
    <div className="bg-slate-50 dark:bg-slate-800/50 rounded-[2rem] p-6 border border-slate-100 dark:border-slate-800 space-y-4">
      <Typography className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest" variant="p">
        {t("monitor.batches", { defaultValue: "Batches Progress" })}
      </Typography>
      <div className="grid grid-cols-3 gap-2 text-center">
        <div className="bg-white dark:bg-slate-900 p-3 rounded-xl border border-slate-100 dark:border-slate-800">
          <Typography className="text-xs text-slate-400 uppercase" variant="p">
            {t("monitor.total", { defaultValue: "Total" })}
          </Typography>
          <Typography className="text-lg font-extrabold text-slate-700 dark:text-slate-200" variant="p">
            {total}
          </Typography>
        </div>
        <div className="bg-white dark:bg-slate-900 p-3 rounded-xl border border-slate-100 dark:border-slate-800">
          <Typography className="text-xs text-slate-400 uppercase" variant="p">
            {t("monitor.success", { defaultValue: "Success" })}
          </Typography>
          <Typography className="text-lg font-extrabold text-emerald-600 dark:text-emerald-400" variant="p">
            {completed}
          </Typography>
        </div>
        <div className="bg-white dark:bg-slate-900 p-3 rounded-xl border border-slate-100 dark:border-slate-800">
          <Typography className="text-xs text-slate-400 uppercase" variant="p">
            {t("monitor.failed", { defaultValue: "Failed" })}
          </Typography>
          <Typography className="text-lg font-extrabold text-rose-600 dark:text-rose-400" variant="p">
            {failed}
          </Typography>
        </div>
      </div>
    </div>
  );
}

function JobTimeInfoCard({
  completedAt,
  createdAt,
  requesterId,
  t,
}: {
  completedAt?: string | null;
  createdAt: string;
  requesterId?: string | null;
  t: TFunction;
}) {
  return (
    <div className="bg-slate-50 dark:bg-slate-800/50 rounded-[2rem] p-6 border border-slate-100 dark:border-slate-800 space-y-3 text-sm">
      <div className="flex justify-between">
        <span className="text-slate-400">{t("monitor.created_at", { defaultValue: "Created At" })}</span>
        <span className="text-slate-700 dark:text-slate-300 font-medium">
          {new Date(createdAt).toLocaleString()}
        </span>
      </div>
      {completedAt ? (
        <div className="flex justify-between">
          <span className="text-slate-400">{t("monitor.completed_at", { defaultValue: "Completed At" })}</span>
          <span className="text-slate-700 dark:text-slate-300 font-medium">
            {new Date(completedAt).toLocaleString()}
          </span>
        </div>
      ) : null}
      {requesterId ? (
        <div className="flex justify-between">
          <span className="text-slate-400">{t("monitor.triggered_by", { defaultValue: "Triggered By" })}</span>
          <span className="text-slate-700 dark:text-slate-300 font-mono text-xs">
            {requesterId}
          </span>
        </div>
      ) : null}
    </div>
  );
}

export function JobDetailsDrawer({ isOpen, onClose, job, t }: JobDetailsDrawerProps) {
  const toast = useToast();
  if (!job) return null;

  const taskTypeKey = job.task_type ?? job.report_type;
  const displayType = taskTypeKey
    ? t(`monitor.task_types.${taskTypeKey}`, { defaultValue: taskTypeKey })
    : "—";

  const start = new Date(job.created_at);
  const end = job.completed_at ? new Date(job.completed_at) : null;
  const durationMs = end ? end.getTime() - start.getTime() : null;

  const formatDuration = (ms: number) => {
    const seconds = Math.floor(ms / 1000);
    if (seconds < 60) return `${seconds}s`;
    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = seconds % 60;
    return `${minutes}m ${remainingSeconds}s`;
  };

  const copyToClipboard = (text: string, successMessage: string) => {
    void navigator.clipboard.writeText(text);
    toast.success(successMessage);
  };

  const durationText = durationMs !== null
    ? formatDuration(durationMs)
    : job.status === "processing"
      ? t("monitor.running", { defaultValue: "Running..." })
      : "—";

  return (
    <Sheet
      isOpen={isOpen}
      onClose={onClose}
      subtitle={displayType}
      title={t("monitor.job_details_title", { defaultValue: "Job Details" })}
    >
      <div className="space-y-8 pb-10">
        <JobHeaderCard jobId={job.job_id} onCopy={copyToClipboard} t={t} />

        <JobStatusCard durationText={durationText} status={job.status} t={t} />

        <JobBatchesCard
          completed={job.completed_batches}
          failed={job.failed_batches}
          t={t}
          total={job.total_batches}
        />

        <JobTimeInfoCard
          completedAt={job.completed_at}
          createdAt={job.created_at}
          requesterId={job.requester_id}
          t={t}
        />

        {job.error_message ? (
          <div className="bg-rose-50 dark:bg-rose-950/30 border border-rose-100 dark:border-rose-900/50 rounded-[2rem] p-6 space-y-3">
            <Typography className="text-xs font-bold text-rose-700 dark:text-rose-400 uppercase tracking-widest flex items-center gap-2" variant="h5">
              <ShieldAlert size={14} />
              {t("monitor.error_details", { defaultValue: "Error Details" })}
            </Typography>
            <pre className="text-xs font-mono text-rose-600 dark:text-rose-400 whitespace-pre-wrap break-all max-h-60 overflow-y-auto bg-white/60 dark:bg-black/20 p-4 rounded-xl border border-rose-100/50 dark:border-rose-900/30 leading-relaxed scrollbar-thin">
              {job.error_message}
            </pre>
          </div>
        ) : null}

        {job.payload ? (
          <div className="space-y-3">
            <Typography className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest" variant="h5">
              {t("monitor.payload_parameters", { defaultValue: "Payload Parameters" })}
            </Typography>
            <pre className="text-[11px] font-mono text-slate-600 dark:text-slate-300 bg-slate-50 dark:bg-slate-800/80 p-4 rounded-xl border border-slate-100 dark:border-slate-800 overflow-x-auto whitespace-pre leading-relaxed max-h-60 scrollbar-thin">
              {JSON.stringify(job.payload, null, 2)}
            </pre>
          </div>
        ) : null}

        <div className="pt-4 flex gap-3">
          <Button
            className="flex-1 inline-flex items-center justify-center gap-2 px-5 py-3 rounded-2xl text-xs font-bold uppercase tracking-wider border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800 active:scale-95 transition-all shadow-sm h-auto"
            onClick={() => {
              copyToClipboard(JSON.stringify(job, null, 2), t("monitor.debug_json_copied", { defaultValue: "Copied Debug JSON!" }));
            }}
            variant="outline"
          >
            <Copy size={14} />
            {t("monitor.copy_debug_json", { defaultValue: "Copy Debug JSON" })}
          </Button>
        </div>
      </div>
    </Sheet>
  );
}
