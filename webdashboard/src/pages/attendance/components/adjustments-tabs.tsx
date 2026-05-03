import { CheckCircle, Clock, XCircle } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";
import { cn } from "@/lib/utils";
import type { TabType } from "../hooks/use-attendance-adjustments-page";

interface AdjustmentsTabsProps {
  activeTab: TabType;
  setActiveTab: (tab: TabType) => void;
}

export function AdjustmentsTabs({ activeTab, setActiveTab }: AdjustmentsTabsProps) {
  const { t } = useTranslation();

  return (
    <div className="flex bg-slate-100 dark:bg-slate-700/50 p-1.5 rounded-[22px] w-fit border border-slate-200 dark:border-slate-700/50 shadow-inner">
      <button
        className={cn(
          "flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all",
          activeTab === "pending"
            ? "bg-white dark:bg-slate-900 text-amber-600 dark:text-amber-300 shadow-sm border border-slate-100 dark:border-slate-800"
            : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200"
        )}
        onClick={() => setActiveTab("pending")}
        type="button"
      >
        <Clock size={14} /> {t("common.status_pending")}
      </button>
      <button
        className={cn(
          "flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all",
          activeTab === "approved"
            ? "bg-white dark:bg-slate-900 text-emerald-600 dark:text-emerald-300 shadow-sm border border-slate-100 dark:border-slate-800"
            : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200"
        )}
        onClick={() => setActiveTab("approved")}
        type="button"
      >
        <CheckCircle size={14} /> {t("common.status_approved")}
      </button>
      <button
        className={cn(
          "flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all",
          activeTab === "rejected"
            ? "bg-white dark:bg-slate-900 text-rose-600 dark:text-rose-300 shadow-sm border border-slate-100 dark:border-slate-800"
            : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200"
        )}
        onClick={() => setActiveTab("rejected")}
        type="button"
      >
        <XCircle size={14} /> {t("common.status_rejected")}
      </button>
    </div>
  );
}
