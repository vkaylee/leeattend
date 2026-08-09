import { CheckCircle2, Download, FileText, Upload, XCircle } from "lucide-react";
import React, { memo } from "react";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";

import type { UploadResult } from "./bulk-upload-types";

// ── FileUploadZone ────────────────────────────────────────────────────

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

export const FileUploadZone = memo(function FileUploadZone({
  isDragOver,
  onDragLeave,
  onDragOver,
  onDrop,
  onFileChange,
  selectedFile,
  setSelectedFile,
  t,
}: FileUploadZoneProps) {
  if (selectedFile) {
    return (
      <div className="border-2 border-dashed rounded-xl p-8 text-center border-slate-200 dark:border-slate-700">
        <div className="space-y-3">
          <FileText className="w-12 h-12 text-primary-600 dark:text-primary-400 mx-auto" />
          <Typography
            className="text-sm font-bold text-slate-700 dark:text-slate-200"
            variant="p"
          >
            {selectedFile.name}
          </Typography>
          <Typography
            className="text-xs text-slate-500 dark:text-slate-400"
            variant="p"
          >
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
      className={`block border-2 border-dashed rounded-xl p-8 text-center transition-all cursor-pointer ${
        isDragOver
          ? "border-primary-500 bg-primary-50 dark:bg-primary-950"
          : "border-slate-200 dark:border-slate-700 hover:border-primary-400"
      }`}
      htmlFor="csv-upload"
      onDragLeave={onDragLeave}
      onDragOver={onDragOver}
      onDrop={onDrop}
    >
      <Upload className="w-12 h-12 text-slate-400 dark:text-slate-500 mx-auto mb-3" />
      <Typography
        className="text-sm font-bold text-slate-700 dark:text-slate-200 mb-1"
        variant="p"
      >
        {t("approvals.bulk_upload.drag_drop")}
      </Typography>
      <Typography
        className="text-xs text-slate-500 dark:text-slate-400 mb-4"
        variant="p"
      >
        {t("approvals.bulk_upload.or_click")}
      </Typography>
      <input
        accept=".csv"
        className="hidden"
        id="csv-upload"
        onChange={(e) => {
          onFileChange(e.target.files?.[0] ?? null);
        }}
        type="file"
      />
      <span className="inline-block px-6 py-2 rounded-xl bg-primary-600 text-white text-xs font-bold uppercase tracking-wider hover:bg-primary-700">
        {t("approvals.bulk_upload.select_file")}
      </span>
    </label>
  );
});

// ── UploadSummary ─────────────────────────────────────────────────────

interface UploadSummaryProps {
  t: (key: string, options?: Record<string, unknown>) => string;
  uploadResult: UploadResult;
}

export const UploadSummary = memo(function UploadSummary({
  t,
  uploadResult,
}: UploadSummaryProps) {
  return (
    <div className="bg-slate-50 dark:bg-slate-800 rounded-xl p-4 space-y-2">
      <Typography
        className="text-sm font-bold text-slate-700 dark:text-slate-200 uppercase tracking-wide"
        variant="h3"
      >
        {t("approvals.bulk_upload.result_summary")}
      </Typography>
      <div className="grid grid-cols-3 gap-3 text-center">
        <div className="bg-white dark:bg-slate-900 rounded-lg p-3 border border-slate-100 dark:border-slate-800">
          <Typography
            className="text-xs text-slate-500 dark:text-slate-400 uppercase tracking-wider mb-1"
            variant="p"
          >
            {t("approvals.bulk_upload.total")}
          </Typography>
          <Typography
            className="text-2xl font-bold text-slate-700 dark:text-slate-200"
            variant="p"
          >
            {uploadResult.total_rows}
          </Typography>
        </div>
        <div className="bg-emerald-50 dark:bg-emerald-950 rounded-lg p-3 border border-emerald-200 dark:border-emerald-800">
          <Typography
            className="text-xs text-emerald-600 dark:text-emerald-300 uppercase tracking-wider mb-1 flex items-center justify-center gap-1"
            variant="p"
          >
            <CheckCircle2 className="w-3 h-3" />
            {t("approvals.bulk_upload.success")}
          </Typography>
          <Typography
            className="text-2xl font-bold text-emerald-700 dark:text-emerald-300"
            variant="p"
          >
            {uploadResult.success_count}
          </Typography>
        </div>
        <div className="bg-rose-50 dark:bg-rose-950 rounded-lg p-3 border border-rose-200">
          <Typography
            className="text-xs text-rose-600 dark:text-rose-300 uppercase tracking-wider mb-1 flex items-center justify-center gap-1"
            variant="p"
          >
            <XCircle className="w-3 h-3" />
            {t("approvals.bulk_upload.failed")}
          </Typography>
          <Typography
            className="text-2xl font-bold text-rose-700 dark:text-rose-300"
            variant="p"
          >
            {uploadResult.failed_count}
          </Typography>
        </div>
      </div>
    </div>
  );
});

// ── ErrorDetailsTable ─────────────────────────────────────────────────

interface ErrorDetailsTableProps {
  onDownloadErrorReport: () => void;
  t: (key: string, options?: Record<string, unknown>) => string;
  uploadResult: UploadResult;
}

export const ErrorDetailsTable = memo(function ErrorDetailsTable({
  onDownloadErrorReport,
  t,
  uploadResult,
}: ErrorDetailsTableProps) {
  if (uploadResult.errors.length === 0) return null;

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <Typography
          className="text-sm font-bold text-slate-700 dark:text-slate-200 uppercase tracking-wide"
          variant="h3"
        >
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

      <div className="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl overflow-auto overscroll-contain max-h-64">
        <table className="w-full min-w-[520px] text-left">
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
              <tr
                className="hover:bg-slate-50 dark:hover:bg-slate-800"
                key={index}
              >
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
});

// ── SuccessMessage ────────────────────────────────────────────────────

interface SuccessMessageProps {
  count: number;
  t: (key: string, options?: Record<string, unknown>) => string;
}

export const SuccessMessage = memo(function SuccessMessage({
  count,
  t,
}: SuccessMessageProps) {
  return (
    <div className="flex items-center gap-3 bg-emerald-50 dark:bg-emerald-950 border border-emerald-200 dark:border-emerald-800 rounded-xl p-4">
      <CheckCircle2 className="w-5 h-5 text-emerald-600 dark:text-emerald-300 flex-shrink-0" />
      <Typography
        className="text-xs font-bold text-emerald-700 dark:text-emerald-300 uppercase tracking-wide"
        variant="p"
      >
        {t("approvals.bulk_upload.success_message", { count }) ||
          `${count.toString()} OT claims entered successfully`}
      </Typography>
    </div>
  );
});
