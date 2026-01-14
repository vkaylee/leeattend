import { useState, useEffect } from "react";
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
  const wsId = getCurrentWorkspaceId();
  const queryClient = useQueryClient();
  const [file, setFile] = useState<File | null>(null);
  const [uploading, setUploading] = useState(false);
  const [activeJobId, setActiveJobId] = useState<string | null>(null);

  // Poll for job status if there's an active job
  const { data: jobStatus } = useQuery({
    queryKey: ["import-job", activeJobId],
    queryFn: () => fetchImportJobStatus(wsId!, activeJobId!),
    enabled: !!activeJobId && !!wsId,
    refetchInterval: (query) => {
        const status = query.state.data?.status;
        return (status === "processing" || status === "pending") ? 1000 : false;
    }
  });

  // Also check if there's a currently running job when opening
  const { data: latestJob } = useQuery({
    queryKey: ["latest-import-job", wsId, targetType],
    queryFn: () => fetchLatestImportJob(wsId!, targetType),
    enabled: isOpen && !!wsId,
  });

  useEffect(() => {
    if (latestJob) {
      setActiveJobId(latestJob.id);
    }
  }, [latestJob]);

  const handleUpload = async () => {
    if (!file || !wsId) return;

    setUploading(true);
    try {
      const result = await uploadAttendance(wsId, file, endpoint);
      setActiveJobId(result.job_id);
      setFile(null);
      queryClient.invalidateQueries({ queryKey: ["latest-import-job"] });
    } catch (error) {
      console.error("Upload error:", error);
      alert("Lỗi khi tải file lên");
    } finally {
      setUploading(false);
    }
  };

  const isFinished = jobStatus?.status === "completed" || jobStatus?.status === "failed";

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title={title}
      subtitle="Nhập dữ liệu hàng loạt"
      icon={<Upload className="w-6 h-6" />}
      maxWidth="lg"
    >
      {activeJobId ? (
        <div className="space-y-6">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              {jobStatus?.status === "processing" ? (
                <Loader2 className="w-6 h-6 text-primary-600 animate-spin" />
              ) : jobStatus?.status === "completed" ? (
                <CheckCircle2 className="w-6 h-6 text-emerald-500" />
              ) : jobStatus?.status === "failed" ? (
                <AlertCircle className="w-6 h-6 text-rose-500" />
              ) : (
                <Loader2 className="w-6 h-6 text-slate-400 animate-spin" />
              )}
              <div>
                <h3 className="font-bold text-slate-900">
                  {jobStatus?.status === "processing" ? "Đang xử lý dữ liệu..." : 
                    jobStatus?.status === "completed" ? "Hoàn thành!" : 
                    jobStatus?.status === "failed" ? "Có lỗi xảy ra" : "Đang chờ..."}
                </h3>
                <p className="text-sm text-slate-500">
                  {jobStatus?.processed_rows} / {jobStatus?.total_rows} dòng đã xử lý
                </p>
              </div>
            </div>
            {jobStatus?.total_rows ? (
              <span className="text-sm font-bold text-primary-600 bg-primary-50 px-3 py-1 rounded-full">
                {Math.round((jobStatus.processed_rows / jobStatus.total_rows) * 100)}%
              </span>
            ) : null}
          </div>

          <div className="h-3 w-full bg-slate-100 rounded-full overflow-hidden">
            <div 
              className={cn(
                "h-full transition-all duration-500",
                jobStatus?.status === "failed" ? "bg-rose-500" : "bg-primary-600"
              )}
              style={{ width: `${jobStatus?.total_rows ? (jobStatus.processed_rows / jobStatus.total_rows) * 100 : 0}%` }}
            />
          </div>

          {jobStatus?.error_log && (
            <div className="bg-rose-50 rounded-2xl p-5 border border-rose-100">
              <h4 className="text-sm font-bold text-rose-900 mb-3 flex items-center gap-2">
                <AlertCircle className="w-4 h-4" /> Danh sách lỗi chi tiết:
              </h4>
              <div className="space-y-2 max-h-60 overflow-y-auto pr-2 custom-scrollbar">
                {jobStatus.error_log.split("\n").map((err, i) => (
                  <div key={i} className="text-xs bg-white/50 p-2 rounded-lg border border-rose-100 text-rose-700 font-medium">
                    {err}
                  </div>
                ))}
              </div>
            </div>
          )}

          {isFinished && (
            <div className="pt-4 flex justify-end">
              <Button onClick={() => { setActiveJobId(null); queryClient.invalidateQueries(); }}>
                Tiếp tục
              </Button>
            </div>
          )}
        </div>
      ) : (
        <div className="space-y-6">
          <div className="bg-primary-50 rounded-2xl p-4 border border-primary-100 flex gap-3">
            <AlertCircle className="w-5 h-5 text-primary-600 shrink-0" />
            <div className="text-sm text-primary-800">
              <p className="font-bold mb-1">Yêu cầu định dạng file:</p>
              <ul className="list-disc list-inside space-y-1 opacity-90">
                <li>Cột A: Mã số nhân viên (Employee ID)</li>
                <li>Cột B: Tên nhân viên (Không bắt buộc)</li>
                <li>Cột C: Thời gian quẹt thẻ (DD/MM/YYYY HH:mm)</li>
                <li>Hỗ trợ file .xlsx, .xls và .csv</li>
              </ul>
            </div>
          </div>

          <div 
            className={cn(
              "border-2 border-dashed rounded-[2rem] p-10 flex flex-col items-center justify-center transition-all cursor-pointer",
              file ? "border-emerald-500 bg-emerald-50/30" : "border-slate-200 hover:border-primary-400 hover:bg-primary-50/30"
            )}
            onClick={() => document.getElementById("file-upload")?.click()}
          >
            <input 
              id="file-upload"
              type="file" 
              className="hidden" 
              accept=".xlsx,.xls,.csv"
              onChange={(e) => setFile(e.target.files?.[0] || null)}
            />
            
            {file ? (
              <>
                <div className="w-16 h-16 bg-emerald-100 rounded-2xl flex items-center justify-center mb-4">
                  <FileText className="w-8 h-8 text-emerald-600" />
                </div>
                <p className="font-bold text-slate-900 mb-1">{file.name}</p>
                <p className="text-sm text-slate-500">{(file.size / 1024).toFixed(1)} KB</p>
                <button 
                  onClick={(e) => { e.stopPropagation(); setFile(null); }}
                  className="mt-4 text-xs font-bold text-rose-600 hover:underline"
                >
                  Chọn file khác
                </button>
              </>
            ) : (
              <>
                <div className="w-16 h-16 bg-slate-100 rounded-2xl flex items-center justify-center mb-4">
                  <Upload className="w-8 h-8 text-slate-400" />
                </div>
                <p className="font-bold text-slate-900 mb-1">Click hoặc kéo thả file vào đây</p>
                <p className="text-sm text-slate-500">Excel hoặc CSV (Tối đa 10MB)</p>
              </>
            )}
          </div>

          <div className="flex gap-3 pt-2">
            <Button 
              variant="outline" 
              className="flex-1"
              onClick={onClose}
            >
              Hủy
            </Button>
            <Button 
              className="flex-1"
              disabled={!file || uploading}
              onClick={handleUpload}
            >
              {uploading ? (
                <>
                  <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                  Đang tải lên...
                </>
              ) : (
                "Bắt đầu nhập dữ liệu"
              )}
            </Button>
          </div>
        </div>
      )}
    </Modal>
  );
}