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

const TABS = [
  { icon: Clock, key: "recorded" as const, tone: "primary" },
  { icon: AlertTriangle, key: "unknown" as const, tone: "amber" },
  { icon: UserX, key: "unrecorded" as const, tone: "rose" },
] as const;

const ACTIVE_TONE = {
  amber: "bg-amber-50 text-amber-600 border-amber-100 dark:bg-amber-950 dark:text-amber-300 dark:border-amber-800",
  primary: "bg-primary-50 text-primary-600 border-primary-100 dark:bg-primary-950 dark:text-primary-400 dark:border-primary-800",
  rose: "bg-rose-50 text-rose-600 border-rose-100 dark:bg-rose-950 dark:text-rose-300 dark:border-rose-800",
} as const;

export function AttendanceLogsTabs({ activeTab, setActiveTab, stagingCount, unrecordedCount }: AttendanceLogsTabsProps) {
  const { t } = useTranslation();
  const counts = { unknown: stagingCount, unrecorded: unrecordedCount };

  return (
    <div aria-label={t("attendance_logs.title")} className="flex w-fit max-w-full items-center gap-1 overflow-x-auto rounded-2xl border border-slate-200 bg-white p-1 shadow-sm dark:border-slate-700 dark:bg-slate-900" role="tablist">
      {TABS.map(({ icon: Icon, key, tone }) => {
        const isActive = activeTab === key;
        const count = counts[key as keyof typeof counts];
        return (
          <Button
            aria-selected={isActive}
            className={cn(
              "shrink-0 whitespace-nowrap border transition-all",
              isActive ? ACTIVE_TONE[tone] : "border-transparent text-slate-500 hover:bg-slate-50 hover:text-slate-700 dark:text-slate-400 dark:hover:bg-slate-800 dark:hover:text-slate-200",
            )}
            data-value={key}
            key={key}
            onClick={() => { setActiveTab(key); }}
            role="tab"
            size="sm"
            tabIndex={isActive ? 0 : -1}
            variant="ghost"
          >
            <Icon aria-hidden="true" size={14} />
            {t(`attendance_logs.tabs.${key}`)}
            {(count ?? 0) > 0 && (
              <span className={cn("ml-1 rounded-full px-2 py-0.5 text-[9px] font-bold", key === "unknown" ? "bg-amber-100 text-amber-700 dark:bg-amber-950 dark:text-amber-200" : "bg-rose-100 text-rose-700 dark:bg-rose-950 dark:text-rose-200")}>
                {count}
              </span>
            )}
          </Button>
        );
      })}
    </div>
  );
}
