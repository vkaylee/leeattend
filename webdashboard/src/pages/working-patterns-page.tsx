import { useNavigate } from "@tanstack/react-router";
import React, { useState } from "react";

import { PageContainer } from "@/components/ui/page-container";

import { PatternHeader } from "./attendance/components/working-patterns/header";
import { PatternModals } from "./attendance/components/working-patterns/modals";
import { PatternGrid } from "./attendance/components/working-patterns/pattern-grid";
import { PatternList } from "./attendance/components/working-patterns/pattern-list";
import { useWorkingPatterns } from "./attendance/hooks/use-working-patterns";

export default function WorkingPatternsPage({ hideHeaderTitle }: { hideHeaderTitle?: boolean }) {
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
  } = hook;

  const [viewMode, setViewMode] = useState<"list" | "grid">(() => {
    const saved = localStorage.getItem("working_patterns_view_mode");
    return saved === "grid" ? "grid" : "list";
  });

  const handleViewModeChange = (mode: "list" | "grid") => {
    setViewMode(mode);
    localStorage.setItem("working_patterns_view_mode", mode);
  };

  

  return (
    <PageContainer>
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
    </PageContainer>
  );
}