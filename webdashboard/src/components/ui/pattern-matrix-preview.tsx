import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import type { Shift, WorkingPatternDetail } from "@/lib/api/types";

import { Popover, PopoverAnchor, PopoverContent } from "@/components/ui/popover";
import {
  ShiftTimelineScrubber,
} from "@/components/ui/shift-timeline-scrubber";
import { Skeleton } from "@/components/ui/skeleton";
import { cn } from "@/lib/utils";
import { PatternPopoverDetails } from "@/pages/attendance/components/working-patterns/pattern-popover-details";

interface PatternMatrixPreviewProps {
  cycleLength: number;
  details?: WorkingPatternDetail[];
  isLoading?: boolean;
  patternType: "weekly" | "rotation";
  shifts?: Shift[];
  /** row = larger cells for pattern list cards; modal = compact for modal previews */
  variant?: "row" | "modal";
}

/** Assign a stable color index to each distinct shift ID for badge tinting */
const SHIFT_COLOR_PALETTE = [
  "bg-indigo-50 text-indigo-700 border-indigo-200 dark:bg-indigo-950/30 dark:text-indigo-300 dark:border-indigo-800/50",
  "bg-emerald-50 text-emerald-700 border-emerald-200 dark:bg-emerald-950/30 dark:text-emerald-300 dark:border-emerald-800/50",
  "bg-violet-50 text-violet-700 border-violet-200 dark:bg-violet-950/30 dark:text-violet-300 dark:border-violet-800/50",
  "bg-amber-50 text-amber-700 border-amber-200 dark:bg-amber-950/30 dark:text-amber-300 dark:border-amber-800/50",
  "bg-sky-50 text-sky-700 border-sky-200 dark:bg-sky-950/30 dark:text-sky-300 dark:border-sky-800/50",
  "bg-rose-50 text-rose-700 border-rose-200 dark:bg-rose-950/30 dark:text-rose-300 dark:border-rose-800/50",
  "bg-teal-50 text-teal-700 border-teal-200 dark:bg-teal-950/30 dark:text-teal-300 dark:border-teal-800/50",
  "bg-orange-50 text-orange-700 border-orange-200 dark:bg-orange-950/30 dark:text-orange-300 dark:border-orange-800/50",
];

function shiftColorClass(
  shiftId: string,
  allShiftIds: string[],
): string {
  const idx = allShiftIds.indexOf(shiftId);
  return SHIFT_COLOR_PALETTE[idx % SHIFT_COLOR_PALETTE.length];
}

export function PatternMatrixPreview({
  details,
  shifts,
  patternType,
  cycleLength,
  isLoading = false,
  variant = "row",
}: PatternMatrixPreviewProps) {
  const { t } = useTranslation();

  const daysOfWeekShort = [
    t("days.sunday_short"),
    t("days.monday_short"),
    t("days.tuesday_short"),
    t("days.wednesday_short"),
    t("days.thursday_short"),
    t("days.friday_short"),
    t("days.saturday_short"),
  ];

  const dayLabels =
    patternType === "rotation"
      ? Array.from({ length: cycleLength }, (_, i) =>
          t("patterns.form.day_index", { index: i + 1 }),
        )
      : daysOfWeekShort;

  const numDays = patternType === "rotation" ? cycleLength : 7;

  // Collect all distinct shift IDs for color assignment
  const allShiftIds = Array.from(
    new Set(
      (details ?? [])
        .filter((d) => d.shift_id)
        .map((d) => d.shift_id!),
    ),
  );

  // Loading skeleton
  if (isLoading) {
    return (
      <div className="flex gap-1.5 overflow-x-auto py-1 scrollbar-thin scrollbar-thumb-slate-100 dark:scrollbar-thumb-slate-800 scrollbar-track-transparent">
        {Array.from({ length: Math.min(numDays, 7) }).map((_, i) => (
          <Skeleton
            className={cn(
              "rounded-xl flex-shrink-0 border border-slate-100 dark:border-slate-800",
              variant === "row" ? "w-28 h-28" : "w-20 h-20",
            )}
            key={i}
          />
        ))}
      </div>
    );
  }

  // Empty state
  if (!details?.length) {
    return (
      <div className="flex items-center justify-center w-full py-8 text-[10px] font-bold uppercase tracking-widest text-slate-300 dark:text-slate-600 italic">
        {t("patterns.empty_desc")}
      </div>
    );
  }

  return (
    <div
      className="flex gap-1.5 overflow-x-auto py-1 scrollbar-thin scrollbar-thumb-slate-100 dark:scrollbar-thumb-slate-800 scrollbar-track-transparent"
      role="list"
    >
      {Array.from({ length: numDays }, (_, idx) => {
        const dayDetails =
          details?.filter((d) => d.schedule_day === idx) ?? [];
        const assignedDetails = dayDetails.filter((d) => d.shift_id);
        const hasShifts = assignedDetails.length > 0;

        // Build interval data for the scrubber
        const intervals = assignedDetails
          .map((d) => {
            const shift = shifts?.find((s) => s.id === d.shift_id);
            if (!shift?.intervals?.length) return null;
            return shift.intervals[0]; // first interval block
          })
          .filter(Boolean);

        return (
          <DayMatrixCell
            allShiftIds={allShiftIds}
            assignedDetails={assignedDetails}
            dayLabel={dayLabels[idx]}
            hasShifts={hasShifts}
            intervals={intervals as {
              start_time?: string;
              end_time?: string;
              break_start?: string;
              break_end?: string;
              break_duration_minutes?: number;
            }[]}
            key={idx}
            shifts={shifts}
            variant={variant}
          />
        );
      })}
    </div>
  );
}

