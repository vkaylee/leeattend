import { useQuery } from "@tanstack/react-query";
import { useState, useMemo, useEffect } from "react";
import { useTranslation } from "react-i18next";

import { createQueryKey } from "@/lib/query-keys";

import type { ExportPhase } from "../report-export-dialog";

import { useReportStream } from "../../../../hooks/use-report-stream";
import {
  createReportExportJob,
  fetchExportJobStatus,
  downloadExportReport,
} from "../../../../lib/api/attendance";


export interface UseExportDialogOptions {
  employeeIds?: string[];
  endDate: string;
  isOpen: boolean;
  onClose: () => void;
  reportType: "detail" | "summary";
  startDate: string;
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

export function useExportDialog({ isOpen,startDate, endDate, reportType, employeeIds, onClose }: UseExportDialogOptions) {
  const { t } = useTranslation();
  const [phaseState, setPhase] = useState<ExportPhase>("idle");
  const [jobId, setJobId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [totalBatches, setTotalBatches] = useState(0);
  const [useFallbackPolling, setUseFallbackPolling] = useState(false);

  // Reset export state when dialog re-opens.
  useEffect(() => {
    if (isOpen) {
      // eslint-disable-next-line react-hooks/set-state-in-effect
      setPhase("idle"); setJobId(null); setError(null); setTotalBatches(0); setUseFallbackPolling(false);
    }
  }, [isOpen]);

  const { status: streamStatus, progress: streamProgress } = useReportStream({
    jobId,
    onComplete: () => { setPhase("completed"); },
    onError: (message: string) => { handleStreamError(message, setUseFallbackPolling, setPhase, setError); },
    maxRetries: 3,
  });

  const phase = derivePhase(jobId, useFallbackPolling, streamStatus, phaseState);

  const { data: pollingStatus } = useQuery({
    queryKey: createQueryKey("export-job-fallback", jobId),
    queryFn: () => fetchExportJobStatus(jobId ?? ""),
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
      const response = await createReportExportJob({ start_date: startDate, end_date: endDate, report_type: reportType, employee_ids: employeeIds });
      setJobId(response.job_id); setTotalBatches(response.total_batches);
    } catch (err) {
      setPhase("failed"); setError(err instanceof Error ? err.message : t("report_export.status.failed"));
    }
  };

  const handleDownload = async () => {
    if (!jobId) return;
    setPhase("downloading");
    try {
      const blob = await downloadExportReport(jobId);
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

function derivePhase(
  jobId: string | null,
  useFallbackPolling: boolean,
  streamStatus: string,
  phaseState: ExportPhase,
): ExportPhase {
  if (!jobId) return phaseState;
  if (phaseState === "downloading" || phaseState === "completed" || phaseState === "failed") return phaseState;
  if (useFallbackPolling) return "polling";
  if (phaseState === "starting") {
    if (streamStatus === "connecting") return "starting";
    if (streamStatus === "streaming") return "polling";
  }
  return phaseState;
}
