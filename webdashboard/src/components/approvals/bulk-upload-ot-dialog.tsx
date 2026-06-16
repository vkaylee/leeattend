/* eslint-disable complexity */
import * as Dialog from "@radix-ui/react-dialog";
import { X, Upload, Download, FileText, AlertCircle, CheckCircle2, XCircle, HelpCircle } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { useBulkUploadPaperOt } from "@/hooks/use-attendance";
import { useConfirm } from "@/hooks/use-confirm";
import { cn } from "@/lib/utils";

interface Props {
  onOpenChange: (open: boolean) => void;
  onSuccess?: () => void;
  open: boolean;
}

interface UploadResult {
  errors: {
    line_number: number;
    employment_id: string;
    error_message: string;
  }[];
  failed_count: number;
  success_count: number;
  total_rows: number;
}

interface FileUploadZoneProps {
  isDragOver: boolean;
  onDragLeave: () => void;
  onDragOver: (e: React.DragEvent) => void;
  onDrop: (e: React.DragEvent) => void;
  onFileChange: (file: File | null) => void;
  selectedFile: File | null;
  setSelectedFile: (file: File | null) => void;
  t: (key: string, options?: Record<string, unknown>) => string;
}

interface UploadSummaryProps {
  t: (key: string, options?: Record<string, unknown>) => string;
  uploadResult: UploadResult;
}

interface ErrorDetailsTableProps {
  onDownloadErrorReport: () => void;
  t: (key: string, options?: Record<string, unknown>) => string;
  uploadResult: UploadResult;
}

interface SuccessMessageProps {
  count: number;
  t: (key: string, options?: Record<string, unknown>) => string;
}

interface BulkUploadDialogContentProps {
  error: string | null;
  handleClose: () => void;
  handleFileChange: (file: File | null) => void;
  handleUpload: () => void;
  isDragOver: boolean;
  isUploading: boolean;
  onDownloadErrorReport: () => void;
  selectedFile: File | null;
  setIsDragOver: (over: boolean) => void;
  setSelectedFile: (file: File | null) => void;
  t: (key: string, options?: Record<string, unknown>) => string;
  uploadResult: UploadResult | null;
}

function FileUploadZone({
  isDragOver,
  onDragLeave,
  onDragOver,
  onDrop,
  onFileChange,
  selectedFile,
  setSelectedFile,
  t
}: FileUploadZoneProps) {
  if (selectedFile) {
    return (
      <div className="border-2 border-dashed rounded-xl p-8 text-center border-slate-200 dark:border-slate-700">
        <div className="space-y-3">
          <FileText className="w-12 h-12 text-primary-600 dark:text-primary-400 mx-auto" />
          <Typography className="text-sm font-bold text-slate-700 dark:text-slate-200" variant="p">{selectedFile.name}</Typography>
          <Typography className="text-xs text-slate-500 dark:text-slate-400" variant="p">
            {(selectedFile.size / 1024).toFixed(2)} KB
          </Typography>
          <Button
            className="h-auto p-0 text-xs text-rose-600 dark:text-rose-300 hover:underline hover:bg-transparent dark:hover:bg-transparent shadow-none"
            onClick={(e) => {
              e.stopPropagation();
              setSelectedFile(null);
            }}
            variant="ghost"
          >
            {t("common.remove")}
          </Button>
        </div>
      </div>
    );
  }

  return (
    <label
      htmlFor="csv-upload"
      className={`block border-2 border-dashed rounded-xl p-8 text-center transition-all cursor-pointer ${
        isDragOver
          ? "border-primary-500 bg-primary-50 dark:bg-primary-950"
          : "border-slate-200 dark:border-slate-700 hover:border-primary-400"
      }`}
      onDragLeave={onDragLeave}
      onDragOver={onDragOver}
      onDrop={onDrop}
    >
      <Upload className="w-12 h-12 text-slate-400 dark:text-slate-500 mx-auto mb-3" />
      <Typography className="text-sm font-bold text-slate-700 dark:text-slate-200 mb-1" variant="p">
        {t("approvals.bulk_upload.drag_drop")}
      </Typography>
      <Typography className="text-xs text-slate-500 dark:text-slate-400 mb-4" variant="p">
        {t("approvals.bulk_upload.or_click")}
      </Typography>
      <input
        accept=".csv"
        className="hidden"
        id="csv-upload"
        onChange={(e) => { onFileChange(e.target.files?.[0] ?? null); }}
        type="file"
      />
      <span
        className="inline-block px-6 py-2 rounded-xl bg-primary-600 text-white text-xs font-bold uppercase tracking-wider hover:bg-primary-700"
      >
        {t("approvals.bulk_upload.select_file")}
      </span>
    </label>
  );
}

