import { Link } from "@tanstack/react-router";
import { CalendarCheck, Download, LayoutGrid, Table } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { cn } from "@/lib/utils";

import type { ReportViewMode } from "../hooks/use-reports-page";

interface ReportsHeaderProps {
  dateRange: { start: string; end: string };
  onExportClick: () => void;
  onViewModeChange?: (mode: ReportViewMode) => void;
  viewMode?: ReportViewMode;
}

export function ReportsHeader({
  dateRange,
  onExportClick,
  onViewModeChange,
  viewMode = "summary",
}: ReportsHeaderProps) {
  const { t } = useTranslation();
  const viewLabel = t("reports.view_mode_group", "Chế độ xem báo cáo");

  return (
    <header className="space-y-4" data-testid="reports-header">
      <div>
        <Typography as="h1" className="mb-1 font-bold normal-case" variant="h3">
          {t("reports.title")}
        </Typography>
        <Typography className="text-sm font-medium text-slate-500 dark:text-slate-400" variant="tagline">
          {t("reports.subtitle")}
        </Typography>
      </div>

      <div className="flex w-full flex-wrap items-center gap-2">
        {!!onViewModeChange && (
          <div
            aria-label={viewLabel}
            className="flex shrink-0 items-center rounded-xl border border-slate-200 bg-slate-100 p-1 dark:border-slate-700 dark:bg-slate-800/80"
            role="group"
          >
            <Button
              aria-controls="attendance-report-panel"
              aria-pressed={viewMode === "matrix"}
              className={cn(
                "min-h-9 shrink-0 rounded-lg px-3 text-xs font-semibold normal-case tracking-normal transition-colors sm:px-4",
                viewMode === "matrix"
                  ? "bg-white text-primary-600 shadow-sm dark:bg-slate-900 dark:text-primary-400"
                  : "text-slate-500 hover:text-slate-800 dark:text-slate-400 dark:hover:text-slate-200",
              )}
              onClick={() => { onViewModeChange("matrix"); }}
              type="button"
              variant="ghost"
            >
              <LayoutGrid aria-hidden="true" size={15} />
              <span>{t("reports.view_matrix", "Bảng công tháng")}</span>
            </Button>
            <Button
              aria-controls="attendance-report-panel"
              aria-pressed={viewMode === "summary"}
              className={cn(
                "min-h-9 shrink-0 rounded-lg px-3 text-xs font-semibold normal-case tracking-normal transition-colors sm:px-4",
                viewMode === "summary"
                  ? "bg-white text-primary-600 shadow-sm dark:bg-slate-900 dark:text-primary-400"
                  : "text-slate-500 hover:text-slate-800 dark:text-slate-400 dark:hover:text-slate-200",
              )}
              onClick={() => { onViewModeChange("summary"); }}
              type="button"
              variant="ghost"
            >
              <Table aria-hidden="true" size={15} />
              <span>{t("reports.view_summary", "Tổng hợp giờ")}</span>
            </Button>
          </div>
        )}

        <div className="ml-auto flex shrink-0 flex-wrap items-center gap-2">
          <Link
            className="inline-flex min-h-10 items-center justify-center gap-2 rounded-xl border border-slate-200 bg-white px-3 text-xs font-semibold text-slate-700 shadow-sm transition-colors hover:bg-slate-50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary-500 dark:border-slate-700 dark:bg-slate-900 dark:text-slate-200 dark:hover:bg-slate-800 sm:px-4"
            search={{
              tab: "recorded" as const,
              mode: "custom" as const,
              from: dateRange.start,
              to: dateRange.end,
            }}
            to="/attendance-logs"
          >
            <CalendarCheck aria-hidden="true" size={16} />
            <span>{t("reports.view_raw_logs", "Xem nhật ký thô")}</span>
          </Link>
          <Button
            className="min-h-10 rounded-xl px-3 text-xs font-semibold normal-case tracking-normal sm:px-4"
            data-testid="export-report-btn"
            icon={<Download aria-hidden="true" size={16} />}
            onClick={onExportClick}
            variant="outline"
          >
            {viewMode === "matrix"
              ? t("reports.export_matrix_csv", "Xuất ma trận (CSV)")
              : t("reports.export_excel", "Xuất Excel")}
          </Button>
        </div>
      </div>
    </header>
  );
}
