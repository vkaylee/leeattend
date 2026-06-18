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
        <Typography className="font-bold mb-0.5 normal-case" data-testid="attendance-logs-title" variant="h3">
          {t("attendance_logs.title")}
        </Typography>
        <Typography className="text-xs text-slate-500 dark:text-slate-400 font-medium opacity-80" variant="small">{t("attendance_logs.subtitle")}</Typography>
      </div>
      <div className="flex items-center gap-3">
        <Link
          className="flex items-center gap-2 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 text-primary-600 dark:text-primary-400 px-4 py-2 rounded-xl hover:bg-primary-50 dark:hover:bg-primary-950 transition-all font-bold text-[10px] uppercase tracking-widest shadow-sm"
          search={{ from: dateRange.start, to: dateRange.end } as Record<string, unknown>}
          to="/reports"
        >
          <BarChart3 size={14} /> {t("attendance_logs.view_reports")}
        </Link>
        <Button className="rounded-xl h-10" icon={<Download size={14} />} onClick={() => { window.print(); }} size="sm" variant="outline">
          {t("attendance_logs.export_btn")}
        </Button>
        <PermissionGate permission="attendance_staging:manage">
          <Button className="rounded-xl h-10" icon={<Upload size={14} />} onClick={() => { setIsImportDialogOpen(true); }} size="sm" variant="ghost">
            {t("attendance_logs.import_btn")}
          </Button>
        </PermissionGate>
      </div>
    </div>
  );
}
