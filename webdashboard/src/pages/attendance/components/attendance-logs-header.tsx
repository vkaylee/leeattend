import { Link } from "@tanstack/react-router";
import { BarChart3, Download, Upload } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import PermissionGate from "@/components/common/permission-gate";
import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";

interface AttendanceLogsHeaderProps {
  dateRange: { start: string; end: string };
  setIsImportDialogOpen: (open: boolean) => void;
}

export function AttendanceLogsHeader({ dateRange, setIsImportDialogOpen }: AttendanceLogsHeaderProps) {
  const { t } = useTranslation();

  return (
    <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 px-2">
      <div>
        <Typography className="text-3xl font-bold tracking-tight text-slate-700 dark:text-slate-200" data-testid="attendance-logs-title" variant="h1">
          {t("attendance_logs.title")}
        </Typography>
        <Typography className="text-slate-500 dark:text-slate-400 mt-1 font-medium" variant="p">{t("attendance_logs.subtitle")}</Typography>
      </div>
      <div className="flex items-center gap-3">
        <Link
          className="flex items-center gap-2 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 text-primary-600 dark:text-primary-400 px-5 py-2.5 rounded-2xl hover:bg-primary-50 dark:hover:bg-primary-950 transition-all font-bold text-[10px] uppercase tracking-widest shadow-sm"
          search={{ from: dateRange.start, to: dateRange.end } as Record<string, unknown>}
          to="/reports"
        >
          <BarChart3 size={14} /> {t("attendance_logs.view_reports")}
        </Link>
        <Button icon={<Download size={14} />} onClick={() => { window.print(); }} size="sm" variant="outline">
          {t("attendance_logs.export_btn")}
        </Button>
        <PermissionGate permission="attendance_staging:manage">
          <Button icon={<Upload size={14} />} onClick={() => { setIsImportDialogOpen(true); }} size="sm">
            {t("attendance_logs.import_btn")}
          </Button>
        </PermissionGate>
      </div>
    </div>
  );
}
