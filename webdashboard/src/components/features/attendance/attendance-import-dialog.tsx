import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Upload, FileText, Loader2, CheckCircle2, AlertCircle } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import { Typography } from "@/components/ui/typography";
import { uploadAttendance, fetchLatestImportJob, fetchImportJobStatus, getCurrentWorkspaceId } from "@/lib/api";
import { AttendanceImportJob } from "@/lib/api/types";
import { cn } from "@/lib/utils";

interface AttendanceImportDialogProps {
  customRequirements?: string[];
  endpoint: string; // e.g., "/attendance/upload" or "/attendance/logs/upload"
  isOpen: boolean;
  onClose: () => void;
  targetType: string; // "adjustment", "log", or "employee"
  title: string;
}

interface ImportProgressProps {
  jobStatus: AttendanceImportJob | undefined;
  t: (key: string) => string;
}

const getStatusConfig = (status: string | undefined, t: (key: string) => string) => {
  switch (status) {
    case "processing":
      return {
        icon: <Loader2 className="text-primary-600 dark:text-primary-400 animate-spin" size={20} />,
        text: t("attendance_import.status.processing"),
      };
    case "completed":
      return {
        icon: <CheckCircle2 className="text-emerald-500" size={20} />,
        text: t("attendance_import.status.completed"),
      };
    case "failed":
      return {
        icon: <AlertCircle className="text-rose-500" size={20} />,
        text: t("attendance_import.status.failed"),
      };
    default:
      return {
        icon: <Loader2 className="text-slate-500 dark:text-slate-400 animate-spin" size={20} />,
        text: t("attendance_import.status.pending"),
      };
  }
};

function ErrorLog({ errorLog, t }: { errorLog: string; t: (key: string) => string }) {
  return (
    <div className="bg-slate-50 dark:bg-slate-900 rounded-2xl p-6 border border-rose-200 dark:border-rose-900/50 shadow-sm relative overflow-hidden">
      <Typography className="text-[10px] font-bold text-rose-600 dark:text-rose-400 mb-4 flex items-center gap-2 uppercase tracking-[0.25em]" variant="h4"><AlertCircle size={14} /> {t("attendance_import.error_list_title")}</Typography>
      <div className="space-y-2 max-h-48 overflow-y-auto pr-2 custom-scrollbar relative z-10">
        {errorLog.split("\n").map((error, index) => {
          if (error.startsWith("DL_LINK:")) {
            const url = error.replace("DL_LINK:", "").trim();
            return (
              <a className="mb-4 text-[11px] bg-rose-50 dark:bg-rose-900/30 p-3 rounded-lg border border-rose-200 dark:border-rose-800 text-rose-700 dark:text-rose-400 font-bold flex items-center gap-2 hover:bg-rose-100 dark:hover:bg-rose-900/50 transition-colors" download href={url} key={index}>
                <FileText size={16} /> Tải file Excel chứa danh sách các dòng lỗi / bỏ qua
              </a>
            );
          }
          return (
            <div className="text-[11px] bg-white dark:bg-slate-800 p-3 rounded-lg border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-300 font-mono leading-relaxed break-words" key={index}>
              <span className="text-rose-500 mr-2 font-bold uppercase">ERR:</span> {error}
            </div>
          );
        })}
      </div>
    </div>
  );
}

function ProgressBar({ percentage, status }: { percentage: number; status: string | undefined }) {
  return (
    <div className="h-2.5 w-full bg-slate-100 dark:bg-slate-700 rounded-full overflow-hidden shadow-inner border border-slate-200 dark:border-slate-700/50 relative">
      <div className={cn("h-full transition-all duration-1000 shadow-xl", status === "failed" ? "bg-rose-400 dark:bg-rose-500" : "bg-primary-600")} style={{ width: `${percentage.toString()}%` }}>
         <div className="absolute inset-0 bg-gradient-to-r from-transparent via-white/20 to-transparent animate-shimmer" style={{ backgroundSize: '200% 100%' }} />
      </div>
    </div>
  );
}

function ImportProgressInfo({ processed, total, percentage, config, t }: { processed: number; total: number; percentage: number; config: { icon: React.ReactNode; text: string }; t: (key: string) => string }) {
  return (
    <div className="flex items-center justify-between px-2">
      <div className="flex items-center gap-4">
        <div className="w-11 h-11 rounded-xl bg-slate-100 dark:bg-slate-800 flex items-center justify-center shadow-inner border border-slate-200 dark:border-slate-700">
          {config.icon}
        </div>
        <div>
          <Typography className="font-bold text-slate-800 dark:text-slate-200 uppercase tracking-tight text-base leading-none mb-1" variant="h3">
            {config.text}
          </Typography>
          <Typography className="text-[10px] font-bold text-slate-600 dark:text-slate-400 uppercase tracking-[0.2em]" variant="p">{processed} / {total} {t("attendance_import.rows_processed")}</Typography>
        </div>
      </div>
      {total > 0 && <span className="text-[10px] font-bold text-primary-700 dark:text-primary-400 bg-primary-50 dark:bg-primary-950 px-3 py-1 rounded-full border border-primary-200 dark:border-primary-800 shadow-sm">{percentage}%</span>}
    </div>
  );
}

