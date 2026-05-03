import { LayoutGrid, List, Zap } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { TabType } from "./hooks/use-shifts";

interface ShiftsTabsProps {
  activeTab: TabType;
  setActiveTab: (tab: TabType) => void;
}

export function ShiftsTabs({ activeTab, setActiveTab }: ShiftsTabsProps) {
  const { t } = useTranslation();

  return (
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
  );
}
