import { Briefcase } from "lucide-react";
import React, { useMemo, useState } from "react";

import { formatDistanceToNow } from "date-fns";
import { TFunction } from "i18next";

import Button from "@/components/ui/button";
import { FilterPanel } from "@/components/ui/filter-panel";
import { FilterTrigger } from "@/components/ui/filter-trigger";
import ResponsiveTable from "@/components/ui/responsive-table";
import TableEmptyRow from "@/components/ui/table-empty-row";
import { Typography } from "@/components/ui/typography";
import { useDebounce } from "@/hooks/use-debounce";
import { useInfiniteScroll } from "@/hooks/use-infinite-scroll";
import { useWorkerJobs } from "@/hooks/use-worker-monitor";
import { MonitorInfraJobRecord, RecentJob } from "@/lib/api/types";
import { cn } from "@/lib/utils";

import { getStatusStyle, JobDetailsDrawer } from "./job-details-drawer";

function jobStatusLabel(t: TFunction, status: string): string {
  const defaults: Record<string, string> = {
    completed: "Completed",
    failed: "Failed",
    processing: "Processing",
    retryable: "Retryable",
    dead_letter: "Dead letter",
    lost: "Lost",
    queued: "Queued",
    dispatching: "Dispatching",
  };
  return t(`monitor.status_${status}`, { defaultValue: defaults[status] ?? t("monitor.unknown_status", { defaultValue: "Status unavailable" }) });
}

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
      className="cursor-pointer rounded-2xl border border-slate-100 bg-white p-5 shadow-sm motion-safe:transition-all active:scale-[0.99] dark:border-slate-800 dark:bg-slate-900"
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
          {jobStatusLabel(t, job.status)}
        </span>
      </div>

      <div className="mt-3 grid grid-cols-3 gap-2 border-t border-slate-100 pt-3 dark:border-slate-800">
        <div>
          <div className="text-[9px] font-bold uppercase tracking-wider text-slate-400">{t("monitor.job_type", { defaultValue: "Type" })}</div>
          <div className="mt-0.5 text-xs font-medium text-slate-700 dark:text-slate-200">{displayType}</div>
        </div>
        <div>
          <div className="text-[9px] font-bold uppercase tracking-wider text-slate-400">{t("monitor.batches", { defaultValue: "Batches" })}</div>
          <div className="mt-0.5 text-xs text-slate-700 dark:text-slate-200">
            {job.completed_batches}/{job.total_batches}
            {job.failed_batches > 0 ? <span className="text-rose-500 ml-1">({job.failed_batches})</span> : null}
          </div>
        </div>
        <div>
          <div className="text-[9px] font-bold uppercase tracking-wider text-slate-400">{t("monitor.created_at", { defaultValue: "Created" })}</div>
          <div className="mt-0.5 text-xs text-slate-500 dark:text-slate-400">
            {formatDistanceToNow(new Date(job.created_at), { addSuffix: true })}
          </div>
        </div>
      </div>

      {job.error_summary ? (
        <div className="mt-2 truncate rounded-lg border border-rose-100 bg-rose-50 px-3 py-1.5 text-[10px] font-medium text-rose-600 dark:border-rose-800 dark:bg-rose-950/30 dark:text-rose-400">
          {job.error_summary}
        </div>
      ) : null}
    </div>
  );
}

function SkeletonRow() {
  return (
    <tr className="border-t border-slate-50 dark:border-slate-800">
      <td className="px-6 py-4"><div className="h-4 w-32 bg-slate-100 dark:bg-slate-800 rounded motion-safe:animate-pulse" /></td>
      <td className="px-6 py-4"><div className="h-6 w-20 bg-slate-100 dark:bg-slate-800 rounded-full motion-safe:animate-pulse" /></td>
      <td className="px-6 py-4"><div className="h-4 w-24 bg-slate-100 dark:bg-slate-800 rounded motion-safe:animate-pulse" /></td>
      <td className="px-6 py-4"><div className="h-4 w-16 bg-slate-100 dark:bg-slate-800 rounded motion-safe:animate-pulse" /></td>
      <td className="px-6 py-4"><div className="h-4 w-28 bg-slate-100 dark:bg-slate-800 rounded motion-safe:animate-pulse" /></td>
      <td className="px-6 py-4"><div className="h-4 w-40 bg-slate-100 dark:bg-slate-800 rounded motion-safe:animate-pulse" /></td>
    </tr>
  );
}

