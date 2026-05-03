import { Plus, Share2 } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { AuthorizedAgent } from "@/lib/api";
import { cn } from "@/lib/utils";

interface ConnectivityHeaderProps {
  agents: AuthorizedAgent[];
  includeInactive: boolean;
  onAddAgent: () => void;
  onAddDevice: () => void;
  onCollapseAll: () => void;
  onExpandAll: () => void;
  onToggleInactive: () => void;
}

export function ConnectivityHeader({
  agents,
  includeInactive,
  onAddAgent,
  onAddDevice,
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
          <h1 className="text-3xl font-bold tracking-tight text-slate-700 dark:text-slate-200 leading-tight">
            {t("common.settings.categories.connectivity")}
          </h1>
          <p className="text-slate-500 dark:text-slate-400 mt-1 font-medium">
            {t("common.settings.categories.connectivity_desc")}
          </p>
        </div>
      </div>

      <div className="flex items-center gap-3">
        {agents.length > 0 && (
          <>
            <button
              className="px-4 py-2 text-xs font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:text-primary-400 transition-colors"
              data-testid="expand-all-btn"
              onClick={onExpandAll}
            >
              {t("common.expand_all")}
            </button>
            <button
              className="px-4 py-2 text-xs font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:text-primary-400 transition-colors"
              data-testid="collapse-all-btn"
              onClick={onCollapseAll}
            >
              {t("common.collapse_all")}
            </button>
            <div className="w-px h-6 bg-slate-200 dark:bg-slate-600" />
          </>
        )}
        <button
          className={cn(
            "flex items-center gap-2 px-6 py-3 rounded-2xl font-bold text-[10px] uppercase tracking-widest transition-all border shadow-sm active:scale-95",
            includeInactive
              ? "bg-slate-900 text-white border-transparent"
              : "bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 border-slate-200 dark:border-slate-700 hover:bg-slate-50 dark:hover:bg-slate-800"
          )}
          data-testid="toggle-inactive-btn"
          onClick={onToggleInactive}
        >
          {includeInactive ? t("common.status_inactive") : t("common.status_active")}
        </button>
        <button
          className="flex items-center gap-3 bg-white dark:bg-slate-900 text-slate-600 dark:text-slate-300 px-6 py-3 rounded-2xl font-bold text-xs uppercase tracking-widest hover:border-primary-200 dark:hover:border-primary-800 transition-all border border-slate-200 dark:border-slate-700 active:scale-95"
          data-testid="add-agent-btn"
          onClick={onAddAgent}
        >
          <Plus size={18} /> {t("agents.add_btn")}
        </button>
        <button
          className="flex items-center gap-3 bg-primary-600 text-white px-8 py-3 rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-primary-100 dark:shadow-black/30 active:scale-95"
          data-testid="global-add-device-btn"
          onClick={onAddDevice}
        >
          <Plus size={18} /> {t("devices.add_btn")}
        </button>
      </div>
    </div>
  );
}
