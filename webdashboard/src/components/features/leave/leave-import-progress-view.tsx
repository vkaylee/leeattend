import type { TFunction } from "i18next";

import { Loader2, CheckCircle2, AlertTriangle, XCircle, Download, AlertCircle } from "lucide-react";
import React from "react";

import type { LeaveImportJob } from "@/lib/api/leaves";

import Button from "@/components/ui/button";
import { toast } from "@/components/ui/toast-imperative";
import { Typography } from "@/components/ui/typography";
import { downloadLeaveImportErrors } from "@/lib/api";
import { logError } from "@/lib/logger";
import { cn } from "@/lib/utils";

export interface LeaveImportProgressViewProps {
  getErrorInfo: (job: LeaveImportJob) => { success_count?: number; error_count?: number; error_file_path?: string } | null;
  importJob: LeaveImportJob;
  isFinished: boolean;
  isProcessing: boolean;
  onClose: () => void;
  setImportJob: (job: LeaveImportJob | null) => void;
  t: TFunction;
  wsId: string | null;
}

function getStatusIcon(status: string, hasError: boolean, isProcessing: boolean) {
  if (isProcessing) return <Loader2 className="text-primary-600 dark:text-primary-400 animate-spin" size={20} />;
  if (status === "completed" && !hasError) return <CheckCircle2 className="text-emerald-500" size={20} />;
  if (status === "completed" && hasError) return <AlertTriangle className="text-amber-500" size={20} />;
  if (status === "partially_failed") return <AlertTriangle className="text-amber-500" size={20} />;
  return <XCircle className="text-rose-500" size={20} />;
}

function getStatusText(status: string, hasError: boolean, isProcessing: boolean, t: TFunction) {
  if (isProcessing) return t("leave_import.status.processing", "Đang xử lý...");
  if (status === "completed" && !hasError) return t("leave_import.status.completed", "Import thành công");
  if (status === "completed" && hasError) return t("leave_import.status.partial", "Import có lỗi");
  if (status === "partially_failed") return t("leave_import.status.partially_failed", "Import một phần thành công");
  return t("leave_import.status.failed", "Import thất bại");
}

function ProgressHeader({ importJob, isProcessing, t }: { importJob: LeaveImportJob; isProcessing: boolean; t: TFunction }) {
  const hasError = Boolean(importJob.error_log);
  return (
    <div className="flex items-center justify-between px-2">
      <div className="flex items-center gap-4">
        <div className="w-11 h-11 rounded-xl bg-slate-100 dark:bg-slate-800 flex items-center justify-center shadow-inner border border-slate-200 dark:border-slate-700">
          {getStatusIcon(importJob.status, hasError, isProcessing)}
        </div>
        <div>
          <Typography className="font-bold text-slate-800 dark:text-slate-200 uppercase tracking-tight text-base leading-none mb-1" variant="h3">
            {getStatusText(importJob.status, hasError, isProcessing, t)}
          </Typography>
          <Typography className="text-[10px] font-bold text-slate-600 dark:text-slate-400 uppercase tracking-[0.2em]" variant="p">
            {importJob.processed_rows || 0} / {importJob.total_rows || "?"} {t("leave_import.rows_label", "dòng đã xử lý")}
          </Typography>
        </div>
      </div>
      {importJob.total_rows ? (
        <span className="text-[10px] font-bold text-primary-700 dark:text-primary-400 bg-primary-50 dark:bg-primary-950 px-3 py-1 rounded-full border border-primary-200 dark:border-primary-800 shadow-sm">
          {Math.round((importJob.processed_rows / importJob.total_rows) * 100)}%
        </span>
      ) : null}
    </div>
  );
}

interface ErrorInfo {
  error_count?: number;
  error_file_path?: string;
  success_count?: number;
}

function PartialSuccessDetails({ info, importJob, t, wsId }: { info: ErrorInfo | null; importJob: LeaveImportJob; t: TFunction; wsId: string | null }) {
  return (
    <div className="bg-slate-50 dark:bg-slate-900 rounded-2xl p-6 border border-amber-200 dark:border-amber-900/50 shadow-sm">
      <Typography className="text-sm font-semibold text-amber-700 dark:text-amber-300 mb-3" variant="p">
        {t("leave_import.partial_detail", "Thành công {{success}} dòng, {{errors}} dòng bị lỗi")
          .replace("{{success}}", String(info?.success_count ?? importJob.processed_rows))
          .replace("{{errors}}", String(info?.error_count ?? 0))}
      </Typography>
      {Boolean(info?.error_file_path) && Boolean(wsId) ? (
        <Button
          className="border-amber-200 bg-amber-100 text-[10px] text-amber-700 hover:bg-amber-200 dark:border-amber-800 dark:bg-amber-900/50 dark:text-amber-300 dark:hover:bg-amber-800/50"
          onClick={() => {
            if (wsId) {
              void downloadLeaveImportErrors(wsId, importJob.id).catch((e: unknown) => { logError(e, { component: 'LeaveImportProgressView' }); toast.error(t("leave_import.download_errors_failed", "Tải file lỗi thất bại")); });
            }
          }}
          size="sm"
          variant="outline"
        >
          <Download size={14} /> {t("leave_import.download_errors", "Tải file lỗi")}
        </Button>
      ) : null}
    </div>
  );
}

