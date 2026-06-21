import { Clock, X } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import { type Shift } from "@/lib/api";
import { cn } from "@/lib/utils";

interface WeeklyDesignEditorProps {
  addShiftToDay: (dayIndex: number, shiftId: string) => void;
  grid: string[][];
  gridRows: string[];
  removeShiftFromDay: (dayIndex: number, shiftId: string) => void;
  shifts: Shift[];
}

function WeeklyDesignEditor({
  addShiftToDay,
  grid,
  gridRows,
  removeShiftFromDay,
  shifts,
}: WeeklyDesignEditorProps) {
  const { t } = useTranslation();
  return (
    <div className="space-y-4 max-h-[60vh] overflow-y-auto pr-2 scrollbar-thin scrollbar-thumb-slate-200 dark:scrollbar-thumb-slate-800 scrollbar-track-transparent">
      {gridRows.map((dayName, index) => {
        const dayShifts = grid[index] || [];
        return (
          <div
            className={cn(
              "flex flex-col sm:flex-row sm:items-center justify-between gap-4 px-6 py-4 rounded-2xl border transition-all duration-300",
              dayShifts.length > 0
                ? "bg-white dark:bg-slate-900 border-primary-100 dark:border-primary-800/50 shadow-sm"
                : "bg-slate-50/50 dark:bg-slate-850/30 border-slate-100 dark:border-slate-850"
            )}
            key={index}
          >
            <div className="w-28 flex-shrink-0">
              <span
                className={cn(
                  "text-[10px] font-extrabold uppercase tracking-widest block",
                  dayShifts.length > 0
                    ? "text-slate-750 dark:text-slate-200"
                    : "text-slate-400 dark:text-slate-500"
                )}
              >
                {dayName}
              </span>
            </div>

            <div className="flex-1 flex items-center gap-2 flex-wrap min-h-[2.5rem]">
              {dayShifts.map((shiftId) => {
                const shiftName = shifts.find((s) => s.id === shiftId)?.name ?? shiftId;
                return (
                  <div
                    className="flex items-center gap-1.5 px-3 py-1.5 bg-primary-50 dark:bg-primary-950 border border-primary-100 dark:border-primary-850 rounded-xl shadow-sm animate-in zoom-in-95 duration-350"
                    key={shiftId}
                  >
                    <Clock className="text-primary-500 dark:text-primary-400 flex-shrink-0" size={10} />
                    <span className="text-[10px] font-bold text-primary-750 dark:text-primary-300 uppercase tracking-tight">
                      {shiftName.toUpperCase()}
                    </span>
                    <Button
                      aria-label={`Remove ${shiftName}`}
                      className="p-0.5 rounded-md text-primary-400 hover:text-rose-500 hover:bg-rose-50 dark:hover:bg-rose-950/50 transition-colors flex-shrink-0 ml-1"
                      onClick={() => {
                        removeShiftFromDay(index, shiftId);
                      }}
                      type="button"
                      variant="destructive"
                    >
                      <X size={12} />
                    </Button>
                  </div>
                );
              })}
              {dayShifts.length === 0 && (
                <span className="text-[10px] font-bold text-slate-300 dark:text-slate-650 uppercase tracking-widest italic">
                  {t("status_off")}
                </span>
              )}
            </div>

            <Combobox
              className="w-36 flex-shrink-0 !py-1.5 !px-3 rounded-xl text-[9px] font-bold uppercase tracking-tight outline-none border border-dashed border-primary-200 dark:border-primary-800 bg-transparent text-primary-500 dark:text-primary-400 cursor-pointer transition-all hover:bg-primary-50 dark:hover:bg-primary-950/50"
              onChange={(val) => {
                if (val) addShiftToDay(index, val);
              }}
              options={shifts
                .filter((s) => !dayShifts.includes(s.id))
                .map((s) => ({ label: s.name.toUpperCase(), value: s.id }))}
              placeholder={`+ ${t("patterns.form.add_shift") || "ADD"}`}
              value=""
            />
          </div>
        );
      })}
    </div>
  );
}

