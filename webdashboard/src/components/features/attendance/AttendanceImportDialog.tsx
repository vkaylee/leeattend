import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Upload, FileText, Loader2, CheckCircle2, AlertCircle } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import Modal from "@/components/ui/Modal";
import { uploadAttendance, fetchLatestImportJob, fetchImportJobStatus, getCurrentWorkspaceId } from "@/lib/api";
import { cn } from "@/lib/utils";

interface AttendanceImportDialogProps {
  customRequirements?: string[];
  endpoint: string; // e.g., "/attendance/upload" or "/attendance/logs/upload"
  isOpen: boolean;
  onClose: () => void;
  targetType: string; // "adjustment", "log", or "employee"
  title: string;
}

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
    queryFn: () => fetchImportJobStatus(wsId!, activeJobId!),
    enabled: !!activeJobId && !!wsId,
    refetchInterval: (query) => {
        const status = query.state.data?.status;
        return (status === "processing" || status === "pending") ? 1000 : false;
    }
  });

  const { data: latestJob } = useQuery({
    queryKey: ["latest-import-job", wsId, targetType],
    queryFn: () => fetchLatestImportJob(wsId!, targetType),
    enabled: isOpen && !!wsId,
  });

  const [previousLatestJob, setPreviousLatestJob] = useState(latestJob);
  if (latestJob !== previousLatestJob) {
    setPreviousLatestJob(latestJob);
    if (latestJob) setActiveJobId(latestJob.id);
  }

  const handleUpload = async () => {
    if (!file || !wsId) return;
    setUploading(true);
    try {
      const result = await uploadAttendance(wsId, file, endpoint);
      setActiveJobId(result.job_id); setFile(null);
      queryClient.invalidateQueries({ queryKey: ["latest-import-job"] });
    } catch (error) { console.error(error); } finally { setUploading(false); }
  };

  const isFinished = jobStatus?.status === "completed" || jobStatus?.status === "failed";

  return (
    <Modal icon={<Upload size={24} />} isOpen={isOpen} maxWidth="lg" onClose={onClose} subtitle={t("attendance_import.subtitle")} title={title}>
      {activeJobId ? (
        <div className="space-y-5 pt-2 animate-in fade-in duration-500">
          <div className="flex items-center justify-between px-2">
            <div className="flex items-center gap-4">
              <div className="w-11 h-11 rounded-xl bg-slate-100 dark:bg-slate-800 flex items-center justify-center shadow-inner border border-slate-200 dark:border-slate-700">
                {jobStatus?.status === "processing" ? <Loader2 className="text-primary-600 dark:text-primary-400 animate-spin" size={20} /> : jobStatus?.status === "completed" ? <CheckCircle2 className="text-emerald-500" size={20} /> : jobStatus?.status === "failed" ? <AlertCircle className="text-rose-500" size={20} /> : <Loader2 className="text-slate-500 dark:text-slate-400 animate-spin" size={20} />}
              </div>
              <div>
                <h3 className="font-bold text-slate-800 dark:text-slate-200 uppercase tracking-tight text-base leading-none mb-1">
                  {jobStatus?.status === "processing" ? t("attendance_import.status.processing") : jobStatus?.status === "completed" ? t("attendance_import.status.completed") : jobStatus?.status === "failed" ? t("attendance_import.status.failed") : t("attendance_import.status.pending")}
                </h3>
                <p className="text-[10px] font-bold text-slate-600 dark:text-slate-400 uppercase tracking-[0.2em]">{jobStatus?.processed_rows || 0} / {jobStatus?.total_rows || 0} {t("attendance_import.rows_processed")}</p>
              </div>
            </div>
            {jobStatus?.total_rows ? <span className="text-[10px] font-bold text-primary-700 dark:text-primary-400 bg-primary-50 dark:bg-primary-950 px-3 py-1 rounded-full border border-primary-200 dark:border-primary-800 shadow-sm">{Math.round((jobStatus.processed_rows / jobStatus.total_rows) * 100)}%</span> : null}
          </div>

          <div className="h-2.5 w-full bg-slate-100 dark:bg-slate-700 rounded-full overflow-hidden shadow-inner border border-slate-200 dark:border-slate-700/50 relative">
            <div className={cn("h-full transition-all duration-1000 shadow-xl", jobStatus?.status === "failed" ? "bg-rose-400 dark:bg-rose-500" : "bg-primary-600")} style={{ width: `${jobStatus?.total_rows ? (jobStatus.processed_rows / jobStatus.total_rows) * 100 : 0}%` }}>
               <div className="absolute inset-0 bg-gradient-to-r from-transparent via-white/20 to-transparent animate-shimmer" style={{ backgroundSize: '200% 100%' }} />
            </div>
          </div>

          {!!jobStatus?.error_log && (
            <div className="bg-slate-50 dark:bg-slate-900 rounded-2xl p-6 border border-rose-200 dark:border-rose-900/50 shadow-sm relative overflow-hidden">
              <h4 className="text-[10px] font-bold text-rose-600 dark:text-rose-400 mb-4 flex items-center gap-2 uppercase tracking-[0.25em]"><AlertCircle size={14} /> {t("attendance_import.error_list_title")}</h4>
              <div className="space-y-2 max-h-48 overflow-y-auto pr-2 custom-scrollbar relative z-10">
                {jobStatus.error_log.split("\n").map((error: string, index: number) => (<div className="text-[11px] bg-white dark:bg-slate-800 p-3 rounded-lg border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-300 font-mono leading-relaxed" key={index}><span className="text-rose-500 mr-2 font-bold uppercase">ERR:</span> {error}</div>))}
              </div>
            </div>
          )}

          {!!isFinished && (
            <div className="pt-3 flex justify-end border-t border-slate-200 dark:border-slate-800">
              <button className="px-10 py-3 bg-slate-800 dark:bg-slate-700 text-white rounded-xl font-bold text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-lg active:scale-95" onClick={() => { setActiveJobId(null); queryClient.invalidateQueries(); }}>{t("common.close").toUpperCase()}</button>
            </div>
          )}
        </div>
      ) : (
        <div className="space-y-6 pt-2 animate-in fade-in duration-500">
          <div className="bg-primary-50 dark:bg-primary-950/50 rounded-2xl p-5 border border-primary-200 dark:border-primary-800/50 flex gap-4">
            <div className="w-9 h-9 rounded-xl bg-white dark:bg-slate-900 flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-sm border border-primary-200 dark:border-primary-800 shrink-0"><AlertCircle size={18} /></div>
            <div className="text-xs text-primary-800 dark:text-primary-200 leading-relaxed font-medium pt-0.5">
              <p className="font-bold uppercase tracking-[0.2em] mb-2 text-slate-700 dark:text-slate-300">{t("attendance_import.file_requirements.title")}</p>
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
            </div>
          </div>

          { }
          <label className={cn("border-3 border-dashed rounded-2xl p-10 flex flex-col items-center justify-center transition-all cursor-pointer group active:scale-[0.98] relative overflow-hidden w-full", file ? "border-emerald-400 bg-emerald-50 dark:bg-emerald-950/20" : "border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-800/30 hover:border-primary-400 hover:bg-primary-50 dark:hover:bg-primary-950/30")} htmlFor="file-upload">
            <input accept=".xlsx,.xls,.csv" aria-label="File Upload" className="hidden" id="file-upload" onChange={(e) => { setFile(e.target.files?.[0] || null); }} type="file" />
            {file ? (
              <div className="text-center animate-in zoom-in-95 duration-500">
                <div className="w-16 h-16 bg-emerald-100 dark:bg-emerald-900 rounded-2xl flex items-center justify-center mx-auto mb-5 shadow-lg shadow-emerald-200/50 dark:shadow-emerald-900/30 rotate-3 group-hover:rotate-0 transition-transform"><FileText className="text-emerald-600 dark:text-emerald-300" size={28} /></div>
                <p className="font-bold text-slate-800 dark:text-slate-200 mb-1 text-lg tracking-tight uppercase">{file.name}</p>
                <p className="text-[11px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{(file.size / 1024).toFixed(1)} KB — READY TO START</p>
                <button className="mt-5 text-[10px] font-bold text-rose-600 dark:text-rose-400 uppercase tracking-widest hover:text-rose-700 dark:hover:text-rose-300 transition-all bg-white dark:bg-slate-900 px-5 py-2 rounded-lg border border-rose-200 dark:border-rose-800 shadow-sm hover:shadow-md active:scale-90" onClick={(e) => { e.stopPropagation(); setFile(null); }}>{t("attendance_import.select_another_file")}</button>
              </div>
            ) : (
              <div className="text-center group-hover:scale-105 transition-transform duration-500">
                <div className="w-16 h-16 bg-white dark:bg-slate-900 rounded-2xl flex items-center justify-center mx-auto mb-5 shadow-lg border border-slate-200 dark:border-slate-700 -rotate-3 group-hover:rotate-0 transition-all"><Upload className="text-slate-400 dark:text-slate-500" size={28} /></div>
                <p className="font-bold text-slate-800 dark:text-slate-200 mb-1 text-lg tracking-tight uppercase">{t("attendance_import.upload_prompt")}</p>
                <p className="text-[11px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("attendance_import.upload_limit")}</p>
              </div>
            )}
          </label>

          <div className="flex gap-4 pt-3 border-t border-slate-200 dark:border-slate-800">
            <button className="flex-1 px-6 py-3 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 text-slate-600 dark:text-slate-400 rounded-xl font-bold text-[10px] uppercase tracking-widest hover:bg-slate-50 dark:hover:bg-slate-800 transition-all active:scale-95" onClick={onClose}>{t("common.cancel")}</button>
            <button className="flex-[1.5] py-3 bg-primary-600 text-white rounded-xl font-bold text-[10px] uppercase tracking-widest hover:bg-primary-700 transition-all shadow-lg shadow-primary-200/50 dark:shadow-primary-900/30 active:scale-95 disabled:opacity-50 flex items-center justify-center gap-3" disabled={!file || uploading} onClick={handleUpload}>
              {uploading ? <><Loader2 className="animate-spin" size={18} /> {t("attendance_import.status.uploading").toUpperCase()}</> : t("attendance_import.start_import").toUpperCase()}
            </button>
          </div>
        </div>
      )}
    </Modal>
  );
}