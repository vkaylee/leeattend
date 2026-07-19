import type { Locale } from "date-fns/locale";

import React, { memo } from "react";

import { format, isToday as isDateToday } from "date-fns";

import type { EmployeeSchedule, Shift } from "@/lib/api/types";

import { RosterCell } from "./roster-cell";
import { isWeekend } from "./roster-utils";

interface RosterGridTableProps {
  containerRef?: React.RefObject<HTMLDivElement | null>;
  currentLocale: Locale;
  employeesLabel: string;
  isDragging: boolean;
  monthDays: Date[];
  monthSchedules: EmployeeSchedule[];
  onCellMouseDown: (
    employeeId: string,
    dateStr: string,
    e: React.MouseEvent,
  ) => void;
  onCellMouseEnter: (employeeId: string, dateStr: string) => void;
  selectedCells: Set<string>;
  shifts?: Shift[];
}

const RosterGridTable = memo(function RosterGridTable({
  containerRef,
  currentLocale,
  employeesLabel,
  isDragging,
  monthDays,
  monthSchedules,
  onCellMouseDown,
  onCellMouseEnter,
  selectedCells,
  shifts,
}: RosterGridTableProps) {
  return (
    <div
      className={`overflow-x-auto relative max-w-full roster-scrollbar ${isDragging ? "select-none" : ""}`}
      ref={containerRef}
    >
      <table className="w-full border-collapse">
        <thead>
          <tr className="border-b border-slate-100 dark:border-slate-800 bg-slate-50/50 dark:bg-slate-850/50">
            <th className="sticky left-0 bg-slate-50 dark:bg-slate-850 border-r border-slate-100 dark:border-slate-800 px-4 py-3 text-left text-[10px] font-bold uppercase tracking-wider text-slate-500 dark:text-slate-400 z-20 min-w-[160px] shadow-[2px_0_5px_rgba(0,0,0,0.02)]">
              {employeesLabel}
            </th>
            {monthDays.map((day) => {
              const dateStr = format(day, "yyyy-MM-dd");
              const dayNum = day.getDate();
              const dayName = format(day, "eee", { locale: currentLocale });
              const isToday = isDateToday(day);
              const isWe = isWeekend(day);

              return (
                <th
                  className={`px-2 py-2 border-r border-slate-100 dark:border-slate-800 text-center min-w-[45px] ${
                    isToday
                      ? "bg-primary-50/50 dark:bg-primary-950/20"
                      : isWe
                      ? "bg-amber-50/30 dark:bg-amber-950/10"
                      : ""
                  }`}
                  key={dateStr}
                >
                  <div
                    className={`text-[10px] font-bold uppercase tracking-tighter ${isToday ? "text-primary-600 dark:text-primary-400" : "text-slate-400 dark:text-slate-500"}`}
                  >
                    {dayName}
                  </div>
                  <div
                    className={`text-sm font-extrabold mt-0.5 ${
                      isToday
                        ? "text-primary-600 dark:text-primary-400 bg-primary-100 dark:bg-primary-900/60 rounded-full w-6 h-6 flex items-center justify-center mx-auto"
                        : isWe
                        ? "text-amber-600 dark:text-amber-500"
                        : "text-slate-600 dark:text-slate-300"
                    }`}
                  >
                    {dayNum}
                  </div>
                </th>
              );
            })}
          </tr>
        </thead>
        <tbody>
          {monthSchedules.map((empSchedule) => (
            <tr
              className="border-b border-slate-100 dark:border-slate-800 last:border-b-0 hover:bg-slate-50/30 dark:hover:bg-slate-850/10"
  // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
              key={empSchedule.employee_id ?? empSchedule.employment_id}
            >
              <td className="sticky left-0 bg-white dark:bg-slate-900 border-r border-slate-100 dark:border-slate-800 px-4 py-3 font-bold text-xs text-slate-700 dark:text-slate-300 z-10 shadow-[2px_0_5px_rgba(0,0,0,0.02)] min-w-[160px] truncate">
                <div className="flex items-center gap-2">
                  <div className="w-6 h-6 rounded-full bg-slate-100 dark:bg-slate-800 flex items-center justify-center text-[10px] font-extrabold text-slate-600 dark:text-slate-400">
                    {empSchedule.full_name.substring(0, 2).toUpperCase()}
                  </div>
                  <span className="truncate">{empSchedule.full_name}</span>
                </div>
              </td>

              {monthDays.map((day) => {
                const dateStr = format(day, "yyyy-MM-dd");
  // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
                const cellKey = `${empSchedule.employee_id ?? ""}:${dateStr}`;
                const isSelected = selectedCells.has(cellKey);
                const daySchedules = empSchedule.schedules.filter(
                  (s) => s.date === dateStr,
                );

                return (
                  <RosterCell
                    dateStr={dateStr}
                    daySchedules={daySchedules}
  // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
                    employeeId={empSchedule.employee_id ?? ""}
                    isSelected={isSelected}
                    key={dateStr}
                    onMouseDown={onCellMouseDown}
                    onMouseEnter={onCellMouseEnter}
                    shifts={shifts}
                  />
                );
              })}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
});
RosterGridTable.displayName = "RosterGridTable";

export { RosterGridTable };
