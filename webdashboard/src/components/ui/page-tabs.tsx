import React, { memo, useCallback } from "react";

import Button from "@/components/ui/button";
import { CONTROL_TEXT_STYLE } from "@/components/ui/control-styles";
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

function getActiveTabClasses(colorScheme: PageTab["colorScheme"]): string {
  const textColor = {
    amber: "text-amber-600 dark:text-amber-300",
    emerald: "text-emerald-600 dark:text-emerald-300",
    rose: "text-rose-600 dark:text-rose-300",
    blue: "text-blue-600 dark:text-blue-300",
    violet: "text-violet-600 dark:text-violet-300",
    primary: "text-primary-600 dark:text-primary-400",
  }[colorScheme ?? "primary"];
  return `bg-white dark:bg-slate-900 ${textColor} shadow-sm border border-slate-100 dark:border-slate-800`;
}

function getNextTabIndex(event: React.KeyboardEvent<HTMLButtonElement>, currentIndex: number, tabCount: number): number {
  if (event.key === "ArrowRight" || event.key === "ArrowDown") return (currentIndex + 1) % tabCount;
  if (event.key === "ArrowLeft" || event.key === "ArrowUp") return (currentIndex - 1 + tabCount) % tabCount;
  if (event.key === "Home") return 0;
  if (event.key === "End") return tabCount - 1;
  return currentIndex;
}

const PageTabButton = memo(function PageTabButton({
  tab,
  isActive,
  onTabChange,
  onKeyDown,
}: {
  tab: PageTab;
  isActive: boolean;
  onTabChange: (tabId: string) => void;
  onKeyDown: (event: React.KeyboardEvent<HTMLButtonElement>) => void;
}) {
  const handleClick = useCallback(() => {
    onTabChange(tab.id);
  }, [onTabChange, tab.id]);

  const activeClasses = isActive
    ? getActiveTabClasses(tab.colorScheme)
    : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 border border-transparent";

  return (
    <Button
      aria-selected={isActive}
      className={cn(
        CONTROL_TEXT_STYLE,
        "flex whitespace-nowrap items-center transition-all duration-200 shadow-none focus-visible:ring-2 focus-visible:ring-primary-500 focus-visible:outline-none",
        activeClasses
      )}
      data-testid={tab.testId}
      data-value={tab.id}
      onClick={handleClick}
      onKeyDown={onKeyDown}
      role="tab"
      size="sm"
      tabIndex={isActive ? 0 : -1}
      type="button"
      variant="ghost"
    >
      {tab.icon}
      <span>{tab.label}</span>
    </Button>
  );
});

export const PageTabs = memo(function PageTabs({ tabs, activeTab, onTabChange, className }: PageTabsProps) {
  const visibleTabs = tabs.filter((tab) => !tab.hidden);

  const handleKeyDown = useCallback(
    (event: React.KeyboardEvent<HTMLButtonElement>) => {
      const tabList = event.currentTarget.parentElement;
      if (!tabList) return;
      const tabButtons = Array.from(tabList.querySelectorAll<HTMLButtonElement>('[role="tab"]:not(:disabled)'));
      const currentIndex = tabButtons.indexOf(event.currentTarget);
      if (currentIndex < 0) return;

      const nextIndex = getNextTabIndex(event, currentIndex, tabButtons.length);
      if (nextIndex === currentIndex) return;

      event.preventDefault();
      const nextButton = tabButtons[nextIndex];
      nextButton.focus();
      const nextTab = nextButton.dataset.value;
      if (nextTab) onTabChange(nextTab);
    },
    [onTabChange]
  );

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
          onKeyDown={handleKeyDown}
          onTabChange={onTabChange}
          tab={tab}
        />
      ))}
    </div>
  );
});

PageTabs.displayName = "PageTabs";
