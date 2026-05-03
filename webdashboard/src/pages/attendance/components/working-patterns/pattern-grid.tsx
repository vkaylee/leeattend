import { CalendarDays } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import EmptyState from "@/components/ui/empty-state";
import { Shift, WorkingPattern, WorkingPatternDetail } from "@/lib/api/types";

import { PatternCard } from "./pattern-card";

interface PatternGridProps {
  expandedPatternId: string | null;
  isLoading: boolean;
  onDelete: (pattern: WorkingPattern) => void;
  onToggleExpand: (id: string) => void;
  patternDetails?: WorkingPatternDetail[];
  patterns?: WorkingPattern[];
  shifts?: Shift[];
}

export function PatternGrid({
  expandedPatternId,
  isLoading,
  onDelete,
  onToggleExpand,
  patternDetails,
  patterns,
  shifts,
}: PatternGridProps) {
  const { t } = useTranslation();

  if (isLoading) {
    return (
      <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-8">
        {Array.from({ length: 3 }).map((_, index) => (
          <div
            className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-10 rounded-[2.5rem] shadow-sm animate-pulse h-80"
            key={index}
          />
        ))}
      </div>
    );
  }

  if (!patterns || patterns.length === 0) {
    return (
      <EmptyState
        description={t("patterns.empty_desc")}
        icon={<CalendarDays size={40} />}
        title={t("patterns.empty_title")}
      />
    );
  }

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-8">
      {patterns.map((pattern) => (
        <PatternCard
          expandedPatternId={expandedPatternId}
          key={pattern.id}
          onDelete={onDelete}
          onToggleExpand={onToggleExpand}
          pattern={pattern}
          patternDetails={
            expandedPatternId === pattern.id ? patternDetails : undefined
          }
          shifts={shifts}
        />
      ))}
    </div>
  );
}
