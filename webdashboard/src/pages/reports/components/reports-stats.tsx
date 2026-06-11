import { Clock, FileBarChart, Users2 } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

interface ReportsStatsProps {
  quickStats: {
    stdHours: string;
    otHours: string;
    totalEmployees: number;
  };
}

export function ReportsStats({ quickStats }: ReportsStatsProps) {
  const { t } = useTranslation();

  return (
    <>
      <div className="flex items-center gap-2 bg-slate-900 dark:bg-slate-800 px-5 py-3.5 rounded-[1.5rem] border border-slate-800 dark:border-slate-700 shadow-md">
        <FileBarChart className="text-primary-400" size={14} />
        <span className="text-[10px] font-bold text-slate-400 uppercase tracking-widest">
          {t("reports.quick_stats.std_hours")}
        </span>
        <span className="font-mono font-bold text-white text-sm tracking-tighter">{quickStats.stdHours}h</span>
      </div>
      <div className="flex items-center gap-2 bg-gradient-to-r from-orange-900 to-orange-950 px-5 py-3.5 rounded-[1.5rem] border border-orange-800 shadow-md">
        <Clock className="text-orange-400" size={14} />
        <span className="text-[10px] font-bold text-orange-400/70 uppercase tracking-widest">
          {t("reports.quick_stats.ot_hours")}
        </span>
        <span className="font-mono font-bold text-orange-100 text-sm tracking-tighter">{quickStats.otHours}h</span>
      </div>
      <div className="flex items-center gap-2 bg-white dark:bg-slate-900 px-5 py-3.5 rounded-[1.5rem] border border-slate-200 dark:border-slate-700 shadow-md">
        <Users2 className="text-primary-600 dark:text-primary-400" size={14} />
        <span className="text-[10px] font-bold text-slate-400 uppercase tracking-widest">{t("common.employees")}</span>
        <span className="font-mono font-bold text-slate-700 dark:text-slate-200 text-sm tracking-tighter">
          {quickStats.totalEmployees}
        </span>
      </div>
    </>
  );
}
