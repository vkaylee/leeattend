import type { TFunction } from "i18next";

import { CheckCircle2, XCircle, AlertTriangle, Download, Loader2, X } from "lucide-react";
import React from "react";

import type { LeaveImportJob } from "@/lib/api/leaves";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { downloadLeaveImportErrors } from "@/lib/api";
import { cn } from "@/lib/utils";

function getImportErrorInfo(job: LeaveImportJob) {
  if (!job.error_log) return null;
  try {
    return JSON.parse(job.error_log) as { success_count?: number; error_count?: number; error_file_path?: string };
  } catch { return null; }
}

export interface LeaveImportBannerProps {
  importBannerDismissed: boolean;
  importJob: LeaveImportJob | null;
  setImportBannerDismissed: (val: boolean) => void;
  t: TFunction;
  wsId: string;
}

function ProcessingState({ importJob, t }: { importJob: LeaveImportJob; t: TFunction }) {
  if (importJob.status !== 'processing' && importJob.status !== 'pending') return null;
  return (
    <>
      <Loader2 className="animate-spin text-blue-500 flex-shrink-0" size={20} />
      <div>
        <Typography className="text-sm font-semibold text-blue-700 dark:text-blue-300" variant="p">
          {t("leave_requests.upload_processing", "Đang xử lý {{processed}}/{{total}} dòng...").replace('{{processed}}', String(importJob.processed_rows)).replace('{{total}}', String(importJob.total_rows || '?'))}
        </Typography>
        {importJob.total_rows > 0 && (
          <div className="mt-2 h-2 rounded-full bg-blue-100 dark:bg-blue-900 overflow-hidden w-48">
            <div
              className="h-full rounded-full bg-blue-500 transition-all duration-500"
              style={{ width: `${Math.min(100, (importJob.processed_rows / importJob.total_rows) * 100).toString()}%` }}
            />
          </div>
        )}
      </div>
    </>
  );
}

function SuccessState({ importJob, t }: { importJob: LeaveImportJob; t: TFunction }) {
  if (importJob.status !== 'completed' || importJob.error_log) return null;
  return (
    <>
      <CheckCircle2 className="text-emerald-500 flex-shrink-0" size={20} />
      <Typography className="text-sm font-semibold text-emerald-700 dark:text-emerald-300" variant="p">
        {t("leave_requests.upload_success", "Import thành công {{success}} đơn nghỉ").replace('{{success}}', String(importJob.processed_rows))}
      </Typography>
    </>
  );
}

function PartialSuccessState({ importJob, wsId, t }: { importJob: LeaveImportJob; wsId: string; t: TFunction }) {
  if (importJob.status !== 'completed' || !importJob.error_log) return null;
  const info = getImportErrorInfo(importJob);
  const successCount = info?.success_count ?? importJob.processed_rows;
  const errorCount = info?.error_count ?? 0;
  const totalCount = successCount + errorCount;
  return (
    <>
      <AlertTriangle className="text-amber-500 flex-shrink-0" size={20} />
      <Typography className="text-sm font-semibold text-amber-700 dark:text-amber-300" variant="p">
        {t("leave_requests.upload_partial", "Thành công {{success}}/{{total}}. {{errors}} dòng lỗi.")
          .replace('{{success}}', String(successCount))
          .replace('{{total}}', String(totalCount))
          .replace('{{errors}}', String(errorCount))}
      </Typography>
      {info?.error_file_path ? (
        <Button
          className="ml-2 flex items-center gap-1.5 text-xs font-bold text-amber-600 hover:text-amber-800 dark:text-amber-400 dark:hover:text-amber-200 underline underline-offset-2 border-none shadow-none h-auto w-auto p-0 hover:bg-transparent"
          onClick={() => { void downloadLeaveImportErrors(wsId, importJob.id).catch(() => { /* empty */ }); }}
          variant="ghost"
        >
          <Download size={14} /> {t("leave_requests.download_errors", "Tải file lỗi")}
        </Button>
      ) : null}
    </>
  );
}

function FailedState({ importJob, wsId, t }: { importJob: LeaveImportJob; wsId: string; t: TFunction }) {
  if (importJob.status !== 'failed') return null;
  const info = getImportErrorInfo(importJob);
  const errorCount = info?.error_count ?? 1;
  return (
    <>
      <XCircle className="text-red-500 flex-shrink-0" size={20} />
      <div className="flex items-center gap-3">
        <Typography className="text-sm font-semibold text-red-700 dark:text-red-300" variant="p">
          {info
            ? t("leave_requests.upload_failed_count", "Import thất bại: {{errors}} dòng lỗi.").replace('{{errors}}', String(errorCount))
            : t("leave_requests.upload_failed", "Import thất bại")
          }
          {!info && importJob.error_log ? <span className="font-normal ml-2 text-xs opacity-75">— {importJob.error_log.substring(0, 100)}</span> : null}
        </Typography>
        {info?.error_file_path ? (
          <Button
            className="flex items-center gap-1.5 text-xs font-bold text-red-600 hover:text-red-800 dark:text-red-400 dark:hover:text-red-200 underline underline-offset-2 border-none shadow-none h-auto w-auto p-0 hover:bg-transparent"
            onClick={() => { void downloadLeaveImportErrors(wsId, importJob.id).catch(() => { /* empty */ }); }}
            variant="ghost"
          >
            <Download size={14} /> {t("leave_requests.download_errors", "Tải file lỗi")}
          </Button>
        ) : null}
      </div>
    </>
  );
}

function getBannerClassName(job: LeaveImportJob) {
  if (job.status === 'processing' || job.status === 'pending') {
    return "bg-blue-50 dark:bg-blue-950/30 border-blue-200 dark:border-blue-800/50";
  }
  if (job.status === 'completed' && !job.error_log) {
    return "bg-emerald-50 dark:bg-emerald-950/30 border-emerald-200 dark:border-emerald-800/50";
  }
  if (job.status === 'completed' && job.error_log) {
    return "bg-amber-50 dark:bg-amber-950/30 border-amber-200 dark:border-amber-800/50";
  }
  return "bg-red-50 dark:bg-red-950/30 border-red-200 dark:border-red-800/50";
}

export function LeaveImportBanner({ importJob, importBannerDismissed, setImportBannerDismissed, t, wsId }: LeaveImportBannerProps) {
  if (!importJob || importBannerDismissed) return null;

  const showCloseButton = importJob.status !== 'processing' && importJob.status !== 'pending';

  return (
    <div className={cn(
      "rounded-2xl px-6 py-4 flex items-center justify-between gap-4 animate-in slide-in-from-top-4 duration-500 border",
      getBannerClassName(importJob)
    )}>
      <div className="flex items-center gap-3 flex-1 min-w-0">
        <ProcessingState importJob={importJob} t={t} />
        <SuccessState importJob={importJob} t={t} />
        <PartialSuccessState importJob={importJob} t={t} wsId={wsId} />
        <FailedState importJob={importJob} t={t} wsId={wsId} />
      </div>
      {showCloseButton ? (
        <Button 
          className="text-primary-400 hover:text-primary-600 transition-colors flex-shrink-0 border-none shadow-none h-auto w-auto p-0 hover:bg-transparent" 
          onClick={() => { setImportBannerDismissed(true); }}
          size="icon"
          variant="ghost"
        >
          <X size={18} />
        </Button>
      ) : null}
    </div>
  );
}
