import type { TFunction } from "i18next";

import { useQuery } from "@tanstack/react-query";
import { Download, Loader2, CheckCircle2, AlertCircle, FileSpreadsheet } from "lucide-react";
import { useState, useMemo, useEffect } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";

import { useReportStream } from "../../../hooks/use-report-stream";
import {
  createReportExportJob,
  fetchExportJobStatus,
  downloadExportReport,
} from "../../../lib/api/attendance";
import { cn } from "../../../lib/utils";
import Modal from "../../ui/modal";

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

interface ExportInfoPanelProps {
  employeeIds?: string[];
  endDate: string;
  reportType: "summary" | "detail";
  startDate: string;
  t: TFunction;
}

function ExportInfoPanel({ startDate, endDate, reportType, employeeIds, t }: ExportInfoPanelProps) {
  return (
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
  );
}

interface ExportProgressPanelProps {
  completedBatches: number;
  error: string | null;
  percentage: number;
  phase: ExportPhase;
  t: TFunction;
  totalBatches: number;
  useFallbackPolling: boolean;
}

function ExportPhaseIcon({ phase }: { phase: ExportPhase }) {
  if (phase === "starting" || phase === "polling" || phase === "downloading") {
    return <Loader2 className="w-5 h-5 text-primary-600 dark:text-primary-400 animate-spin" />;
  }
  if (phase === "completed") return <CheckCircle2 className="w-5 h-5 text-emerald-500" />;
  if (phase === "failed") return <AlertCircle className="w-5 h-5 text-rose-500" />;
  return null;
}

function ExportStatusText({ phase, t }: { phase: ExportPhase; t: TFunction }) {
  const colorClass =
    phase === "failed" ? "text-rose-600 dark:text-rose-300" :
    phase === "completed" ? "text-emerald-600 dark:text-emerald-300" :
    "text-slate-700 dark:text-slate-200";
  const messages: Partial<Record<ExportPhase, string>> = {
    starting: t("report_export.status.preparing"),
    polling: t("report_export.status.processing"),
    completed: t("report_export.status.completed"),
    failed: t("report_export.status.failed"),
    downloading: t("report_export.status.downloading"),
  };
  return (
    <span className={`font-medium ${colorClass}`} data-testid="report-export-status-text">
      {messages[phase]}
    </span>
  );
}

function ExportProgressBar({ phase, percentage }: { percentage: number; phase: ExportPhase }) {
  const fillClass =
    phase === "failed" ? "bg-rose-50 dark:bg-rose-950" :
    phase === "completed" ? "bg-emerald-50 dark:bg-emerald-950" :
    "bg-primary-600 bg-gradient-to-r from-primary-500 to-primary-600";
  return (
    <div className="h-4 w-full bg-slate-100 dark:bg-slate-700 rounded-full overflow-hidden shadow-inner" data-testid="report-export-progress-bar">
      <div
        className={cn("h-full transition-all duration-500 ease-out", fillClass)}
        data-testid="report-export-progress-fill"
        style={{ width: `${phase === "completed" ? 100 : percentage}%` }}
      />
    </div>
  );
}

function ExportProgressPanel({ phase, percentage, totalBatches, completedBatches, error, useFallbackPolling, t }: ExportProgressPanelProps) {
  return (
    <div className="space-y-4">
      <ExportProgressBar percentage={percentage} phase={phase} />

      <div className="flex items-center justify-center gap-3">
        <ExportPhaseIcon phase={phase} />
        <ExportStatusText phase={phase} t={t} />
      </div>

      {phase === "polling" && totalBatches > 0 && (
        <Typography className="text-center text-sm text-slate-500 dark:text-slate-400" data-testid="report-export-batch-counter" variant="p">
          {t("report_export.batches_completed", { completed: completedBatches, total: totalBatches })}
        </Typography>
      )}

      {error ? (
        <div className="bg-rose-50 dark:bg-rose-950 border border-rose-200 rounded-xl p-4" data-testid="report-export-error">
          <Typography className="text-sm text-rose-700 dark:text-rose-300" variant="p">{error}</Typography>
        </div>
      ) : null}

      {useFallbackPolling && phase === "polling" ? (
        <div className="bg-amber-50 dark:bg-amber-950 border border-amber-200 rounded-xl p-3" data-testid="report-export-fallback-notice">
          <Typography className="text-xs text-amber-700 dark:text-amber-300" variant="p">{t("report_export.fallback_mode")}</Typography>
        </div>
      ) : null}
    </div>
  );
}

interface ExportActionsPanelProps {
  handleDownload: () => Promise<void>;
  handleStartExport: () => Promise<void>;
  onClose: () => void;
  phase: ExportPhase;
  t: TFunction;
}

