import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import type { ExportPhase } from "../report-export-dialog";
import { exportAttendanceReport } from "../../../../lib/api/attendance-monitoring";
import { downloadBlob } from "../../../../lib/api/download-helper";

export interface UseExportDialogOptions {
  employeeIds?: string[];
  endDate: string;
  isOpen: boolean;
  onClose: () => void;
  reportType: "detail" | "summary";
  startDate: string;
}

export function useExportDialog({ isOpen, startDate, endDate, reportType, employeeIds, onClose }: UseExportDialogOptions) {
  const { t } = useTranslation();
  const [phase, setPhase] = useState<ExportPhase>("idle");
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (isOpen) {
      setPhase("idle");
      setError(null);
    }
  }, [isOpen]);

  const handleStartExport = async () => {
    setPhase("starting");
    setError(null);
    try {
      setPhase("downloading");
      const blob = await exportAttendanceReport({
        start_date: startDate,
        end_date: endDate,
        format: "xlsx",
        report_type: reportType,
        employee_ids: employeeIds,
      });
      downloadBlob(blob, `attendance-report-${startDate}-to-${endDate}.xlsx`);
      setPhase("completed");
      onClose();
    } catch (err) {
      setPhase("failed");
      setError(err instanceof Error ? err.message : t("report_export.status.failed"));
    }
  };

  return {
    t,
    phase,
    error,
    percentage: phase === "completed" ? 100 : 0,
    completedBatches: phase === "completed" ? 1 : 0,
    totalBatches: phase === "idle" ? 0 : 1,
    useFallbackPolling: false,
    handleStartExport,
    handleDownload: handleStartExport,
  };
}
