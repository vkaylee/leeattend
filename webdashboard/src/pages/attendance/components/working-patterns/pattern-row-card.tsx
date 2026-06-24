import { useQuery } from "@tanstack/react-query";
import { CalendarDays, Edit, Trash2, Clock, Coffee } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Popover, PopoverContent, PopoverAnchor } from "@/components/ui/popover";
import StatusBadge from "@/components/ui/status-badge";
import { getCurrentWorkspaceId, fetchPatternDetails } from "@/lib/api";
import { Shift, WorkingPattern, WorkingPatternDetail } from "@/lib/api/types";
import { cn } from "@/lib/utils";

interface PatternRowCardProps {
  expandedPatternId: string | null;
  onDelete: (pattern: WorkingPattern) => void;
  onEdit: (pattern: WorkingPattern) => void;
  onToggleExpand: (id: string) => void;
  pattern: WorkingPattern;
  patternDetails?: WorkingPatternDetail[];
  shifts?: Shift[];
}

function DayTimelinePill({
  dayName,
  dayDetails,
  shifts,
}: {
  dayName: string;
  dayDetails: WorkingPatternDetail[];
  shifts?: Shift[];
}) {
  const { t } = useTranslation();
  const assignedDetails = dayDetails.filter((d) => d.shift_id);
  const hasShifts = assignedDetails.length > 0;
  const [isOpen, setIsOpen] = useState(false);

  return (
    <Popover modal={false} onOpenChange={setIsOpen} open={isOpen}>
      <PopoverAnchor asChild>
        <Button
          aria-label={`${dayName}: ${hasShifts ? assignedDetails.map((d) => shifts?.find((s) => s.id === d.shift_id)?.name ?? "").join(", ") : t("status_off")}`}
          className={cn(
            "flex flex-col items-center justify-between min-w-[8rem] h-14 p-2 rounded-xl border transition-all text-left outline-none focus-visible:ring-2 focus-visible:ring-primary-500 focus-visible:ring-offset-2 focus-visible:ring-offset-white dark:focus-visible:ring-offset-slate-900 font-normal normal-case active:scale-95",
            hasShifts
              ? "bg-primary-50/50 dark:bg-primary-950/20 border-primary-100 dark:border-primary-800/50 hover:border-primary-300 dark:hover:border-primary-700 text-slate-700 dark:text-slate-200"
              : "bg-slate-50/50 dark:bg-slate-800/20 border-slate-100 dark:border-slate-800 opacity-60 hover:opacity-100 hover:border-slate-300 dark:hover:border-slate-700 text-slate-500 dark:text-slate-400"
          )}
          onBlur={() => { setIsOpen(false); }}
          onFocus={() => { setIsOpen(true); }}
          onMouseEnter={() => { setIsOpen(true); }}
          onMouseLeave={() => { setIsOpen(false); }}
          size="sm"
          variant="ghost"
        >
          <span
            className={cn(
              "text-[8px] font-bold uppercase tracking-wider",
              hasShifts
                ? "text-primary-600 dark:text-primary-400"
                : "text-slate-400 dark:text-slate-500"
            )}
          >
            {dayName}
          </span>
          <div className="flex items-center justify-center w-full">
            {hasShifts ? (
              <div className="flex flex-col gap-0.5 items-center w-full">
                {assignedDetails.map((d, i) => {
                  const shiftName = shifts?.find((s) => s.id === d.shift_id)?.name ?? "—";
                  // take first 12 characters for compact view
                  const shortName = shiftName.length > 12 ? shiftName.substring(0, 12) + "." : shiftName;
                  return (
                    <span
                      className="text-[9px] font-bold text-slate-700 dark:text-slate-200 truncate max-w-[7.5rem] uppercase text-center"
                      key={i}
                    >
                      {shortName}
                    </span>
                  );
                })}
              </div>
            ) : (
              <span className="text-[8px] font-semibold text-slate-300 dark:text-slate-600 uppercase italic">
                {t("status_off")}
              </span>
            )}
          </div>
        </Button>
      </PopoverAnchor>
      <PopoverContent
        className="w-64 p-4 rounded-2xl shadow-xl border border-slate-100 dark:border-slate-800 bg-white dark:bg-slate-900 pointer-events-none animate-in fade-in zoom-in-95 duration-200"
        side="top"
        sideOffset={8}
      >
        <div className="space-y-3">
          <div className="flex items-center justify-between border-b border-slate-100 dark:border-slate-800/60 pb-2">
            <span className="text-[10px] font-bold uppercase text-slate-400 dark:text-slate-500 tracking-wider">
              {dayName}
            </span>
            {!!hasShifts && !!assignedDetails[0].shift_id && (
              <span className="px-2 py-0.5 rounded-md bg-primary-50 dark:bg-primary-950 text-[9px] font-bold text-primary-600 dark:text-primary-400 uppercase">
                {shifts?.find((s) => s.id === assignedDetails[0].shift_id)?.report_symbol ?? "—"}
              </span>
            )}
          </div>

          {hasShifts ? (
            assignedDetails.map((d, idx) => {
              const shift = shifts?.find((s) => s.id === d.shift_id);
              if (!shift) return null;
              return (
                <div className="space-y-2.5" key={idx}>
                  <h5 className="font-bold text-sm text-slate-800 dark:text-slate-100 uppercase tracking-tight leading-snug">
                    {shift.name}
                  </h5>
                  <div className="flex flex-col gap-1.5 text-xs text-slate-600 dark:text-slate-350">
                    <div className="flex items-center gap-2">
                      <Clock className="text-primary-500 dark:text-primary-400" size={14} />
                      <span className="font-semibold">
                        {shift.start_time} - {shift.end_time}
                      </span>
                    </div>
                    {!!shift.break_start && !!shift.break_end && (
                      <div className="flex items-center gap-2">
                        <Coffee className="text-amber-500 dark:text-amber-400" size={14} />
                        <span>
                          {t("shifts.table.break_time")}: {shift.break_start} - {shift.break_end} ({shift.break_duration_minutes}m)
                        </span>
                      </div>
                    )}
                  </div>
                </div>
              );
            })
          ) : (
            <div className="py-2 text-center text-xs font-semibold text-slate-400 dark:text-slate-500 italic uppercase tracking-wider">
              {t("shifts.monthly.off_day") || "Rest Day"}
            </div>
          )}
        </div>
      </PopoverContent>
    </Popover>
  );
}

