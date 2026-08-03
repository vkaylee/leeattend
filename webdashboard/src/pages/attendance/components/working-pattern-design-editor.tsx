import React, { useCallback, useState } from "react";
import { useTranslation } from "react-i18next";

import { ShiftPickerModal } from "@/components/shifts/shift-picker-modal";
import { type Shift } from "@/lib/api";
import { cn } from "@/lib/utils";

import { AddShiftTrigger } from "./working-patterns/add-shift-trigger";
import { AssignedShiftChip } from "./working-patterns/assigned-shift-chip";

interface WeeklyDesignEditorProps {
  grid: string[][];
  gridRows: string[];
  onOpenPicker: (dayIndex: number) => void;
  removeShiftFromDay: (dayIndex: number, shiftId: string) => void;
  shifts: Shift[];
}

function WeeklyDesignEditor({
  grid,
  gridRows,
  onOpenPicker,
  removeShiftFromDay,
  shifts,
}: WeeklyDesignEditorProps) {
  const { t } = useTranslation();
  return (
    <div className="space-y-4 max-h-[60vh] overflow-y-auto pr-2 scrollbar-thin scrollbar-thumb-slate-200 dark:scrollbar-thumb-slate-800 scrollbar-track-transparent">
      {gridRows.map((dayName, index) => {
        const dayShifts = grid[index] || [];
        const availableShifts = shifts.filter((s) => !dayShifts.includes(s.id));
        return (
          <div
            className={cn(
              "flex flex-col sm:flex-row sm:items-center justify-between gap-4 px-6 py-4 rounded-2xl border transition-all duration-300",
              dayShifts.length > 0
                ? "bg-white dark:bg-slate-900 border-primary-100 dark:border-primary-800/50 shadow-sm"
                : "bg-slate-50/50 dark:bg-slate-850/30 border-slate-100 dark:border-slate-850",
            )}
            key={index}
          >
            <div className="w-28 flex-shrink-0">
              <span
                className={cn(
                  "text-[10px] font-extrabold uppercase tracking-widest block",
                  dayShifts.length > 0
                    ? "text-slate-750 dark:text-slate-200"
                    : "text-slate-400 dark:text-slate-500",
                )}
              >
                {dayName}
              </span>
            </div>

            <div className="flex-1 flex items-center gap-2 flex-wrap min-h-[2.5rem]">
              {dayShifts.map((shiftId) => {
                const shift = shifts.find((s) => s.id === shiftId);
                if (!shift) return null;
                return (
                  <AssignedShiftChip
                    key={shiftId}
                    onRemove={() => {
                      removeShiftFromDay(index, shiftId);
                    }}
                    shift={shift}
                  />
                );
              })}
              {dayShifts.length === 0 && (
                <span className="text-[10px] font-bold text-slate-300 dark:text-slate-650 uppercase tracking-widest italic">
                  {t("status_off")}
                </span>
              )}
            </div>

            <AddShiftTrigger
              availableCount={availableShifts.length}
              onClick={() => {
                onOpenPicker(index);
              }}
            />
          </div>
        );
      })}
    </div>
  );
}

interface RotationDesignEditorProps {
  grid: string[][];
  gridRows: string[];
  onOpenPicker: (dayIndex: number) => void;
  removeShiftFromDay: (dayIndex: number, shiftId: string) => void;
  shifts: Shift[];
}

