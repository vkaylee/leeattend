import type { TFunction } from "i18next";

import { CheckCircle2, XCircle, AlertTriangle, Download, Loader2, X } from "lucide-react";
import React from "react";

import type { LeaveImportJob } from "@/lib/api/leaves";

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

export function LeaveImportBanner({ importJob, importBannerDismissed, setImportBannerDismissed, t, wsId }: LeaveImportBannerProps) {
  if (!importJob || importBannerDismissed) return null;

  return (
    <div className={cn(
      "rounded-2xl px-6 py-4 flex items-center justify-between gap-4 animate-in slide-in-from-top-4 duration-500 border",
      importJob.status === 'processing' || importJob.status === 'pending'
        ? "bg-blue-50 dark:bg-blue-950/30 border-blue-200 dark:border-blue-800/50"
        : importJob.status === 'completed' && !importJob.error_log
          ? "bg-emerald-50 dark:bg-emerald-950/30 border-emerald-200 dark:border-emerald-800/50"
          : importJob.status === 'completed' && importJob.error_log
            ? "bg-amber-50 dark:bg-amber-950/30 border-amber-200 dark:border-amber-800/50"
            : "bg-red-50 dark:bg-red-950/30 border-red-200 dark:border-red-800/50"
    )}>
      <div className="flex items-center gap-3 flex-1 min-w-0">
        {(importJob.status === 'processing' || importJob.status === 'pending') && (
          <>
            <Loader2 className="animate-spin text-blue-500 flex-shrink-0" size={20} />
            <div>
              <p className="text-sm font-semibold text-blue-700 dark:text-blue-300">
                {t("leave_requests.upload_processing", "Đang xử lý {{processed}}/{{total}} dòng...").replace('{{processed}}', String(importJob.processed_rows)).replace('{{total}}', String(importJob.total_rows || '?'))}
              </p>
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
        )}
        {importJob.status === 'completed' && !importJob.error_log && (
          <>
            <CheckCircle2 className="text-emerald-500 flex-shrink-0" size={20} />
            <p className="text-sm font-semibold text-emerald-700 dark:text-emerald-300">
              {t("leave_requests.upload_success", "Import thành công {{success}} đơn nghỉ").replace('{{success}}', String(importJob.processed_rows))}
            </p>
          </>
        )}
        {importJob.status === 'completed' && !!importJob.error_log && (() => {
          const info = getImportErrorInfo(importJob);
          return (
            <>
              <AlertTriangle className="text-amber-500 flex-shrink-0" size={20} />
              <p className="text-sm font-semibold text-amber-700 dark:text-amber-300">
                {t("leave_requests.upload_partial", "Thành công {{success}}/{{total}}. {{errors}} dòng lỗi.")
                  .replace('{{success}}', String(info?.success_count ?? importJob.processed_rows))
                  .replace('{{total}}', String((info?.success_count ?? 0) + (info?.error_count ?? 0)))
                  .replace('{{errors}}', String(info?.error_count ?? 0))}
              </p>
              {info?.error_file_path ? (
                <button
                  className="ml-2 flex items-center gap-1.5 text-xs font-bold text-amber-600 hover:text-amber-800 dark:text-amber-400 dark:hover:text-amber-200 underline underline-offset-2"
                  onClick={() => { void downloadLeaveImportErrors(wsId, importJob.id).catch(() => { /* empty */ }); }}
                >
                  <Download size={14} /> {t("leave_requests.download_errors", "Tải file lỗi")}
                </button>
              ) : null}
            </>
          );
        })()}
        {importJob.status === 'failed' && (
          <>
            <XCircle className="text-red-500 flex-shrink-0" size={20} />
            <p className="text-sm font-semibold text-red-700 dark:text-red-300">
              {t("leave_requests.upload_failed", "Import thất bại")}
              {importJob.error_log ? <span className="font-normal ml-2 text-xs opacity-75">— {importJob.error_log.substring(0, 100)}</span> : null}
            </p>
          </>
        )}
      </div>
      {importJob.status !== 'processing' && importJob.status !== 'pending' && (
        <button className="text-primary-400 hover:text-primary-600 transition-colors flex-shrink-0" onClick={() => { setImportBannerDismissed(true); }}>
          <X size={18} />
        </button>
      )}
    </div>
  );
}