function ImportProgress({ jobStatus, t }: ImportProgressProps) {
  const processed = jobStatus?.processed_rows ?? 0;
  const total = jobStatus?.total_rows ?? 0;
  const percentage = total > 0 ? Math.round((processed / total) * 100) : 0;
  const config = getStatusConfig(jobStatus?.status, t);

  return (
    <div className="space-y-5 pt-2 animate-in fade-in duration-500">
      <ImportProgressInfo config={config} percentage={percentage} processed={processed} t={t} total={total} />
      <ProgressBar percentage={percentage} status={jobStatus?.status} />
      {!!jobStatus?.error_log && <ErrorLog errorLog={jobStatus.error_log} t={t} />}
    </div>
  );
}

function ImportRequirements({ t, customRequirements, targetType }: { t: (key: string) => string; customRequirements?: string[]; targetType?: string }) {
  return (
    <div className="bg-primary-50 dark:bg-primary-950/50 rounded-2xl p-5 border border-primary-200 dark:border-primary-800/50 flex gap-4">
      <div className="w-9 h-9 rounded-xl bg-white dark:bg-slate-900 flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-sm border border-primary-200 dark:border-primary-800 shrink-0"><AlertCircle size={18} /></div>
      <div className="text-xs text-primary-800 dark:text-primary-200 leading-relaxed font-medium pt-0.5 flex-1">
        <Typography className="font-bold uppercase tracking-[0.2em] mb-2 text-slate-700 dark:text-slate-300" variant="p">{t("attendance_import.file_requirements.title")}</Typography>
        <ul className="grid grid-cols-1 md:grid-cols-2 gap-x-8 gap-y-1.5 text-slate-600 dark:text-slate-400 font-bold uppercase tracking-tight">
          {customRequirements ? (
            customRequirements.map((request, index) => (
              <li className="flex items-center gap-2" key={index}><div className="w-1.5 h-1.5 bg-primary-500 dark:bg-primary-400 rounded-full" /> {request}</li>
            ))
          ) : (
            <>
              <li className="flex items-center gap-2"><div className="w-1.5 h-1.5 bg-primary-500 dark:bg-primary-400 rounded-full" /> {t("attendance_import.file_requirements.col_a")}</li>
              <li className="flex items-center gap-2"><div className="w-1.5 h-1.5 bg-primary-500 dark:bg-primary-400 rounded-full" /> {t("attendance_import.file_requirements.col_b")}</li>
              <li className="flex items-center gap-2"><div className="w-1.5 h-1.5 bg-primary-500 dark:bg-primary-400 rounded-full" /> {t("attendance_import.file_requirements.col_c")}</li>
              <li className="flex items-center gap-2"><div className="w-1.5 h-1.5 bg-primary-500 dark:bg-primary-400 rounded-full" /> {t("attendance_import.file_requirements.formats")}</li>
            </>
          )}
        </ul>
        {targetType === "employee" && (
          <div className="mt-4 flex items-center">
            <a className="text-[10px] font-bold text-primary-600 dark:text-primary-400 uppercase tracking-widest hover:text-primary-700 hover:underline flex items-center gap-1.5" download href="/templates/employee_import_template.xlsx">
              <FileText size={14} /> Tải file Excel mẫu (Template)
            </a>
          </div>
        )}
      </div>
    </div>
  );
}

function FileUploadZone({ file, setFile, t }: { file: File | null; setFile: (file: File | null) => void; t: (key: string) => string }) {
  return (
    <label className={cn("border-3 border-dashed rounded-2xl p-10 flex flex-col items-center justify-center transition-all cursor-pointer group active:scale-[0.98] relative overflow-hidden w-full", file ? "border-emerald-400 bg-emerald-50 dark:bg-emerald-950/20" : "border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-800/30 hover:border-primary-400 hover:bg-primary-50 dark:hover:bg-primary-950/30")} htmlFor="file-upload">
      <input accept=".xlsx,.xls,.csv" aria-label="File Upload" className="hidden" id="file-upload" onChange={(e) => { setFile(e.target.files?.[0] ?? null); }} type="file" />
      {file ? (
        <div className="text-center animate-in zoom-in-95 duration-500">
          <div className="w-16 h-16 bg-emerald-100 dark:bg-emerald-900 rounded-2xl flex items-center justify-center mx-auto mb-5 shadow-lg shadow-emerald-200/50 dark:shadow-emerald-900/30 rotate-3 group-hover:rotate-0 transition-transform"><FileText className="text-emerald-600 dark:text-emerald-300" size={28} /></div>
          <Typography className="font-bold text-slate-800 dark:text-slate-200 mb-1 text-lg tracking-tight uppercase" variant="p">{file.name}</Typography>
          <Typography className="text-[11px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest" variant="p">{(file.size / 1024).toFixed(1)} KB — READY TO START</Typography>
          <Button className="mt-5 border-rose-200 dark:border-rose-800 text-rose-600 dark:text-rose-400 hover:text-rose-700 dark:hover:text-rose-300 hover:bg-rose-50 dark:hover:bg-rose-950 shadow-sm hover:shadow-md" onClick={(e) => { e.stopPropagation(); setFile(null); }} size="sm" variant="outline">{t("attendance_import.select_another_file")}</Button>
        </div>
      ) : (
        <div className="text-center group-hover:scale-105 transition-transform duration-500">
          <div className="w-16 h-16 bg-white dark:bg-slate-900 rounded-2xl flex items-center justify-center mx-auto mb-5 shadow-lg border border-slate-200 dark:border-slate-700 -rotate-3 group-hover:rotate-0 transition-all"><Upload className="text-slate-400 dark:text-slate-500" size={28} /></div>
          <Typography className="font-bold text-slate-800 dark:text-slate-200 mb-1 text-lg tracking-tight uppercase" variant="p">{t("attendance_import.upload_prompt")}</Typography>
          <Typography className="text-[11px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest" variant="p">{t("attendance_import.upload_limit")}</Typography>
        </div>
      )}
    </label>
  );
}

