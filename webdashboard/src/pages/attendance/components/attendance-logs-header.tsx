import { Link } from "@tanstack/react-router";
import { BarChart3, Download, Upload } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import PermissionGate from "@/components/common/permission-gate";
import Button from "@/components/ui/button";
import { PageHeader } from "@/components/ui/page-header";
interface AttendanceLogsHeaderProps {
  dateRange: { start: string; end: string };
  setIsImportDialogOpen: (open: boolean) => void;
}

export function AttendanceLogsHeader({ dateRange, setIsImportDialogOpen }: AttendanceLogsHeaderProps) {
  const { t } = useTranslation();

  return (
    <PageHeader
      actions={
        <div className="flex flex-wrap items-center gap-2">
          <Link
            className="flex h-[var(--ui-height-sm)] items-center gap-2 rounded-xl border border-slate-200 bg-white px-4 text-sm font-medium text-primary-600 shadow-sm transition-all hover:bg-primary-50 dark:border-slate-700 dark:bg-slate-900 dark:text-primary-400 dark:hover:bg-primary-950"
            search={{ from: dateRange.start, to: dateRange.end } as { from?: string; to?: string }}
            to="/reports"
          >
            <BarChart3 size={14} /> {t("attendance_logs.view_reports")}
          </Link>
          <Button className="rounded-xl" icon={<Download size={14} />} onClick={() => { window.print(); }} size="sm" variant="outline">
            {t("attendance_logs.export_btn")}
          </Button>
          <PermissionGate permission="attendance_staging:manage">
            <Button className="rounded-xl" icon={<Upload size={14} />} onClick={() => { setIsImportDialogOpen(true); }} size="sm" variant="ghost">
              {t("attendance_logs.import_btn")}
            </Button>
          </PermissionGate>
        </div>
      }
      className="px-2"
      data-testid="attendance-logs-title"
      subtitle={t("attendance_logs.subtitle")}
      title={t("attendance_logs.title")}
    />
  );
}