export function PatternRowCard({
  onDelete,
  onEdit,
  pattern,
  shifts,
}: PatternRowCardProps) {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId() ?? "";

  const { data: details, isLoading: isLoadingDetails } = useQuery<WorkingPatternDetail[]>({
    queryKey: ["pattern-details", wsId, pattern.id],
    queryFn: () => fetchPatternDetails(wsId, pattern.id),
    enabled: !!wsId && !!pattern.id,
  });

  const daysOfWeekShort = [
    t("days.sunday_short"),
    t("days.monday_short"),
    t("days.tuesday_short"),
    t("days.wednesday_short"),
    t("days.thursday_short"),
    t("days.friday_short"),
    t("days.saturday_short"),
  ];

  const isRotation = pattern.pattern_type === "rotation";
  const cycleLength = pattern.cycle_length_days ?? 7;
  const detailRows = isRotation
    ? Array.from({ length: cycleLength }, (_, i) => ({
        day: t("patterns.form.day_index", { index: i + 1 }),
        index: i,
      }))
    : daysOfWeekShort.map((day, index) => ({
        day,
        index,
      }));

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
            <h4 className="font-bold text-base text-slate-700 dark:text-slate-200 leading-tight uppercase tracking-tight truncate">
              {pattern.name}
            </h4>
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

      {/* Row 2: Timeline */}
      <div className="w-full flex items-center gap-1.5 overflow-x-auto py-1 scrollbar-thin scrollbar-thumb-slate-100 dark:scrollbar-thumb-slate-800 scrollbar-track-transparent">
        {isLoadingDetails ? (
          <div className="flex gap-1.5">
            {Array.from({ length: Math.min(cycleLength, 7) }).map((_, i) => (
              <div
                className="w-32 h-14 bg-slate-50 dark:bg-slate-800/40 border border-slate-100 dark:border-slate-800 rounded-xl animate-pulse flex-shrink-0"
                key={i}
              />
            ))}
          </div>
        ) : (
          detailRows.map(({ day, index }) => (
            <DayTimelinePill
              dayDetails={details?.filter((d) => d.schedule_day === index) ?? []}
              dayName={day}
              key={index}
              shifts={shifts}
            />
          ))
        )}
      </div>
    </div>
  );
}
