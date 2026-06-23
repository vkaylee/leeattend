import { CalendarDays, Plus, Users, LayoutGrid, List } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import PageHeader from "@/components/ui/page-header";
import { cn } from "@/lib/utils";

interface PatternHeaderProps {
  hideTitle?: boolean;
  onBulkAssignClick: () => void;
  onCreateClick: () => void;
  onViewModeChange: (mode: "list" | "grid") => void;
  viewMode: "list" | "grid";
}

export function PatternHeader({
  onBulkAssignClick,
  onCreateClick,
  viewMode,
  onViewModeChange,
  hideTitle,
}: PatternHeaderProps) {
  const { t } = useTranslation();

  return (
    <PageHeader
      actions={
        <>
          <div className="flex items-center gap-0.5 bg-slate-100 dark:bg-slate-800/60 p-1 rounded-xl mr-2 border border-slate-200/50 dark:border-slate-700/50">
            <Button
              aria-label={t("patterns.view_mode_list")}
              className={cn(
                "p-2 rounded-lg transition-all focus-visible:ring-2 focus-visible:ring-primary-500 focus-visible:outline-none h-auto min-w-0 px-2 py-2 font-normal normal-case active:scale-95",
                viewMode === "list"
                  ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm"
                  : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 hover:bg-slate-200/50 dark:hover:bg-slate-800/50"
              )}
              onClick={() => { onViewModeChange("list"); }}
              size="sm"
              title={t("patterns.view_mode_list")}
              variant="ghost"
            >
              <List size={18} />
            </Button>
            <Button
              aria-label={t("patterns.view_mode_grid")}
              className={cn(
                "p-2 rounded-lg transition-all focus-visible:ring-2 focus-visible:ring-primary-500 focus-visible:outline-none h-auto min-w-0 px-2 py-2 font-normal normal-case active:scale-95",
                viewMode === "grid"
                  ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm"
                  : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 hover:bg-slate-200/50 dark:hover:bg-slate-800/50"
              )}
              onClick={() => { onViewModeChange("grid"); }}
              size="sm"
              title={t("patterns.view_mode_grid")}
              variant="ghost"
            >
              <LayoutGrid size={18} />
            </Button>
          </div>
          <Button
            icon={<Users size={18} />}
            onClick={() => {
              onBulkAssignClick();
            }}
            variant="outline"
          >
            {t("patterns.bulk_assign_btn").toUpperCase()}
          </Button>
          <Button icon={<Plus size={18} />}
            onClick={() => {
              onCreateClick();
            }}
            variant="primary"
          >
            {t("patterns.create_btn").toUpperCase()}
          </Button>
        </>
      }
      data-testid="patterns-title"
      hideTitle={hideTitle}
      icon={<CalendarDays size={24} />}
      subtitle={t("patterns.subtitle")}
      title={t("patterns.title")}
    />
  );
}
