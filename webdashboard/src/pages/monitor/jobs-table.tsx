import type { TFunction } from "i18next";
import type { MouseEventHandler, ReactNode } from "react";

import { Briefcase } from "lucide-react";
import { useMemo, useState } from "react";

import { formatDistanceToNow } from "date-fns";

import type { MonitorInfraJobRecord, RecentJob } from "@/lib/api/types";

import Button from "@/components/ui/button";
import { FilterPanel } from "@/components/ui/filter-panel";
import { FilterTrigger } from "@/components/ui/filter-trigger";
import ResponsiveTable from "@/components/ui/responsive-table";
import TableEmptyStateRow from "@/components/ui/table-empty-row";
import { Typography } from "@/components/ui/typography";
import { useDebounce } from "@/hooks/use-debounce";
import { useHorizontalDragScroll } from "@/hooks/use-horizontal-drag-scroll";
import { useInfiniteScroll } from "@/hooks/use-infinite-scroll";
import { useWorkerJobs } from "@/hooks/use-worker-monitor";
import { cn } from "@/lib/utils";

import { JobDetailsDrawer, JobStatusBadge } from "./job-details-drawer";

const STATUS_FILTERS = [
  "completed",
  "queued",
  "processing",
  "retryable",
  "failed",
  "dead_letter",
  "lost",
  "pending",
] as const;

const STATUS_FILTER_LABELS: Record<string, string> = {
  completed: "Completed",
  failed: "Failed",
  pending: "Pending",
  processing: "Processing",
  queued: "Queued",
  retryable: "Retryable",
  dead_letter: "Dead letter",
  lost: "Lost",
};

interface JobsTableProps {
  t: TFunction;
  uniqueTypes: string[];
}

interface JobMobileCardProps {
  job: RecentJob;
  onClick: () => void;
  t: TFunction;
}

function getJobDisplayType(job: RecentJob, t: TFunction): string {
  const taskTypeKey = job.task_type ?? job.report_type;
  return taskTypeKey
    ? t(`monitor.task_types.${taskTypeKey}`, { defaultValue: taskTypeKey })
    : "—";
}

