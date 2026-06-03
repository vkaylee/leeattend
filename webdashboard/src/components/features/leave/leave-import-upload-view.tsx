import type { TFunction, i18n as I18nType } from "i18next";

import { Upload, FileText, Loader2, AlertCircle, Download } from "lucide-react";
import React from "react";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { downloadLeaveImportTemplate } from "@/lib/api";
import { cn } from "@/lib/utils";

export interface LeaveImportUploadViewProps {
  downloadingTemplate: boolean;
  file: File | null;
  handleUpload: () => Promise<void>;
  i18n: I18nType;
  onClose: () => void;
  setDownloadingTemplate: (val: boolean) => void;
  setFile: (file: File | null) => void;
  t: TFunction;
  uploading: boolean;
  wsId: string | null;
}

function RequirementsPanel({ t }: { t: TFunction }) {
  return (
    <div className="bg-primary-50 dark:bg-primary-950/50 rounded-2xl p-5 border border-primary-200 dark:border-primary-800/50 flex gap-4">
      <div className="w-9 h-9 rounded-xl bg-white dark:bg-slate-900 flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-sm border border-primary-200 dark:border-primary-800 shrink-0">
        <AlertCircle size={18} />
      </div>
      <div className="text-xs text-primary-800 dark:text-primary-200 leading-relaxed font-medium pt-0.5 w-full">
        <Typography className="font-bold uppercase tracking-[0.2em] mb-1.5 text-slate-700 dark:text-slate-300" variant="p">
          {t("leave_import.requirements_title", "Yêu cầu file")}
        </Typography>
        <Typography className="text-[11px] text-slate-500 dark:text-slate-400 mb-3 font-normal normal-case leading-normal" variant="p">
          {t("leave_import.req_auto_detect", "💡 Hệ thống nhận diện cột tự động theo tiêu đề ở hàng đầu tiên (Header). Thứ tự các cột không quan trọng.")}
        </Typography>
        <ul className="grid grid-cols-1 md:grid-cols-2 gap-x-8 gap-y-1.5 text-slate-600 dark:text-slate-400 font-medium tracking-normal">
          <li className="flex items-center gap-2"><div className="w-1.5 h-1.5 bg-primary-500 dark:bg-primary-400 rounded-full" /> {t("leave_import.req_col_a", "Cột employee_code: Mã nhân viên")}</li>
          <li className="flex items-center gap-2"><div className="w-1.5 h-1.5 bg-primary-500 dark:bg-primary-400 rounded-full" /> {t("leave_import.req_col_b", "Cột leave_type_code: Mã loại nghỉ")}</li>
          <li className="flex items-center gap-2"><div className="w-1.5 h-1.5 bg-primary-500 dark:bg-primary-400 rounded-full" /> {t("leave_import.req_col_c", "Cột start_date: Ngày bắt đầu")}</li>
          <li className="flex items-center gap-2"><div className="w-1.5 h-1.5 bg-primary-500 dark:bg-primary-400 rounded-full" /> {t("leave_import.req_col_d", "Cột end_date: Ngày kết thúc")}</li>
          <li className="flex items-center gap-2"><div className="w-1.5 h-1.5 bg-primary-500 dark:bg-primary-400 rounded-full" /> {t("leave_import.req_col_e", "Cột reason: Lý do (tùy chọn)")}</li>
          <li className="flex items-center gap-2"><div className="w-1.5 h-1.5 bg-primary-500 dark:bg-primary-400 rounded-full" /> {t("leave_import.req_col_f", "Cột duration_type: Loại thời gian")}</li>
        </ul>
        <div className="mt-3 pt-3 border-t border-primary-100 dark:border-primary-900/50 text-[10px] text-slate-500 dark:text-slate-400 font-semibold tracking-wider flex items-center gap-2">
          <div className="w-1.5 h-1.5 bg-primary-500 dark:bg-primary-400 rounded-full" />
          {t("leave_import.req_format", "Định dạng: .xlsx, .xls, .csv")}
        </div>
      </div>
    </div>
  );
}

