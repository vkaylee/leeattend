import { useQuery } from "@tanstack/react-query";
import { CalendarDays, Edit, Trash2 } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { PatternMatrixPreview } from "@/components/ui/pattern-matrix-preview";
import StatusBadge from "@/components/ui/status-badge";
import { Typography } from "@/components/ui/typography";
import { fetchPatternDetails } from "@/lib/api";
import { Shift, WorkingPattern, WorkingPatternDetail } from "@/lib/api/types";
import { createQueryKey } from "@/lib/query-keys";


interface PatternRowCardProps {
  expandedPatternId: string | null;
  onDelete: (pattern: WorkingPattern) => void;
  onEdit: (pattern: WorkingPattern) => void;
  onToggleExpand: (id: string) => void;
  pattern: WorkingPattern;
  patternDetails?: WorkingPatternDetail[];
  shifts?: Shift[];
}


export function PatternRowCard({
  onDelete,
  onEdit,
  pattern,
  shifts,
}: PatternRowCardProps) {
  const { t } = useTranslation();


  const { data: details, isLoading: isLoadingDetails } = useQuery<WorkingPatternDetail[]>({
    queryKey: createQueryKey("pattern-details", pattern.id),
    queryFn: () => fetchPatternDetails(pattern.id),
    enabled: !!pattern.id,
  });

  const isRotation = pattern.pattern_type === "rotation";
  const cycleLength = pattern.cycle_length_days ?? 7;

  return (
    <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[2rem] p-6 shadow-sm hover:shadow-md hover:border-slate-200 dark:hover:border-slate-700 transition-all flex flex-col gap-4 group relative overflow-hidden">
      {/* Row 1: Info, Description & Actions */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 w-full">
        {/* Left: Icon, Title & Badges */}
        <div className="flex items-center gap-4 min-w-0 max-w-md">
          <div className="w-12 h-12 bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-400 rounded-2xl flex items-center justify-center border border-primary-100 dark:border-primary-800/50 group-hover:scale-110 transition-all duration-300 flex-shrink-0">
            <CalendarDays size={22} />
          </div>
          <div className="min-w-0">
            <Typography className="font-bold text-base text-slate-700 dark:text-slate-200 leading-tight uppercase tracking-tight truncate" variant="h4">
              {pattern.name}
            </Typography>
            <div className="flex flex-wrap items-center gap-1.5 mt-1.5">
              <StatusBadge variant={pattern.is_active ? "success" : "neutral"}>
                {pattern.is_active ? t("status_active_caps") : t("status_off")}
              </StatusBadge>
              <StatusBadge variant="primary">
                {pattern.pattern_type === "rotation"
                  ? (t("patterns.badge_rotation", { days: pattern.cycle_length_days }) || `Rotation (${pattern.cycle_length_days}d)`)
                  : t("patterns.badge_weekly") || "Weekly"}
              </StatusBadge>
            </div>
          </div>
        </div>

        {/* Middle: Description */}
        <div className="flex-1 text-slate-500 dark:text-slate-400 text-xs font-semibold line-clamp-1 italic opacity-60 leading-relaxed max-w-xs md:max-w-md lg:max-w-lg hidden sm:block">
          {pattern.description ?? t("common.no_description")}
        </div>

        {/* Right: Actions */}
        <div className="flex items-center gap-2 flex-shrink-0 justify-end">
          <Button
            className="p-2.5 bg-slate-50 hover:bg-slate-100 dark:bg-slate-800/50 dark:hover:bg-slate-800 text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:hover:text-primary-400 rounded-xl border border-slate-100/50 dark:border-slate-800/50 transition-all focus-visible:ring-2 focus-visible:ring-primary-500 focus-visible:outline-none"
            onClick={() => { onEdit(pattern); }}
            title={t("common.edit") || "Edit"}
            type="button"
            variant="ghost"
          >
            <Edit size={16} />
          </Button>
          <Button
            className="p-2.5 bg-slate-50 hover:bg-slate-100 dark:bg-slate-800/50 dark:hover:bg-slate-800 text-slate-500 dark:text-slate-400 hover:text-rose-650 dark:hover:text-rose-400 rounded-xl border border-slate-100/50 dark:border-slate-800/50 transition-all focus-visible:ring-2 focus-visible:ring-primary-500 focus-visible:outline-none"
            onClick={() => { onDelete(pattern); }}
            title={t("common.delete")}
            type="button"
            variant="ghost"
          >
            <Trash2 size={16} />
          </Button>
        </div>
      </div>

      {/* Divider */}
      <div className="border-t border-slate-100 dark:border-slate-800/60 w-full" />

      {/* Row 2: Visual Matrix Preview */}
      <div className="w-full">
        <PatternMatrixPreview
          cycleLength={cycleLength}
          details={details}
          isLoading={isLoadingDetails}
          patternType={isRotation ? "rotation" : "weekly"}
          shifts={shifts}
          variant="row"
        />
      </div>
    </div>
  );
}
