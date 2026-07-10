import { Building2, UserX, Users, X } from "lucide-react";
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

const STATUS_chip_STYLES = "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-400 border-emerald-100 dark:border-emerald-800 hover:bg-emerald-100 dark:hover:bg-emerald-900";

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
  const isStatusFiltered = filterStatus !== "active";
  const hasActiveFilters = isStatusFiltered || filterDeptIds.length > 0 || activeCfKeys.length > 0;

  const statusLabel = filterStatus === "terminated"
    ? t("employees.table.terminated", "Đã nghỉ việc")
    : filterStatus === "all"
      ? t("common.all", "Tất cả")
      : null;

  const StatusIcon = filterStatus === "terminated" ? UserX : Users;

  return (
    <div className="space-y-4">
      {hasActiveFilters ? (
        <div className="flex items-center gap-2 flex-wrap animate-in slide-in-from-top-4 duration-300" data-testid="active-filter-chips">
          <span className="text-[9px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-[0.2em] mr-1">
            {t("employees.filtering_by", "Đang lọc")}
          </span>

          {/* Status Chip */}
          {isStatusFiltered ? (
            <Button
              className={cn(
                "h-auto w-auto px-3 py-1 rounded-full text-[10px] font-bold uppercase tracking-wide border transition-colors group shadow-none",
                STATUS_chip_STYLES
              )}
              data-testid={`status-chip-${filterStatus}`}
              onClick={() => { setFilterStatus("active"); }}
              variant="ghost"
            >
              <StatusIcon className="opacity-60" size={10} />
              {statusLabel}
              <X className="opacity-40 group-hover:opacity-100 transition-opacity" size={10} />
            </Button>
          ) : null}

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
              setFilterStatus("active");
              setFilterDeptIds([]);
              setCustomFieldFilters({});
            }}
            variant="ghost"
          >
            {t("employees.clear_all", "Xóa tất cả")}
          </Button>
        </div>
      ) : null}
    </div>
  );
}