function UploadSummary({ t, uploadResult }: UploadSummaryProps) {
  return (
    <div className="bg-slate-50 dark:bg-slate-800 rounded-xl p-4 space-y-2">
      <Typography className="text-sm font-bold text-slate-700 dark:text-slate-200 uppercase tracking-wide" variant="h3">
        {t("approvals.bulk_upload.result_summary")}
      </Typography>
      <div className="grid grid-cols-3 gap-3 text-center">
        <div className="bg-white dark:bg-slate-900 rounded-lg p-3 border border-slate-100 dark:border-slate-800">
          <Typography className="text-xs text-slate-500 dark:text-slate-400 uppercase tracking-wider mb-1" variant="p">
            {t("approvals.bulk_upload.total")}
          </Typography>
          <Typography className="text-2xl font-bold text-slate-700 dark:text-slate-200" variant="p">{uploadResult.total_rows}</Typography>
        </div>
        <div className="bg-emerald-50 dark:bg-emerald-950 rounded-lg p-3 border border-emerald-200 dark:border-emerald-800">
          <Typography className="text-xs text-emerald-600 dark:text-emerald-300 uppercase tracking-wider mb-1 flex items-center justify-center gap-1" variant="p">
            <CheckCircle2 className="w-3 h-3" />
            {t("approvals.bulk_upload.success")}
          </Typography>
          <Typography className="text-2xl font-bold text-emerald-700 dark:text-emerald-300" variant="p">{uploadResult.success_count}</Typography>
        </div>
        <div className="bg-rose-50 dark:bg-rose-950 rounded-lg p-3 border border-rose-200">
          <Typography className="text-xs text-rose-600 dark:text-rose-300 uppercase tracking-wider mb-1 flex items-center justify-center gap-1" variant="p">
            <XCircle className="w-3 h-3" />
            {t("approvals.bulk_upload.failed")}
          </Typography>
          <Typography className="text-2xl font-bold text-rose-700 dark:text-rose-300" variant="p">{uploadResult.failed_count}</Typography>
        </div>
      </div>
    </div>
  );
}

