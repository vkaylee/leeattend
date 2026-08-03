import { ArrowLeft, Clock } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { type Shift } from "@/lib/api";
import { cn } from "@/lib/utils";

interface CalendarCell {
  cycleIndex: number;
  date: Date;
  dayShifts: string[];
  isAnchor: boolean;
  isCurrentMonth: boolean;
  isToday: boolean;
}

function DayOfWeekHeaders({ t }: { t: (key: string) => string }) {
  const days = [
    t("days.sunday"),
    t("days.monday"),
    t("days.tuesday"),
    t("days.wednesday"),
    t("days.thursday"),
    t("days.friday"),
    t("days.saturday"),
  ];

  return (
    <div className="grid grid-cols-7 gap-2 mb-2 text-center">
      {days.map((dName, idx) => (
        <span
          className="text-[9px] font-extrabold text-slate-400 dark:text-slate-500 uppercase tracking-widest py-1"
          key={idx}
        >
          {dName.slice(0, 3)}
        </span>
      ))}
    </div>
  );
}

interface WorkingPatternCalendarPreviewProps {
  cycleLengthDays: number;
  generateCalendarCells: (
    cycleLength: number,
    grid: string[][]
  ) => CalendarCell[];
  grid: string[][];
  handleNextMonth: () => void;
  handlePreviousMonth: () => void;
  previewAnchorDate: string;
  previewMonth: number;
  previewYear: number;
  setPreviewAnchorDate: (val: string) => void;
  shifts: Shift[];
}

export function WorkingPatternCalendarPreview({
  cycleLengthDays,
  generateCalendarCells,
  grid,
  handleNextMonth,
  handlePreviousMonth,
  previewAnchorDate,
  previewMonth,
  previewYear,
  setPreviewAnchorDate,
  shifts,
}: WorkingPatternCalendarPreviewProps) {
  const { t } = useTranslation();
  const { formatDate } = useDateTimeFormatter();

  return (
    <div className="flex-1 flex flex-col space-y-6">
      {/* Calendar Config Controls */}
      <div className="flex flex-col sm:flex-row items-stretch sm:items-center justify-between gap-4 p-5 bg-slate-50/50 dark:bg-slate-800/30 border border-slate-100 dark:border-slate-800 rounded-2xl">
        {/* Anchor Date Picker */}
        <div className="flex-1 flex flex-col gap-2">
          <label className="text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-wider">
            {t("patterns.form.anchor_date_preview")}
          </label>
          <input
            className="w-full h-10 px-4 rounded-xl border border-slate-100 dark:border-slate-700 bg-white dark:bg-slate-900 text-sm font-bold text-slate-700 dark:text-slate-200 outline-none focus:border-primary-500 transition-colors"
            onChange={(e: React.ChangeEvent<HTMLInputElement>) => {
              setPreviewAnchorDate(e.target.value);
            }}
            type="date"
            value={previewAnchorDate}
          />
        </div>

        {/* Month Selector */}
        <div className="flex-1 flex flex-col gap-2">
          <label className="text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-wider text-right sm:text-right">
            {t("patterns.form.preview_month")}
          </label>
          <div className="flex items-center justify-end gap-3 h-10">
            <Button
              className="p-2 border border-slate-100 dark:border-slate-700 bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 rounded-xl hover:bg-slate-50 dark:hover:bg-slate-800 active:scale-95 transition-all"
              onClick={handlePreviousMonth}
              type="button"
              variant="ghost"
            >
              <ArrowLeft size={16} />
            </Button>
            <span className="text-sm font-extrabold text-slate-700 dark:text-slate-200 uppercase tracking-tight min-w-[8rem] text-center">
              {formatDate(new Date(previewYear, previewMonth))}
            </span>
            <Button
              className="p-2 border border-slate-100 dark:border-slate-700 bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 rounded-xl hover:bg-slate-50 dark:hover:bg-slate-800 active:scale-95 transition-all rotate-180"
              onClick={handleNextMonth}
              type="button"
              variant="ghost"
            >
              <ArrowLeft size={16} />
            </Button>
          </div>
        </div>
      </div>

      {/* Month Calendar Grid */}
      <div className="flex-1 flex flex-col">
        {/* Day of Week Headers */}
        <DayOfWeekHeaders t={t} />

        {/* Calendar Cells */}
        <div className="grid grid-cols-7 gap-2 flex-1 auto-rows-fr">
          {(() => {
            const cells = generateCalendarCells(cycleLengthDays, grid);

            return cells.map(({ cycleIndex, date, dayShifts, isAnchor, isCurrentMonth, isToday }, idx) => {
              return (
                <div
                  className={cn(
                    "min-h-[5.5rem] p-3 rounded-2xl border transition-all duration-300 flex flex-col justify-between",
                    isCurrentMonth
                      ? "bg-white dark:bg-slate-900 border-slate-100 dark:border-slate-800"
                      : "bg-slate-50/20 dark:bg-slate-900/10 border-slate-50 dark:border-slate-900/50 opacity-40",
                    isToday && "ring-2 ring-primary-500/50 border-primary-100 dark:border-primary-850",
                    isAnchor && "border-secondary-500 bg-secondary-50/10 dark:bg-secondary-950/10"
                  )}
                  key={idx}
                >
                  <div className="flex justify-between items-start">
                    <span
                      className={cn(
                        "text-xs font-extrabold",
                        isToday
                          ? "text-primary-500"
                          : isCurrentMonth
                          ? "text-slate-700 dark:text-slate-200"
                          : "text-slate-400 dark:text-slate-600"
                      )}
                    >
                      {date.getDate()}
                    </span>
                    {cycleIndex >= 0 && (
                      <span className="text-[8px] font-extrabold uppercase tracking-wider px-1.5 py-0.5 rounded bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-750 text-slate-400 dark:text-slate-500">
                        D{cycleIndex + 1}
                      </span>
                    )}
                  </div>

                  <div className="flex flex-col gap-1 mt-2">
                    {dayShifts.length > 0 ? (
                      dayShifts.map((shiftId) => {
                        const shiftName = shifts.find((s) => s.id === shiftId)?.name ?? shiftId;
                        return (
                          <div
                            className="flex items-center gap-1 px-1.5 py-0.5 bg-primary-50 dark:bg-primary-950 border border-primary-100/50 dark:border-primary-900 rounded-md shadow-sm"
                            key={shiftId}
                          >
                            <Clock className="text-primary-500 flex-shrink-0" size={8} />
                            <span className="text-[8px] font-bold text-primary-750 dark:text-primary-400 truncate uppercase tracking-tight">
                              {shiftName}
                            </span>
                          </div>
                        );
                      })
                    ) : (
                      <div className="text-center py-1">
                        <span className="text-[8px] font-extrabold text-slate-300 dark:text-slate-600 uppercase tracking-wider italic">
                          {t("status_off")}
                        </span>
                      </div>
                    )}
                  </div>
                </div>
              );
            });
          })()}
        </div>
      </div>
    </div>
  );
}
