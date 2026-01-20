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

      console.error("Upload error:", error);

    } finally { setUploading(false); }

  };



  const isFinished = jobStatus?.status === "completed" || jobStatus?.status === "failed";



  return (

    <Modal isOpen={isOpen} onClose={onClose} title={title} subtitle={t("attendance_import.subtitle")} icon={<Upload size={24} />} maxWidth="lg">

      {activeJobId ? (

        <div className="space-y-8 pt-4 animate-in fade-in duration-500">

          <div className="flex items-center justify-between px-2">

            <div className="flex items-center gap-4">

              <div className="w-12 h-12 rounded-2xl bg-slate-50 flex items-center justify-center shadow-inner border border-slate-100">

                {jobStatus?.status === "processing" ? <Loader2 className="w-6 h-6 text-primary-600 animate-spin" /> : jobStatus?.status === "completed" ? <CheckCircle2 className="w-6 h-6 text-emerald-500" /> : jobStatus?.status === "failed" ? <AlertCircle className="w-6 h-6 text-rose-500" /> : <Loader2 className="w-6 h-6 text-slate-400 animate-spin" />}

              </div>

              <div>

                <h3 className="font-black text-slate-900 uppercase tracking-tight text-lg">

                  {jobStatus?.status === "processing" ? t("attendance_import.status.processing") : jobStatus?.status === "completed" ? t("attendance_import.status.completed") : jobStatus?.status === "failed" ? t("attendance_import.status.failed") : t("attendance_import.status.pending")}

                </h3>

                <p className="text-[10px] font-black text-slate-400 uppercase tracking-[0.15em] mt-0.5">{jobStatus?.processed_rows || 0} / {jobStatus?.total_rows || 0} {t("attendance_import.rows_processed")}</p>

              </div>

            </div>

            {jobStatus?.total_rows ? <span className="text-[10px] font-black text-primary-600 bg-primary-50 px-3 py-1.5 rounded-full border border-primary-100 shadow-sm">{Math.round((jobStatus.processed_rows / jobStatus.total_rows) * 100)}%</span> : null}

          </div>



          <div className="h-3 w-full bg-slate-100 rounded-full overflow-hidden shadow-inner border border-slate-200/50">

            <div className={cn("h-full transition-all duration-1000 shadow-sm", jobStatus?.status === "failed" ? "bg-rose-500" : "bg-primary-600")} style={{ width: `${jobStatus?.total_rows ? (jobStatus.processed_rows / jobStatus.total_rows) * 100 : 0}%` }} />

          </div>



          {jobStatus?.error_log && (

            <div className="bg-slate-900 rounded-[2rem] p-8 border border-white/5 shadow-2xl shadow-rose-900/10">

              <h4 className="text-[10px] font-black text-rose-400 mb-4 flex items-center gap-2 uppercase tracking-[0.2em]"><AlertCircle size={14} /> {t("attendance_import.error_list_title")}</h4>

              <div className="space-y-3 max-h-60 overflow-y-auto pr-2 custom-scrollbar">

                {jobStatus.error_log.split("\n").map((err, i) => (<div key={i} className="text-[11px] bg-white/5 p-3 rounded-xl border border-white/5 text-slate-300 font-mono leading-relaxed">{err}</div>))}

              </div>

            </div>

          )}



          {isFinished && (

            <div className="pt-4 flex justify-end">

              <button onClick={() => { setActiveJobId(null); queryClient.invalidateQueries(); }} className="px-10 py-3.5 bg-slate-900 text-white rounded-2xl font-black text-xs uppercase tracking-widest hover:bg-black transition-all shadow-xl shadow-slate-200 active:scale-95">{t("common.close").toUpperCase()}</button>

            </div>

          )}

        </div>

      ) : (

        <div className="space-y-8 pt-4 animate-in fade-in duration-500">

          <div className="bg-primary-50/50 rounded-[2rem] p-6 border border-primary-100/50 flex gap-4 shadow-inner">

            <AlertCircle className="w-6 h-6 text-primary-600 shrink-0" />

            <div className="text-xs text-primary-900 leading-relaxed font-medium">

              <p className="font-black uppercase tracking-widest mb-2 opacity-80">{t("attendance_import.file_requirements.title")}</p>

              <ul className="space-y-1.5 opacity-70">

                <li className="flex items-center gap-2"><div className="w-1 h-1 bg-primary-400 rounded-full" /> {t("attendance_import.file_requirements.col_a")}</li>

                <li className="flex items-center gap-2"><div className="w-1 h-1 bg-primary-400 rounded-full" /> {t("attendance_import.file_requirements.col_b")}</li>

                <li className="flex items-center gap-2"><div className="w-1 h-1 bg-primary-400 rounded-full" /> {t("attendance_import.file_requirements.col_c")}</li>

                <li className="flex items-center gap-2"><div className="w-1 h-1 bg-primary-400 rounded-full" /> {t("attendance_import.file_requirements.formats")}</li>

              </ul>

            </div>

          </div>



          <div className={cn("border-2 border-dashed rounded-[2.5rem] p-12 flex flex-col items-center justify-center transition-all cursor-pointer group active:scale-95", file ? "border-emerald-500 bg-emerald-50/30" : "border-slate-200 hover:border-primary-400 hover:bg-primary-50/30")} onClick={() => document.getElementById("file-upload")?.click()}>

            <input id="file-upload" type="file" className="hidden" accept=".xlsx,.xls,.csv" onChange={(e) => setFile(e.target.files?.[0] || null)} />

            {file ? (

              <>

                <div className="w-20 h-20 bg-emerald-100 rounded-[1.5rem] flex items-center justify-center mb-6 shadow-lg shadow-emerald-100/50 transition-transform group-hover:scale-110 rotate-3"><FileText size={32} className="text-emerald-600" /></div>

                <p className="font-black text-slate-900 mb-1.5 text-lg tracking-tight uppercase">{file.name}</p>

                <p className="text-[10px] font-black text-slate-400 uppercase tracking-[0.2em]">{(file.size / 1024).toFixed(1)} KB</p>

                <button onClick={(e) => { e.stopPropagation(); setFile(null); }} className="mt-6 text-[10px] font-black text-rose-500 uppercase tracking-[0.2em] hover:text-rose-700 transition-colors bg-rose-50 px-4 py-2 rounded-xl border border-rose-100">{t("attendance_import.select_another_file")}</button>

              </>

            ) : (

              <>

                <div className="w-20 h-20 bg-slate-50 rounded-[1.5rem] flex items-center justify-center mb-6 shadow-inner border border-slate-100 transition-transform group-hover:scale-110 -rotate-3"><Upload size={32} className="text-slate-300" /></div>

                <p className="font-black text-slate-900 mb-1.5 text-lg tracking-tight uppercase">{t("attendance_import.upload_prompt")}</p>

                <p className="text-[10px] font-black text-slate-400 uppercase tracking-[0.2em] opacity-60">{t("attendance_import.upload_limit")}</p>

              </>

            )}

          </div>



          <div className="flex gap-4 pt-2">

            <button onClick={onClose} className="flex-1 px-8 py-4 bg-white border border-slate-200 text-slate-500 rounded-2xl font-black text-xs uppercase tracking-widest hover:bg-slate-50 transition-all">{t("common.cancel")}</button>

            <button disabled={!file || uploading} onClick={handleUpload} className="flex-1 bg-primary-600 text-white rounded-2xl font-black text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-primary-100 active:scale-95 disabled:opacity-50 flex items-center justify-center gap-3">

              {uploading ? <><Loader2 size={18} className="animate-spin" /> {t("attendance_import.status.uploading").toUpperCase()}</> : t("attendance_import.start_import")}

            </button>

          </div>

        </div>

      )}

    </Modal>

  );

}
