import { useNavigate } from "@tanstack/react-router";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import { PatternHeader } from "./attendance/components/working-patterns/header";
import { PatternModals } from "./attendance/components/working-patterns/modals";
import { PatternGrid } from "./attendance/components/working-patterns/pattern-grid";
import { PatternList } from "./attendance/components/working-patterns/pattern-list";
import { useWorkingPatterns } from "./attendance/hooks/use-working-patterns";

export default function WorkingPatternsPage({ hideHeaderTitle }: { hideHeaderTitle?: boolean }) {
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

  const [viewMode, setViewMode] = useState<"list" | "grid">(() => {
    const saved = localStorage.getItem("working_patterns_view_mode");
    return saved === "grid" ? "grid" : "list";
  });

  const handleViewModeChange = (mode: "list" | "grid") => {
    setViewMode(mode);
    localStorage.setItem("working_patterns_view_mode", mode);
  };

  if (!wsId) {
    return (
      <div className="p-12 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">
        {t("common.select_workspace")}
      </div>
    );
  }

  return (
    <div className="space-y-6 animate-in fade-in duration-700 pb-20">
      <PatternHeader
        hideTitle={hideHeaderTitle}
        onBulkAssignClick={() => {
          setIsBulkModalOpen(true);
        }}
        onCreateClick={() => {
          void navigate({ to: "/shifts/rules/working-patterns/new" });
        }}
        onViewModeChange={handleViewModeChange}
        viewMode={viewMode}
      />

      {viewMode === "list" ? (
        <PatternList
          expandedPatternId={expandedPatternId}
          isLoading={isLoading}
          onDelete={handleDelete}
          onEdit={(pattern) => {
            void navigate({
              to: "/shifts/rules/working-patterns/$patternId/edit",
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
      ) : (
        <PatternGrid
          expandedPatternId={expandedPatternId}
          isLoading={isLoading}
          onDelete={handleDelete}
          onEdit={(pattern) => {
            void navigate({
              to: "/shifts/rules/working-patterns/$patternId/edit",
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
      )}

      <PatternModals hook={hook} />
      <ConfirmationDialog />
    </div>
  );
}