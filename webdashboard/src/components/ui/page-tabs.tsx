import React, { memo, useCallback } from "react";

import Button from "@/components/ui/button";
import { cn } from "@/lib/utils";

export interface PageTab {
  colorScheme?: "amber" | "emerald" | "rose" | "violet" | "blue" | "primary";
  hidden?: boolean;
  icon?: React.ReactNode;
  id: string;
  label: string;
  testId?: string;
}

export interface PageTabsProps {
  activeTab: string;
  className?: string;
  onTabChange: (tabId: string) => void;
  tabs: PageTab[];
}

const PageTabButton = memo(function PageTabButton({
  tab,
  isActive,
  onTabChange,
}: {
  tab: PageTab;
  isActive: boolean;
  onTabChange: (tabId: string) => void;
}) {
  const handleClick = useCallback(() => {
    onTabChange(tab.id);
  }, [onTabChange, tab.id]);

  let activeClasses = "";
  if (isActive) {
    switch (tab.colorScheme) {
      case "amber":
        activeClasses =
          "bg-white dark:bg-slate-900 text-amber-600 dark:text-amber-300 shadow-sm border border-slate-100 dark:border-slate-800";
        break;
      case "emerald":
        activeClasses =
          "bg-white dark:bg-slate-900 text-emerald-600 dark:text-emerald-300 shadow-sm border border-slate-100 dark:border-slate-800";
        break;
      case "rose":
        activeClasses =
          "bg-white dark:bg-slate-900 text-rose-600 dark:text-rose-300 shadow-sm border border-slate-100 dark:border-slate-800";
        break;
      case "blue":
        activeClasses =
          "bg-white dark:bg-slate-900 text-blue-600 dark:text-blue-300 shadow-sm border border-slate-100 dark:border-slate-800";
        break;
      case "violet":
        activeClasses =
          "bg-white dark:bg-slate-900 text-violet-600 dark:text-violet-300 shadow-sm border border-slate-100 dark:border-slate-800";
        break;
      case "primary":
      default:
        activeClasses =
          "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800";
        break;
    }
  } else {
    activeClasses =
      "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 border border-transparent";
  }

  return (
    <Button
      aria-selected={isActive}
      className={cn(
        "flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all duration-200 whitespace-nowrap shadow-none focus-visible:ring-2 focus-visible:ring-primary-500 focus-visible:outline-none",
        activeClasses
      )}
      data-testid={tab.testId}
      onClick={handleClick}
      role="tab"
      type="button"
      variant="ghost"
    >
      {tab.icon}
      <span>{tab.label}</span>
    </Button>
  );
});

export const PageTabs = memo(function PageTabs({ tabs, activeTab, onTabChange, className }: PageTabsProps) {
  // Filter out permission-gated (hidden) tabs
  const visibleTabs = tabs.filter((tab) => !tab.hidden);

  return (
    <div
      className={cn(
        "flex bg-slate-100 dark:bg-slate-700/50 p-1.5 rounded-[22px] w-fit border border-slate-200 dark:border-slate-700/50 shadow-inner overflow-x-auto custom-scrollbar",
        className
      )}
      role="tablist"
    >
      {visibleTabs.map((tab) => (
        <PageTabButton
          isActive={activeTab === tab.id}
          key={tab.id}
          onTabChange={onTabChange}
          tab={tab}
        />
      ))}
    </div>
  );
});

PageTabs.displayName = "PageTabs";