function ExportActionsPanel({ phase, onClose, handleStartExport, handleDownload, t }: ExportActionsPanelProps) {
  return (
    <div className="flex gap-3 pt-2">
      {phase === "idle" && (
        <>
          <Button
            className="flex-1 h-auto py-3 px-6 font-medium"
            data-testid="report-export-cancel-button"
            onClick={onClose}
            variant="outline"
          >
            {t("common.cancel")}
          </Button>
          <Button
            className="flex-1 h-auto py-3 px-6 font-medium"
            data-testid="report-export-start-button"
            icon={<Download size={18} />}
            onClick={handleStartExport}
            variant="primary"
          >
            {t("report_export.start_export")}
          </Button>
        </>
      )}

      {(phase === "starting" || phase === "polling") && (
        <Button
          className="flex-1 h-auto py-3 px-6 font-medium"
          data-testid="report-export-background-button"
          onClick={onClose}
          variant="outline"
        >
          {t("report_export.run_background")}
        </Button>
      )}

      {phase === "completed" && (
        <>
          <Button
            className="flex-1 h-auto py-3 px-6 font-medium"
            data-testid="report-export-close-button"
            onClick={onClose}
            variant="outline"
          >
            {t("common.close")}
          </Button>
          <Button
            className="flex-1 h-auto py-3 px-6 bg-emerald-600 hover:bg-emerald-700 text-white font-medium"
            data-testid="report-export-download-button"
            icon={<Download size={18} />}
            onClick={handleDownload}
            variant="primary"
          >
            {t("report_export.download_button")}
          </Button>
        </>
      )}

      {phase === "failed" && (
        <>
          <Button
            className="flex-1 h-auto py-3 px-6 font-medium"
            data-testid="report-export-close-button"
            onClick={onClose}
            variant="outline"
          >
            {t("common.close")}
          </Button>
          <Button
            className="flex-1 h-auto py-3 px-6 font-medium"
            data-testid="report-export-retry-button"
            onClick={handleStartExport}
            variant="primary"
          >
            {t("common.retry")}
          </Button>
        </>
      )}

      {phase === "downloading" && (
        <Button
          className="flex-1 h-auto py-3 px-6 bg-slate-300 text-slate-500 dark:text-slate-400 font-medium"
          data-testid="report-export-downloading-button"
          disabled
          loading
          variant="ghost"
        >
          {t("report_export.status.downloading")}
        </Button>
      )}
    </div>
  );
}

function derivePhase(jobId: string | null, useFallbackPolling: boolean, streamStatus: string, phaseState: ExportPhase): ExportPhase {
  if (jobId && !useFallbackPolling) {
    if (streamStatus === "connecting") return "starting";
    if (streamStatus === "streaming") return "polling";
  }
  return phaseState;
}

interface UseExportDialogOptions {
  employeeIds?: string[];
  endDate: string;
  isOpen: boolean;
  onClose: () => void;
  reportType: "detail" | "summary";
  startDate: string;
  wsId: string;
}

function handleStreamError(
  message: string,
  setUseFallbackPolling: (v: boolean) => void,
  setPhase: (p: ExportPhase) => void,
  setError: (e: string | null) => void,
) {
  if (message.includes("Max retries exceeded") || message.includes("Connection lost")) {
    setUseFallbackPolling(true); setPhase("polling"); setError(null);
  } else {
    setPhase("failed"); setError(message);
  }
}

function calcPercentage({ jobId, phase, useFallbackPolling, pollingProgress, streamProgress }: {
  jobId: string | null; phase: ExportPhase; pollingProgress: number | undefined; streamProgress: number; useFallbackPolling: boolean;
}): number {
  if (!jobId || phase !== "polling") return 0;
  return useFallbackPolling && pollingProgress != null ? pollingProgress : streamProgress;
}

function calcCompletedBatches(useFallbackPolling: boolean, pollingCompleted: number | undefined, totalBatches: number, streamProgress: number): number {
  if (useFallbackPolling && pollingCompleted != null) return pollingCompleted;
  return totalBatches > 0 ? Math.floor((streamProgress / 100) * totalBatches) : 0;
}