function ImportActions({ onClose, file, uploading, handleUpload, t }: { onClose: () => void; file: File | null; uploading: boolean; handleUpload: () => void; t: (key: string) => string }) {
  return (
    <div className="flex gap-4 pt-3 border-t border-slate-200 dark:border-slate-800">
      <Button className="flex-1" onClick={onClose} size="lg" variant="outline">{t("common.cancel")}</Button>
      <Button className="flex-[1.5]" disabled={!file || uploading} loading={uploading} onClick={handleUpload} size="lg" variant="primary">
        {uploading ? t("attendance_import.status.uploading").toUpperCase() : t("attendance_import.start_import").toUpperCase()}
      </Button>
    </div>
  );
}

// eslint-disable-next-line complexity
export default function AttendanceImportDialog({ 
  isOpen, 
  onClose, 
  endpoint, 
  title, 
  targetType,
  customRequirements
}: AttendanceImportDialogProps) {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId();
  const queryClient = useQueryClient();
  const [file, setFile] = useState<File | null>(null);
  const [uploading, setUploading] = useState(false);
  const [activeJobId, setActiveJobId] = useState<string | null>(null);

  const { data: jobStatus } = useQuery({
    queryKey: ["import-job", activeJobId],
    queryFn: () => wsId && activeJobId ? fetchImportJobStatus(wsId, activeJobId) : Promise.reject(new Error("Missing workspace ID or Job ID")),
    enabled: !!activeJobId && !!wsId,
    refetchInterval: (query) => {
        const status = query.state.data?.status;
        return (status === "processing" || status === "pending") ? 1000 : false;
    }
  });

  const { data: latestJob } = useQuery({
    queryKey: ["latest-import-job", wsId, targetType],
    queryFn: () => wsId ? fetchLatestImportJob(wsId, targetType) : Promise.reject(new Error("Missing workspace ID")),
    enabled: isOpen && !!wsId,
  });

  const [previousLatestJob, setPreviousLatestJob] = useState(latestJob);
  if (latestJob !== previousLatestJob) {
    setPreviousLatestJob(latestJob);
    if (latestJob && (latestJob.status === "processing" || latestJob.status === "pending")) {
      setActiveJobId(latestJob.id);
    }
  }

  const handleUpload = async () => {
    if (!file || !wsId) return;
    setUploading(true);
    try {
      const result = await uploadAttendance(wsId, file, endpoint);
      setActiveJobId(result.job_id); setFile(null);
      void queryClient.invalidateQueries({ queryKey: ["latest-import-job"] });
    } catch (error) { console.error(error); } finally { setUploading(false); }
  };

  const isFinished = jobStatus?.status === "completed" || jobStatus?.status === "failed";

  return (
    <Modal icon={<Upload size={24} />} isOpen={isOpen} maxWidth="lg" onClose={onClose} subtitle={t("attendance_import.subtitle")} title={title}>
      {activeJobId ? (
        <>
          <ImportProgress jobStatus={jobStatus} t={t} />
          {isFinished ? (
            <div className="pt-3 flex justify-end border-t border-slate-200 dark:border-slate-800">
              <Button 
                className="px-10" 
                onClick={() => { 
                  onClose(); 
                  setTimeout(() => {
                    setActiveJobId(null); 
                    setFile(null);
                    void queryClient.invalidateQueries(); 
                  }, 300);
                }}
                size="lg"
                variant="secondary"
              >
                {t("common.close").toUpperCase()}
              </Button>
            </div>
          ) : null}
        </>
      ) : (
        <div className="space-y-6 pt-2 animate-in fade-in duration-500">
          <ImportRequirements customRequirements={customRequirements} t={t} targetType={targetType} />

          <FileUploadZone file={file} setFile={setFile} t={t} />

          <ImportActions file={file} handleUpload={handleUpload} onClose={onClose} t={t} uploading={uploading} />
        </div>
      )}
    </Modal>
  );
}