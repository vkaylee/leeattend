import { useSearch } from "@tanstack/react-router";
import React from "react";
import { useTranslation } from "react-i18next";

import { getStandaloneOrganizationId } from "@/lib/api";

import AssignmentRulesPage from "./assignment-rules-page";
import WorkingPatternsPage from "./working-patterns-page";

export default function ShiftsRulesPage() {
  const { t } = useTranslation();
  const wsId = getStandaloneOrganizationId();

  // Safe search param parsing
  const search = useSearch({ strict: false });
  const activeTab = search.tab ?? "patterns";

  if (!wsId) {
    return (
      <div className="p-12 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">
        {t("common.select_workspace")}
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="animate-in fade-in slide-in-from-bottom-4 duration-500">
        {activeTab === "patterns" ? (
          <WorkingPatternsPage hideHeaderTitle />
        ) : (
          <AssignmentRulesPage hideHeaderTitle />
        )}
      </div>
    </div>
  );
}
