import { useQuery } from "@tanstack/react-query";
import { Download, Loader2, CheckCircle2, AlertCircle, FileSpreadsheet } from "lucide-react";
import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";

import { useReportStream } from "../../../hooks/use-report-stream";
import {
  createReportExportJob,
  fetchExportJobStatus,
  downloadExportReport,
} from "../../../lib/api/attendance";
import { cn } from "../../../lib/utils";
import Modal from "../../ui/Modal";

interface ReportExportDialogProps {
  employeeIds?: string[];
  endDate: string;
  isOpen: boolean;
  onClose: () => void;
  reportType: "summary" | "detail";
  startDate: string;
  wsId: string;
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
  const [phaseState, setPhase] = useState<ExportPhase>("idle");
  const [jobId, setJobId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [totalBatches, setTotalBatches] = useState(0);
  const [useFallbackPolling, setUseFallbackPolling] = useState(false);

  // Reset state when dialog opens
  useEffect(() => {
    if (isOpen) {
      // eslint-disable-next-line react-hooks/set-state-in-effect -- Derived state synced from props; intentional pattern
      setPhase("idle");
      setJobId(null);
      setError(null);
      setTotalBatches(0);
      setUseFallbackPolling(false);
    }
  }, [isOpen]);

  // WebSocket streaming for real-time updates
  const {
    status: streamStatus,
    progress: streamProgress,
    error: _streamError,
  } = useReportStream({
    jobId,
    onComplete: () => {
      console.debug("[ReportExport] Stream completed");
      setPhase("completed");
    },
    onError: (message) => {
      console.error("[ReportExport] Stream error:", message);

      // Check if max retries exceeded - fallback to polling
      if (
        message.includes("Max retries exceeded") ||
        message.includes("Connection lost")
      ) {
        console.warn("[ReportExport] WebSocket failed, falling back to polling");
        setUseFallbackPolling(true);
        setPhase("polling"); // Transition to polling phase so the query is enabled
        setError(null); // Clear error, fallback will handle
      } else {
        setPhase("failed");
        setError(message);
      }
    },
    maxRetries: 3,
  });

  // Calculate derived phase from stream status to avoid sync state updates
  const phase = (() => {
    if (jobId && !useFallbackPolling) {
      if (streamStatus === "connecting") return "starting";
      if (streamStatus === "streaming") return "polling";
    }
    return phaseState;
  })();

  console.error(`[ReportDialog] Render: phase=${phase}, phaseState=${phaseState}, jobId=${jobId}, streamStatus=${streamStatus}, useFallbackPolling=${useFallbackPolling}`);

  // Fallback polling when WebSocket fails
  const { data: pollingStatus } = useQuery({
    queryKey: ["export-job-fallback", jobId],
    queryFn: () => fetchExportJobStatus(wsId, jobId!),
    enabled: !!jobId && useFallbackPolling && phase === "polling",
    refetchInterval: (query) => {
      const status = query.state.data?.status;
      return status === "processing" || status === "pending" ? 2000 : false; // 2s (less aggressive)
    },
  });

  const pollingStatusValue = pollingStatus?.status;
  const pollingError = pollingStatus?.error;

  // Handle fallback polling status
  useEffect(() => {
    if (!useFallbackPolling || !pollingStatus) return;

    if (pollingStatusValue === "completed" && phaseState !== "completed") {
      // eslint-disable-next-line react-hooks/set-state-in-effect -- Derived state synced from props; intentional pattern
      setPhase("completed");
    } else if (pollingStatusValue === "failed" && phaseState !== "failed") {
      setPhase("failed");
      setError(pollingError || t("report_export.status.failed"));
    }
  }, [pollingStatusValue, pollingError, useFallbackPolling, t, phaseState, pollingStatus]);

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
      // Phase will be set automatically by useEffect watching streamStatus
    } catch (error_) {
      setPhase("failed");
      setError(error_ instanceof Error ? error_.message : t("report_export.status.failed"));
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
    } catch (error_) {
      setPhase("failed");
      setError(error_ instanceof Error ? error_.message : "Download failed");
    }
  };

  // Calculate percentage from appropriate source
  const percentage =
    jobId && phase === "polling"
      ? useFallbackPolling && pollingStatus?.progress?.percentage
        ? pollingStatus.progress.percentage
        : streamProgress
      : 0;

  // Calculate batch count
  const completedBatches = useFallbackPolling && pollingStatus?.progress
    ? pollingStatus.progress.completed_batches
    : totalBatches > 0
      ? Math.floor((streamProgress / 100) * totalBatches)
      : 0;

  return (
    <Modal
      icon={<FileSpreadsheet size={24} />}
      isOpen={isOpen}
      maxWidth="md"
      onClose={onClose}
      subtitle={t("report_export.subtitle")}
      testId="report-export-dialog"
      title={t("report_export.title")}
    >
      <div className="p-6 space-y-6">
        {/* Export Info */}
        <div className="bg-slate-50 dark:bg-slate-800 rounded-2xl p-4 space-y-2" data-testid="report-export-info">
          <div className="flex justify-between text-sm">
            <span className="text-slate-500 dark:text-slate-400">{t("reports.date_range")}:</span>
            <span className="font-medium">{startDate} ~ {endDate}</span>
          </div>
          <div className="flex justify-between text-sm">
            <span className="text-slate-500 dark:text-slate-400">{t("reports.report_type")}:</span>
            <span className="font-medium capitalize">{reportType}</span>
          </div>
          {!!employeeIds && employeeIds.length > 0 && (
            <div className="flex justify-between text-sm">
              <span className="text-slate-500 dark:text-slate-400">{t("reports.employees")}:</span>
              <span className="font-medium">{employeeIds.length} {t("common.selected")}</span>
            </div>
          )}
        </div>

        {/* Progress Section */}
        {phase !== "idle" && (
          <div className="space-y-4">
            {/* Progress Bar */}
            <div className="h-4 w-full bg-slate-100 dark:bg-slate-700 rounded-full overflow-hidden shadow-inner" data-testid="report-export-progress-bar">
              <div
                className={cn(
                  "h-full transition-all duration-500 ease-out",
                  phase === "failed" ? "bg-rose-50 dark:bg-rose-950" :
                    phase === "completed" ? "bg-emerald-50 dark:bg-emerald-950" :
                      "bg-primary-600 bg-gradient-to-r from-primary-500 to-primary-600"
                )}
                data-testid="report-export-progress-fill"
                style={{ width: `${phase === "completed" ? 100 : percentage}%` }}
              />
            </div>

            {/* Status */}
            <div className="flex items-center justify-center gap-3">
              {phase === "starting" || phase === "polling" ? (
                <Loader2 className="w-5 h-5 text-primary-600 dark:text-primary-400 animate-spin" />
              ) : phase === "completed" ? (
                <CheckCircle2 className="w-5 h-5 text-emerald-500" />
              ) : phase === "failed" ? (
                <AlertCircle className="w-5 h-5 text-rose-500" />
              ) : phase === "downloading" ? (
                <Loader2 className="w-5 h-5 text-primary-600 dark:text-primary-400 animate-spin" />
              ) : null}

              <span className={cn(
                "font-medium",
                phase === "failed" ? "text-rose-600 dark:text-rose-300" :
                  phase === "completed" ? "text-emerald-600 dark:text-emerald-300" :
                    "text-slate-700 dark:text-slate-200"
              )}
                data-testid="report-export-status-text"
              >
                {phase === "starting" && t("report_export.status.preparing")}
                {phase === "polling" && t("report_export.status.processing")}
                {phase === "completed" && t("report_export.status.completed")}
                {phase === "failed" && t("report_export.status.failed")}
                {phase === "downloading" && t("report_export.status.downloading")}
              </span>
            </div>

            {/* Batch Progress */}
            {phase === "polling" && totalBatches > 0 && (
              <p className="text-center text-sm text-slate-500 dark:text-slate-400" data-testid="report-export-batch-counter">
                {t("report_export.batches_completed", {
                  completed: completedBatches,
                  total: totalBatches,
                })}
              </p>
            )}

            {/* Error Message */}
            {!!error && (
              <div className="bg-rose-50 dark:bg-rose-950 border border-rose-200 rounded-xl p-4" data-testid="report-export-error">
                <p className="text-sm text-rose-700 dark:text-rose-300">{error}</p>
              </div>
            )}

            {/* Fallback Mode Notification */}
            {!!useFallbackPolling && phase === "polling" && (
              <div className="bg-amber-50 dark:bg-amber-950 border border-amber-200 rounded-xl p-3" data-testid="report-export-fallback-notice">
                <p className="text-xs text-amber-700 dark:text-amber-300">
                  {t("report_export.fallback_mode")}
                </p>
              </div>
            )}
          </div>
        )}

        {/* Actions */}
        <div className="flex gap-3 pt-2">
          {phase === "idle" && (
            <>
              <button
                className="flex-1 px-6 py-3 border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-200 rounded-xl hover:bg-slate-50 dark:hover:bg-slate-800 transition-colors font-medium"
                data-testid="report-export-cancel-button"
                onClick={onClose}
              >
                {t("common.cancel")}
              </button>
              <button
                className="flex-1 flex items-center justify-center gap-2 px-6 py-3 bg-primary-600 text-white rounded-xl hover:bg-primary-700 transition-colors font-medium"
                data-testid="report-export-start-button"
                onClick={handleStartExport}
              >
                <Download size={18} />
                {t("report_export.start_export")}
              </button>
            </>
          )}

          {(phase === "starting" || phase === "polling") && (
            <button
              className="flex-1 px-6 py-3 border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-200 rounded-xl hover:bg-slate-50 dark:hover:bg-slate-800 transition-colors font-medium"
              data-testid="report-export-background-button"
              onClick={onClose}
            >
              {t("report_export.run_background")}
            </button>
          )}

          {phase === "completed" && (
            <>
              <button
                className="flex-1 px-6 py-3 border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-200 rounded-xl hover:bg-slate-50 dark:hover:bg-slate-800 transition-colors font-medium"
                data-testid="report-export-close-button"
                onClick={onClose}
              >
                {t("common.close")}
              </button>
              <button
                className="flex-1 flex items-center justify-center gap-2 px-6 py-3 bg-emerald-600 text-white rounded-xl hover:bg-emerald-700 transition-colors font-medium"
                data-testid="report-export-download-button"
                onClick={handleDownload}
              >
                <Download size={18} />
                {t("report_export.download_button")}
              </button>
            </>
          )}

          {phase === "failed" && (
            <>
              <button
                className="flex-1 px-6 py-3 border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-200 rounded-xl hover:bg-slate-50 dark:hover:bg-slate-800 transition-colors font-medium"
                data-testid="report-export-close-button"
                onClick={onClose}
              >
                {t("common.close")}
              </button>
              <button
                className="flex-1 flex items-center justify-center gap-2 px-6 py-3 bg-primary-600 text-white rounded-xl hover:bg-primary-700 transition-colors font-medium"
                data-testid="report-export-retry-button"
                onClick={handleStartExport}
              >
                {t("common.retry")}
              </button>
            </>
          )}

          {phase === "downloading" && (
            <button
              className="flex-1 flex items-center justify-center gap-2 px-6 py-3 bg-slate-300 text-slate-500 dark:text-slate-400 rounded-xl cursor-not-allowed font-medium"
              data-testid="report-export-downloading-button"
              disabled
            >
              <Loader2 className="animate-spin" size={18} />
              {t("report_export.status.downloading")}
            </button>
          )}
        </div>
      </div>
    </Modal>
  );
}
