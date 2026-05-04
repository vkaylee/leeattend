import { Users, Cpu, Settings2 } from "lucide-react";

import { type TFunction } from "i18next";

import DashboardViewTabs from "@/components/features/dashboard/dashboard-view-tabs";
import { type DashboardView } from "@/hooks/use-dashboard-layout";
import { cn } from "@/lib/utils";

interface DashboardHeaderProps {
  activeView: DashboardView;
  setActiveView: (view: DashboardView) => void;
  setCustomizeOpen: (open: boolean) => void;
  setShowSystemTab: (show: boolean) => void;
  showSystemTab: boolean;
  t: TFunction;
}

export function DashboardHeader({ t, showSystemTab, setShowSystemTab, activeView, setActiveView, setCustomizeOpen }: DashboardHeaderProps) {
  return (
    <header className="flex flex-col gap-6 px-2">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-6">
        <div className="space-y-1">
          <h1 className="text-5xl font-bold text-slate-700 dark:text-slate-200 tracking-tighter uppercase leading-tight" data-testid="dashboard-title">{t("dashboard.title")}</h1>
          <p className="text-slate-500 dark:text-slate-400 text-lg font-bold uppercase tracking-[0.25em] opacity-60">{t("dashboard.welcome")}</p>
        </div>
        <div className="flex bg-slate-100 dark:bg-slate-700/50 p-1.5 rounded-[22px] w-fit border border-slate-200 dark:border-slate-700/50 shadow-inner">
          <button
            className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all", !showSystemTab ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200")}
            onClick={() => { setShowSystemTab(false); }}
          >
            <Users size={14} /> {t("dashboard.tabs.hr")}
          </button>
          <button
            className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all", showSystemTab ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200")}
            onClick={() => { setShowSystemTab(true); }}
          >
            <Cpu size={14} /> {t("dashboard.tabs.system")}
          </button>
        </div>
      </div>
      {!showSystemTab ? (
        <div className="flex items-center justify-between gap-4 flex-wrap">
          <DashboardViewTabs activeView={activeView} onViewChange={setActiveView} />
          <button
            className="flex items-center gap-2 px-5 py-2.5 rounded-2xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-xs font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:hover:text-primary-400 hover:border-primary-200 dark:hover:border-primary-700 transition-all shadow-sm"
            onClick={() => { setCustomizeOpen(true); }}
          >
            <Settings2 size={14} />
            <span className="hidden sm:inline">{t("dashboard.customize.btn")}</span>
          </button>
        </div>
      ) : null}
    </header>
  );
}