function RotationDesignEditor({
  grid,
  gridRows,
  onOpenPicker,
  removeShiftFromDay,
  shifts,
}: RotationDesignEditorProps) {
  const { t } = useTranslation();
  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-4 max-h-[60vh] overflow-y-auto pr-2 scrollbar-thin scrollbar-thumb-slate-200 dark:scrollbar-thumb-slate-800 scrollbar-track-transparent p-1">
      {gridRows.map((dayName, index) => {
        const dayShifts = grid[index] || [];
        const availableShifts = shifts.filter((s) => !dayShifts.includes(s.id));
        return (
          <div
            className={cn(
              "flex flex-col justify-between p-5 rounded-[1.5rem] border transition-all duration-300 gap-4 min-h-[10rem]",
              dayShifts.length > 0
                ? "bg-white dark:bg-slate-900 border-primary-100 dark:border-primary-800/50 shadow-sm"
                : "bg-slate-50/50 dark:bg-slate-850/30 border-slate-100 dark:border-slate-850",
            )}
            key={index}
          >
            <div className="flex items-center justify-between">
              <span
                className={cn(
                  "text-[10px] font-extrabold uppercase tracking-widest",
                  dayShifts.length > 0
                    ? "text-slate-755 dark:text-slate-200"
                    : "text-slate-400 dark:text-slate-500",
                )}
              >
                {dayName}
              </span>
            </div>

            <div className="flex-1 flex flex-col gap-1.5 justify-center min-h-[3rem]">
              {dayShifts.length > 0 ? (
                dayShifts.map((shiftId) => {
                  const shift = shifts.find((s) => s.id === shiftId);
                  if (!shift) return null;
                  return (
                    <AssignedShiftChip
                      key={shiftId}
                      onRemove={() => {
                        removeShiftFromDay(index, shiftId);
                      }}
                      shift={shift}
                    />
                  );
                })
              ) : (
                <div className="flex items-center justify-center flex-1 border border-dashed border-slate-200 dark:border-slate-800 rounded-2xl bg-slate-50/20 dark:bg-slate-900/10 min-h-[3rem]">
                  <span className="text-[10px] font-bold text-slate-350 dark:text-slate-655 uppercase tracking-widest italic">
                    {t("status_off")}
                  </span>
                </div>
              )}
            </div>

            <AddShiftTrigger
              availableCount={availableShifts.length}
              fullWidth
              onClick={() => {
                onOpenPicker(index);
              }}
            />
          </div>
        );
      })}
    </div>
  );
}

interface WorkingPatternDesignEditorProps {
  addShiftToDay: (dayIndex: number, shiftId: string) => void;
  grid: string[][];
  gridRows: string[];
  patternType: "weekly" | "rotation";
  removeShiftFromDay: (dayIndex: number, shiftId: string) => void;
  shifts: Shift[];
}

export function WorkingPatternDesignEditor({
  addShiftToDay,
  grid,
  gridRows,
  patternType,
  removeShiftFromDay,
  shifts,
}: WorkingPatternDesignEditorProps) {
  const [pickerDayIndex, setPickerDayIndex] = useState<number | null>(null);

  const handleOpenPicker = useCallback((dayIndex: number) => {
    setPickerDayIndex(dayIndex);
  }, []);

  const handleClosePicker = useCallback(() => {
    setPickerDayIndex(null);
  }, []);

  const handleSelectShift = useCallback(
    (shiftId: string | null) => {
      if (pickerDayIndex === null) return;
      if (shiftId === null) {
        // OFF day — clear all shifts for this day
        const dayShifts = grid[pickerDayIndex] ?? [];
        for (const sid of dayShifts) {
          removeShiftFromDay(pickerDayIndex, sid);
        }
      } else {
        addShiftToDay(pickerDayIndex, shiftId);
      }
    },
    [pickerDayIndex, grid, addShiftToDay, removeShiftFromDay],
  );

  const pickerDayName = pickerDayIndex !== null ? (gridRows[pickerDayIndex] ?? "") : "";

  // Shifts available for the currently-picked day (filter out already-assigned)
  const pickerShifts =
    pickerDayIndex !== null
      ? shifts.filter((s) => !(grid[pickerDayIndex] ?? []).includes(s.id))
      : shifts;

  return (
    <div className="flex-1">
      {patternType === "weekly" ? (
        <WeeklyDesignEditor
          grid={grid}
          gridRows={gridRows}
          onOpenPicker={handleOpenPicker}
          removeShiftFromDay={removeShiftFromDay}
          shifts={shifts}
        />
      ) : (
        <RotationDesignEditor
          grid={grid}
          gridRows={gridRows}
          onOpenPicker={handleOpenPicker}
          removeShiftFromDay={removeShiftFromDay}
          shifts={shifts}
        />
      )}

      {pickerDayIndex !== null && (
        <ShiftPickerModal
          isOpen
          onClose={handleClosePicker}
          onSelectShift={handleSelectShift}
          shifts={pickerShifts}
          targetDates={[pickerDayName]}
        />
      )}
    </div>
  );
}
