import * as Popover from "@radix-ui/react-popover";
import { Building2, Check, ListFilter, X } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { cn } from "@/lib/utils";

import type { Employee, Department } from "../../../lib/api/types";

interface EmployeeNameCellProps {
  employee: Employee;
  isSelected: boolean;
}

export const EmployeeNameCell = ({ employee, isSelected }: EmployeeNameCellProps) => (
  <div className="flex items-center gap-5">
    <div className={cn(
      "w-12 h-12 rounded-2xl flex items-center justify-center font-bold text-sm shadow-inner border transition-all uppercase",
      isSelected 
        ? "bg-primary-600 text-white border-primary-500" 
        : "bg-slate-100 dark:bg-slate-700 text-slate-500 dark:text-slate-400 border-slate-100 dark:border-slate-800 group-hover:bg-white dark:hover:bg-slate-900 group-hover:text-primary-600 dark:group-hover:text-primary-400 group-hover:border-primary-100"
    )}>
      {employee.full_name[0]}
    </div>
    <div>
      <div className="font-bold text-slate-700 dark:text-slate-200 text-sm tracking-tight uppercase leading-none">{employee.full_name}</div>
      <div className="font-mono font-bold text-[10px] text-slate-400 dark:text-slate-500 uppercase tracking-tighter mt-1.5">{employee.employee_id_number}</div>
    </div>
  </div>
);

interface DepartmentFilterHeaderProps {
  departments: Department[] | undefined;
  filterDeptIds: string[];
  setFilterDeptIds: (ids: string[]) => void;
  toggleDeptFilter: (id: string) => void;
}

export const DepartmentFilterHeader = ({ 
  departments, 
  filterDeptIds, 
  setFilterDeptIds, 
  toggleDeptFilter 
}: DepartmentFilterHeaderProps) => {
  const { t } = useTranslation();
  
  return (
    <Popover.Root>
      <Popover.Trigger asChild>
        <Button
          className={cn(
            "relative h-5 w-5 rounded transition-colors p-0 border-none shadow-none hover:bg-transparent",
            filterDeptIds.length > 0
              ? "text-primary-600 dark:text-primary-400"
              : "text-slate-400 dark:text-slate-500 opacity-60 hover:opacity-100 hover:text-primary-600"
          )}
          data-testid="dept-filter-btn"
          onClick={(e) => { e.stopPropagation(); }}
          title={t("employees.filter_department", "Filter by department")}
          variant="ghost"
        >
          <ListFilter size={12} />
          {filterDeptIds.length > 0 && (
            <span className="absolute -top-1 -right-1 min-w-[14px] h-[14px] px-0.5 flex items-center justify-center bg-primary-500 text-white text-[8px] font-bold rounded-full leading-none animate-in zoom-in">
              {filterDeptIds.length}
            </span>
          )}
        </Button>
      </Popover.Trigger>
      <Popover.Portal>
        <Popover.Content
          align="start"
          className="z-[10000] w-56 overflow-hidden rounded-2xl border border-slate-100 dark:border-slate-700 bg-white dark:bg-slate-900 p-2 shadow-2xl dark:shadow-black/30 animate-in fade-in zoom-in-95 duration-200"
          onClick={(e) => { e.stopPropagation(); }}
          sideOffset={8}
        >
          <div className="px-3 pt-2 pb-1.5">
            <div className="text-[9px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-[0.2em]">
              {t("employees.filter_department", "Filter by department")}
            </div>
          </div>
          <div className="max-h-60 overflow-y-auto custom-scrollbar">
            {filterDeptIds.length > 0 && (
              <Button
                className="h-auto w-full px-3 py-2 flex items-center gap-3 rounded-xl text-[11px] font-bold uppercase tracking-wide text-red-500 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-950/30 transition-colors mb-1 border-none shadow-none justify-start"
                onClick={() => { setFilterDeptIds([]); }}
                variant="ghost"
              >
                <X className="shrink-0" size={12} />
                {t("employees.clear_filter", "Xóa bộ lọc")}
              </Button>
            )}

            {departments && departments.length > 0 ? (
              departments.map(d => {
                const isChecked = filterDeptIds.includes(d.id);
                return (
                  <Button
                    className={cn(
                      "h-auto w-full px-3 py-2.5 flex items-center gap-3 rounded-xl text-[11px] font-bold uppercase tracking-wide transition-colors border-none shadow-none justify-start",
                      isChecked
                        ? "bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-400 hover:bg-primary-100 dark:hover:bg-primary-900"
                        : "text-slate-600 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800"
                    )}
                    data-testid={`dept-filter-option-${d.id}`}
                    key={d.id}
                    onClick={() => { toggleDeptFilter(d.id); }}
                    variant="ghost"
                  >
                    <div className={cn("w-4 h-4 rounded border flex items-center justify-center shrink-0 transition-colors",
                      isChecked
                        ? "bg-primary-600 border-primary-600 text-white"
                        : "border-slate-300 dark:border-slate-600"
                    )}>
                      {isChecked ? <Check size={10} strokeWidth={3} /> : null}
                    </div>
                    <Building2 className="opacity-50 shrink-0" size={12} />
                    <span className="truncate">{d.name}</span>
                  </Button>
                );
              })
            ) : (
              <div className="px-3 py-6 text-center text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest opacity-60">
                {t("employees.no_departments", "Chưa có phòng ban")}
              </div>
            )}
          </div>
        </Popover.Content>
      </Popover.Portal>
    </Popover.Root>
  );
};