function ErrorDetailsTable({ onDownloadErrorReport, t, uploadResult }: ErrorDetailsTableProps) {
  if (uploadResult.errors.length === 0) return null;

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <Typography className="text-sm font-bold text-slate-700 dark:text-slate-200 uppercase tracking-wide" variant="h3">
          {t("approvals.bulk_upload.error_details")}
        </Typography>
        <Button
          className="h-auto px-4 py-2 rounded-xl bg-rose-100 dark:bg-rose-900 hover:bg-rose-200 text-rose-700 dark:text-rose-300 text-xs font-bold uppercase tracking-wider flex items-center"
          icon={<Download className="w-3 h-3" />}
          onClick={onDownloadErrorReport}
          variant="outline"
        >
          {t("approvals.bulk_upload.download_errors")}
        </Button>
      </div>

      <div className="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl overflow-hidden max-h-64 overflow-y-auto">
        <table className="w-full text-left">
          <thead className="bg-slate-50 dark:bg-slate-800 sticky top-0">
            <tr>
              <th className="px-4 py-2 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider">
                {t("approvals.bulk_upload.line")}
              </th>
              <th className="px-4 py-2 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider">
                {t("common.employee")}
              </th>
              <th className="px-4 py-2 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider">
                {t("common.error")}
              </th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-100 dark:divide-slate-800">
            {uploadResult.errors.map((error_, index) => (
              <tr className="hover:bg-slate-50 dark:hover:bg-slate-800" key={index}>
                <td className="px-4 py-2 text-xs font-mono text-slate-600 dark:text-slate-300">
                  {error_.line_number}
                </td>
                <td className="px-4 py-2 text-xs font-mono text-slate-600 dark:text-slate-300">
                  {error_.employment_id}
                </td>
                <td className="px-4 py-2 text-xs text-rose-600 dark:text-rose-300">
                  {error_.error_message}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

function downloadTemplate() {
  const csvContent = `employee_id,date,minutes,reason
E001,2026-01-28,120,Production line overtime - paper form #123
E002,2026-01-28,90,Urgent order completion`;

  const blob = new Blob([csvContent], { type: "text/csv" });
  const url = window.URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = "ot_upload_template.csv";
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  window.URL.revokeObjectURL(url);
}

function downloadErrorReport(uploadResult: UploadResult) {
  if (uploadResult.errors.length === 0) return;

  const headers = "line_number,employment_id,error_message";
  const rows = uploadResult.errors
    .map(e => `${e.line_number.toString()},${e.employment_id},"${e.error_message}"`)
    .join("\n");

  const csvContent = `${headers}\n${rows}`;
  const blob = new Blob([csvContent], { type: "text/csv" });
  const url = window.URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = `ot_upload_errors_${new Date().getTime().toString()}.csv`;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  window.URL.revokeObjectURL(url);
}

function SuccessMessage({ count, t }: SuccessMessageProps) {
  return (
    <div className="flex items-center gap-3 bg-emerald-50 dark:bg-emerald-950 border border-emerald-200 dark:border-emerald-800 rounded-xl p-4">
      <CheckCircle2 className="w-5 h-5 text-emerald-600 dark:text-emerald-300 flex-shrink-0" />
      <Typography className="text-xs font-bold text-emerald-700 dark:text-emerald-300 uppercase tracking-wide" variant="p">
        {t("approvals.bulk_upload.success_message", { count }) ||
          `${count.toString()} OT claims entered successfully`}
      </Typography>
    </div>
  );
}

function useBulkUploadForm(onSuccess?: () => void) {
  const { t } = useTranslation();
  const { bulkUploadPaperOt, isUploading } = useBulkUploadPaperOt();

  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [uploadResult, setUploadResult] = useState<UploadResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isDragOver, setIsDragOver] = useState(false);

  const resetForm = () => {
    setSelectedFile(null);
    setUploadResult(null);
    setError(null);
    setIsDragOver(false);
  };

  const handleFileChange = (file: File | null) => {
    if (!file) return;
    if (!file.name.endsWith(".csv")) {
      setError(t("approvals.bulk_upload.error_csv_only"));
      return;
    }
    setSelectedFile(file);
    setError(null);
    setUploadResult(null);
  };

  const handleUpload = async () => {
    if (!selectedFile) return;
    setError(null);
    try {
      const result = await bulkUploadPaperOt(selectedFile);
      setUploadResult(result);
      if (result.success_count > 0) onSuccess?.();
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : "Failed to upload file");
    }
  };

  return {
    error,
    handleFileChange,
    handleUpload,
    isDragOver,
    isUploading,
    resetForm,
    selectedFile,
    setError,
    setIsDragOver,
    setSelectedFile,
    setUploadResult,
    t,
    uploadResult
  };
}

function BulkUploadDialogContent({
  error,
  handleClose,
  handleFileChange,
  handleUpload,
  isDragOver,
  isUploading,
  onDownloadErrorReport,
  selectedFile,
  setIsDragOver,
  setSelectedFile,
  t,
  uploadResult,
}: BulkUploadDialogContentProps) {
  return (
    <Dialog.Content className="fixed left-1/2 top-1/2 z-50 w-full max-w-2xl -translate-x-1/2 -translate-y-1/2 border border-slate-150 dark:border-slate-800 bg-white dark:bg-slate-900 p-7 shadow-2xl rounded-[2.5rem] max-h-[92vh] overflow-y-auto custom-scrollbar animate-in fade-in zoom-in-95 duration-200">
      <Dialog.Title className="text-xl font-black text-slate-800 dark:text-slate-100 uppercase tracking-tight flex items-center gap-2">
        <Upload className="w-5.5 h-5.5 text-primary-500" />
        {t("approvals.bulk_upload.title")}
      </Dialog.Title>
      <Dialog.Description className="text-xs text-slate-500 dark:text-slate-400 mt-1.5 leading-relaxed">
        {t("approvals.bulk_upload.description")}
      </Dialog.Description>
      <Button
        className="absolute right-5 top-5 h-8 w-8 p-0 rounded-full hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors focus-visible:ring-2 focus-visible:ring-primary-500 shadow-none border-none"
        icon={<X className="h-4 w-4" />}
        onClick={handleClose}
        size="icon"
        variant="ghost"
        aria-label="Close dialog"
      />
      <div className="mt-6 space-y-6">
        {!uploadResult && (
          <div className="bg-slate-50 dark:bg-slate-800/40 border border-slate-200/50 dark:border-slate-800 rounded-2xl p-5 space-y-3.5 shadow-sm">
            <div className="flex items-center gap-2 border-b border-slate-200/60 dark:border-slate-700/60 pb-2">
              <HelpCircle className="w-4.5 h-4.5 text-primary-500" />
              <Typography className="text-xs font-extrabold text-slate-650 dark:text-slate-350 uppercase tracking-wider" variant="p">
                {t("approvals.bulk_upload.instructions_title")}
              </Typography>
            </div>
            
            <ul className="space-y-3 text-xs text-slate-600 dark:text-slate-400">
              <li className="flex gap-2.5">
                <span className="flex items-center justify-center w-5 h-5 rounded-full bg-primary-50 dark:bg-primary-950/50 text-[10px] font-black text-primary-600 dark:text-primary-400 shrink-0">1</span>
                <span className="pt-0.5">
                  {t("approvals.bulk_upload.step1")}
                </span>
              </li>
              <li className="flex gap-2.5">
                <span className="flex items-center justify-center w-5 h-5 rounded-full bg-primary-50 dark:bg-primary-950/50 text-[10px] font-black text-primary-600 dark:text-primary-400 shrink-0">2</span>
                <div className="space-y-2 pt-0.5 w-full">
                  <span>
                    {t("approvals.bulk_upload.step2")}
                  </span>
                  <div className="grid grid-cols-1 sm:grid-cols-2 gap-2 mt-1.5 bg-white dark:bg-slate-900/60 p-3 rounded-xl border border-slate-100 dark:border-slate-800/60 font-mono text-[10px] text-slate-500 dark:text-slate-400">
                    <div className="flex items-start gap-1"><strong className="text-slate-700 dark:text-slate-200">employee_id:</strong> <span>{t("approvals.bulk_upload.step2_col_id")}</span></div>
                    <div className="flex items-start gap-1"><strong className="text-slate-700 dark:text-slate-200">date:</strong> <span>{t("approvals.bulk_upload.step2_col_date")}</span></div>
                    <div className="flex items-start gap-1"><strong className="text-slate-700 dark:text-slate-200">minutes:</strong> <span>{t("approvals.bulk_upload.step2_col_mins")}</span></div>
                    <div className="flex items-start gap-1"><strong className="text-slate-700 dark:text-slate-200">reason:</strong> <span>{t("approvals.bulk_upload.step2_col_reason")}</span></div>
                  </div>
                </div>
              </li>
              <li className="flex gap-2.5">
                <span className="flex items-center justify-center w-5 h-5 rounded-full bg-primary-50 dark:bg-primary-950/50 text-[10px] font-black text-primary-600 dark:text-primary-400 shrink-0">3</span>
                <span className="pt-0.5">
                  {t("approvals.bulk_upload.step3")}
                </span>
              </li>
              <li className="flex gap-2.5">
                <span className="flex items-center justify-center w-5 h-5 rounded-full bg-primary-50 dark:bg-primary-950/50 text-[10px] font-black text-primary-600 dark:text-primary-400 shrink-0">4</span>
                <span className="pt-0.5">
                  {t("approvals.bulk_upload.step4")}
                </span>
              </li>
            </ul>
          </div>
        )}
        <div className="flex justify-center">
          <Button
            className="h-10 px-5 rounded-xl border border-slate-200 dark:border-slate-800 bg-white hover:bg-slate-50 text-slate-700 dark:bg-slate-900 dark:hover:bg-slate-850 dark:text-slate-300 text-xs font-bold uppercase tracking-wider shadow-sm transition-all duration-200 active:scale-98"
            icon={<Download className="w-4 h-4" />}
            onClick={downloadTemplate}
            variant="ghost"
          >
            {t("approvals.bulk_upload.download_template")}
          </Button>
        </div>
        {!uploadResult && (
          <FileUploadZone
            isDragOver={isDragOver}
            onDragLeave={() => { setIsDragOver(false); }}
            onDragOver={(e) => { e.preventDefault(); setIsDragOver(true); }}
            onDrop={(e) => { e.preventDefault(); setIsDragOver(false); handleFileChange(e.dataTransfer.files.item(0)); }}
            onFileChange={handleFileChange}
            selectedFile={selectedFile}
            setSelectedFile={setSelectedFile}
            t={t}
          />
        )}
        {!!selectedFile && !uploadResult && (
          <div className="flex justify-center">
            <Button
              className="h-11 px-8 rounded-xl font-bold uppercase tracking-wider shadow-md transition-all active:scale-98"
              disabled={isUploading}
              icon={!isUploading && <Upload className="w-4 h-4" />}
              loading={isUploading}
              onClick={handleUpload}
              variant="primary"
            >
              {isUploading ? t("common.uploading") : t("approvals.bulk_upload.upload")}
            </Button>
          </div>
        )}
        {!!error && (
          <div className="flex items-center gap-3 bg-rose-50 dark:bg-rose-950 border border-rose-200 rounded-xl p-4">
            <AlertCircle className="w-5 h-5 text-rose-600 dark:text-rose-300 flex-shrink-0" />
            <Typography className="text-xs font-bold text-rose-600 dark:text-rose-300 uppercase tracking-wide" variant="p">{error}</Typography>
          </div>
        )}
        {!!uploadResult && (
          <div className="space-y-4">
            <UploadSummary t={t} uploadResult={uploadResult} />
            <ErrorDetailsTable onDownloadErrorReport={onDownloadErrorReport} t={t} uploadResult={uploadResult} />
            {uploadResult.success_count > 0 && <SuccessMessage count={uploadResult.success_count} t={t} />}
          </div>
        )}
        <div className="flex justify-end gap-3 pt-2">
          <Button
            className={cn(
              "h-10 px-6 rounded-xl text-xs font-bold uppercase tracking-wider transition-all active:scale-98",
              uploadResult
                ? "bg-primary-600 text-white shadow-md hover:bg-primary-700"
                : "text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800"
            )}
            onClick={handleClose}
            variant={uploadResult ? "primary" : "ghost"}
          >
            {uploadResult ? t("common.close") : t("common.cancel")}
          </Button>
        </div>
      </div>
    </Dialog.Content>
  );
}

export function BulkUploadOtDialog({ open, onOpenChange, onSuccess }: Props) {
  const { confirm, ConfirmationDialog } = useConfirm();
  const form = useBulkUploadForm(onSuccess);
  const { t, uploadResult, resetForm, isDragOver, setIsDragOver, handleFileChange, selectedFile, setSelectedFile, isUploading, handleUpload, error } = form;

  const handleClose = async () => {
    if (uploadResult && uploadResult.failed_count > 0) {
      const ok = await confirm({
        title: t("common.warning", { defaultValue: "Warning" }),
        message: t("approvals.bulk_upload.confirm_close_with_errors", { defaultValue: "Some rows failed. Are you sure you want to close?" }),
        variant: "danger"
      });
      if (!ok) return;
    }
    resetForm();
    onOpenChange(false);
  };

  return (
    <Dialog.Root onOpenChange={(o) => { if (!o) { void handleClose(); } }} open={open}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-black/40 z-50 backdrop-blur-sm" />
        <BulkUploadDialogContent
          error={error}
          handleClose={() => { void handleClose(); }}
          handleFileChange={handleFileChange}
          handleUpload={() => { void handleUpload(); }}
          isDragOver={isDragOver}
          isUploading={isUploading}
          onDownloadErrorReport={() => { if (uploadResult) downloadErrorReport(uploadResult); }}
          selectedFile={selectedFile}
          setIsDragOver={setIsDragOver}
          setSelectedFile={setSelectedFile}
          t={t}
          uploadResult={uploadResult}
        />
      </Dialog.Portal>
      <ConfirmationDialog />
    </Dialog.Root>
  );
}