function FileDropZone({ file, setFile, t }: { file: File | null; setFile: (file: File | null) => void; t: TFunction }) {
  return (
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
          <Typography className="font-bold text-slate-800 dark:text-slate-200 mb-1 text-lg tracking-tight uppercase" variant="p">{file.name}</Typography>
          <Typography className="text-[11px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest" variant="p">
            {(file.size / 1024).toFixed(1)} KB — {t("leave_import.ready", "SẴN SÀNG UPLOAD")}
          </Typography>
          <Button
            className="mt-5 text-[10px] px-5 py-2 h-auto rounded-lg shadow-sm border-rose-200 text-rose-600 hover:text-rose-700 dark:border-rose-800 dark:text-rose-400 dark:hover:text-rose-300"
            onClick={(e) => { e.stopPropagation(); setFile(null); }}
            size="sm"
            variant="outline"
          >
            {t("leave_import.select_another", "Chọn file khác")}
          </Button>
        </div>
      ) : (
        <div className="text-center group-hover:scale-105 transition-transform duration-500">
          <div className="w-16 h-16 bg-white dark:bg-slate-900 rounded-2xl flex items-center justify-center mx-auto mb-5 shadow-lg border border-slate-200 dark:border-slate-700 -rotate-3 group-hover:rotate-0 transition-all">
            <Upload className="text-slate-400 dark:text-slate-500" size={28} />
          </div>
          <Typography className="font-bold text-slate-800 dark:text-slate-200 mb-1 text-lg tracking-tight uppercase" variant="p">
            {t("leave_import.drop_prompt", "Nhấn để chọn file Excel")}
          </Typography>
          <Typography className="text-[11px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest" variant="p">
            {t("leave_import.drop_hint", "Hỗ trợ .xlsx, .xls, .csv")}
          </Typography>
        </div>
      )}
    </div>
  );
}

export function LeaveImportUploadView({ file, setFile, wsId, i18n, t, uploading, handleUpload, onClose, downloadingTemplate, setDownloadingTemplate }: LeaveImportUploadViewProps) {
  return (
    <div className="space-y-6 pt-2 animate-in fade-in duration-500">
      {/* Instructions Panel */}
      <RequirementsPanel t={t} />

      {/* Download Template Button */}
      <Button
        className="w-full h-auto py-3.5 border-emerald-200 bg-emerald-50 text-[10px] text-emerald-700 hover:bg-emerald-100 dark:border-emerald-800/50 dark:bg-emerald-950/30 dark:text-emerald-300 dark:hover:bg-emerald-900/30 group"
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
        size="md"
        variant="outline"
      >
        {downloadingTemplate ? <Loader2 className="animate-spin" size={16} /> : <Download className="group-hover:animate-bounce" size={16} />}
        {t("leave_import.download_template", "Tải file mẫu Excel")}
      </Button>

      {/* File Drop Zone */}
      <FileDropZone file={file} setFile={setFile} t={t} />

      {/* Footer buttons */}
      <div className="flex gap-4 pt-3 border-t border-slate-200 dark:border-slate-800">
        <Button
          className="flex-1 h-auto py-3 px-6 text-[10px]"
          onClick={onClose}
          size="sm"
          variant="outline"
        >
          {t("common.cancel", "Hủy")}
        </Button>
        <Button
          className="flex-[1.5] h-auto py-3 text-[10px]"
          disabled={!file || uploading}
          onClick={() => { void handleUpload(); }}
          size="sm"
          variant="primary"
        >
          {uploading ? (
            <><Loader2 className="animate-spin" size={18} /> {t("leave_import.uploading", "ĐANG UPLOAD...")}</>
          ) : (
            t("leave_import.start_import", "BẮT ĐẦU IMPORT").toUpperCase()
          )}
        </Button>
      </div>
    </div>
  );
}
