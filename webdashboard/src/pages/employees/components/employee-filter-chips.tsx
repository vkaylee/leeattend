import { Building2, X } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";

interface EmployeeFilterChipsProps {
  departments: { id: string; name: string }[] | undefined;
  filterDeptIds: string[];
  setFilterDeptIds: (ids: string[]) => void;
  toggleDeptFilter: (id: string) => void;
}

export function EmployeeFilterChips({
  filterDeptIds,
  departments,
  toggleDeptFilter,
  setFilterDeptIds,
}: EmployeeFilterChipsProps) {
  const { t } = useTranslation();

  if (filterDeptIds.length === 0 || !departments) return null;

  return (
    <div className="flex items-center gap-2 flex-wrap animate-in slide-in-from-top-4 duration-300" data-testid="active-filter-chips">
      <span className="text-[9px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-[0.2em] mr-1">
        {t("employees.filtering_by", "Đang lọc")}
      </span>
      {filterDeptIds.map((id) => {
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
      <Button
        className="h-auto w-auto p-0 text-[10px] font-bold text-slate-400 dark:text-slate-500 hover:text-red-500 dark:hover:text-red-400 uppercase tracking-wide transition-colors ml-1 border-none shadow-none hover:bg-transparent"
        onClick={() => { setFilterDeptIds([]); }}
        variant="ghost"
      >
        {t("employees.clear_all", "Xóa tất cả")}
      </Button>
    </div>
  );
}

