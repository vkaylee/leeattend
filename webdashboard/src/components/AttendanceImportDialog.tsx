import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { fetchLatestImportJob, fetchImportJobStatus, getCurrentWorkspaceId, uploadAttendance } from "@/lib/api";
import { 
  Upload,
  AlertCircle,
  CheckCircle2,
  Loader2,
  FileText
} from "lucide-react";
import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";
import Modal from "./ui/Modal";

function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

// Minimal Button for the dialog
const Button = ({ children, variant = "default", className, disabled, onClick, type = "button" }: { children: React.ReactNode, variant?: string, className?: string, disabled?: boolean, onClick?: () => void, type?: "button" | "submit" }) => {
  const variants: Record<string, string> = {
    default: "bg-slate-900 text-white hover:bg-slate-800 shadow-sm",
    outline: "border border-slate-200 bg-white hover:bg-slate-50 text-slate-700",
    ghost: "hover:bg-slate-100 text-slate-600",
  };
  return (
    <button 
      type={type}
      disabled={disabled}
      onClick={onClick}
      className={cn(
        "inline-flex items-center justify-center rounded-xl px-4 py-3 text-sm font-bold transition-all active:scale-95 disabled:opacity-50 disabled:pointer-events-none",
        variants[variant || "default"],
        className
      )}
    >
      {children}
    </button>
  );
};

interface AttendanceImportDialogProps {
  isOpen: boolean;
  onClose: () => void;
  endpoint: string; // e.g., "/attendance/upload" or "/attendance/logs/upload"
  title: string;
  targetType: string; // "adjustment" or "log"
}