/** Individual day cell in the matrix */
function DayMatrixCell({
  allShiftIds,
  assignedDetails,
  dayLabel,
  hasShifts,
  intervals,
  shifts,
  variant,
}: {
  allShiftIds: string[];
  assignedDetails: WorkingPatternDetail[];
  dayLabel: string;
  hasShifts: boolean;
  intervals: {
    start_time?: string;
    end_time?: string;
    break_start?: string;
    break_end?: string;
    break_duration_minutes?: number;
  }[];
  shifts?: Shift[];
  variant: "row" | "modal";
}) {
  const { t } = useTranslation();
  const [isOpen, setIsOpen] = useState(false);
  const isCompact = variant === "modal";

  const cellContent = (
    <div
      className={cn(
        "flex flex-col gap-1 transition-all",
        isCompact ? "w-20 min-w-[5rem]" : "w-28 min-w-[7rem]",
      )}
    >
      {/* Day label */}
      <span
        className={cn(
          "font-bold uppercase tracking-wider text-center",
          isCompact ? "text-[8px]" : "text-[9px]",
          hasShifts
            ? "text-slate-600 dark:text-slate-350"
            : "text-slate-300 dark:text-slate-600",
        )}
      >
        {dayLabel}
      </span>

      {/* Shift badges */}
      <div className="flex flex-col gap-0.5 items-center min-h-[1.25rem]">
        {hasShifts ? (
          assignedDetails.map((d, i) => {
            const shift = shifts?.find((s) => s.id === d.shift_id);
            const shiftName = shift?.name ?? "—";
            const shortName =
              shiftName.length > 10
                ? shiftName.substring(0, 10) + "…"
                : shiftName;
            const colorCls = shiftColorClass(
              d.shift_id ?? "",
              allShiftIds,
            );

            return (
              <span
                className={cn(
                  "px-1.5 py-0.5 rounded-md border text-[7px] font-bold uppercase truncate max-w-full text-center leading-tight",
                  colorCls,
                )}
                key={i}
              >
                {shortName}
              </span>
            );
          })
        ) : (
          <span className="text-[7px] font-semibold text-slate-300 dark:text-slate-600 uppercase italic">
            {t("status_off")}
          </span>
        )}
      </div>

      {/* Mini timeline scrubber */}
      {!!hasShifts && intervals.length > 0 && (
        <ShiftTimelineScrubber
          className="mt-0.5"
          intervals={intervals}
          shiftName={undefined}
          variant={isCompact ? "compact" : "compact"}
        />
      )}

      {/* Empty day fill */}
      {!hasShifts && (
        <div className="h-3 mt-0.5 rounded-lg bg-slate-100/50 dark:bg-slate-800/30 border border-dashed border-slate-200/50 dark:border-slate-700/30" />
      )}
    </div>
  );

  // Popover for detail on hover (only for row variant with shifts)
  if (variant === "row" && hasShifts) {
    return (
      <Popover modal={false} onOpenChange={setIsOpen} open={isOpen}>
        <PopoverAnchor asChild>
          <button
            aria-label={`${dayLabel}: ${assignedDetails
              .map(
                (d) =>
                  shifts?.find((s) => s.id === d.shift_id)?.name ?? "",
              )
              .join(", ")}`}
            className={cn(
              "flex flex-col items-center rounded-2xl border p-2 transition-all text-left outline-none focus-visible:ring-2 focus-visible:ring-primary-500 focus-visible:ring-offset-2 focus-visible:ring-offset-white dark:focus-visible:ring-offset-slate-900 active:scale-95",
              hasShifts
                ? "bg-white dark:bg-slate-900 border-slate-200 dark:border-slate-800 hover:border-primary-300 dark:hover:border-primary-700 hover:shadow-sm"
                : "bg-slate-50/50 dark:bg-slate-800/20 border-slate-100 dark:border-slate-800 opacity-60 hover:opacity-100",
            )}
            onBlur={() => {
              setIsOpen(false);
            }}
            onFocus={() => {
              setIsOpen(true);
            }}
            onMouseEnter={() => {
              setIsOpen(true);
            }}
            onMouseLeave={() => {
              setIsOpen(false);
            }}
            type="button"
          >
            {cellContent}
          </button>
        </PopoverAnchor>
        <PopoverContent
          className="w-64 p-4 rounded-2xl shadow-xl border border-slate-100 dark:border-slate-800 bg-white dark:bg-slate-900 pointer-events-none animate-in fade-in zoom-in-95 duration-200"
          side="top"
          sideOffset={8}
        >
          <PatternPopoverDetails
            dayDetails={assignedDetails}
            dayName={dayLabel}
            shifts={shifts}
          />
        </PopoverContent>
      </Popover>
    );
  }

  // Modal compact variant — just the cell, no popover
  return (
    <div
      className={cn(
        "flex flex-col items-center rounded-xl p-1.5 transition-all",
        hasShifts
          ? "bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800"
          : "bg-slate-50/50 dark:bg-slate-800/20 border border-slate-100 dark:border-slate-800 opacity-60",
      )}
    >
      {cellContent}
    </div>
  );
}
