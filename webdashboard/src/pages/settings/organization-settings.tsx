import { Building2, Layers, Briefcase } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import EmploymentPhaseManager from "@/components/people/employment-phase-manager";
import { Typography } from "@/components/ui/typography";
import { getStandaloneOrganizationId } from "@/lib/api";
import { cn } from "@/lib/utils";

import EmployersPage from "./employers-page";

type TabType = "employers" | "employment_phases";

export default function OrganizationSettings() {
  const { t } = useTranslation();
  const wsId = getStandaloneOrganizationId();
  const [activeTab, setActiveTab] = useState<TabType>("employers");

  return (
    <div className="space-y-6 animate-in fade-in duration-300">
      {/* Apple HIG Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 pb-2 border-b border-slate-200/60 dark:border-slate-800/60">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-primary-50 dark:bg-primary-950/70 flex items-center justify-center text-primary-600 dark:text-primary-400 border border-primary-100/50 dark:border-primary-900/50 shadow-2xs">
            <Building2 size={20} />
          </div>
          <div>
            <Typography className="text-xl font-bold tracking-tight text-slate-800 dark:text-slate-100" data-testid="organization-settings-title" variant="h1">
              {t("common.settings.categories.organization", { defaultValue: "Pháp nhân & Mốc công tác" })}
            </Typography>
            <Typography className="text-xs text-slate-500 dark:text-slate-400 font-medium" variant="small">
              {t("common.settings.categories.organization_desc", { defaultValue: "Quản lý pháp nhân và lộ trình công tác" })}
            </Typography>
          </div>
        </div>
      </div>

      {/* Apple HIG Segmented Control */}
      <div className="flex items-center">
        <div className="inline-flex items-center gap-1 rounded-xl bg-slate-100/90 dark:bg-slate-800/80 p-1 border border-slate-200/60 dark:border-slate-700/60" role="tablist">
          <button
            aria-selected={activeTab === "employers"}
            className={cn(
              "flex items-center gap-2 px-4 py-1.5 rounded-lg text-xs font-semibold transition-all duration-150 select-none",
              activeTab === "employers"
                ? "bg-white dark:bg-slate-700 text-slate-900 dark:text-slate-100 shadow-2xs font-bold"
                : "text-slate-500 dark:text-slate-400 hover:text-slate-800 dark:hover:text-slate-200"
            )}
            data-testid="tab-employers"
            onClick={() => setActiveTab("employers")}
            role="tab"
            type="button"
          >
            <Briefcase size={14} />
            <span>Pháp nhân</span>
          </button>
          <button
            aria-selected={activeTab === "employment_phases"}
            className={cn(
              "flex items-center gap-2 px-4 py-1.5 rounded-lg text-xs font-semibold transition-all duration-150 select-none",
              activeTab === "employment_phases"
                ? "bg-white dark:bg-slate-700 text-slate-900 dark:text-slate-100 shadow-2xs font-bold"
                : "text-slate-500 dark:text-slate-400 hover:text-slate-800 dark:hover:text-slate-200"
            )}
            data-testid="tab-phases"
            onClick={() => setActiveTab("employment_phases")}
            role="tab"
            type="button"
          >
            <Layers size={14} />
            <span>Lộ trình & Mốc</span>
          </button>
        </div>
      </div>

      <div className="animate-in fade-in duration-300">


        {activeTab === "employers" ? (
          <div className="bg-white dark:bg-slate-900/90 rounded-2xl border border-slate-200/80 dark:border-slate-800/80 overflow-hidden shadow-2xs p-6">
            <EmployersPage />
          </div>
        ) : null}
        {activeTab === "employment_phases" ? (
          <div className="bg-white dark:bg-slate-900/90 rounded-2xl border border-slate-200/80 dark:border-slate-800/80 overflow-hidden shadow-2xs p-6">
            <EmploymentPhaseManager wsId={wsId} />
          </div>
        ) : null}
      </div>
    </div>
  );
}
