import { Server } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { Typography } from "@/components/ui/typography";

interface EmptyAgentsStateProps {
  onAddAgent: () => void;
}

export function EmptyAgentsState({ onAddAgent }: EmptyAgentsStateProps) {
  const { t } = useTranslation();

  return (
    <div className="py-32 text-center border-2 border-dashed border-slate-100 dark:border-slate-800 rounded-[3rem] bg-slate-50 dark:bg-slate-800/30 flex flex-col items-center animate-in zoom-in-95 duration-500" data-testid="empty-agents-state">
      <div className="w-24 h-24 bg-white dark:bg-slate-900 rounded-full flex items-center justify-center shadow-sm mb-6 border border-slate-100 dark:border-slate-800">
        <Server className="text-slate-200 dark:text-slate-700" size={32} />
      </div>
      <Typography className="text-slate-500 dark:text-slate-400 font-bold uppercase tracking-widest text-[10px] opacity-60 mb-2" variant="p">
        {t("agents.table.no_agents")}
      </Typography>
      <Typography className="text-slate-400 dark:text-slate-500 text-xs max-w-md mb-8" variant="p">
        {t("connectivity.empty_guide")}
      </Typography>
      <button
        className="px-10 py-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-2xl font-bold text-[10px] uppercase tracking-[0.2em] text-slate-600 dark:text-slate-300 hover:border-primary-200 dark:border-primary-800 hover:text-primary-600 dark:text-primary-400 transition-all shadow-sm active:scale-95"
        data-testid="empty-add-agent-btn"
        onClick={onAddAgent}
      >
        {t("agents.add_btn")}
      </button>
    </div>
  );
}
