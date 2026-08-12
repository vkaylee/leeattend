import { Ban, CheckCircle2, Clock, Copy, Loader2, RefreshCw, ShieldAlert, XCircle } from "lucide-react";
import React from "react";

import Button from "@/components/ui/button";

import { TFunction } from "i18next";

import Sheet from "@/components/ui/sheet";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { MonitorInfraJobRecord, RecentJob } from "@/lib/api/types";
import { cn } from "@/lib/utils";

interface StatusStyle {
  className: string;
  icon: React.ReactNode;
}

  // eslint-disable-next-line react-refresh/only-export-components
export function getStatusStyle(status: string): StatusStyle {
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
        icon: <Loader2 className="motion-safe:animate-spin" size={12} />,
      };
    case "retryable":
      return {
        className: "bg-sky-50 dark:bg-sky-950 text-sky-600 dark:text-sky-300",
        icon: <RefreshCw size={12} />,
      };
    case "dead_letter":
      return {
        className: "bg-orange-50 dark:bg-orange-950 text-orange-600 dark:text-orange-300",
        icon: <ShieldAlert size={12} />,
      };
    case "lost":
      return {
        className: "bg-red-100 dark:bg-red-950 text-red-700 dark:text-red-300",
        icon: <Ban size={12} />,
      };
    case "queued":
    case "dispatching":
      return {
        className: "bg-blue-50 dark:bg-blue-950 text-blue-600 dark:text-blue-300",
        icon: <Clock size={12} />,
      };
    default:
      return {
        className: "bg-slate-100 dark:bg-slate-700 text-slate-500 dark:text-slate-400",
        icon: <Clock size={12} />,
      };
  }
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
          className="p-2 hover:bg-slate-100 dark:hover:bg-slate-700 rounded-lg motion-safe:transition-colors text-slate-500 dark:text-slate-400 h-auto min-w-0"
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

function localizedJobStatus(t: TFunction, status: string): string {
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
            {localizedJobStatus(t, status)}
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
  errorSummary,
  t,
}: {
  completedAt?: string | null;
  createdAt: string;
  errorSummary?: string | null;
  t: TFunction;
}) {
  const { formatDateTimeLocale } = useDateTimeFormatter();
  return (
    <div className="bg-slate-50 dark:bg-slate-800/50 rounded-[2rem] p-6 border border-slate-100 dark:border-slate-800 space-y-3 text-sm">
      <div className="flex justify-between">
        <span className="text-slate-400">{t("monitor.created_at", { defaultValue: "Created At" })}</span>
        <span className="text-slate-700 dark:text-slate-300 font-medium">
          {formatDateTimeLocale(createdAt)}
        </span>
      </div>
      {completedAt ? (
        <div className="flex justify-between">
          <span className="text-slate-400">{t("monitor.completed_at", { defaultValue: "Completed At" })}</span>
          <span className="text-slate-700 dark:text-slate-300 font-medium">
            {formatDateTimeLocale(completedAt)}
          </span>
        </div>
      ) : null}
      {errorSummary ? (
        <div className="flex justify-between gap-4">
          <span className="text-slate-400">{t("monitor.error_summary", { defaultValue: "Error Summary" })}</span>
          <span className="max-w-[65%] truncate text-right text-slate-700 dark:text-slate-300 font-medium" title={errorSummary}>
            {errorSummary}
          </span>
        </div>
      ) : null}
    </div>
  );
}

interface JobDetailsDrawerProps {
  durableJob?: MonitorInfraJobRecord | null;
  isOpen: boolean;
  job: RecentJob | null;
  onClose: () => void;
  t: TFunction;
}

