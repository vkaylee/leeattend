import { Upload, FileText, Loader2, CheckCircle2, AlertCircle, Download, AlertTriangle, XCircle } from "lucide-react";
import { useState, useEffect, useCallback, useRef } from "react";
import { useTranslation } from "react-i18next";

import type { LeaveImportJob } from "@/lib/api/leaves";

import Modal from "@/components/ui/modal";
import {
  uploadLeaveExcel,
  fetchLatestLeaveImport,
  downloadLeaveImportTemplate,
  downloadLeaveImportErrors,
  getCurrentWorkspaceId,
} from "@/lib/api";
import { cn } from "@/lib/utils";

interface LeaveImportDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onImportComplete?: () => void;
}

export default function LeaveImportDialog({
  isOpen,
  onClose,
  onImportComplete,
}: LeaveImportDialogProps) {
  const { t, i18n } = useTranslation();
  const wsId = getCurrentWorkspaceId();
  const [file, setFile] = useState<File | null>(null);
  const [uploading, setUploading] = useState(false);
  const [downloadingTemplate, setDownloadingTemplate] = useState(false);
  const [importJob, setImportJob] = useState<LeaveImportJob | null>(null);
  const pollingRef = useRef<ReturnType<typeof setInterval> | null>(null);

  // Check for latest import when dialog opens
  useEffect(() => {
    if (!isOpen || !wsId) return;
    void fetchLatestLeaveImport(wsId).then((job) => {
      if (job) {
        const created = new Date(job.created_at);
        const now = new Date();
        if (now.getTime() - created.getTime() < 24 * 60 * 60 * 1000) {
          setImportJob(job);
        }
      }
    }).catch(() => { /* empty */ });
  }, [isOpen, wsId]);

  // Polling for job status
  const startPolling = useCallback((wsIdValue: string) => {
    if (pollingRef.current) clearInterval(pollingRef.current);
    pollingRef.current = setInterval(async () => {
      try {
        const job = await fetchLatestLeaveImport(wsIdValue);
        if (job) {
          setImportJob(job);
          if (job.status !== "processing" && job.status !== "pending") {
            if (pollingRef.current) clearInterval(pollingRef.current);
            pollingRef.current = null;
            onImportComplete?.();
          }
        }
      } catch {
        // Network error — keep polling
      }
    }, 2000);
  }, [onImportComplete]);

  useEffect(() => {
    if (importJob?.status === "processing" || importJob?.status === "pending") {
      if (wsId) startPolling(wsId);
    }
    return () => {
      if (pollingRef.current) clearInterval(pollingRef.current);
    };
  }, [importJob?.status, wsId, startPolling]);

  // Reset state when dialog closes
  const [previousIsOpen, setPreviousIsOpen] = useState(isOpen);
  if (isOpen !== previousIsOpen) {
    setPreviousIsOpen(isOpen);
    if (!isOpen) setFile(null);
  }

  const handleUpload = async () => {
    if (!file || !wsId) return;
    setUploading(true);
    try {
      const result = await uploadLeaveExcel(wsId, file);
      setImportJob({
        id: result.job_id,
        status: "pending",
        total_rows: 0,
        processed_rows: 0,
        error_log: null,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      });
      setFile(null);
      startPolling(wsId);
    } catch {
      // Error handled by toast in parent or here
    } finally {
      setUploading(false);
    }
  };

  const getErrorInfo = (job: LeaveImportJob) => {
    if (!job.error_log) return null;
    try {
      return JSON.parse(job.error_log) as { success_count?: number; error_count?: number; error_file_path?: string };
    } catch { return null; }
  };

  const isProcessing = importJob?.status === "processing" || importJob?.status === "pending";
  const isFinished = importJob?.status === "completed" || importJob?.status === "failed";

  return (
    <Modal
      icon={<Upload size={24} />}
      isOpen={isOpen}
      maxWidth="lg"
      onClose={onClose}
      subtitle={t("leave_import.subtitle", "Upload file Excel để tạo đơn nghỉ phép hàng loạt")}
      testId="leave-import-dialog"
      title={t("leave_import.title", "Import đơn nghỉ phép")}
    >
      {importJob && (isProcessing || isFinished) ? (
        /* ===== PROGRESS / RESULT VIEW ===== */
        <div className="space-y-5 pt-2 animate-in fade-in duration-500">
          <div className="flex items-center justify-between px-2">
            <div className="flex items-center gap-4">
              <div className="w-11 h-11 rounded-xl bg-slate-100 dark:bg-slate-800 flex items-center justify-center shadow-inner border border-slate-200 dark:border-slate-700">
                {isProcessing ? (
                  <Loader2 className="text-primary-600 dark:text-primary-400 animate-spin" size={20} />
                ) : importJob.status === "completed" && !importJob.error_log ? (
                  <CheckCircle2 className="text-emerald-500" size={20} />
                ) : importJob.status === "completed" && importJob.error_log ? (
                  <AlertTriangle className="text-amber-500" size={20} />
                ) : (
                  <XCircle className="text-rose-500" size={20} />
                )}
              </div>
              <div>
                <h3 className="font-bold text-slate-800 dark:text-slate-200 uppercase tracking-tight text-base leading-none mb-1">
                  {isProcessing
                    ? t("leave_import.status.processing", "Đang xử lý...")
                    : importJob.status === "completed" && !importJob.error_log
                      ? t("leave_import.status.completed", "Import thành công")
                      : importJob.status === "completed" && importJob.error_log
                        ? t("leave_import.status.partial", "Import có lỗi")
                        : t("leave_import.status.failed", "Import thất bại")}
                </h3>
                <p className="text-[10px] font-bold text-slate-600 dark:text-slate-400 uppercase tracking-[0.2em]">
                  {importJob.processed_rows || 0} / {importJob.total_rows || "?"} {t("leave_import.rows_label", "dòng đã xử lý")}
                </p>
              </div>
            </div>
            {importJob.total_rows ? (
              <span className="text-[10px] font-bold text-primary-700 dark:text-primary-400 bg-primary-50 dark:bg-primary-950 px-3 py-1 rounded-full border border-primary-200 dark:border-primary-800 shadow-sm">
                {Math.round((importJob.processed_rows / importJob.total_rows) * 100)}%
              </span>
            ) : null}
          </div>

          {/* Progress bar */}
          <div className="h-2.5 w-full bg-slate-100 dark:bg-slate-700 rounded-full overflow-hidden shadow-inner border border-slate-200 dark:border-slate-700/50 relative">
            <div
              className={cn(
                "h-full transition-all duration-1000 shadow-xl",
                importJob.status === "failed"
                  ? "bg-rose-400 dark:bg-rose-500"
                  : importJob.status === "completed" && importJob.error_log
                    ? "bg-amber-400 dark:bg-amber-500"
                    : "bg-primary-600"
              )}
              style={{ width: `${importJob.total_rows ? (importJob.processed_rows / importJob.total_rows) * 100 : (isProcessing ? 15 : 0)}%` }}
            >
              {!!isProcessing && <div className="absolute inset-0 bg-gradient-to-r from-transparent via-white/20 to-transparent animate-shimmer" style={{ backgroundSize: '200% 100%' }} />}
            </div>
          </div>

          {/* Error details */}
          {importJob.status === "completed" && !!importJob.error_log && (() => {
            const info = getErrorInfo(importJob);
            return (
              <div className="bg-slate-50 dark:bg-slate-900 rounded-2xl p-6 border border-amber-200 dark:border-amber-900/50 shadow-sm">
                <p className="text-sm font-semibold text-amber-700 dark:text-amber-300 mb-3">
                  {t("leave_import.partial_detail", "Thành công {{success}} dòng, {{errors}} dòng bị lỗi")
                    .replace("{{success}}", String(info?.success_count ?? importJob.processed_rows))
                    .replace("{{errors}}", String(info?.error_count ?? 0))}
                </p>
                {!!info?.error_file_path && !!wsId && (
                  <button
                    className="inline-flex items-center gap-2 px-5 py-2.5 bg-amber-100 dark:bg-amber-900/50 text-amber-700 dark:text-amber-300 rounded-xl text-[10px] font-bold uppercase tracking-widest hover:bg-amber-200 dark:hover:bg-amber-800/50 transition-all border border-amber-200 dark:border-amber-800 active:scale-95"
                    onClick={() => { void downloadLeaveImportErrors(wsId, importJob.id).catch(() => { /* empty */ }); }}
                  >
                    <Download size={14} /> {t("leave_import.download_errors", "Tải file lỗi")}
                  </button>
                )}
              </div>
            );
          })()}

          {importJob.status === "failed" && !!importJob.error_log && (
            <div className="bg-slate-50 dark:bg-slate-900 rounded-2xl p-6 border border-rose-200 dark:border-rose-900/50 shadow-sm">
              <h4 className="text-[10px] font-bold text-rose-600 dark:text-rose-400 mb-2 flex items-center gap-2 uppercase tracking-[0.25em]">
                <AlertCircle size={14} /> {t("leave_import.error_detail", "Chi tiết lỗi")}
              </h4>
              <p className="text-xs text-slate-600 dark:text-slate-400 font-mono">{importJob.error_log.substring(0, 200)}</p>
            </div>
          )}

          {/* Footer buttons */}
          <div className="pt-3 flex gap-4 border-t border-slate-200 dark:border-slate-800">
            {!!isFinished && <>
                <button
                  className="flex-1 px-6 py-3 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 text-slate-600 dark:text-slate-400 rounded-xl font-bold text-[10px] uppercase tracking-widest hover:bg-slate-50 dark:hover:bg-slate-800 transition-all active:scale-95"
                  onClick={() => { setImportJob(null); }}
                >
                  {t("leave_import.upload_another", "Upload file khác")}
                </button>
                <button
                  className="flex-1 px-10 py-3 bg-slate-800 dark:bg-slate-700 text-white rounded-xl font-bold text-[10px] uppercase tracking-widest hover:bg-primary-700 transition-all shadow-lg active:scale-95"
                  onClick={onClose}
                >
                  {t("common.close", "Đóng").toUpperCase()}
                </button>
              </>}
          </div>
        </div>
      ) : (
        /* ===== UPLOAD VIEW ===== */
        <div className="space-y-6 pt-2 animate-in fade-in duration-500">
          {/* Instructions Panel */}
          <div className="bg-primary-50 dark:bg-primary-950/50 rounded-2xl p-5 border border-primary-200 dark:border-primary-800/50 flex gap-4">
            <div className="w-9 h-9 rounded-xl bg-white dark:bg-slate-900 flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-sm border border-primary-200 dark:border-primary-800 shrink-0">
              <AlertCircle size={18} />
            </div>
            <div className="text-xs text-primary-800 dark:text-primary-200 leading-relaxed font-medium pt-0.5">
              <p className="font-bold uppercase tracking-[0.2em] mb-2 text-slate-700 dark:text-slate-300">
                {t("leave_import.requirements_title", "Yêu cầu file")}
              </p>
              <ul className="grid grid-cols-1 md:grid-cols-2 gap-x-8 gap-y-1.5 text-slate-600 dark:text-slate-400 font-bold uppercase tracking-tight">
                <li className="flex items-center gap-2"><div className="w-1.5 h-1.5 bg-primary-500 dark:bg-primary-400 rounded-full" /> {t("leave_import.req_col_a", "Cột A: Mã nhân viên")}</li>
                <li className="flex items-center gap-2"><div className="w-1.5 h-1.5 bg-primary-500 dark:bg-primary-400 rounded-full" /> {t("leave_import.req_col_b", "Cột B: Mã loại nghỉ")}</li>
                <li className="flex items-center gap-2"><div className="w-1.5 h-1.5 bg-primary-500 dark:bg-primary-400 rounded-full" /> {t("leave_import.req_col_c", "Cột C-D: Ngày bắt đầu / kết thúc")}</li>
                <li className="flex items-center gap-2"><div className="w-1.5 h-1.5 bg-primary-500 dark:bg-primary-400 rounded-full" /> {t("leave_import.req_col_e", "Cột E: Lý do (tùy chọn)")}</li>
                <li className="flex items-center gap-2"><div className="w-1.5 h-1.5 bg-primary-500 dark:bg-primary-400 rounded-full" /> {t("leave_import.req_col_f", "Cột F: Loại thời gian")}</li>
                <li className="flex items-center gap-2"><div className="w-1.5 h-1.5 bg-primary-500 dark:bg-primary-400 rounded-full" /> {t("leave_import.req_format", "Định dạng: .xlsx, .xls, .csv")}</li>
              </ul>
            </div>
          </div>

          {/* Download Template Button */}
          <button
            className="w-full flex items-center justify-center gap-3 px-6 py-3.5 bg-emerald-50 dark:bg-emerald-950/30 border border-emerald-200 dark:border-emerald-800/50 rounded-2xl text-emerald-700 dark:text-emerald-300 font-bold text-[10px] uppercase tracking-widest hover:bg-emerald-100 dark:hover:bg-emerald-900/30 transition-all active:scale-[0.98] group disabled:opacity-50"
            disabled={downloadingTemplate || !wsId}
            onClick={async () => {
              if (!wsId) return;
              setDownloadingTemplate(true);
              try {
                await downloadLeaveImportTemplate(wsId, i18n.language);
              } catch { /* handled */ } finally {
                setDownloadingTemplate(false);
              }
            }}
          >
            {downloadingTemplate ? <Loader2 className="animate-spin" size={16} /> : <Download className="group-hover:animate-bounce" size={16} />}
            {t("leave_import.download_template", "Tải file mẫu Excel")}
          </button>

          {/* File Drop Zone */}
          { }
          <div
            className={cn(
              "border-3 border-dashed rounded-2xl p-10 flex flex-col items-center justify-center transition-all cursor-pointer group active:scale-[0.98] relative overflow-hidden",
              file
                ? "border-emerald-400 bg-emerald-50 dark:bg-emerald-950/20"
                : "border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-800/30 hover:border-primary-400 hover:bg-primary-50 dark:hover:bg-primary-950/30"
            )}
            onClick={() => document.getElementById("leave-import-file-input")?.click()}
            onKeyDown={(e) => {
              if (e.key === 'Enter' || e.key === ' ') {
                e.preventDefault();
                document.getElementById("leave-import-file-input")?.click();
              }
            }}
            role="button"
            tabIndex={0}
          >
            <input
              accept=".xlsx,.xls,.csv"
              aria-label="File Upload"
              className="hidden"
              id="leave-import-file-input"
              onChange={(e) => { setFile(e.target.files?.[0] ?? null); }}
              type="file"
            />
            {file ? (
              <div className="text-center animate-in zoom-in-95 duration-500">
                <div className="w-16 h-16 bg-emerald-100 dark:bg-emerald-900 rounded-2xl flex items-center justify-center mx-auto mb-5 shadow-lg shadow-emerald-200/50 dark:shadow-emerald-900/30 rotate-3 group-hover:rotate-0 transition-transform">
                  <FileText className="text-emerald-600 dark:text-emerald-300" size={28} />
                </div>
                <p className="font-bold text-slate-800 dark:text-slate-200 mb-1 text-lg tracking-tight uppercase">{file.name}</p>
                <p className="text-[11px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
                  {(file.size / 1024).toFixed(1)} KB — {t("leave_import.ready", "SẴN SÀNG UPLOAD")}
                </p>
                <button
                  className="mt-5 text-[10px] font-bold text-rose-600 dark:text-rose-400 uppercase tracking-widest hover:text-rose-700 dark:hover:text-rose-300 transition-all bg-white dark:bg-slate-900 px-5 py-2 rounded-lg border border-rose-200 dark:border-rose-800 shadow-sm hover:shadow-md active:scale-90"
                  onClick={(e) => { e.stopPropagation(); setFile(null); }}
                >
                  {t("leave_import.select_another", "Chọn file khác")}
                </button>
              </div>
            ) : (
              <div className="text-center group-hover:scale-105 transition-transform duration-500">
                <div className="w-16 h-16 bg-white dark:bg-slate-900 rounded-2xl flex items-center justify-center mx-auto mb-5 shadow-lg border border-slate-200 dark:border-slate-700 -rotate-3 group-hover:rotate-0 transition-all">
                  <Upload className="text-slate-400 dark:text-slate-500" size={28} />
                </div>
                <p className="font-bold text-slate-800 dark:text-slate-200 mb-1 text-lg tracking-tight uppercase">
                  {t("leave_import.drop_prompt", "Nhấn để chọn file Excel")}
                </p>
                <p className="text-[11px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
                  {t("leave_import.drop_hint", "Hỗ trợ .xlsx, .xls, .csv")}
                </p>
              </div>
            )}
          </div>

          {/* Footer buttons */}
          <div className="flex gap-4 pt-3 border-t border-slate-200 dark:border-slate-800">
            <button
              className="flex-1 px-6 py-3 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 text-slate-600 dark:text-slate-400 rounded-xl font-bold text-[10px] uppercase tracking-widest hover:bg-slate-50 dark:hover:bg-slate-800 transition-all active:scale-95"
              onClick={onClose}
            >
              {t("common.cancel", "Hủy")}
            </button>
            <button
              className="flex-[1.5] py-3 bg-primary-600 text-white rounded-xl font-bold text-[10px] uppercase tracking-widest hover:bg-primary-700 transition-all shadow-lg shadow-primary-200/50 dark:shadow-primary-900/30 active:scale-95 disabled:opacity-50 flex items-center justify-center gap-3"
              disabled={!file || uploading}
              onClick={() => { void handleUpload(); }}
            >
              {uploading ? (
                <><Loader2 className="animate-spin" size={18} /> {t("leave_import.uploading", "ĐANG UPLOAD...")}</>
              ) : (
                t("leave_import.start_import", "BẮT ĐẦU IMPORT").toUpperCase()
              )}
            </button>
          </div>
        </div>
      )}
    </Modal>
  );
}
