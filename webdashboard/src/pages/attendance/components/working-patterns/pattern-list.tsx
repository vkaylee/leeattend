import { CalendarDays } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import EmptyState from "@/components/ui/empty-state";
import { Shift, WorkingPattern, WorkingPatternDetail } from "@/lib/api/types";

import { PatternRowCard } from "./pattern-row-card";

interface PatternListProps {
  expandedPatternId: string | null;
  isLoading: boolean;
  onDelete: (pattern: WorkingPattern) => void;
  onEdit: (pattern: WorkingPattern) => void;
  onToggleExpand: (id: string) => void;
  patternDetails?: WorkingPatternDetail[];
  patterns?: WorkingPattern[];
  shifts?: Shift[];
}

export function PatternList({
  expandedPatternId,
  isLoading,
  onDelete,
  onEdit,
  onToggleExpand,
  patternDetails,
  patterns,
  shifts,
}: PatternListProps) {
  const { t } = useTranslation();

  if (isLoading) {
    return (
      <div className="flex flex-col gap-6">
        {Array.from({ length: 3 }).map((_, index) => (
          <div
            className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-8 rounded-[2rem] shadow-sm animate-pulse h-28 w-full"
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
    <div className="flex flex-col gap-6 animate-in fade-in duration-500">
      {patterns.map((pattern) => (
        <PatternRowCard
          expandedPatternId={expandedPatternId}
          key={pattern.id}
          onDelete={onDelete}
          onEdit={onEdit}
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
