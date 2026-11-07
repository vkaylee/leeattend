import { AlertTriangle, Clock, UserX } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { cn } from "@/lib/utils";

import type { TabType } from "../hooks/use-attendance-logs-page";

interface AttendanceLogsTabsProps {
  activeTab: TabType;
  setActiveTab: (tab: TabType) => void;
  stagingCount?: number;
  unrecordedCount?: number;
}

export function AttendanceLogsTabs({ activeTab, setActiveTab, stagingCount, unrecordedCount }: AttendanceLogsTabsProps) {
  const { t } = useTranslation();

  return (
    <div className="bg-white dark:bg-slate-900 rounded-2xl border border-slate-200 dark:border-slate-700 shadow-sm p-1 flex items-center gap-1 w-fit">
      <Button className={cn(
          "px-6 py-2.5 rounded-xl text-[10px] font-bold uppercase tracking-widest transition-all flex items-center gap-2",
          activeTab === "recorded"
            ? "bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-400 shadow-sm border border-primary-100 dark:border-primary-800"
            : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 hover:bg-slate-50 dark:hover:bg-slate-800"
        )}
        onClick={() => { setActiveTab("recorded"); }}
        type="button"
        variant="ghost"
      >
        <Clock size={14} />
        {t("attendance_logs.tabs.recorded")}
      </Button>
      <Button className={cn(
          activeTab === "unknown"
            ? "bg-amber-50 dark:bg-amber-950 text-amber-600 dark:text-amber-300 shadow-sm border border-amber-100 dark:border-amber-800"
            : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 hover:bg-slate-50 dark:hover:bg-slate-800"
        )}
        onClick={() => { setActiveTab("unknown"); }}
        type="button"
        variant="ghost"
      >
        <AlertTriangle size={14} />
        {t("attendance_logs.tabs.unknown")}
        {(stagingCount ?? 0) > 0 && (
          <span className="ml-1 px-2 py-0.5 bg-amber-100 dark:bg-amber-950 text-amber-700 dark:text-amber-200 rounded-full text-[9px] font-bold">
            {stagingCount}
          </span>
        )}
      </Button>
      <Button className={cn(
          "px-6 py-2.5 rounded-xl text-[10px] font-bold uppercase tracking-widest transition-all flex items-center gap-2",
          activeTab === "unrecorded"
            ? "bg-rose-50 dark:bg-rose-950 text-rose-600 dark:text-rose-300 shadow-sm border border-rose-100 dark:border-rose-800"
            : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 hover:bg-slate-50 dark:hover:bg-slate-800"
        )}
        onClick={() => { setActiveTab("unrecorded"); }}
        type="button"
        variant="ghost"
      >
        <UserX size={14} />
        {t("attendance_logs.tabs.unrecorded")}
        {(unrecordedCount ?? 0) > 0 && (
          <span className="ml-1 px-2 py-0.5 bg-rose-100 dark:bg-rose-950 text-rose-700 dark:text-rose-200 rounded-full text-[9px] font-bold">
            {unrecordedCount}
          </span>
        )}
      </Button>
    </div>
  );
}

