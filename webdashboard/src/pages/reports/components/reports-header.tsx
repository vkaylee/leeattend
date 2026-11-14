import { Link } from "@tanstack/react-router";
import { CalendarCheck, Download, LayoutGrid, Table } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { PageHeader } from "@/components/ui/page-header";
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

  return (
    <PageHeader
      actions={
        <div className="flex items-center gap-3 flex-wrap">
          {/* View Mode Switcher (Matrix Grid vs Summary Hours) */}
          {!!onViewModeChange && (
            <div
              aria-label={t("reports.view_mode_group", "Chế độ xem báo cáo")}
              className="flex items-center bg-slate-100 dark:bg-slate-800/80 p-1 rounded-2xl border border-slate-200 dark:border-slate-700"
              role="group"
            >
              <Button
                aria-label={t("reports.view_matrix", "Bảng công tháng (Lưới 31 ngày)")}
                aria-pressed={viewMode === "matrix"}
                className={cn(
                  "flex items-center gap-1.5 px-3.5 py-1.5 rounded-xl text-xs font-semibold normal-case tracking-normal transition-all",
                  viewMode === "matrix"
                    ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm"
                    : "text-slate-500 hover:text-slate-800 dark:hover:text-slate-200"
                )}
                onClick={() => {
                  onViewModeChange("matrix");
                }}
                variant="ghost"
              >
                <LayoutGrid size={15} />
                <span>{t("reports.view_matrix", "Bảng công tháng")}</span>
              </Button>

              <Button
                aria-label={t("reports.view_summary", "Tổng hợp số giờ")}
                aria-pressed={viewMode === "summary"}
                className={cn(
                  "flex items-center gap-1.5 px-3.5 py-1.5 rounded-xl text-xs font-semibold normal-case tracking-normal transition-all",
                  viewMode === "summary"
                    ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm"
                    : "text-slate-500 hover:text-slate-800 dark:hover:text-slate-200"
                )}
                onClick={() => {
                  onViewModeChange("summary");
                }}
                variant="ghost"
              >
                <Table size={15} />
                <span>{t("reports.view_summary", "Tổng hợp giờ")}</span>
              </Button>
            </div>
          )}

          <Link
            className="flex items-center gap-2 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-200 px-4 py-2 rounded-xl hover:bg-slate-50 dark:hover:bg-slate-800 transition-all font-semibold text-xs tracking-wide shadow-sm active:scale-95"
            search={{
              tab: "recorded" as const,
              mode: "custom" as const,
              from: dateRange.start,
              to: dateRange.end,
            }}
            to="/attendance-logs"
          >
            <CalendarCheck size={16} /> {t("reports.view_raw_logs", "Xem nhật ký thô")}
          </Link>

          <Button
            className="flex items-center gap-2 h-10 px-4 rounded-xl font-semibold text-xs normal-case tracking-normal shadow-sm"
            data-testid="export-report-btn"
            icon={<Download size={16} />}
            onClick={onExportClick}
            variant="outline"
          >
            {viewMode === "matrix"
              ? t("reports.export_matrix_csv", "Xuất ma trận (CSV)")
              : t("reports.export_excel", "Xuất Excel")}
          </Button>
        </div>
      }
      className="px-2"
      subtitle={t("reports.subtitle")}
      title={t("reports.title")}
    />
  );
}
