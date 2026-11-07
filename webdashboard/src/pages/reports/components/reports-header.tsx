import { Link } from "@tanstack/react-router";
import { CalendarCheck, Download } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { PageHeader } from "@/components/ui/page-header";

interface ReportsHeaderProps {
  dateRange: { start: string; end: string };
  onExportClick: () => void;
}

export function ReportsHeader({ dateRange, onExportClick }: ReportsHeaderProps) {
  const { t } = useTranslation();

  return (
    <PageHeader
      actions={
        <div className="flex items-center gap-3 flex-wrap">
          <Link
            className="flex items-center gap-2 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 text-primary-600 dark:text-primary-400 px-5 py-2.5 rounded-xl hover:bg-primary-50 dark:hover:bg-primary-950 transition-all font-bold text-xs uppercase tracking-widest shadow-sm active:scale-95"
            search={{ tab: "recorded" as const, mode: "custom" as const, from: dateRange.start, to: dateRange.end }}
            to="/attendance-logs"
          >
            <CalendarCheck size={18} /> {t("reports.view_raw_logs")}
          </Link>
          <Button
            className="flex items-center gap-2 h-10 px-5 rounded-xl font-bold text-xs uppercase tracking-widest"
            data-testid="export-report-btn"
            icon={<Download size={18} />}
            onClick={onExportClick}
            variant="outline"
          >
            {t("reports.export_excel")}
          </Button>
        </div>
      }
      className="px-2"
      subtitle={t("reports.subtitle")}
      title={t("reports.title")}
    />
  );
}
