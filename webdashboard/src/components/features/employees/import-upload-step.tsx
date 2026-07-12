import { AlertCircle, FileText, Upload } from "lucide-react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { cn } from "@/lib/utils";

interface Props {
  file: File | null;
  loading: boolean;
  onFileChange: (file: File | null) => void;
  onDownloadTemplate: () => void;
  onUpload: () => void;
  onClose: () => void;
}

export default function ImportUploadStep({
  file,
  loading,
  onFileChange,
  onDownloadTemplate,
  onUpload,
  onClose,
}: Props) {
  const { t } = useTranslation("employees");

  return (
    <div className="space-y-6 pt-2 animate-in fade-in duration-500">
      {/* File Requirements & Guidelines */}
      <div className="bg-primary-50 dark:bg-primary-950/50 rounded-2xl p-5 border border-primary-200 dark:border-primary-800/50 flex gap-4">
        <div className="w-9 h-9 rounded-xl bg-white dark:bg-slate-900 flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-sm border border-primary-200 dark:border-primary-800 shrink-0">
          <AlertCircle size={18} />
        </div>
        <div className="text-xs text-primary-800 dark:text-primary-200 leading-relaxed font-medium pt-0.5 flex-1">
          <Typography className="font-bold uppercase tracking-[0.2em] mb-2 text-slate-700 dark:text-slate-300 block" variant="p">
            {t("employees.bulk_termination.format_req_title")}
          </Typography>
          <Typography className="text-slate-500 dark:text-slate-400 text-[11px] italic mb-3 block" variant="p">
            {t("employees.bulk_termination.format_req_desc")}
          </Typography>
          <ul className="space-y-3 mt-2 text-slate-700 dark:text-slate-300 font-medium text-xs">
            {(["employee_code", "termination_date", "termination_reason", "termination_notes"] as const).map(
              (key) => {
                const isOptional = key === "termination_notes";
                return (
                  <li key={key} className="flex flex-col space-y-1">
                    <div className="flex items-center gap-2">
                      <div className="w-1.5 h-1.5 bg-primary-500 dark:bg-primary-400 rounded-full shrink-0" />
                      <span className="font-mono text-[11px] font-bold bg-white dark:bg-slate-900 px-2 py-0.5 rounded border border-slate-200 dark:border-slate-800 text-primary-700 dark:text-primary-400 shadow-sm">
                        {key}
                      </span>
                      <span className="text-xs font-bold text-slate-800 dark:text-slate-200">
                        — {t(`employees.bulk_termination.${key}_title`)}
                      </span>
                      <span
                        className={cn(
                          "text-[9px] font-bold px-2 py-0.5 rounded-full uppercase tracking-wider",
                          isOptional
                            ? "bg-slate-50 dark:bg-slate-800 text-slate-500 border border-slate-200 dark:border-slate-700"
                            : "bg-rose-50 dark:bg-rose-950/30 text-rose-600 dark:text-rose-400 border border-rose-100 dark:border-rose-900/50",
                        )}
                      >
                        {t(isOptional ? "employees.bulk_termination.optional_badge" : "employees.bulk_termination.required_badge")}
                      </span>
                    </div>
                  </li>
                );
              },
            )}
          </ul>
          <div className="mt-4 flex items-center">
            <Button
              className="text-[10px] font-bold text-primary-600 dark:text-primary-400 uppercase tracking-widest hover:text-primary-700 hover:underline flex items-center gap-1.5 cursor-pointer bg-transparent border-0 p-0 h-auto min-w-0"
              onClick={onDownloadTemplate}
              type="button"
              variant="ghost"
            >
              <FileText size={14} /> {t("employees.bulk_termination.download_template_btn")}
            </Button>
          </div>
        </div>
      </div>

      {/* Drag & Drop File upload */}
      <label
        className={cn(
          "border-3 border-dashed rounded-2xl p-10 flex flex-col items-center justify-center transition-all cursor-pointer group active:scale-[0.98] relative overflow-hidden w-full",
          file
            ? "border-emerald-400 bg-emerald-50 dark:bg-emerald-950/20"
            : "border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-800/30 hover:border-primary-400 hover:bg-primary-50 dark:hover:bg-primary-950/30",
        )}
        htmlFor="file-upload"
      >
        <input
          accept=".xlsx,.xls"
          aria-label={t("employees.bulk_termination.uploading_btn")}
          className="hidden"
          id="file-upload"
          onChange={(e) => onFileChange(e.target.files?.[0] ?? null)}
          type="file"
        />
        {file ? (
          <div className="text-center animate-in zoom-in-95 duration-500">
            <div className="w-16 h-16 bg-emerald-100 dark:bg-emerald-900 rounded-2xl flex items-center justify-center mx-auto mb-5 shadow-lg shadow-emerald-200/50 dark:shadow-emerald-900/30 rotate-3 group-hover:rotate-0 transition-transform">
              <FileText className="text-emerald-600 dark:text-emerald-300" size={28} />
            </div>
            <Typography className="font-bold text-slate-800 dark:text-slate-200 mb-1 text-lg tracking-tight uppercase" variant="p">
              {file.name}
            </Typography>
            <Typography className="text-[11px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest" variant="p">
              {(file.size / 1024).toFixed(1)} KB — {t("employees.bulk_termination.file_ready_label")}
            </Typography>
            <Button
              className="mt-5 border-rose-200 dark:border-rose-800 text-rose-600 dark:text-rose-400 hover:text-rose-700 dark:hover:text-rose-300 hover:bg-rose-50 dark:hover:bg-rose-950 shadow-sm hover:shadow-md"
              onClick={(e) => { e.stopPropagation(); onFileChange(null); }}
              size="sm"
              variant="outline"
            >
              {t("employees.bulk_termination.choose_other_file")}
            </Button>
          </div>
        ) : (
          <div className="text-center group-hover:scale-105 transition-transform duration-500">
            <div className="w-16 h-16 bg-white dark:bg-slate-900 rounded-2xl flex items-center justify-center mx-auto mb-5 shadow-lg border border-slate-200 dark:border-slate-700 -rotate-3 group-hover:rotate-0 transition-all">
              <Upload className="text-slate-400 dark:text-slate-500" size={28} />
            </div>
            <Typography className="font-bold text-slate-800 dark:text-slate-200 mb-1 text-lg tracking-tight uppercase" variant="p">
              {t("employees.bulk_termination.drag_drop_placeholder")}
            </Typography>
            <Typography className="text-[11px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest" variant="p">
              {t("employees.bulk_termination.file_support_tip")}
            </Typography>
          </div>
        )}
      </label>

      {/* Actions */}
      <div className="flex gap-4 pt-3 border-t border-slate-200 dark:border-slate-800">
        <Button className="flex-1" onClick={onClose} size="lg" variant="outline">
          {t("employees.bulk_termination.cancel_btn")}
        </Button>
        <Button
          className="flex-[1.5]"
          disabled={!file || loading}
          loading={loading}
          onClick={onUpload}
          size="lg"
          variant="primary"
        >
          {loading ? t("employees.bulk_termination.uploading_btn") : t("employees.bulk_termination.continue_btn")}
        </Button>
      </div>
    </div>
  );
}