function FailedDetails({ errorLog, importJob, t, wsId }: { errorLog: string; importJob: LeaveImportJob; t: TFunction; wsId: string | null }) {
  let info: ErrorInfo | null = null;
  try {
    info = JSON.parse(errorLog) as ErrorInfo;
  } catch { /* JSON.parse on error_log — invalid JSON returns null to show raw error text */ }

  return (
    <div className="bg-slate-50 dark:bg-slate-900 rounded-2xl p-6 border border-rose-200 dark:border-rose-900/50 shadow-sm">
      <Typography className="text-[10px] font-bold text-rose-600 dark:text-rose-400 mb-2 flex items-center gap-2 uppercase tracking-[0.25em]" variant="h4">
        <AlertCircle size={14} /> {t("leave_import.error_detail", "Chi tiết lỗi")}
      </Typography>
      {info ? (
        <div className="space-y-3">
          <Typography className="text-xs text-slate-600 dark:text-slate-400" variant="p">
            {t("leave_import.failed_detail_count", "Import thất bại: {{errors}} dòng bị lỗi").replace("{{errors}}", String(info.error_count ?? 1))}
          </Typography>
          {Boolean(info.error_file_path) && Boolean(wsId) ? (
            <Button
              className="border-rose-200 bg-rose-50 text-[10px] text-rose-700 hover:bg-rose-100 dark:border-rose-800 dark:bg-rose-950/30 dark:text-rose-300 dark:hover:bg-rose-900/50"
              onClick={() => {
                if (wsId) {
                  void downloadLeaveImportErrors(wsId, importJob.id).catch((e: unknown) => { logError(e, { component: 'LeaveImportProgressView' }); toast.error(t("leave_import.download_errors_failed", "Tải file lỗi thất bại")); });
                }
              }}
              size="sm"
              variant="outline"
            >
              <Download size={14} /> {t("leave_import.download_errors", "Tải file lỗi")}
            </Button>
          ) : null}
        </div>
      ) : (
        <Typography className="text-xs text-slate-600 dark:text-slate-400 font-mono" variant="p">
          {errorLog.substring(0, 200)}
        </Typography>
      )}
    </div>
  );
}

function ErrorDetails({ getErrorInfo, importJob, t, wsId }: { getErrorInfo: (job: LeaveImportJob) => { success_count?: number; error_count?: number; error_file_path?: string } | null; importJob: LeaveImportJob; t: TFunction; wsId: string | null }) {
  if ((importJob.status === "completed" || importJob.status === "partially_failed") && importJob.error_log) {
    return <PartialSuccessDetails importJob={importJob} info={getErrorInfo(importJob)} t={t} wsId={wsId} />;
  }
  if (importJob.status === "failed" && importJob.error_log) {
    return <FailedDetails errorLog={importJob.error_log} importJob={importJob} t={t} wsId={wsId} />;
  }
  return null;
}

export function LeaveImportProgressView({ importJob, isProcessing, isFinished, wsId, getErrorInfo, t, setImportJob, onClose }: LeaveImportProgressViewProps) {
  return (
    <div className="space-y-5 pt-2 animate-in fade-in duration-500">
      <ProgressHeader importJob={importJob} isProcessing={isProcessing} t={t} />

      {/* Progress bar */}
      <div className="h-2.5 w-full bg-slate-100 dark:bg-slate-700 rounded-full overflow-hidden shadow-inner border border-slate-200 dark:border-slate-700/50 relative">
        <div
          className={cn(
            "h-full transition-all duration-1000 shadow-xl",
            importJob.status === "failed"
              ? "bg-rose-400 dark:bg-rose-500"
              : (importJob.status === "completed" && importJob.error_log) || importJob.status === "partially_failed"
                ? "bg-amber-400 dark:bg-amber-500"
                : "bg-primary-600"
          )}
          style={{ width: `${importJob.total_rows ? (importJob.processed_rows / importJob.total_rows) * 100 : (isProcessing ? 15 : 0)}%` }}
        >
          {isProcessing ? <div className="absolute inset-0 bg-gradient-to-r from-transparent via-white/20 to-transparent animate-shimmer bg-[length:200%_100%]" /> : null}
        </div>
      </div>

      {/* Error details */}
      <ErrorDetails getErrorInfo={getErrorInfo} importJob={importJob} t={t} wsId={wsId} />

      {/* Footer buttons */}
      <div className="pt-3 flex gap-4 border-t border-slate-200 dark:border-slate-800">
        {isFinished ? (
          <>
            <Button
              className="flex-1 h-auto py-3 px-6 text-[10px]"
              onClick={() => { setImportJob(null); }}
              size="sm"
              variant="outline"
            >
              {t("leave_import.upload_another", "Upload file khác")}
            </Button>
            <Button
              className="flex-1 h-auto py-3 px-10 text-[10px] bg-slate-800 dark:bg-slate-700"
              onClick={onClose}
              size="sm"
              variant="secondary"
            >
              {t("common.close", "Đóng").toUpperCase()}
            </Button>
          </>
        ) : null}
      </div>
    </div>
  );
}
