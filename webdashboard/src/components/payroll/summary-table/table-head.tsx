import React from "react";
 
import { cn } from "@/lib/utils";

interface PayrollSummaryHeaderProps {
  isPageSelected: boolean;
  onSelectPage: () => void;
  t: (key: string) => string;
}

function Check({ size, strokeWidth }: { size: number, strokeWidth: number }) {
  return (
    <svg 
      fill="none" 
      height={size} 
      stroke="currentColor" 
      strokeLinecap="round" 
      strokeLinejoin="round" 
      strokeWidth={strokeWidth} 
      viewBox="0 0 24 24" 
      width={size}
    >
      <polyline points="20 6 9 17 4 12" />
    </svg>
  );
}

export function TableHead({ isPageSelected, onSelectPage, t }: PayrollSummaryHeaderProps) {
  return (
    <thead className="bg-slate-50 dark:bg-slate-800/50 sticky top-0 backdrop-blur-md z-10 border-b border-slate-100 dark:border-slate-800">
      <tr>
        <th className="px-10 py-5 w-16">
          <button className={cn("w-6 h-6 rounded-xl border-2 flex items-center justify-center transition-all", isPageSelected ? "bg-primary-600 border-primary-600 text-white shadow-lg shadow-primary-100" : "border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-900 hover:border-primary-300")} onClick={onSelectPage}>
            {isPageSelected ? <Check size={14} strokeWidth={4} /> : null}
          </button>
        </th>
        <th className="px-4 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("employees.table.name")}</th>
        <th className="px-4 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest text-right">{t("common.payroll.table.base_pay")}</th>
        <th className="px-4 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest text-right">{t("common.payroll.table.overtime")}</th>
        <th className="px-4 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest text-right">{t("common.payroll.table.bonus")}</th>
        <th className="px-4 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest text-right">{t("common.payroll.table.net_pay")}</th>
        <th className="px-8 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest text-center">{t("common.payroll.table.status")}</th>
        <th className="px-4 py-5 w-12 text-right pr-8"></th>
      </tr>
    </thead>
  );
}
