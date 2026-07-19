import * as Dialog from "@radix-ui/react-dialog";
import { AlertCircle, Download, HelpCircle, Upload, X } from "lucide-react";
import React, { memo } from "react";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { downloadCsvTemplate } from "@/lib/csv-utils";
import { cn } from "@/lib/utils";

import type { UploadResult } from "./bulk-upload-types";

import {
  ErrorDetailsTable,
  FileUploadZone,
  SuccessMessage,
  UploadSummary,
} from "./bulk-upload-components";

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

export const BulkUploadDialogContent = memo(
  // eslint-disable-next-line complexity
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
          aria-label={t("common.close")}
          className="absolute right-5 top-5 h-8 w-8 p-0 rounded-full hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors focus-visible:ring-2 focus-visible:ring-primary-500 shadow-none border-none"
          icon={<X className="h-4 w-4" />}
          onClick={handleClose}
          size="icon"
          variant="ghost"
        />
        <div className="mt-6 space-y-6">
          {!uploadResult && (
            <InstructionsPanel t={t} />
          )}
          <div className="flex justify-center">
            <Button
              className="h-10 px-5 rounded-xl border border-slate-200 dark:border-slate-800 bg-white hover:bg-slate-50 text-slate-700 dark:bg-slate-900 dark:hover:bg-slate-850 dark:text-slate-300 text-xs font-bold uppercase tracking-wider shadow-sm transition-all duration-200 active:scale-98"
              icon={<Download className="w-4 h-4" />}
              onClick={downloadCsvTemplate}
              variant="ghost"
            >
              {t("approvals.bulk_upload.download_template")}
            </Button>
          </div>
          {!uploadResult && (
            <FileUploadZone
              isDragOver={isDragOver}
              onDragLeave={() => {
                setIsDragOver(false);
              }}
              onDragOver={(e) => {
                e.preventDefault();
                setIsDragOver(true);
              }}
              onDrop={(e) => {
                e.preventDefault();
                setIsDragOver(false);
                handleFileChange(e.dataTransfer.files.item(0));
              }}
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
                {isUploading
                  ? t("common.uploading")
                  : t("approvals.bulk_upload.upload")}
              </Button>
            </div>
          )}
          {!!error && (
            <div className="flex items-center gap-3 bg-rose-50 dark:bg-rose-950 border border-rose-200 rounded-xl p-4">
              <AlertCircle className="w-5 h-5 text-rose-600 dark:text-rose-300 flex-shrink-0" />
              <Typography
                className="text-xs font-bold text-rose-600 dark:text-rose-300 uppercase tracking-wide"
                variant="p"
              >
                {error}
              </Typography>
            </div>
          )}
          {!!uploadResult && (
            <div className="space-y-4">
              <UploadSummary t={t} uploadResult={uploadResult} />
              <ErrorDetailsTable
                onDownloadErrorReport={onDownloadErrorReport}
                t={t}
                uploadResult={uploadResult}
              />
              {uploadResult.success_count > 0 && (
                <SuccessMessage count={uploadResult.success_count} t={t} />
              )}
            </div>
          )}
          <div className="flex justify-end gap-3 pt-2">
            <Button
              className={cn(
                "h-10 px-6 rounded-xl text-xs font-bold uppercase tracking-wider transition-all active:scale-98",
                uploadResult
                  ? "bg-primary-600 text-white shadow-md hover:bg-primary-700"
                  : "text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800",
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
  },
);

// ── Instructions Panel (internal) ──────────────────────────────────────

const InstructionsPanel = memo(function InstructionsPanel({
  t,
}: {
  t: (key: string, options?: Record<string, unknown>) => string;
}) {
  return (
    <div className="bg-slate-50 dark:bg-slate-800/40 border border-slate-200/50 dark:border-slate-800 rounded-2xl p-5 space-y-3.5 shadow-sm">
      <div className="flex items-center gap-2 border-b border-slate-200/60 dark:border-slate-700/60 pb-2">
        <HelpCircle className="w-4.5 h-4.5 text-primary-500" />
        <Typography
          className="text-xs font-extrabold text-slate-650 dark:text-slate-350 uppercase tracking-wider"
          variant="p"
        >
          {t("approvals.bulk_upload.instructions_title")}
        </Typography>
      </div>

      <ul className="space-y-3 text-xs text-slate-600 dark:text-slate-400">
        <li className="flex gap-2.5">
          <span className="flex items-center justify-center w-5 h-5 rounded-full bg-primary-50 dark:bg-primary-950/50 text-[10px] font-black text-primary-600 dark:text-primary-400 shrink-0">
            1
          </span>
          <span className="pt-0.5">{t("approvals.bulk_upload.step1")}</span>
        </li>
        <li className="flex gap-2.5">
          <span className="flex items-center justify-center w-5 h-5 rounded-full bg-primary-50 dark:bg-primary-950/50 text-[10px] font-black text-primary-600 dark:text-primary-400 shrink-0">
            2
          </span>
          <div className="space-y-2 pt-0.5 w-full">
            <span>{t("approvals.bulk_upload.step2")}</span>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-2 mt-1.5 bg-white dark:bg-slate-900/60 p-3 rounded-xl border border-slate-100 dark:border-slate-800/60 font-mono text-[10px] text-slate-500 dark:text-slate-400">
              <div className="flex items-start gap-1">
                <strong className="text-slate-700 dark:text-slate-200">
                  employee_id:
                </strong>{" "}
                <span>{t("approvals.bulk_upload.step2_col_id")}</span>
              </div>
              <div className="flex items-start gap-1">
                <strong className="text-slate-700 dark:text-slate-200">
                  date:
                </strong>{" "}
                <span>{t("approvals.bulk_upload.step2_col_date")}</span>
              </div>
              <div className="flex items-start gap-1">
                <strong className="text-slate-700 dark:text-slate-200">
                  minutes:
                </strong>{" "}
                <span>{t("approvals.bulk_upload.step2_col_mins")}</span>
              </div>
              <div className="flex items-start gap-1">
                <strong className="text-slate-700 dark:text-slate-200">
                  reason:
                </strong>{" "}
                <span>{t("approvals.bulk_upload.step2_col_reason")}</span>
              </div>
            </div>
          </div>
        </li>
        <li className="flex gap-2.5">
          <span className="flex items-center justify-center w-5 h-5 rounded-full bg-primary-50 dark:bg-primary-950/50 text-[10px] font-black text-primary-600 dark:text-primary-400 shrink-0">
            3
          </span>
          <span className="pt-0.5">{t("approvals.bulk_upload.step3")}</span>
        </li>
        <li className="flex gap-2.5">
          <span className="flex items-center justify-center w-5 h-5 rounded-full bg-primary-50 dark:bg-primary-950/50 text-[10px] font-black text-primary-600 dark:text-primary-400 shrink-0">
            4
          </span>
          <span className="pt-0.5">{t("approvals.bulk_upload.step4")}</span>
        </li>
      </ul>
    </div>
  );
});
