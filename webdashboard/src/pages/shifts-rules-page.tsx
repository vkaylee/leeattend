import { useNavigate, useSearch } from "@tanstack/react-router";
import { CalendarRange, Zap } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import PageHeader from "@/components/ui/page-header";
import { getCurrentWorkspaceId } from "@/lib/api";

import AssignmentRulesPage from "./assignment-rules-page";
import WorkingPatternsPage from "./working-patterns-page";

export default function ShiftsRulesPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const wsId = getCurrentWorkspaceId();
  
  // Safe search param parsing
  const search = useSearch({ strict: false });
  const activeTab = search.tab ?? "patterns";

  const handleTabChange = (tab: "patterns" | "rules") => {
    void navigate({
      to: "/shifts/rules",
      search: { tab },
    });
  };

  if (!wsId) {
    return (
      <div className="p-12 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">
        {t("common.select_workspace")}
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Main Unified Header */}
      <PageHeader
        data-testid="shifts-rules-header"
        subtitle={t("shifts.rules.subtitle", "Manage recurring schedule templates and automated shift routing rules")}
        title={t("common.sections.shifts_rules", "Automation Policy")}
      />

      {/* Pill Tab Switcher */}
      <div className="flex items-center gap-1 bg-slate-100 dark:bg-slate-700/50 p-1.5 rounded-[22px] w-fit border border-slate-200 dark:border-slate-700/50">
        <Button
          className={`flex items-center gap-2 px-6 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all ${
            activeTab === "patterns"
              ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800"
              : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200"
          }`}
          onClick={() => {
            handleTabChange("patterns");
          }}
          type="button"
          variant="ghost"
        >
          <CalendarRange size={14} /> {t("patterns.title", "Working Patterns")}
        </Button>
        <Button
          className={`flex items-center gap-2 px-6 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all ${
            activeTab === "rules"
              ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800"
              : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200"
          }`}
          onClick={() => {
            handleTabChange("rules");
          }}
          type="button"
          variant="ghost"
        >
          <Zap size={14} /> {t("rules.title", "Assignment Rules")}
        </Button>
      </div>

      {/* Tab Contents */}
      <div className="animate-in fade-in slide-in-from-bottom-4 duration-500">
        {activeTab === "patterns" ? (
          <WorkingPatternsPage hideHeaderTitle={true} />
        ) : (
          <AssignmentRulesPage hideHeaderTitle={true} />
        )}
      </div>
    </div>
  );
}
