import { Plus, Share2 } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { Button } from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { AuthorizedAgent } from "@/lib/api";
import { cn } from "@/lib/utils";

interface ConnectivityHeaderProps {
  agents: AuthorizedAgent[];
  includeInactive: boolean;
  onAddAgent: () => void;
  onCollapseAll: () => void;
  onExpandAll: () => void;
  onToggleInactive: () => void;
}

export function ConnectivityHeader({
  agents,
  includeInactive,
  onAddAgent,
  onCollapseAll,
  onExpandAll,
  onToggleInactive,
}: ConnectivityHeaderProps) {
  const { t } = useTranslation();

  return (
    <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 px-2">
      <div className="flex items-center gap-4">
        <div className="w-12 h-12 rounded-[1.25rem] bg-primary-50 dark:bg-primary-950 flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-sm border border-primary-100 dark:border-primary-800/50">
          <Share2 size={24} />
        </div>
        <div>
          <Typography className="text-3xl font-bold tracking-tight text-slate-700 dark:text-slate-200 leading-tight" variant="h1">
            {t("common.settings.categories.connectivity")}
          </Typography>
          <Typography className="text-slate-500 dark:text-slate-400 mt-1 font-medium" variant="p">
            {t("common.settings.categories.connectivity_desc")}
          </Typography>
        </div>
      </div>

      <div className="flex items-center gap-3">
        {agents.length > 0 && (
          <>
            <Button
              className="h-auto px-4 py-2 text-xs text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:hover:text-primary-400 hover:bg-transparent dark:hover:bg-transparent shadow-none"
              data-testid="expand-all-btn"
              onClick={onExpandAll}
              variant="ghost"
            >
              {t("common.expand_all")}
            </Button>
            <Button
              className="h-auto px-4 py-2 text-xs text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:hover:text-primary-400 hover:bg-transparent dark:hover:bg-transparent shadow-none"
              data-testid="collapse-all-btn"
              onClick={onCollapseAll}
              variant="ghost"
            >
              {t("common.collapse_all")}
            </Button>
            <div className="w-px h-6 bg-slate-200 dark:bg-slate-600" />
          </>
        )}
        <Button
          className={cn(
            "h-auto px-6 py-3 rounded-2xl font-bold text-[10px] uppercase tracking-widest border shadow-sm",
            includeInactive
              ? "bg-slate-900 text-white border-transparent hover:bg-slate-800"
              : "bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 border-slate-200 dark:border-slate-700 hover:bg-slate-50 dark:hover:bg-slate-800 hover:text-slate-600 dark:hover:text-slate-300"
          )}
          data-testid="toggle-inactive-btn"
          onClick={onToggleInactive}
          variant={includeInactive ? "secondary" : "outline"}
        >
          {includeInactive ? t("common.status_inactive") : t("common.status_active")}
        </Button>
        <Button
          className="h-auto bg-white dark:bg-slate-900 text-slate-600 dark:text-slate-300 px-6 py-3 rounded-2xl hover:border-primary-200 dark:hover:border-primary-800 border border-slate-200 dark:border-slate-700"
          data-testid="add-agent-btn"
          icon={<Plus size={18} />}
          onClick={onAddAgent}
          variant="outline"
        >
          {t("agents.add_btn")}
        </Button>
      </div>
    </div>
  );
}

