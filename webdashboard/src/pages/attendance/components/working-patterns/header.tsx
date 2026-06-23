import { CalendarDays, Plus, Users, LayoutGrid, List } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import PageHeader from "@/components/ui/page-header";
import { cn } from "@/lib/utils";

interface PatternHeaderProps {
  onBulkAssignClick: () => void;
  onCreateClick: () => void;
  viewMode: "list" | "grid";
  onViewModeChange: (mode: "list" | "grid") => void;
}

export function PatternHeader({
  onBulkAssignClick,
  onCreateClick,
  viewMode,
  onViewModeChange,
}: PatternHeaderProps) {
  const { t } = useTranslation();

  return (
    <PageHeader
      actions={
        <>
          <div className="flex items-center gap-0.5 bg-slate-100 dark:bg-slate-800/60 p-1 rounded-xl mr-2 border border-slate-200/50 dark:border-slate-700/50">
            <button
              aria-label={t("patterns.view_mode_list")}
              className={cn(
                "p-2 rounded-lg transition-all focus-visible:ring-2 focus-visible:ring-primary-500 focus-visible:outline-none",
                viewMode === "list"
                  ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm"
                  : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200"
              )}
              onClick={() => onViewModeChange("list")}
              title={t("patterns.view_mode_list")}
              type="button"
            >
              <List size={18} />
            </button>
            <button
              aria-label={t("patterns.view_mode_grid")}
              className={cn(
                "p-2 rounded-lg transition-all focus-visible:ring-2 focus-visible:ring-primary-500 focus-visible:outline-none",
                viewMode === "grid"
                  ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm"
                  : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200"
              )}
              onClick={() => onViewModeChange("grid")}
              title={t("patterns.view_mode_grid")}
              type="button"
            >
              <LayoutGrid size={18} />
            </button>
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
      icon={<CalendarDays size={24} />}
      subtitle={t("patterns.subtitle")}
      title={t("patterns.title")}
    />
  );
}
