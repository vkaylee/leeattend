import type { TFunction } from "i18next";

import { Download, Loader2, CheckCircle2, AlertCircle, FileSpreadsheet } from "lucide-react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";

import { cn } from "../../../lib/utils";
import Modal from "../../ui/modal";
import { useExportDialog } from "./hooks/use-export-dialog";

interface ReportExportDialogProps {
  employeeIds?: string[];
  endDate: string;
  isOpen: boolean;
  onClose: () => void;
  reportType: "summary" | "detail";
  startDate: string;
}

export type ExportPhase = "idle" | "starting" | "downloading" | "completed" | "failed";

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
  error: string | null;
  phase: ExportPhase;
  t: TFunction;
}

function ExportPhaseIcon({ phase }: { phase: ExportPhase }) {
  if (phase === "starting" || phase === "downloading") {
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

function ExportProgressPanel({ phase, error, t }: ExportProgressPanelProps) {
  return (
    <div className="space-y-4">
      <ExportProgressBar percentage={phase === "completed" ? 100 : 0} phase={phase} />

      <div className="flex items-center justify-center gap-3">
        <ExportPhaseIcon phase={phase} />
        <ExportStatusText phase={phase} t={t} />
      </div>

      {error ? (
        <div className="bg-rose-50 dark:bg-rose-950 border border-rose-200 rounded-xl p-4" data-testid="report-export-error">
          <Typography className="text-sm text-rose-700 dark:text-rose-300" variant="p">{error}</Typography>
        </div>
      ) : null}

    </div>
  );
}

interface ExportActionsPanelProps {
  handleStartExport: () => Promise<void>;
  onClose: () => void;
  phase: ExportPhase;
  t: TFunction;
}

function ExportActionsPanel({ phase, onClose, handleStartExport, t }: ExportActionsPanelProps) {
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

      {phase === "completed" && (
        <Button
          className="flex-1 h-auto py-3 px-6"
          data-testid="report-export-close-button"
          onClick={onClose}
          variant="outline"
        >
          {t("common.close")}
        </Button>
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

    </div>
  );
}

export default function ReportExportDialog({ isOpen, onClose,startDate, endDate, reportType, employeeIds }: ReportExportDialogProps) {
  const { t } = useTranslation();
  const { phase, error, handleStartExport } = useExportDialog({
    isOpen,startDate, endDate, reportType, employeeIds, onClose,
  });

  return (
    <Modal icon={<FileSpreadsheet size={24} />} isOpen={isOpen} maxWidth="md" onClose={onClose} subtitle={t("report_export.subtitle")} testId="report-export-dialog" title={t("report_export.title")}>
      <div className="p-6 space-y-6">
        <ExportInfoPanel employeeIds={employeeIds} endDate={endDate} reportType={reportType} startDate={startDate} t={t} />
        {phase !== "idle" ? (
          <ExportProgressPanel error={error} phase={phase} t={t} />
        ) : null}
        <ExportActionsPanel handleStartExport={handleStartExport} onClose={onClose} phase={phase} t={t} />
      </div>
    </Modal>
  );
}

