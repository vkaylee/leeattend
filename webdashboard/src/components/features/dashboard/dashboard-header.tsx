import { Cpu, Settings2, Users } from "lucide-react";

import { type TFunction } from "i18next";

import DashboardViewTabs from "@/components/features/dashboard/dashboard-view-tabs";
import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
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
          <Typography className="text-2xl font-bold tracking-tight text-slate-850 dark:text-slate-100 normal-case mb-0.5" data-testid="dashboard-title" variant="h3">{t("dashboard.title")}</Typography>
          <Typography className="text-xs text-slate-500 dark:text-slate-400 font-medium normal-case tracking-normal" variant="small">{t("dashboard.welcome")}</Typography>
        </div>
        <div className="flex bg-slate-100 dark:bg-slate-700/50 p-1.5 rounded-[22px] w-fit border border-slate-200 dark:border-slate-700/50 shadow-inner">
          <Button
            className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all h-auto shadow-none border border-transparent", !showSystemTab ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border-slate-100 dark:border-slate-800 hover:bg-white dark:hover:bg-slate-900" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 hover:bg-transparent")}
            onClick={() => { setShowSystemTab(false); }}
            type="button"
            variant="ghost"
          >
            <Users size={14} /> {t("dashboard.tabs.hr")}
          </Button>
          <Button
            className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all h-auto shadow-none border border-transparent", showSystemTab ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border-slate-100 dark:border-slate-800 hover:bg-white dark:hover:bg-slate-900" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 hover:bg-transparent")}
            onClick={() => { setShowSystemTab(true); }}
            type="button"
            variant="ghost"
          >
            <Cpu size={14} /> {t("dashboard.tabs.system")}
          </Button>
        </div>
      </div>
      {!showSystemTab ? (
        <div className="flex items-center justify-between gap-4 flex-wrap">
          <DashboardViewTabs activeView={activeView} onViewChange={setActiveView} />
          <Button
            className="flex items-center gap-2 px-5 py-2.5 rounded-2xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-xs font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:hover:text-primary-400 hover:border-primary-200 dark:hover:border-primary-700 transition-all shadow-sm h-auto"
            onClick={() => { setCustomizeOpen(true); }}
            type="button"
            variant="outline"
          >
            <Settings2 size={14} />
            <span className="hidden sm:inline">{t("dashboard.customize.btn")}</span>
          </Button>
        </div>
      ) : null}
    </header>
  );
}
