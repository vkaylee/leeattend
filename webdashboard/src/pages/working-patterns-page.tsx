import { useNavigate } from "@tanstack/react-router";
import React from "react";
import { useTranslation } from "react-i18next";

import { PatternHeader } from "./attendance/components/working-patterns/header";
import { PatternModals } from "./attendance/components/working-patterns/modals";
import { PatternGrid } from "./attendance/components/working-patterns/pattern-grid";
import { useWorkingPatterns } from "./attendance/hooks/use-working-patterns";

export default function WorkingPatternsPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const hook = useWorkingPatterns();
  const {
    ConfirmationDialog,
    expandedPatternId,
    handleDelete,
    isLoading,
    patternDetails,
    patterns,
    setExpandedPatternId,
    setIsBulkModalOpen,
    shifts,
    wsId,
  } = hook;

  if (!wsId) {
    return (
      <div className="p-12 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">
        {t("common.select_workspace")}
      </div>
    );
  }

  return (
    <div className="space-y-10 animate-in fade-in duration-700 pb-20">
      <PatternHeader
        onBulkAssignClick={() => {
          setIsBulkModalOpen(true);
        }}
        onCreateClick={() => {
          void navigate({ to: "/working-patterns/new" });
        }}
      />

      <PatternGrid
        expandedPatternId={expandedPatternId}
        isLoading={isLoading}
        onDelete={handleDelete}
        onEdit={(pattern) => {
          void navigate({
            to: "/working-patterns/$patternId/edit",
            params: { patternId: pattern.id },
          });
        }}
        onToggleExpand={(id) => {
          setExpandedPatternId(expandedPatternId === id ? null : id);
        }}
        patternDetails={patternDetails}
        patterns={patterns}
        shifts={shifts}
      />

      <PatternModals hook={hook} />
      <ConfirmationDialog />
    </div>
  );
}