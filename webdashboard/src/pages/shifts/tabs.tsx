import { LayoutGrid, List, Zap, CalendarDays } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { TabType, ViewMode } from "./hooks/use-shifts-types";

interface ShiftsTabsProps {
  activeTab: TabType;
  setActiveTab: (tab: TabType) => void;
  setViewMode?: (v: ViewMode) => void;
  viewMode?: ViewMode;
}

export function ShiftsTabs({ activeTab, setActiveTab, viewMode, setViewMode }: ShiftsTabsProps) {
  const { t } = useTranslation();

  return (
    <div className="flex flex-wrap items-center justify-between gap-3">
      <div className="flex items-center gap-1 bg-slate-100 dark:bg-slate-700/50 p-1.5 rounded-[22px] w-fit border border-slate-200 dark:border-slate-700/50">
        <button
          className={`flex items-center gap-2 px-6 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all ${
            activeTab === "weekly-roster"
              ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800"
              : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200"
          }`}
          onClick={() => {
            setActiveTab("weekly-roster");
          }}
          type="button"
        >
          <LayoutGrid size={14} /> {t("shifts.tabs.weekly_roster")}
        </button>
        <button
          className={`flex items-center gap-2 px-6 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all ${
            activeTab === "assignments"
              ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800"
              : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200"
          }`}
          onClick={() => {
            setActiveTab("assignments");
          }}
          type="button"
        >
          <Zap size={14} /> {t("shifts.tabs.fixed_assignments", "Patterns & Rules")}
        </button>
        <button
          className={`flex items-center gap-2 px-6 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all ${
            activeTab === "directory"
              ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800"
              : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200"
          }`}
          onClick={() => {
            setActiveTab("directory");
          }}
          type="button"
        >
          <List size={14} /> {t("shifts.tabs.directory")}
        </button>
      </div>

      {activeTab === "weekly-roster" && !!viewMode && !!setViewMode && (
        <div className="flex items-center gap-1 bg-slate-100 dark:bg-slate-700/50 p-1.5 rounded-[22px] border border-slate-200 dark:border-slate-700/50">
          <button
            className={`flex items-center gap-1.5 px-4 py-2 rounded-[18px] text-[10px] font-bold uppercase tracking-wider transition-all ${
              viewMode === "week"
                ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800"
                : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200"
            }`}
            onClick={() => { setViewMode("week"); }}
            type="button"
          >
            <LayoutGrid size={12} />
            <span>{t("shifts.monthly.view_week", "Tuần")}</span>
          </button>
          <button
            className={`flex items-center gap-1.5 px-4 py-2 rounded-[18px] text-[10px] font-bold uppercase tracking-wider transition-all ${
              viewMode === "month"
                ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800"
                : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200"
            }`}
            onClick={() => { setViewMode("month"); }}
            type="button"
          >
            <CalendarDays size={12} />
            <span>{t("shifts.monthly.view_month", "Tháng")}</span>
          </button>
        </div>
      )}
    </div>
  );
}
