import { CheckCircle2, Clock, Copy, Loader2, ShieldAlert, XCircle } from "lucide-react";
import React from "react";

import { TFunction } from "i18next";

import Button from "@/components/ui/button";
import Sheet from "@/components/ui/sheet";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { RecentJob } from "@/lib/api/types";
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
        icon: <Loader2 className="animate-spin" size={12} />,
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

interface JobDetailsDrawerProps {
  isOpen: boolean;
  job: RecentJob | null;
  onClose: () => void;
  t: TFunction;
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