interface RotationDesignEditorProps {
  addShiftToDay: (dayIndex: number, shiftId: string) => void;
  grid: string[][];
  gridRows: string[];
  removeShiftFromDay: (dayIndex: number, shiftId: string) => void;
  shifts: Shift[];
}

function RotationDesignEditor({
  addShiftToDay,
  grid,
  gridRows,
  removeShiftFromDay,
  shifts,
}: RotationDesignEditorProps) {
  const { t } = useTranslation();
  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-4 max-h-[60vh] overflow-y-auto pr-2 scrollbar-thin scrollbar-thumb-slate-200 dark:scrollbar-thumb-slate-800 scrollbar-track-transparent p-1">
      {gridRows.map((dayName, index) => {
        const dayShifts = grid[index] || [];
        return (
          <div
            className={cn(
              "flex flex-col justify-between p-5 rounded-[1.5rem] border transition-all duration-300 gap-4 min-h-[10rem]",
              dayShifts.length > 0
                ? "bg-white dark:bg-slate-900 border-primary-100 dark:border-primary-800/50 shadow-sm"
                : "bg-slate-50/50 dark:bg-slate-850/30 border-slate-100 dark:border-slate-850"
            )}
            key={index}
          >
            <div className="flex items-center justify-between">
              <span
                className={cn(
                  "text-[10px] font-extrabold uppercase tracking-widest",
                  dayShifts.length > 0
                    ? "text-slate-755 dark:text-slate-200"
                    : "text-slate-400 dark:text-slate-500"
                )}
              >
                {dayName}
              </span>
            </div>

            <div className="flex-1 flex flex-col gap-1.5 justify-center min-h-[3rem]">
              {dayShifts.length > 0 ? (
                dayShifts.map((shiftId) => {
                  const shiftName = shifts.find((s) => s.id === shiftId)?.name ?? shiftId;
                  return (
                    <div
                      className="flex items-center justify-between gap-1.5 px-2.5 py-1.5 bg-primary-50 dark:bg-primary-950 border border-primary-100 dark:border-primary-850 rounded-xl shadow-sm animate-in zoom-in-95 duration-350"
                      key={shiftId}
                    >
                      <div className="flex items-center gap-1.5 overflow-hidden">
                        <Clock
                          className="text-primary-500 dark:text-primary-400 flex-shrink-0"
                          size={10}
                        />
                        <span className="text-[10px] font-bold text-primary-750 dark:text-primary-300 uppercase tracking-tight truncate">
                          {shiftName.toUpperCase()}
                        </span>
                      </div>
                      <Button
                        aria-label={`Remove ${shiftName}`}
                        className="p-0.5 rounded-md text-primary-400 hover:text-rose-500 hover:bg-rose-50 dark:hover:bg-rose-950/50 transition-colors flex-shrink-0 ml-1"
                        onClick={() => {
                          removeShiftFromDay(index, shiftId);
                        }}
                        type="button"
                        variant="destructive"
                      >
                        <X size={12} />
                      </Button>
                    </div>
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

            <Combobox
              className="w-full !py-1.5 !px-3 rounded-xl text-[9px] font-bold uppercase tracking-tight outline-none border border-dashed border-primary-200 dark:border-primary-800 bg-transparent text-primary-500 dark:text-primary-400 cursor-pointer transition-all hover:bg-primary-50 dark:hover:bg-primary-950/50"
              onChange={(val) => {
                if (val) addShiftToDay(index, val);
              }}
              options={shifts
                .filter((s) => !dayShifts.includes(s.id))
                .map((s) => ({ label: s.name.toUpperCase(), value: s.id }))}
              placeholder={`+ ${t("patterns.form.add_shift") || "ADD"}`}
              value=""
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
  return (
    <div className="flex-1">
      {patternType === "weekly" ? (
        <WeeklyDesignEditor
          addShiftToDay={addShiftToDay}
          grid={grid}
          gridRows={gridRows}
          removeShiftFromDay={removeShiftFromDay}
          shifts={shifts}
        />
      ) : (
        <RotationDesignEditor
          addShiftToDay={addShiftToDay}
          grid={grid}
          gridRows={gridRows}
          removeShiftFromDay={removeShiftFromDay}
          shifts={shifts}
        />
      )}
    </div>
  );
}