export default function AttendanceImportDialog({ isOpen, onClose, endpoint, title, targetType }: AttendanceImportDialogProps) {
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

  useEffect(() => { if (latestJob) setActiveJobId(latestJob.id); }, [latestJob]);

  const handleUpload = async () => {
    if (!file || !wsId) return;
    setUploading(true);
    try {
      const result = await uploadAttendance(wsId, file, endpoint);
      setActiveJobId(result.job_id); setFile(null);
      queryClient.invalidateQueries({ queryKey: ["latest-import-job"] });
    } catch (error) {
    } finally { setUploading(false); }
  };

  const isFinished = jobStatus?.status === "completed" || jobStatus?.status === "failed";

  return (
    <Modal isOpen={isOpen} onClose={onClose} title={title} subtitle={t("attendance_import.subtitle")} icon={<Upload size={24} />} maxWidth="lg">
      {activeJobId ? (
        <div className="space-y-8 pt-4 animate-in fade-in duration-500">
          <div className="flex items-center justify-between px-2">
            <div className="flex items-center gap-5">
              <div className="w-14 h-14 rounded-2xl bg-slate-50 flex items-center justify-center shadow-inner border border-slate-100">
                {jobStatus?.status === "processing" ? <Loader2 size={24} className="text-primary-600 animate-spin" /> : jobStatus?.status === "completed" ? <CheckCircle2 size={24} className="text-emerald-500" /> : jobStatus?.status === "failed" ? <AlertCircle size={24} className="text-rose-500" /> : <Loader2 size={24} className="text-slate-400 animate-spin" />}
              </div>
              <div>
                <h3 className="font-black text-slate-900 uppercase tracking-tight text-xl leading-none mb-1.5">
                  {jobStatus?.status === "processing" ? t("attendance_import.status.processing") : jobStatus?.status === "completed" ? t("attendance_import.status.completed") : jobStatus?.status === "failed" ? t("attendance_import.status.failed") : t("attendance_import.status.pending")}
                </h3>
                <p className="text-[10px] font-black text-slate-400 uppercase tracking-[0.2em]">{jobStatus?.processed_rows || 0} / {jobStatus?.total_rows || 0} {t("attendance_import.rows_processed")}</p>
              </div>
            </div>
            {jobStatus?.total_rows ? <span className="text-[10px] font-black text-primary-600 bg-primary-50 px-4 py-1.5 rounded-full border border-primary-100 shadow-sm">{Math.round((jobStatus.processed_rows / jobStatus.total_rows) * 100)}%</span> : null}
          </div>

          <div className="h-4 w-full bg-slate-100 rounded-full overflow-hidden shadow-inner border border-slate-200/50 relative">
            <div className={cn("h-full transition-all duration-1000 shadow-xl", jobStatus?.status === "failed" ? "bg-rose-500" : "bg-primary-600")} style={{ width: `${jobStatus?.total_rows ? (jobStatus.processed_rows / jobStatus.total_rows) * 100 : 0}%` }}>
               <div className="absolute inset-0 bg-gradient-to-r from-transparent via-white/20 to-transparent animate-shimmer" style={{ backgroundSize: '200% 100%' }} />
            </div>
          </div>

          {jobStatus?.error_log && (
            <div className="bg-slate-900 rounded-[2.5rem] p-10 border border-white/5 shadow-2xl shadow-rose-900/20 relative overflow-hidden">
              <h4 className="text-[10px] font-black text-rose-400 mb-6 flex items-center gap-3 uppercase tracking-[0.25em]"><AlertCircle size={16} /> {t("attendance_import.error_list_title")}</h4>
              <div className="space-y-4 max-h-64 overflow-y-auto pr-2 custom-scrollbar relative z-10">
                {jobStatus.error_log.split("\n").map((err, i) => (<div key={i} className="text-[11px] bg-white/5 p-4 rounded-xl border border-white/5 text-slate-300 font-mono leading-relaxed group hover:bg-white/10 transition-colors"><span className="text-rose-500/50 mr-2 font-black uppercase">ERR:</span> {err}</div>))}
              </div>
              <div className="absolute -right-8 -bottom-8 w-32 h-32 text-rose-500 opacity-5 -rotate-12 pointer-events-none"><AlertCircle size={128} /></div>
            </div>
          )}

          {isFinished && (
            <div className="pt-4 flex justify-end border-t border-slate-50">
              <button onClick={() => { setActiveJobId(null); queryClient.invalidateQueries(); }} className="px-12 py-4 bg-slate-900 text-white rounded-2xl font-black text-xs uppercase tracking-widest hover:bg-black transition-all shadow-xl shadow-slate-200 active:scale-95">{t("common.close").toUpperCase()}</button>
            </div>
          )}
        </div>
      ) : (
        <div className="space-y-10 pt-4 animate-in fade-in duration-500">
          <div className="bg-primary-50/50 rounded-[2.5rem] p-8 border border-primary-100/50 flex gap-6 shadow-inner">
            <div className="w-12 h-12 rounded-2xl bg-white flex items-center justify-center text-primary-600 shadow-sm border border-primary-100 shrink-0"><AlertCircle size={24} /></div>
            <div className="text-xs text-primary-900 leading-relaxed font-medium pt-1">
              <p className="font-black uppercase tracking-[0.2em] mb-3 text-primary-950 opacity-80">{t("attendance_import.file_requirements.title")}</p>
              <ul className="grid grid-cols-1 md:grid-cols-2 gap-x-8 gap-y-2 opacity-70 font-bold uppercase tracking-tight">
                <li className="flex items-center gap-2.5"><div className="w-1.5 h-1.5 bg-primary-400 rounded-full" /> {t("attendance_import.file_requirements.col_a")}</li>
                <li className="flex items-center gap-2.5"><div className="w-1.5 h-1.5 bg-primary-400 rounded-full" /> {t("attendance_import.file_requirements.col_b")}</li>
                <li className="flex items-center gap-2.5"><div className="w-1.5 h-1.5 bg-primary-400 rounded-full" /> {t("attendance_import.file_requirements.col_c")}</li>
                <li className="flex items-center gap-2.5"><div className="w-1.5 h-1.5 bg-primary-400 rounded-full" /> {t("attendance_import.file_requirements.formats")}</li>
              </ul>
            </div>
          </div>

          <div className={cn("border-4 border-dashed rounded-[3rem] p-16 flex flex-col items-center justify-center transition-all cursor-pointer group active:scale-95 relative overflow-hidden", file ? "border-emerald-500 bg-emerald-50/20" : "border-slate-100 bg-slate-50/30 hover:border-primary-400 hover:bg-primary-50/30 shadow-inner")} onClick={() => document.getElementById("file-upload")?.click()}>
            <input id="file-upload" type="file" className="hidden" accept=".xlsx,.xls,.csv" onChange={(e) => setFile(e.target.files?.[0] || null)} />
            {file ? (
              <div className="text-center animate-in zoom-in-95 duration-500">
                <div className="w-24 h-24 bg-emerald-100 rounded-[2rem] flex items-center justify-center mx-auto mb-8 shadow-2xl shadow-emerald-200/50 rotate-3 group-hover:rotate-0 transition-transform"><FileText size={40} className="text-emerald-600" /></div>
                <p className="font-black text-slate-900 mb-2 text-2xl tracking-tighter uppercase">{file.name}</p>
                <p className="text-[11px] font-black text-slate-400 uppercase tracking-[0.25em] opacity-60">{(file.size / 1024).toFixed(1)} KB — READY TO START</p>
                <button onClick={(e) => { e.stopPropagation(); setFile(null); }} className="mt-8 text-[10px] font-black text-rose-500 uppercase tracking-[0.3em] hover:text-rose-700 transition-all bg-white px-6 py-3 rounded-xl border border-rose-100 shadow-sm hover:shadow-md active:scale-90">{t("attendance_import.select_another_file")}</button>
              </div>
            ) : (
              <div className="text-center group-hover:scale-105 transition-transform duration-500">
                <div className="w-24 h-24 bg-white rounded-[2rem] flex items-center justify-center mx-auto mb-8 shadow-xl border border-slate-100 -rotate-3 group-hover:rotate-0 transition-all"><Upload size={40} className="text-slate-300" /></div>
                <p className="font-black text-slate-900 mb-2 text-2xl tracking-tighter uppercase">{t("attendance_import.upload_prompt")}</p>
                <p className="text-[11px] font-black text-slate-400 uppercase tracking-[0.25em] opacity-50">{t("attendance_import.upload_limit")}</p>
              </div>
            )}
          </div>

          <div className="flex gap-6 pt-4 border-t border-slate-50">
            <button onClick={onClose} className="flex-1 px-8 py-4 bg-white border border-slate-200 text-slate-500 rounded-2xl font-black text-[10px] uppercase tracking-[0.3em] hover:bg-slate-50 transition-all active:scale-95">{t("common.cancel")}</button>
            <button disabled={!file || uploading} onClick={handleUpload} className="flex-[1.5] bg-primary-600 text-white rounded-2xl font-black text-[10px] uppercase tracking-[0.3em] hover:bg-primary-700 transition-all shadow-2xl shadow-primary-100 active:scale-95 disabled:opacity-50 flex items-center justify-center gap-4">
              {uploading ? <><Loader2 size={20} className="animate-spin" /> {t("attendance_import.status.uploading").toUpperCase()}</> : t("attendance_import.start_import").toUpperCase()}
            </button>
          </div>
        </div>
      )}
    </Modal>
  );
}

  );

}
