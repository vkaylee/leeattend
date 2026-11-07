import { Download, Users, X } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import type { Employee } from "@/lib/api/types";

interface EmployeeBulkActionBarProps {
  employees: Employee[];
  onClearSelection: () => void;
  selectedIds: string[];
}

export function EmployeeBulkActionBar({
  selectedIds,
  employees,
  onClearSelection,
}: EmployeeBulkActionBarProps) {
  const { t } = useTranslation();

  if (selectedIds.length === 0) return null;

  const handleExportCSV = () => {
    const selectedEmps = employees.filter((e) => selectedIds.includes(e.id));
    const headers = [
      "ID",
      "Employee ID",
      "Full Name",
      "Department",
      "Job Title",
      "Hire Date",
      "Status",
    ];
    const rows = selectedEmps.map((e) => [
      e.id,
      e.employee_id_number,
      `"${e.full_name.replace(/"/g, '""')}"`,
      `"${(e.department_name ?? "").replace(/"/g, '""')}"`,
      `"${(e.job_title ?? "").replace(/"/g, '""')}"`,
      e.hire_date ?? "",
      e.active_employment_id ? "Active" : "Terminated",
    ]);

    const csvContent = "data:text/csv;charset=utf-8,\uFEFF" + [headers.join(","), ...rows.map(r => r.join(","))].join("\n");
    const encodedUri = encodeURI(csvContent);
    const link = document.createElement("a");
    link.setAttribute("href", encodedUri);
    link.setAttribute("download", `employees_export_${new Date().toISOString().slice(0, 10)}.csv`);
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
  };

  return (
    <div
      className="fixed bottom-6 left-1/2 -translate-x-1/2 z-40 flex items-center gap-3 bg-slate-900/95 dark:bg-white/95 text-white dark:text-slate-900 px-5 py-3 rounded-2xl shadow-2xl backdrop-blur-md border border-slate-700/50 dark:border-slate-200/50 animate-in slide-in-from-bottom-4 duration-300"
      data-testid="employee-bulk-action-bar"
    >
      <div className="flex items-center gap-2 pr-3 border-r border-slate-700 dark:border-slate-300">
        <Users className="text-primary-400 dark:text-primary-600" size={16} />
        <span className="text-xs font-bold uppercase tracking-wider">
          {t("employees.bulk_selected", { count: selectedIds.length, defaultValue: `Đã chọn ${selectedIds.length}` })}
        </span>
      </div>

      <div className="flex items-center gap-2">
        <Button
          className="h-8 px-3 text-xs font-bold rounded-xl bg-white/10 hover:bg-white/20 dark:bg-slate-900/10 dark:hover:bg-slate-900/20 text-white dark:text-slate-900 border-none transition-all cursor-pointer flex items-center gap-1.5"
          data-testid="bulk-export-btn"
          onClick={handleExportCSV}
          type="button"
          variant="ghost"
        >
          <Download size={14} />
          <span>{t("common.export_csv", "Xuất file CSV")}</span>
        </Button>

        <Button
          aria-label={t("common.clear_selection", "Bỏ chọn")}
          className="size-8 p-0 rounded-xl hover:bg-white/20 dark:hover:bg-slate-900/20 text-slate-400 hover:text-white dark:hover:text-slate-900 transition-colors"
          data-testid="bulk-clear-btn"
          onClick={onClearSelection}
          type="button"
          variant="ghost"
        >
          <X size={14} />
        </Button>
      </div>
    </div>
  );
}
