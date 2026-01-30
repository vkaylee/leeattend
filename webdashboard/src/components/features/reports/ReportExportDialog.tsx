import { useState, useEffect } from "react";
import { useQuery } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";
import { Download, Loader2, CheckCircle2, AlertCircle, FileSpreadsheet, X } from "lucide-react";
import Modal from "../../ui/Modal";
import {
  createReportExportJob,
  fetchExportJobStatus,
  downloadExportReport,
} from "../../../lib/api/attendance";
import { cn } from "../../../lib/utils";

interface ReportExportDialogProps {
  isOpen: boolean;
  onClose: () => void;
  wsId: string;
  startDate: string;
  endDate: string;
  reportType: "summary" | "detail";
  employeeIds?: string[];
}

type ExportPhase = "idle" | "starting" | "polling" | "downloading" | "completed" | "failed";

export default function ReportExportDialog({
  isOpen,
  onClose,
  wsId,
  startDate,
  endDate,
  reportType,
  employeeIds,
}: ReportExportDialogProps) {
  const { t } = useTranslation();
  const [phase, setPhase] = useState<ExportPhase>("idle");
  const [jobId, setJobId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [totalBatches, setTotalBatches] = useState(0);

  // Reset state when dialog opens
  useEffect(() => {
    if (isOpen) {
      setPhase("idle");
      setJobId(null);
      setError(null);
      setTotalBatches(0);
    }
  }, [isOpen]);

  // Poll job status
  const { data: jobStatus } = useQuery({
    queryKey: ["export-job", jobId],
    queryFn: () => fetchExportJobStatus(wsId, jobId!),
    enabled: !!jobId && phase === "polling",
    refetchInterval: (query) => {
      const status = query.state.data?.status;
      return status === "processing" || status === "pending" ? 1000 : false;
    },
  });

  // Handle job status changes
  useEffect(() => {
    if (!jobStatus) return;

    if (jobStatus.status === "completed") {
      setPhase("completed");
    } else if (jobStatus.status === "failed") {
      setPhase("failed");
      setError(jobStatus.error || t("report_export.status.failed"));
    }
  }, [jobStatus, t]);

  const handleStartExport = async () => {
    setPhase("starting");
    setError(null);

    try {
      const response = await createReportExportJob(wsId, {
        start_date: startDate,
        end_date: endDate,
        report_type: reportType,
        employee_ids: employeeIds,
      });

      setJobId(response.job_id);
      setTotalBatches(response.total_batches);
      setPhase("polling");
    } catch (err) {
      setPhase("failed");
      setError(err instanceof Error ? err.message : t("report_export.status.failed"));
    }
  };

  const handleDownload = async () => {
    if (!jobId) return;

    setPhase("downloading");
    try {
      const blob = await downloadExportReport(wsId, jobId);
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `attendance-report-${startDate}-to-${endDate}.xlsx`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      window.URL.revokeObjectURL(url);
      onClose();
    } catch (err) {
      setPhase("failed");
      setError(err instanceof Error ? err.message : "Download failed");
    }
  };

  const progress = jobStatus?.progress;
  const percentage = progress?.percentage ?? 0;

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title={t("report_export.title")}
      subtitle={t("report_export.subtitle")}
      icon={<FileSpreadsheet size={24} />}
      maxWidth="md"
    >
      <div className="p-6 space-y-6">
        {/* Export Info */}
        <div className="bg-slate-50 rounded-2xl p-4 space-y-2">
          <div className="flex justify-between text-sm">
            <span className="text-slate-500">{t("reports.date_range")}:</span>
            <span className="font-medium">{startDate} ~ {endDate}</span>
          </div>
          <div className="flex justify-between text-sm">
            <span className="text-slate-500">{t("reports.report_type")}:</span>
            <span className="font-medium capitalize">{reportType}</span>
          </div>
          {employeeIds && employeeIds.length > 0 && (
            <div className="flex justify-between text-sm">
              <span className="text-slate-500">{t("reports.employees")}:</span>
              <span className="font-medium">{employeeIds.length} {t("common.selected")}</span>
            </div>
          )}
        </div>

        {/* Progress Section */}
        {phase !== "idle" && (
          <div className="space-y-4">
            {/* Progress Bar */}
            <div className="h-4 w-full bg-slate-100 rounded-full overflow-hidden shadow-inner">
              <div
                className={cn(
                  "h-full transition-all duration-500 ease-out",
                  phase === "failed" ? "bg-rose-500" :
                  phase === "completed" ? "bg-emerald-500" :
                  "bg-primary-600 bg-gradient-to-r from-primary-500 to-primary-600"
                )}
                style={{ width: `${phase === "completed" ? 100 : percentage}%` }}
              />
            </div>

            {/* Status */}
            <div className="flex items-center justify-center gap-3">
              {phase === "starting" || phase === "polling" ? (
                <Loader2 className="w-5 h-5 text-primary-600 animate-spin" />
              ) : phase === "completed" ? (
                <CheckCircle2 className="w-5 h-5 text-emerald-500" />
              ) : phase === "failed" ? (
                <AlertCircle className="w-5 h-5 text-rose-500" />
              ) : phase === "downloading" ? (
                <Loader2 className="w-5 h-5 text-primary-600 animate-spin" />
              ) : null}

              <span className={cn(
                "font-medium",
                phase === "failed" ? "text-rose-600" :
                phase === "completed" ? "text-emerald-600" :
                "text-slate-700"
              )}>
                {phase === "starting" && t("report_export.status.preparing")}
                {phase === "polling" && t("report_export.status.processing")}
                {phase === "completed" && t("report_export.status.completed")}
                {phase === "failed" && t("report_export.status.failed")}
                {phase === "downloading" && t("report_export.status.downloading")}
              </span>
            </div>

            {/* Batch Progress */}
            {phase === "polling" && progress && totalBatches > 0 && (
              <p className="text-center text-sm text-slate-500">
                {t("report_export.batches_completed", {
                  completed: progress.completed_batches,
                  total: totalBatches,
                })}
              </p>
            )}

            {/* Error Message */}
            {error && (
              <div className="bg-rose-50 border border-rose-200 rounded-xl p-4">
                <p className="text-sm text-rose-700">{error}</p>
              </div>
            )}
          </div>
        )}

        {/* Actions */}
        <div className="flex gap-3 pt-2">
          {phase === "idle" && (
            <>
              <button
                onClick={onClose}
                className="flex-1 px-6 py-3 border border-slate-200 text-slate-700 rounded-xl hover:bg-slate-50 transition-colors font-medium"
              >
                {t("common.cancel")}
              </button>
              <button
                onClick={handleStartExport}
                className="flex-1 flex items-center justify-center gap-2 px-6 py-3 bg-primary-600 text-white rounded-xl hover:bg-primary-700 transition-colors font-medium"
              >
                <Download size={18} />
                {t("report_export.start_export")}
              </button>
            </>
          )}

          {(phase === "starting" || phase === "polling") && (
            <button
              onClick={onClose}
              className="flex-1 px-6 py-3 border border-slate-200 text-slate-700 rounded-xl hover:bg-slate-50 transition-colors font-medium"
            >
              {t("report_export.run_background")}
            </button>
          )}

          {phase === "completed" && (
            <>
              <button
                onClick={onClose}
                className="flex-1 px-6 py-3 border border-slate-200 text-slate-700 rounded-xl hover:bg-slate-50 transition-colors font-medium"
              >
                {t("common.close")}
              </button>
              <button
                onClick={handleDownload}
                className="flex-1 flex items-center justify-center gap-2 px-6 py-3 bg-emerald-600 text-white rounded-xl hover:bg-emerald-700 transition-colors font-medium"
              >
                <Download size={18} />
                {t("report_export.download_button")}
              </button>
            </>
          )}

          {phase === "failed" && (
            <>
              <button
                onClick={onClose}
                className="flex-1 px-6 py-3 border border-slate-200 text-slate-700 rounded-xl hover:bg-slate-50 transition-colors font-medium"
              >
                {t("common.close")}
              </button>
              <button
                onClick={handleStartExport}
                className="flex-1 flex items-center justify-center gap-2 px-6 py-3 bg-primary-600 text-white rounded-xl hover:bg-primary-700 transition-colors font-medium"
              >
                {t("common.retry")}
              </button>
            </>
          )}

          {phase === "downloading" && (
            <button
              disabled
              className="flex-1 flex items-center justify-center gap-2 px-6 py-3 bg-slate-300 text-slate-500 rounded-xl cursor-not-allowed font-medium"
            >
              <Loader2 size={18} className="animate-spin" />
              {t("report_export.status.downloading")}
            </button>
          )}
        </div>
      </div>
    </Modal>
  );
}
