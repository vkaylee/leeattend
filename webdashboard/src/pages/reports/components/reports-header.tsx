import { Link } from "@tanstack/react-router";
import { CalendarCheck, Download } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";

interface ReportsHeaderProps {
  dateRange: { start: string; end: string };
  onExportClick: () => void;
}

export function ReportsHeader({ dateRange, onExportClick }: ReportsHeaderProps) {
  const { t } = useTranslation();

  return (
    <div className="flex flex-col md:flex-row md:items-center justify-between gap-8 px-2">
      <div className="space-y-1">
        <Typography className="text-4xl font-bold tracking-tighter text-slate-700 dark:text-slate-200 uppercase leading-none" variant="h1">
          {t("reports.title")}
        </Typography>
        <Typography className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] opacity-60" variant="p">
          {t("reports.subtitle")}
        </Typography>
      </div>

      <div className="flex items-center gap-3">
        <Link
          className="flex items-center gap-3 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 text-primary-600 dark:text-primary-400 px-8 py-3 rounded-2xl hover:bg-primary-50 dark:hover:bg-primary-950 transition-all font-bold text-xs uppercase tracking-widest shadow-xl active:scale-95"
          search={{ from: dateRange.start, to: dateRange.end } as Record<string, string>}
          to="/attendance-logs"
        >
          <CalendarCheck size={18} /> {t("reports.view_raw_logs").toUpperCase()}
        </Link>
        <Button
          className="flex items-center gap-3 h-auto py-3 px-8 rounded-2xl font-bold text-xs uppercase tracking-widest"
          data-testid="export-report-btn"
          icon={<Download size={18} />}
          onClick={onExportClick}
          variant="outline"
        >
          {t("reports.export_excel").toUpperCase()}
        </Button>
      </div>
    </div>
  );
}
