import { CalendarDays, ShieldCheck, UserPlus } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { cn } from "@/lib/utils";

import type { TabType } from "../../hooks/use-leave-settings";
interface LeaveSettingsTabsProps {
  activeTab: TabType;
  onTabChange: (tab: TabType) => void;
}

export function LeaveSettingsTabs({ activeTab, onTabChange }: LeaveSettingsTabsProps) {
  const { t } = useTranslation();

  const tabs = [
    { icon: CalendarDays, id: "types", label: t("settings.leave.tabs.types") },
    { icon: ShieldCheck, id: "policies", label: t("settings.leave.tabs.policies") },
    { icon: UserPlus, id: "assignments", label: t("settings.leave.tabs.assignments") },
  ] as const;

  return (
    <div className="inline-flex items-center gap-1 rounded-xl bg-slate-100/90 dark:bg-slate-800/80 p-1 border border-slate-200/60 dark:border-slate-700/60 mb-6" role="tablist">
      {tabs.map((tab) => {
        const Icon = tab.icon;
        const isSelected = activeTab === tab.id;
        return (
          <Button
            aria-selected={isSelected}
            className={cn(
              "flex items-center gap-2 px-4 py-1.5 rounded-lg text-xs font-semibold transition-all duration-150 select-none",
              isSelected
                ? "bg-white dark:bg-slate-700 text-slate-900 dark:text-slate-100 shadow-2xs font-bold"
                : "text-slate-500 dark:text-slate-400 hover:text-slate-800 dark:hover:text-slate-200"
            )}
            data-testid={`leave-tab-${tab.id}`}
            key={tab.id}
            onClick={() => { onTabChange(tab.id); }}
            role="tab"
            type="button"
          >
            <Icon className="w-3.5 h-3.5" />
            <span>{tab.label}</span>
          </Button>
        );
      })}
    </div>
  );
}
