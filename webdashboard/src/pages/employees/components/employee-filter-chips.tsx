import { Building2, X, Users, UserX, UserCheck } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { cn } from "@/lib/utils";

interface EmployeeFilterChipsProps {
  customFieldFilters: Record<string, string>;
  departments: { id: string; name: string }[] | undefined;
  filterDeptIds: string[];
  filterStatus: string;
  setCustomFieldFilters: (filters: Record<string, string>) => void;
  setFilterDeptIds: (ids: string[]) => void;
  setFilterStatus: (status: string) => void;
  toggleDeptFilter: (id: string) => void;
  updateCustomFieldFilter: (key: string, val: string) => void;
}

export function EmployeeFilterChips({
  filterDeptIds,
  departments,
  toggleDeptFilter,
  setFilterDeptIds,
  customFieldFilters,
  setCustomFieldFilters,
  updateCustomFieldFilter,
  filterStatus,
  setFilterStatus,
}: EmployeeFilterChipsProps) {
  const { t } = useTranslation();

  const activeCfKeys = Object.keys(customFieldFilters);
  const hasActiveFilters = filterDeptIds.length > 0 || activeCfKeys.length > 0;

  return (
    <div className="space-y-4">
      {/* Status Selection Segmented Control */}
      <div className="flex items-center gap-1 bg-slate-100 dark:bg-slate-800/80 p-0.5 rounded-xl border border-slate-200/50 dark:border-slate-700/50 w-fit">
        <Button
          className={cn(
            "h-8 px-4 text-xs font-bold rounded-lg transition-all border-none shadow-none uppercase tracking-wide",
            filterStatus === "active"
              ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm"
              : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-300 hover:bg-transparent"
          )}
          onClick={() => { setFilterStatus("active"); }}
          variant="ghost"
        >
          <UserCheck className="mr-1.5 opacity-70" size={12} />
          {t("employees.table.active", "Hoạt động")}
        </Button>
        <Button
          className={cn(
            "h-8 px-4 text-xs font-bold rounded-lg transition-all border-none shadow-none uppercase tracking-wide",
            filterStatus === "terminated"
              ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm"
              : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-300 hover:bg-transparent"
          )}
          onClick={() => { setFilterStatus("terminated"); }}
          variant="ghost"
        >
          <UserX className="mr-1.5 opacity-70" size={12} />
          {t("employees.table.terminated", "Đã nghỉ việc")}
        </Button>
        <Button
          className={cn(
            "h-8 px-4 text-xs font-bold rounded-lg transition-all border-none shadow-none uppercase tracking-wide",
            filterStatus === "all"
              ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm"
              : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-300 hover:bg-transparent"
          )}
          onClick={() => { setFilterStatus("all"); }}
          variant="ghost"
        >
          <Users className="mr-1.5 opacity-70" size={12} />
          {t("common.all", "Tất cả")}
        </Button>
      </div>

      {!!hasActiveFilters && (
        <div className="flex items-center gap-2 flex-wrap animate-in slide-in-from-top-4 duration-300" data-testid="active-filter-chips">
          <span className="text-[9px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-[0.2em] mr-1">
            {t("employees.filtering_by", "Đang lọc")}
          </span>
          
          {/* Department Chips */}
          {!!departments && filterDeptIds.map((id) => {
            const dept = departments.find((d) => d.id === id);
            if (!dept) return null;
            return (
              <Button
                className="h-auto w-auto px-3 py-1 bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-400 rounded-full text-[10px] font-bold uppercase tracking-wide border border-primary-100 dark:border-primary-800 hover:bg-primary-100 dark:hover:bg-primary-900 transition-colors group shadow-none"
                data-testid={`dept-chip-${id}`}
                key={id}
                onClick={() => { toggleDeptFilter(id); }}
                variant="ghost"
              >
                <Building2 className="opacity-60" size={10} />
                {dept.name}
                <X className="opacity-40 group-hover:opacity-100 transition-opacity" size={10} />
              </Button>
            );
          })}

          {/* Custom Field Chips */}
          {activeCfKeys.map((key) => {
            const val = customFieldFilters[key];
            return (
              <Button
                className="h-auto w-auto px-3 py-1 bg-amber-50 dark:bg-amber-950 text-amber-600 dark:text-amber-400 rounded-full text-[10px] font-bold uppercase tracking-wide border border-amber-100 dark:border-amber-800 hover:bg-amber-100 dark:hover:bg-amber-900 transition-colors group shadow-none"
                key={key}
                onClick={() => { updateCustomFieldFilter(key, ""); }}
                variant="ghost"
              >
                <span className="opacity-80 font-mono">{key}:</span>
                <span>{val}</span>
                <X className="opacity-40 group-hover:opacity-100 transition-opacity" size={10} />
              </Button>
            );
          })}

          <Button
            className="h-auto w-auto p-0 text-[10px] font-bold text-slate-400 dark:text-slate-500 hover:text-red-500 dark:hover:text-red-400 uppercase tracking-wide transition-colors ml-1 border-none shadow-none hover:bg-transparent"
            onClick={() => {
              setFilterDeptIds([]);
              setCustomFieldFilters({});
            }}
            variant="ghost"
          >
            {t("employees.clear_all", "Xóa tất cả")}
          </Button>
        </div>
      )}
    </div>
  );
}