function JobMobileCard({ job, onClick, t }: JobMobileCardProps) {
  return (
    <div
      className="cursor-pointer rounded-2xl border border-slate-100 bg-white p-5 shadow-sm motion-safe:transition-all active:scale-[0.99] dark:border-slate-800 dark:bg-slate-900"
      onClick={onClick}
      onKeyDown={(event) => {
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
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
        <JobStatusBadge className="inline-flex items-center gap-1.5 rounded-full px-2.5 py-0.5 text-[9px] font-bold uppercase tracking-wider" status={job.status} t={t} />
      </div>
      <div className="mt-3 grid grid-cols-3 gap-2 border-t border-slate-100 pt-3 dark:border-slate-800">
        <JobMobileValue label={t("monitor.job_type", { defaultValue: "Type" })} value={getJobDisplayType(job, t)} />
        <JobMobileValue
          label={t("monitor.batches", { defaultValue: "Batches" })}
          value={
            <>
              {job.completed_batches}/{job.total_batches}
              {job.failed_batches > 0 ? <span className="ml-1 text-rose-500">({job.failed_batches})</span> : null}
            </>
          }
        />
        <JobMobileValue
          label={t("monitor.created_at", { defaultValue: "Created" })}
          value={formatDistanceToNow(new Date(job.created_at), { addSuffix: true })}
        />
      </div>
      {job.error_summary ? (
        <div className="mt-2 truncate rounded-lg border border-rose-100 bg-rose-50 px-3 py-1.5 text-[10px] font-medium text-rose-600 dark:border-rose-800 dark:bg-rose-950/30 dark:text-rose-400">
          {job.error_summary}
        </div>
      ) : null}
    </div>
  );
}

interface JobMobileValueProps {
  label: string;
  value: ReactNode;
}

function JobMobileValue({ label, value }: JobMobileValueProps) {
  return (
    <div>
      <div className="text-[9px] font-bold uppercase tracking-wider text-slate-400">{label}</div>
      <div className="mt-0.5 text-xs text-slate-700 dark:text-slate-200">{value}</div>
    </div>
  );
}

function SkeletonRow() {
  return (
    <tr className="border-t border-slate-50 dark:border-slate-800">
      <td className="px-6 py-4"><div className="h-4 w-32 rounded bg-slate-100 motion-safe:animate-pulse dark:bg-slate-800" /></td>
      <td className="px-6 py-4"><div className="h-6 w-20 rounded-full bg-slate-100 motion-safe:animate-pulse dark:bg-slate-800" /></td>
      <td className="px-6 py-4"><div className="h-4 w-24 rounded bg-slate-100 motion-safe:animate-pulse dark:bg-slate-800" /></td>
      <td className="px-6 py-4"><div className="h-4 w-16 rounded bg-slate-100 motion-safe:animate-pulse dark:bg-slate-800" /></td>
      <td className="px-6 py-4"><div className="h-4 w-28 rounded bg-slate-100 motion-safe:animate-pulse dark:bg-slate-800" /></td>
      <td className="px-6 py-4"><div className="h-4 w-40 rounded bg-slate-100 motion-safe:animate-pulse dark:bg-slate-800" /></td>
    </tr>
  );
}

interface JobFiltersProps {
  searchTerm: string;
  setSearchTerm: (value: string) => void;
  setStatusFilter: (value: string) => void;
  setTypeFilter: (value: string) => void;
  statusFilter: string;
  t: TFunction;
  typeFilter: string;
  uniqueTypes: string[];
}

function JobFilters({ searchTerm, setSearchTerm, statusFilter, setStatusFilter, t, typeFilter, setTypeFilter, uniqueTypes }: JobFiltersProps) {
  return (
    <div className="space-y-4 border-b border-slate-100 p-6 dark:border-slate-800">
      <div>
        <Typography className="text-lg font-bold uppercase tracking-tight text-slate-700 dark:text-slate-200" variant="h4">
          {t("monitor.recent_jobs", { defaultValue: "Recent Jobs" })}
        </Typography>
        <Typography className="mt-1 text-[10px] font-bold uppercase tracking-[0.2em] text-slate-500 opacity-60 dark:text-slate-400" variant="p">
          {t("monitor.recent_jobs_subtitle", { defaultValue: "Job metadata from last 24 hours" })}
        </Typography>
      </div>
      <div className="flex flex-col gap-3 pt-2 md:flex-row">
        <div className="flex-1">
          <label className="sr-only" htmlFor="monitor-job-search">
            {t("monitor.filter_search_label", { defaultValue: "Search jobs" })}
          </label>
          <input
            aria-label={t("monitor.filter_search_label", { defaultValue: "Search jobs" })}
            className="w-full rounded-2xl border border-slate-100 bg-slate-50 px-4 py-2.5 text-sm text-slate-700 placeholder:text-slate-400 motion-safe:transition-all focus:outline-none focus:ring-2 focus:ring-blue-500/30 dark:border-slate-800/80 dark:bg-slate-800/40 dark:text-slate-200 dark:placeholder:text-slate-500"
            id="monitor-job-search"
            onChange={(event) => { setSearchTerm(event.target.value); }}
            placeholder={t("monitor.filter_search_placeholder", { defaultValue: "Search Job ID or error code..." })}
            type="text"
            value={searchTerm}
          />
        </div>
        <StatusFilter selected={statusFilter} setSelected={setStatusFilter} t={t} />
        <TypeFilter selected={typeFilter} setSelected={setTypeFilter} t={t} uniqueTypes={uniqueTypes} />
      </div>
    </div>
  );
}

interface FilterProps {
  selected: string;
  setSelected: (value: string) => void;
  t: TFunction;
}

function StatusFilter({ selected, setSelected, t }: FilterProps) {
  const options = [
    { value: "all", label: t("monitor.filter_status_all", { defaultValue: "All Statuses" }) },
    ...STATUS_FILTERS.map((status) => ({ value: status, label: t(`monitor.status_${status}`, { defaultValue: STATUS_FILTER_LABELS[status] }) })),
  ];

  return (
    <div className="w-full md:w-48">
      <FilterPanel
        container="popover"
        mode="single"
        onClear={() => { setSelected("all"); }}
        onToggle={setSelected}
        options={options}
        selected={selected !== "all" ? [selected] : []}
        showClearButton={false}
        trigger={
          <FilterTrigger
            chevron
            hasActive={selected !== "all"}
            label={selected !== "all" ? t(`monitor.status_${selected}`, { defaultValue: selected.replace("_", " ") }) : t("monitor.filter_status", { defaultValue: "Status" })}
            variant="pill-button"
          />
        }
      />
    </div>
  );
}

interface TypeFilterProps extends FilterProps {
  uniqueTypes: string[];
}

function TypeFilter({ selected, setSelected, t, uniqueTypes }: TypeFilterProps) {
  const options = [
    { value: "all", label: t("monitor.filter_type_all", { defaultValue: "All Types" }) },
    ...uniqueTypes.map((type) => ({ value: type, label: t(`monitor.task_types.${type}`, { defaultValue: type }) })),
  ];

  return (
    <div className="w-full md:w-56">
      <FilterPanel
        container="popover"
        mode="single"
        onClear={() => { setSelected("all"); }}
        onToggle={setSelected}
        options={options}
        selected={selected !== "all" ? [selected] : []}
        showClearButton={false}
        trigger={
          <FilterTrigger
            chevron
            hasActive={selected !== "all"}
            label={selected !== "all" ? t(`monitor.task_types.${selected}`, { defaultValue: selected }) : t("monitor.filter_type", { defaultValue: "Type" })}
            variant="pill-button"
          />
        }
      />
    </div>
  );
}

interface DesktopTableProps {
  dragClick: MouseEventHandler<HTMLDivElement>;
  isLoading: boolean;
  jobs: RecentJob[];
  selectJob: (job: RecentJob) => void;
  setDragRef: (node: HTMLDivElement | null) => void;
  t: TFunction;
}

function JobDesktopTable({ dragClick, jobs, isLoading, selectJob, setDragRef, t }: DesktopTableProps) {
  return (
    <div className="overflow-x-auto overscroll-contain" onClickCapture={dragClick} ref={setDragRef}>
      <table className="w-full min-w-[1100px]">
        <thead>
          <tr className="bg-slate-50 dark:bg-slate-800/50">
            {[
              t("monitor.job_id", { defaultValue: "Job ID" }),
              t("monitor.status"),
              t("monitor.job_type", { defaultValue: "Type" }),
              t("monitor.batches", { defaultValue: "Batches" }),
              t("monitor.created_at", { defaultValue: "Created" }),
              t("monitor.error", { defaultValue: "Error" }),
            ].map((heading) => (
              <th className="px-6 py-4 text-left text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400" key={heading}>
                {heading}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {isLoading ? <LoadingRows /> : jobs.length === 0 ? (
            <TableEmptyStateRow
              colSpan={6}
              description={t("monitor.no_jobs_description", { defaultValue: "Thử điều chỉnh bộ lọc hoặc kiểm tra lại sau." })}
              title={t("monitor.no_jobs_found", { defaultValue: "Không có tác vụ nào phù hợp bộ lọc" })}
            />
          ) : jobs.map((job) => <JobDesktopRow job={job} key={job.job_id} onSelect={selectJob} t={t} />)}
        </tbody>
      </table>
    </div>
  );
}

function LoadingRows() {
  return <>{Array.from({ length: 5 }, (_, index) => <SkeletonRow key={`skel-${index}`} />)}</>;
}

interface JobDesktopRowProps {
  job: RecentJob;
  onSelect: (job: RecentJob) => void;
  t: TFunction;
}

function JobDesktopRow({ job, onSelect, t }: JobDesktopRowProps) {
  return (
    <tr
      aria-label={t("monitor.open_job_details", { defaultValue: "Open job details" })}
      className="cursor-pointer border-t border-slate-50 motion-safe:transition-colors hover:bg-slate-50 focus:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-primary-500 dark:border-slate-800 dark:hover:bg-slate-800/50"
      key={job.job_id}
      onClick={() => { onSelect(job); }}
      onKeyDown={(event) => {
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          onSelect(job);
        }
      }}
      role="button"
      tabIndex={0}
    >
      <td className="px-6 py-4"><code className="font-mono text-xs text-slate-600 dark:text-slate-400">{job.job_id.slice(0, 8)}…</code></td>
      <td className="px-6 py-4"><JobStatusBadge className="inline-flex items-center gap-2 rounded-full px-3 py-1 text-xs font-bold uppercase tracking-wider" status={job.status} t={t} /></td>
      <td className="px-6 py-4"><span className="text-sm font-medium text-slate-700 dark:text-slate-200">{getJobDisplayType(job, t)}</span></td>
      <td className="px-6 py-4"><span className="text-sm text-slate-700 dark:text-slate-200">{job.completed_batches}/{job.total_batches}{job.failed_batches > 0 ? <span className="ml-1 text-rose-500">({job.failed_batches} failed)</span> : null}</span></td>
      <td className="px-6 py-4"><span className="text-sm text-slate-500 dark:text-slate-400">{formatDistanceToNow(new Date(job.created_at), { addSuffix: true })}</span></td>
      <td className="max-w-[200px] px-6 py-4">{job.error_summary ? <span className="block cursor-help truncate text-xs text-rose-500 dark:text-rose-400" title={job.error_summary}>{job.error_summary}</span> : <span className="text-sm text-slate-400 dark:text-slate-500">—</span>}</td>
    </tr>
  );
}

interface DurableJobsSectionProps {
  jobs: MonitorInfraJobRecord[];
  onSelect: (job: MonitorInfraJobRecord) => void;
  t: TFunction;
  unavailable: boolean;
}

function DurableJobsSection({ jobs, onSelect, t, unavailable }: DurableJobsSectionProps) {
  return (
    <section aria-labelledby="durable-monitor-jobs-heading" className="overflow-hidden rounded-[2.5rem] border border-slate-100 bg-white shadow-sm dark:border-slate-800 dark:bg-slate-900 dark:shadow-black/10">
      <div className="border-b border-slate-100 p-6 dark:border-slate-800">
        <Typography className="text-lg font-bold uppercase tracking-tight text-slate-700 dark:text-slate-200" id="durable-monitor-jobs-heading" variant="h4">
          {t("monitor.durable_jobs", { defaultValue: "Durable infrastructure jobs" })}
        </Typography>
        <Typography className="mt-1 text-[10px] font-bold uppercase tracking-[0.2em] text-slate-500 opacity-60 dark:text-slate-400" variant="p">
          {t("monitor.durable_jobs_subtitle", { defaultValue: "Authoritative lifecycle records and sanitized events" })}
        </Typography>
      </div>
      {unavailable ? <div className="p-6 text-sm text-amber-700 dark:text-amber-300">{t("monitor.durable_jobs_unavailable", { defaultValue: "Durable job data is temporarily unavailable." })}</div> : jobs.length === 0 ? <div className="p-6 text-sm text-slate-500 dark:text-slate-400">{t("monitor.no_durable_jobs", { defaultValue: "No durable infrastructure jobs are available." })}</div> : <div className="grid gap-3 p-4 sm:p-6">{jobs.map((job) => <DurableJobButton job={job} key={job.job_id} onSelect={onSelect} t={t} />)}</div>}
    </section>
  );
}

interface DurableJobButtonProps {
  job: MonitorInfraJobRecord;
  onSelect: (job: MonitorInfraJobRecord) => void;
  t: TFunction;
}

function DurableJobButton({ job, onSelect, t }: DurableJobButtonProps) {
  const relativeTime = (value: string | null | undefined) => value ? formatDistanceToNow(new Date(value), { addSuffix: true }) : "—";

  return (
    <Button className="flex w-full flex-col gap-3 rounded-2xl border border-slate-100 bg-slate-50 p-4 text-left motion-safe:transition-colors hover:border-blue-200 hover:bg-blue-50/50 focus:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 dark:border-slate-800 dark:bg-slate-800/40 dark:hover:border-blue-800 dark:hover:bg-blue-950/20" onClick={() => { onSelect(job); }} type="button">
      <div className="flex flex-wrap items-center justify-between gap-3"><code className="font-mono text-xs text-slate-600 dark:text-slate-300">{job.job_id.slice(0, 8)}…</code><JobStatusBadge className="inline-flex items-center gap-1.5 rounded-full px-2.5 py-1 text-[10px] font-bold uppercase tracking-wider" status={job.state} t={t} /></div>
      <div className="grid grid-cols-2 gap-3 text-xs text-slate-500 dark:text-slate-400 sm:grid-cols-4">
        <DurableValue label={t("monitor.job_type", { defaultValue: "Type" })} value={job.task_type} />
        <DurableValue label={t("monitor.priority", { defaultValue: "Priority" })} value={job.priority} />
        <DurableValue label={t("monitor.attempt", { defaultValue: "Attempt" })} value={job.attempt_count} />
        <DurableValue label={t("monitor.worker", { defaultValue: "Worker" })} value={job.worker ?? "—"} />
        <DurableValue label={t("monitor.parent_job", { defaultValue: "Parent job" })} mono value={job.parent_job_id ? `${job.parent_job_id.slice(0, 8)}…` : "—"} />
        <DurableValue label={t("monitor.events", { defaultValue: "Events" })} value={job.events.length} />
        <DurableValue label={t("monitor.queued_at", { defaultValue: "Queued" })} title={job.queued_at ?? undefined} value={relativeTime(job.queued_at)} />
        <DurableValue label={t("monitor.started_at", { defaultValue: "Started" })} title={job.started_at ?? undefined} value={relativeTime(job.started_at)} />
        <DurableValue label={t("monitor.finished_at", { defaultValue: "Finished" })} title={job.finished_at ?? undefined} value={relativeTime(job.finished_at)} />
        <DurableValue label={t("monitor.updated_at", { defaultValue: "Updated" })} title={job.updated_at} value={relativeTime(job.updated_at)} />
      </div>
      <div className="grid gap-1 text-xs text-slate-500 dark:text-slate-400 sm:grid-cols-2"><DurableValue label={t("monitor.request_id", { defaultValue: "Request ID" })} mono value={job.request_id ?? "—"} /><DurableValue label={t("monitor.trace_id", { defaultValue: "Trace ID" })} mono value={job.trace_id ?? "—"} /></div>
      {job.error_summary ? <div className="truncate text-xs text-rose-600 dark:text-rose-400">{job.error_code ? `${job.error_code}: ` : ""}{job.error_summary}</div> : null}
    </Button>
  );
}

interface DurableValueProps {
  label: string;
  mono?: boolean;
  title?: string;
  value: ReactNode;
}

function DurableValue({ label, mono, title, value }: DurableValueProps) {
  return <span title={title}>{label}: <strong className={cn("text-slate-700 dark:text-slate-200", mono && "font-mono")}>{value}</strong></span>;
}

interface JobsPaginationProps {
  count: number;
  fetchNextPage: () => void;
  hasNextPage: boolean;
  isFetchingNextPage: boolean;
  t: TFunction;
}

function JobsPagination({ count, fetchNextPage, hasNextPage, isFetchingNextPage, t }: JobsPaginationProps) {
  if (count === 0) return null;

  const loadMore = () => {
    fetchNextPage();
  };

  return (
    <div className="flex flex-wrap items-center justify-between gap-3 border-t border-slate-100 bg-slate-50/50 px-6 py-4 text-xs text-slate-500 dark:border-slate-800/80 dark:bg-slate-800/20 dark:text-slate-400">
      <span>{t("monitor.pagination_loaded", { defaultValue: "Loaded {{count}} jobs", count })}</span>
      {hasNextPage ? <Button aria-label={t("monitor.load_more_jobs", { defaultValue: "Load more jobs" })} disabled={isFetchingNextPage} loading={isFetchingNextPage} onClick={loadMore} size="sm" type="button" variant="outline">{t("monitor.load_more_jobs", { defaultValue: "Load more jobs" })}</Button> : null}
    </div>
  );
}

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
  const { data, fetchNextPage, hasNextPage, isFetchingNextPage, isLoading } = useWorkerJobs(queryParams);
  const jobs = useMemo(() => data?.pages.flatMap((page) => page.jobs) ?? [], [data]);
  const durableJobs = useMemo(() => {
    const jobsById = new Map<string, MonitorInfraJobRecord>();
    for (const page of data?.pages ?? []) {
      if (page.durable_jobs.status !== "available") continue;
      for (const job of page.durable_jobs.jobs ?? []) jobsById.set(job.job_id, job);
    }
    return [...jobsById.values()];
  }, [data]);
  const firstPage = data?.pages[0];
  const durableJobsUnavailable = firstPage?.durable_jobs.status === "unavailable";
  const redisJobsUnavailable = firstPage?.quality.redis.status === "unavailable";
  const mobileLoadMoreRef = useInfiniteScroll<HTMLDivElement>({ fetchNextPage: () => { void fetchNextPage(); }, hasNextPage, isFetchingNextPage });
  const { onClickCapture: handleHorizontalDragClick, setRef: setHorizontalDragScrollRef } = useHorizontalDragScroll<HTMLDivElement>();
  const closeDrawer = () => { setSelectedJob(null); setSelectedDurableJob(null); };
  const selectDurableJob = (job: MonitorInfraJobRecord) => { setSelectedJob(null); setSelectedDurableJob(job); };

  return (
    <>
      <div className="overflow-hidden rounded-[2.5rem] border border-slate-100 bg-white shadow-sm dark:border-slate-800 dark:bg-slate-900 dark:shadow-black/10">
        <JobFilters searchTerm={searchTerm} setSearchTerm={setSearchTerm} setStatusFilter={setStatusFilter} setTypeFilter={setTypeFilter} statusFilter={statusFilter} t={t} typeFilter={typeFilter} uniqueTypes={uniqueTypes} />
        <ResponsiveTable<RecentJob>
          data={jobs}
          emptyState={{ icon: <Briefcase className="h-10 w-10" />, title: redisJobsUnavailable ? t("monitor.metric_unavailable", { defaultValue: "Recent jobs source unavailable" }) : t("monitor.no_matching_jobs", { defaultValue: "No jobs match your search filters" }) }}
          getCardKey={(job) => job.job_id}
          isLoading={isLoading}
          mobileFooter={hasNextPage ? <div className="flex min-h-10 items-center justify-center" ref={mobileLoadMoreRef}>{isFetchingNextPage ? <span className="text-[10px] font-bold uppercase tracking-widest text-slate-400">{t("common.loading")}</span> : null}</div> : undefined}
          renderCard={(job) => <JobMobileCard job={job} onClick={() => { setSelectedJob(job); }} t={t} />}
          renderDesktopTable={() => <JobDesktopTable dragClick={handleHorizontalDragClick} isLoading={isLoading} jobs={jobs} selectJob={setSelectedJob} setDragRef={setHorizontalDragScrollRef} t={t} />}
        />
        {!isLoading ? <JobsPagination count={jobs.length} fetchNextPage={() => { void fetchNextPage(); }} hasNextPage={hasNextPage} isFetchingNextPage={isFetchingNextPage} t={t} /> : null}
      </div>
      <DurableJobsSection jobs={durableJobs} onSelect={selectDurableJob} t={t} unavailable={durableJobsUnavailable} />
      <JobDetailsDrawer durableJob={selectedDurableJob} isOpen={selectedJob !== null || selectedDurableJob !== null} job={selectedJob} onClose={closeDrawer} t={t} />
    </>
  );
}