function useExportDialog({ isOpen, wsId, startDate, endDate, reportType, employeeIds, onClose }: UseExportDialogOptions) {
  const { t } = useTranslation();
  const [phaseState, setPhase] = useState<ExportPhase>("idle");
  const [jobId, setJobId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [totalBatches, setTotalBatches] = useState(0);
  const [useFallbackPolling, setUseFallbackPolling] = useState(false);

  // Reset export state when dialog re-opens. This is a legitimate use of
  // useEffect: the effect is responding to a modal-open event (a side effect),
  // not deriving state from props.
  useEffect(() => {
    if (isOpen) {
      setPhase("idle"); setJobId(null); setError(null); setTotalBatches(0); setUseFallbackPolling(false);
    }
  }, [isOpen]);

  const { status: streamStatus, progress: streamProgress } = useReportStream({
    jobId,
    onComplete: () => { setPhase("completed"); },
    onError: (message) => { handleStreamError(message, setUseFallbackPolling, setPhase, setError); },
    maxRetries: 3,
  });

  const phase = derivePhase(jobId, useFallbackPolling, streamStatus, phaseState);

  const { data: pollingStatus } = useQuery({
    queryKey: ["export-job-fallback", jobId],
    queryFn: () => fetchExportJobStatus(wsId, jobId ?? ""),
    enabled: !!jobId && useFallbackPolling && phase === "polling",
    refetchInterval: (query) => {
      const status = query.state.data?.status;
      return status === "processing" || status === "pending" ? 2000 : false;
    },
  });

  // Derive phase from fallback polling — useMemo avoids setState-in-useEffect.
  const pollingPhase = useMemo((): ExportPhase | null => {
    if (!useFallbackPolling || !pollingStatus) return null;
    if (pollingStatus.status === "completed") return "completed";
    if (pollingStatus.status === "failed") return "failed";
    return null;
  }, [useFallbackPolling, pollingStatus]);

  // Effective phase: polling result wins when fallback is active
  const effectivePhase = pollingPhase ?? phase;

  // Polling error message — derived via useMemo
  const pollingErrorMessage = useMemo(() => {
    if (useFallbackPolling && pollingStatus?.status === "failed") {
      return pollingStatus.error ?? t("report_export.status.failed");
    }
    return null;
  }, [useFallbackPolling, pollingStatus, t]);

  // Effective error: polling error overrides local error when available
  const effectiveError = pollingErrorMessage ?? error;

  const handleStartExport = async () => {
    setPhase("starting"); setError(null);
    try {
      const response = await createReportExportJob(wsId, { start_date: startDate, end_date: endDate, report_type: reportType, employee_ids: employeeIds });
      setJobId(response.job_id); setTotalBatches(response.total_batches);
    } catch (err) {
      setPhase("failed"); setError(err instanceof Error ? err.message : t("report_export.status.failed"));
    }
  };

  const handleDownload = async () => {
    if (!jobId) return;
    setPhase("downloading");
    try {
      const blob = await downloadExportReport(wsId, jobId);
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url; a.download = `attendance-report-${startDate}-to-${endDate}.xlsx`;
      document.body.appendChild(a); a.click(); document.body.removeChild(a);
      window.URL.revokeObjectURL(url); onClose();
    } catch (err) {
      setPhase("failed"); setError(err instanceof Error ? err.message : "Download failed");
    }
  };

  const percentage = calcPercentage({ jobId, phase: effectivePhase, useFallbackPolling, pollingProgress: pollingStatus?.progress.percentage, streamProgress });
  const completedBatches = calcCompletedBatches(useFallbackPolling, pollingStatus?.progress.completed_batches, totalBatches, streamProgress);

  return { t, phase: effectivePhase, error: effectiveError, totalBatches, useFallbackPolling, percentage, completedBatches, handleStartExport, handleDownload };
}

export default function ReportExportDialog({ isOpen, onClose, wsId, startDate, endDate, reportType, employeeIds }: ReportExportDialogProps) {
  const { t } = useTranslation();
  const { phase, error, totalBatches, useFallbackPolling, percentage, completedBatches, handleStartExport, handleDownload } = useExportDialog({
    isOpen, wsId, startDate, endDate, reportType, employeeIds, onClose,
  });

  return (
    <Modal icon={<FileSpreadsheet size={24} />} isOpen={isOpen} maxWidth="md" onClose={onClose} subtitle={t("report_export.subtitle")} testId="report-export-dialog" title={t("report_export.title")}>
      <div className="p-6 space-y-6">
        <ExportInfoPanel employeeIds={employeeIds} endDate={endDate} reportType={reportType} startDate={startDate} t={t} />
        {phase !== "idle" ? (
          <ExportProgressPanel completedBatches={completedBatches} error={error} percentage={percentage} phase={phase} t={t} totalBatches={totalBatches} useFallbackPolling={useFallbackPolling} />
        ) : null}
        <ExportActionsPanel handleDownload={handleDownload} handleStartExport={handleStartExport} onClose={onClose} phase={phase} t={t} />
      </div>
    </Modal>
  );
}