// eslint-disable-next-line max-lines-per-function -- table component with search, pagination, status filters, modals
export function JobsTable({ uniqueTypes, t }: JobsTableProps) {
  const [selectedJob, setSelectedJob] = useState<RecentJob | null>(null);
  const [selectedDurableJob, setSelectedDurableJob] = useState<MonitorInfraJobRecord | null>(null);
  const [searchTerm, setSearchTerm] = useState("");
  const [statusFilter, setStatusFilter] = useState("all");
  const [typeFilter, setTypeFilter] = useState("all");

  const debouncedSearchTerm = useDebounce(searchTerm, 300);

  const queryParams = useMemo(() => ({
    limit: 20,
    search: debouncedSearchTerm || undefined,
    state: statusFilter === "all" ? undefined : statusFilter,
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

  const durableJobs = useMemo(() => {
    const jobsById = new Map<string, MonitorInfraJobRecord>();
    for (const page of data?.pages ?? []) {
      if (page.durable_jobs?.status !== "available") continue;
      for (const job of page.durable_jobs.jobs ?? []) {
        jobsById.set(job.job_id, job);
      }
    }
    return [...jobsById.values()];
  }, [data]);

  const durableJobsUnavailable = data?.pages[0]?.durable_jobs?.status === "unavailable";
  const redisJobsUnavailable = data?.pages[0]?.quality?.redis.status === "unavailable";

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
              <label className="sr-only" htmlFor="monitor-job-search">
                {t("monitor.filter_search_label", { defaultValue: "Search jobs" })}
              </label>
              <input
                aria-label={t("monitor.filter_search_label", { defaultValue: "Search jobs" })}
                className="w-full rounded-2xl border border-slate-100 bg-slate-50 px-4 py-2.5 text-sm text-slate-700 placeholder:text-slate-400 motion-safe:transition-all focus:outline-none focus:ring-2 focus:ring-blue-500/30 dark:border-slate-800/80 dark:bg-slate-800/40 dark:text-slate-200 dark:placeholder:text-slate-500"
                id="monitor-job-search"
                onChange={(e) => { setSearchTerm(e.target.value); }}
                placeholder={t("monitor.filter_search_placeholder", { defaultValue: "Search Job ID or error code..." })}
                type="text"
                value={searchTerm}
              />
            </div>
            <div className="w-full md:w-48">
              <FilterPanel
                container="popover"
                mode="single"
                onClear={() => { setStatusFilter("all"); }}
                onToggle={(val) => { setStatusFilter(val); }}
                options={[
                  { value: "all", label: t("monitor.filter_status_all", { defaultValue: "All Statuses" }) },
                  { value: "completed", label: t("monitor.status_completed", { defaultValue: "Completed" }) },
                  { value: "queued", label: t("monitor.status_queued", { defaultValue: "Queued" }) },
                  { value: "processing", label: t("monitor.status_processing", { defaultValue: "Processing" }) },
                  { value: "retryable", label: t("monitor.status_retryable", { defaultValue: "Retryable" }) },
                  { value: "failed", label: t("monitor.status_failed", { defaultValue: "Failed" }) },
                  { value: "dead_letter", label: t("monitor.status_dead_letter", { defaultValue: "Dead letter" }) },
                  { value: "lost", label: t("monitor.status_lost", { defaultValue: "Lost" }) },
                  { value: "pending", label: t("monitor.status_pending", { defaultValue: "Pending" }) },
                ]}
                selected={statusFilter !== "all" ? [statusFilter] : []}
                showClearButton={false}
                trigger={
                  <FilterTrigger
                    chevron
                    hasActive={statusFilter !== "all"}
                    label={statusFilter !== "all"
                      ? t(`monitor.status_${statusFilter}`, { defaultValue: statusFilter.replace("_", " ") })
                      : t("monitor.filter_status", { defaultValue: "Status" })}
                    variant="pill-button"
                  />
                }
              />
            </div>
            <div className="w-full md:w-56">
              <FilterPanel
                container="popover"
                mode="single"
                onClear={() => { setTypeFilter("all"); }}
                onToggle={(val) => { setTypeFilter(val); }}
                options={[
                  { value: "all", label: t("monitor.filter_type_all", { defaultValue: "All Types" }) },
                  ...uniqueTypes.map((type) => ({
                    value: type,
                    label: t(`monitor.task_types.${type}`, { defaultValue: type }),
                  })),
                ]}
                selected={typeFilter !== "all" ? [typeFilter] : []}
                showClearButton={false}
                trigger={
                  <FilterTrigger
                    chevron
                    hasActive={typeFilter !== "all"}
                    label={typeFilter !== "all"
                      ? t(`monitor.task_types.${typeFilter}`, { defaultValue: typeFilter })
                      : t("monitor.filter_type", { defaultValue: "Type" })}
                    variant="pill-button"
                  />
                }
              />
            </div>
          </div>
        </div>

        <ResponsiveTable<RecentJob>
          data={jobs}
          emptyState={{
            icon: <Briefcase className="h-10 w-10" />,
            title: redisJobsUnavailable
              ? t("monitor.metric_unavailable", { defaultValue: "Recent jobs source unavailable" })
              : t("monitor.no_matching_jobs", { defaultValue: "No jobs match your search filters" }),
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
                  ) : jobs.length === 0 ? (
                    <TableEmptyRow
                      colSpan={6}
                      description={t("monitor.no_jobs_description", { defaultValue: "Thử điều chỉnh bộ lọc hoặc kiểm tra lại sau." })}
                      title={t("monitor.no_jobs_found", { defaultValue: "Không có tác vụ nào phù hợp bộ lọc" })}
                    />
                  ) : (
                    jobs.map((job) => {
                      const taskTypeKey = job.task_type ?? job.report_type;
                      const displayType = taskTypeKey
                        ? t(`monitor.task_types.${taskTypeKey}`, { defaultValue: taskTypeKey })
                        : "—";

                      const { className: statusClassName, icon: statusIcon } = getStatusStyle(job.status);

                      return (
                        <tr
                          className="cursor-pointer border-t border-slate-50 motion-safe:transition-colors hover:bg-slate-50 focus:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-primary-500 dark:border-slate-800 dark:hover:bg-slate-800/50"
                          key={job.job_id}
                          aria-label={t("monitor.open_job_details", { defaultValue: "Open job details" })}
                          onClick={() => { setSelectedJob(job); }}
                          onKeyDown={(e) => {
                            if (e.key === "Enter" || e.key === " ") {
                              e.preventDefault();
                              setSelectedJob(job);
                            }
                          }}
                          role="button"
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
                              {jobStatusLabel(t, job.status)}
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
                            {job.error_summary ? (
                              <span className="block cursor-help truncate text-xs text-rose-500 dark:text-rose-400" title={job.error_summary}>
                                {job.error_summary}
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
          <div className="flex flex-wrap items-center justify-between gap-3 border-t border-slate-100 bg-slate-50/50 px-6 py-4 text-xs text-slate-500 dark:border-slate-800/80 dark:bg-slate-800/20 dark:text-slate-400">
            <span>
              {t("monitor.pagination_loaded", {
                defaultValue: "Loaded {{count}} jobs",
                count: jobs.length,
              })}
            </span>
            {hasNextPage ? (
              <Button
                aria-label={t("monitor.load_more_jobs", { defaultValue: "Load more jobs" })}
                disabled={isFetchingNextPage}
                loading={isFetchingNextPage}
                onClick={() => { void fetchNextPage(); }}
                size="sm"
                type="button"
                variant="outline"
              >
                {t("monitor.load_more_jobs", { defaultValue: "Load more jobs" })}
              </Button>
            ) : null}
          </div>
        ) : null}
      </div>

      <section
        aria-labelledby="durable-monitor-jobs-heading"
        className="overflow-hidden rounded-[2.5rem] border border-slate-100 bg-white shadow-sm dark:border-slate-800 dark:bg-slate-900 dark:shadow-black/10"
      >
        <div className="border-b border-slate-100 p-6 dark:border-slate-800">
          <Typography className="text-lg font-bold uppercase tracking-tight text-slate-700 dark:text-slate-200" id="durable-monitor-jobs-heading" variant="h4">
            {t("monitor.durable_jobs", { defaultValue: "Durable infrastructure jobs" })}
          </Typography>
          <Typography className="mt-1 text-[10px] font-bold uppercase tracking-[0.2em] text-slate-500 opacity-60 dark:text-slate-400" variant="p">
            {t("monitor.durable_jobs_subtitle", { defaultValue: "Authoritative lifecycle records and sanitized events" })}
          </Typography>
        </div>
        {durableJobsUnavailable ? (
          <div className="p-6 text-sm text-amber-700 dark:text-amber-300">
            {t("monitor.durable_jobs_unavailable", { defaultValue: "Durable job data is temporarily unavailable." })}
          </div>
        ) : durableJobs.length === 0 ? (
          <div className="p-6 text-sm text-slate-500 dark:text-slate-400">
            {t("monitor.no_durable_jobs", { defaultValue: "No durable infrastructure jobs are available." })}
          </div>
        ) : (
          <div className="grid gap-3 p-4 sm:p-6">
            {durableJobs.map((job) => {
              const { className: statusClassName, icon: statusIcon } = getStatusStyle(job.state);
              return (
                <Button
                  className="flex w-full flex-col gap-3 rounded-2xl border border-slate-100 bg-slate-50 p-4 text-left motion-safe:transition-colors hover:border-blue-200 hover:bg-blue-50/50 focus:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 dark:border-slate-800 dark:bg-slate-800/40 dark:hover:border-blue-800 dark:hover:bg-blue-950/20"
                  key={job.job_id}
                  onClick={() => {
                    setSelectedJob(null);
                    setSelectedDurableJob(job);
                  }}
                  type="button"
                >
                  <div className="flex flex-wrap items-center justify-between gap-3">
                    <code className="font-mono text-xs text-slate-600 dark:text-slate-300">{job.job_id.slice(0, 8)}…</code>
                    <span className={cn("inline-flex items-center gap-1.5 rounded-full px-2.5 py-1 text-[10px] font-bold uppercase tracking-wider", statusClassName)}>
                      {statusIcon}
                      {jobStatusLabel(t, job.state)}
                    </span>
                  </div>
                  <div className="grid grid-cols-2 gap-3 text-xs text-slate-500 dark:text-slate-400 sm:grid-cols-4">
                    <span>{t("monitor.job_type", { defaultValue: "Type" })}: <strong className="text-slate-700 dark:text-slate-200">{job.task_type}</strong></span>
                    <span>{t("monitor.priority", { defaultValue: "Priority" })}: <strong className="text-slate-700 dark:text-slate-200">{job.priority}</strong></span>
                    <span>{t("monitor.attempt", { defaultValue: "Attempt" })}: <strong className="text-slate-700 dark:text-slate-200">{job.attempt_count}</strong></span>
                    <span>{t("monitor.worker", { defaultValue: "Worker" })}: <strong className="text-slate-700 dark:text-slate-200">{job.worker ?? "—"}</strong></span>
                    <span>{t("monitor.parent_job", { defaultValue: "Parent job" })}: <strong className="font-mono text-slate-700 dark:text-slate-200">{job.parent_job_id ? `${job.parent_job_id.slice(0, 8)}…` : "—"}</strong></span>
                    <span>{t("monitor.events", { defaultValue: "Events" })}: <strong className="text-slate-700 dark:text-slate-200">{job.events.length}</strong></span>
                    <span title={job.queued_at ?? undefined}>{t("monitor.queued_at", { defaultValue: "Queued" })}: <strong className="text-slate-700 dark:text-slate-200">{job.queued_at ? formatDistanceToNow(new Date(job.queued_at), { addSuffix: true }) : "—"}</strong></span>
                    <span title={job.started_at ?? undefined}>{t("monitor.started_at", { defaultValue: "Started" })}: <strong className="text-slate-700 dark:text-slate-200">{job.started_at ? formatDistanceToNow(new Date(job.started_at), { addSuffix: true }) : "—"}</strong></span>
                    <span title={job.finished_at ?? undefined}>{t("monitor.finished_at", { defaultValue: "Finished" })}: <strong className="text-slate-700 dark:text-slate-200">{job.finished_at ? formatDistanceToNow(new Date(job.finished_at), { addSuffix: true }) : "—"}</strong></span>
                    <span title={job.updated_at}>{t("monitor.updated_at", { defaultValue: "Updated" })}: <strong className="text-slate-700 dark:text-slate-200">{formatDistanceToNow(new Date(job.updated_at), { addSuffix: true })}</strong></span>
                  </div>
                  <div className="grid gap-1 text-xs text-slate-500 dark:text-slate-400 sm:grid-cols-2">
                    <span>{t("monitor.request_id", { defaultValue: "Request ID" })}: <code className="text-slate-700 dark:text-slate-200">{job.request_id ?? "—"}</code></span>
                    <span>{t("monitor.trace_id", { defaultValue: "Trace ID" })}: <code className="text-slate-700 dark:text-slate-200">{job.trace_id ?? "—"}</code></span>
                  </div>
                  {job.error_summary ? <div className="truncate text-xs text-rose-600 dark:text-rose-400">{job.error_code ? `${job.error_code}: ` : ""}{job.error_summary}</div> : null}
                </Button>
              );
            })}
          </div>
        )}
      </section>

      <JobDetailsDrawer
        durableJob={selectedDurableJob}
        isOpen={selectedJob !== null || selectedDurableJob !== null}
        job={selectedJob}
        onClose={() => {
          setSelectedJob(null);
          setSelectedDurableJob(null);
        }}
        t={t}
      />
    </>
  );
}