export function JobDetailsDrawer({ durableJob, isOpen, onClose, job, t }: JobDetailsDrawerProps) {
  const toast = useToast();
  const { formatDateTimeLocale } = useDateTimeFormatter();
  if (!job && !durableJob) return null;

  const copyToClipboard = (text: string, successMessage: string) => {
    void navigator.clipboard.writeText(text);
    toast.success(successMessage);
  };

  if (durableJob) {
    const { className: statusClassName, icon: statusIcon } = getStatusStyle(durableJob.state);
    return (
      <Sheet
        isOpen={isOpen}
        onClose={onClose}
        subtitle={durableJob.task_type}
        title={t("monitor.durable_job_details_title", { defaultValue: "Durable Job Details" })}
      >
        <div className="space-y-6 pb-10">
          <JobHeaderCard jobId={durableJob.job_id} onCopy={copyToClipboard} t={t} />
          <div className="grid grid-cols-2 gap-4">
            <div className="rounded-[2rem] border border-slate-100 bg-slate-50 p-6 dark:border-slate-800 dark:bg-slate-800/50">
              <Typography className="text-[10px] font-bold uppercase tracking-widest text-slate-400" variant="p">{t("monitor.status", { defaultValue: "Status" })}</Typography>
              <span className={cn("mt-3 inline-flex items-center gap-2 rounded-full px-3 py-1.5 text-xs font-bold uppercase tracking-wider", statusClassName)}>
                {statusIcon}
                {localizedJobStatus(t, durableJob.state)}
              </span>
            </div>
            <div className="rounded-[2rem] border border-slate-100 bg-slate-50 p-6 dark:border-slate-800 dark:bg-slate-800/50">
              <Typography className="text-[10px] font-bold uppercase tracking-widest text-slate-400" variant="p">{t("monitor.attempt", { defaultValue: "Attempt" })}</Typography>
              <Typography className="mt-3 text-lg font-bold text-slate-700 dark:text-slate-200" variant="h4">{durableJob.attempt_count}</Typography>
            </div>
          </div>
          <div className="rounded-[2rem] border border-slate-100 bg-slate-50 p-6 text-sm dark:border-slate-800 dark:bg-slate-800/50">
            <Typography className="text-xs font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400" variant="p">{t("monitor.job_metadata", { defaultValue: "Job Metadata" })}</Typography>
            <div className="mt-4 grid gap-3 sm:grid-cols-2">
              <div><span className="text-slate-400">{t("monitor.job_type", { defaultValue: "Type" })}</span><div className="font-medium text-slate-700 dark:text-slate-300">{durableJob.task_type}</div></div>
              <div><span className="text-slate-400">{t("monitor.priority", { defaultValue: "Priority" })}</span><div className="font-medium text-slate-700 dark:text-slate-300">{durableJob.priority}</div></div>
              <div><span className="text-slate-400">{t("monitor.parent_job", { defaultValue: "Parent job" })}</span><code className="block break-all text-xs text-slate-700 dark:text-slate-300">{durableJob.parent_job_id ?? "—"}</code></div>
              <div><span className="text-slate-400">{t("monitor.worker", { defaultValue: "Worker" })}</span><div className="break-all font-medium text-slate-700 dark:text-slate-300">{durableJob.worker ?? "—"}</div></div>
              <div><span className="text-slate-400">{t("monitor.request_id", { defaultValue: "Request ID" })}</span><code className="block break-all text-xs text-slate-700 dark:text-slate-300">{durableJob.request_id ?? "—"}</code></div>
              <div><span className="text-slate-400">{t("monitor.trace_id", { defaultValue: "Trace ID" })}</span><code className="block break-all text-xs text-slate-700 dark:text-slate-300">{durableJob.trace_id ?? "—"}</code></div>
            </div>
          </div>
          <div className="rounded-[2rem] border border-slate-100 bg-slate-50 p-6 text-sm dark:border-slate-800 dark:bg-slate-800/50">
            <Typography className="text-xs font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400" variant="p">{t("monitor.lifecycle_timestamps", { defaultValue: "Lifecycle timestamps" })}</Typography>
            <div className="mt-4 space-y-3">
              {([
                ["created_at", durableJob.created_at, "Created At"],
                ["queued_at", durableJob.queued_at, "Queued At"],
                ["started_at", durableJob.started_at, "Started At"],
                ["finished_at", durableJob.finished_at, "Finished At"],
                ["updated_at", durableJob.updated_at, "Updated At"],
              ] as const).map(([key, value, fallbackLabel]) => (
                <div className="flex justify-between gap-4" key={key}>
                  <span className="text-slate-400">{t(`monitor.${key}`, { defaultValue: fallbackLabel })}</span>
                  <span className="text-right text-slate-700 dark:text-slate-300">{value ? formatDateTimeLocale(value) : "—"}</span>
                </div>
              ))}
            </div>
            {durableJob.error_code || durableJob.error_summary ? <div className="mt-4 flex justify-between gap-4 border-t border-slate-200 pt-3 dark:border-slate-700"><span className="text-slate-400">{t("monitor.error_summary", { defaultValue: "Error Summary" })}</span><span className="max-w-[65%] text-right text-slate-700 dark:text-slate-300">{durableJob.error_code ? `${durableJob.error_code}: ` : ""}{durableJob.error_summary ?? "—"}</span></div> : null}
          </div>
          <div className="rounded-[2rem] border border-slate-100 bg-slate-50 p-6 dark:border-slate-800 dark:bg-slate-800/50">
            <Typography className="text-xs font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400" variant="p">{t("monitor.lifecycle_timeline", { defaultValue: "Lifecycle Timeline" })}</Typography>
            <ol className="mt-5 space-y-4 border-l border-slate-200 pl-5 dark:border-slate-700">
              {durableJob.events.map((event) => (
                <li className="relative" key={event.event_id}>
                  <span aria-hidden="true" className="absolute -left-[1.35rem] top-1.5 h-2 w-2 rounded-full bg-blue-500" />
                  <div className="flex flex-wrap items-center justify-between gap-2 text-xs">
                    <strong className="text-slate-700 dark:text-slate-200">{event.to_state}</strong>
                    <time className="text-slate-400" dateTime={event.created_at}>{formatDateTimeLocale(event.created_at)}</time>
                  </div>
                  <div className="mt-1 text-xs text-slate-500 dark:text-slate-400">{event.event_type}{event.worker_id ? ` · ${event.worker_id}` : ""}{event.error_code ? ` · ${event.error_code}` : ""}</div>
                  {event.error_summary ? <div className="mt-1 text-xs text-rose-600 dark:text-rose-400">{event.error_summary}</div> : null}
                </li>
              ))}
            </ol>
          </div>
          <div className="rounded-[2rem] border border-slate-100 bg-slate-50 p-6 text-sm text-slate-500 dark:border-slate-800 dark:bg-slate-800/50 dark:text-slate-400">
            {t("monitor.diagnostics_redacted", { defaultValue: "Sensitive diagnostic payloads are not displayed in the monitor." })}
          </div>
        </div>
      </Sheet>
    );
  }

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
          errorSummary={job.error_summary}
          t={t}
        />

        {job.error_summary ? (
          <div className="bg-rose-50 dark:bg-rose-950/30 border border-rose-100 dark:border-rose-900/50 rounded-[2rem] p-6 space-y-3">
            <Typography className="text-xs font-bold text-rose-700 dark:text-rose-400 uppercase tracking-widest flex items-center gap-2" variant="h5">
              <ShieldAlert aria-hidden="true" size={14} />
              {t("monitor.error_details", { defaultValue: "Error Summary" })}
            </Typography>
            <Typography className="text-xs font-mono text-rose-600 dark:text-rose-400 whitespace-pre-wrap break-all bg-white/60 dark:bg-black/20 p-4 rounded-xl border border-rose-100/50 dark:border-rose-900/30 leading-relaxed" variant="p">
              {job.error_summary}
            </Typography>
          </div>
        ) : null}

        <div className="rounded-[2rem] border border-slate-100 bg-slate-50 p-6 text-sm text-slate-500 dark:border-slate-800 dark:bg-slate-800/50 dark:text-slate-400">
          {t("monitor.diagnostics_redacted", { defaultValue: "Sensitive diagnostic payloads are not displayed in the monitor." })}
        </div>
      </div>
    </Sheet>
  );
}
