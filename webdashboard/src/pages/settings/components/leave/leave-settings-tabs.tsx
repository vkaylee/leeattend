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
    <div className="flex bg-slate-100 dark:bg-slate-700/50 p-1.5 rounded-[22px] w-fit border border-slate-200 dark:border-slate-700/50 shadow-inner mb-8">
      {tabs.map((tab) => {
        const Icon = tab.icon;
        return (
          <Button className={cn(
              "flex items-center gap-3 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all duration-300",
              activeTab === tab.id
                ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800"
                : "text-slate-500 dark:text-slate-400 hover:text-slate-800 dark:text-slate-100"
            )}
            data-testid={`leave-tab-${tab.id}`}
            key={tab.id}
            onClick={() => { onTabChange(tab.id); }}
            variant="ghost"
          >
            <Icon className="w-4 h-4" />
            {tab.label}
          </Button>
        );
      })}
    </div>
  );
}
