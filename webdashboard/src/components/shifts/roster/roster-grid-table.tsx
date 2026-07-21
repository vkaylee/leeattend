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
  onCellPointerDown: (
    employeeId: string,
    dateStr: string,
    e: React.PointerEvent,
  ) => void;
  onCellPointerEnter: (employeeId: string, dateStr: string) => void;
  rowStatus?: Map<string, "saving" | "saved" | "error">;
  selectedCells: Set<string>;
  shifts?: Shift[];
}

/** Resolve the roster cell coordinates from a DOM node, if any. */
const cellCoordsFromNode = (
  node: Element | null,
): { dateStr: string; employeeId: string } | null => {
  const cell = node?.closest<HTMLElement>("[data-employee-id][data-date]");
  if (!cell) return null;
  const { date, employeeId } = cell.dataset;
  if (employeeId === undefined || date === undefined) return null;
  return { dateStr: date, employeeId };
};

const RosterGridTable = memo(function RosterGridTable({
  containerRef,
  currentLocale,
  employeesLabel,
  isDragging,
  monthDays,
  monthSchedules,
  onCellPointerDown,
  onCellPointerEnter,
  rowStatus,
  selectedCells,
  shifts,
}: RosterGridTableProps) {
  const handlePointerDown = (e: React.PointerEvent) => {
    const coords = cellCoordsFromNode(e.target as Element);
    if (!coords) return;
    // Prevent native text selection / touch scroll while painting cells.
    e.preventDefault();
    onCellPointerDown(coords.employeeId, coords.dateStr, e);
  };

  // On touch, pointer events are captured by the origin cell, so subsequent
  // pointermove events never surface as pointerenter on the cells the finger
  // travels over. Hit-test the point under the pointer instead so drag-select
  // works identically for mouse, touch, and pen.
  const handlePointerMove = (e: React.PointerEvent) => {
    if (!isDragging) return;
    const coords = cellCoordsFromNode(
      document.elementFromPoint(e.clientX, e.clientY),
    );
    if (!coords) return;
    e.preventDefault();
    onCellPointerEnter(coords.employeeId, coords.dateStr);
  };

  return (
    <div
      className={`overflow-x-auto relative max-w-full roster-scrollbar ${isDragging ? "select-none touch-none" : ""}`}
      onPointerDown={handlePointerDown}
      onPointerMove={handlePointerMove}
      ref={containerRef}
    >
      <table className="w-full border-collapse">
        <thead>
          <tr className="border-b border-slate-100 dark:border-slate-800 bg-slate-50/50 dark:bg-slate-850/50">
            <th className="sticky left-0 bg-slate-50 dark:bg-slate-850 border-r border-slate-100 dark:border-slate-800 px-3 sm:px-4 py-3 text-left text-[10px] font-bold uppercase tracking-wider text-slate-500 dark:text-slate-400 z-20 min-w-[140px] sm:min-w-[160px] shadow-[2px_0_5px_rgba(0,0,0,0.02)]">
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
                  className={`px-1.5 sm:px-2 py-2 border-r border-slate-100 dark:border-slate-800 text-center min-w-[40px] sm:min-w-[45px] ${
                    isToday
                      ? "bg-primary-50/50 dark:bg-primary-950/20"
                      : isWe
                      ? "bg-amber-50/30 dark:bg-amber-950/10"
                      : ""
                  }`}
                  key={dateStr}
                >
                  <div
                    className={`text-[9px] sm:text-[10px] font-bold uppercase tracking-tighter ${isToday ? "text-primary-600 dark:text-primary-400" : "text-slate-400 dark:text-slate-500"}`}
                  >
                    {dayName}
                  </div>
                  <div
                    className={`text-xs sm:text-sm font-extrabold mt-0.5 ${
                      isToday
                        ? "text-primary-600 dark:text-primary-400 bg-primary-100 dark:bg-primary-900/60 rounded-full w-5 h-5 sm:w-6 sm:h-6 flex items-center justify-center mx-auto"
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
          {monthSchedules.map((empSchedule) => {
            const subtitle = [empSchedule.employee_id_number, empSchedule.department_name]
              .filter(Boolean)
              .join(" · ");
  // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
            const status = rowStatus?.get(empSchedule.employee_id ?? "");

            return (
            <tr
              className="border-b border-slate-100 dark:border-slate-800 last:border-b-0 hover:bg-slate-50/30 dark:hover:bg-slate-850/10"
  // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
              key={empSchedule.employee_id ?? empSchedule.employment_id}
            >
              <td className="sticky left-0 bg-white dark:bg-slate-900 border-r border-slate-100 dark:border-slate-800 px-3 sm:px-4 py-3 font-bold text-xs text-slate-700 dark:text-slate-300 z-10 shadow-[2px_0_5px_rgba(0,0,0,0.02)] min-w-[140px] sm:min-w-[200px]">
                <div className="flex items-center gap-2">
                  <div className="w-5 h-5 sm:w-6 sm:h-6 shrink-0 rounded-full bg-slate-100 dark:bg-slate-800 flex items-center justify-center text-[9px] sm:text-[10px] font-extrabold text-slate-600 dark:text-slate-400">
                    {empSchedule.full_name.substring(0, 2).toUpperCase()}
                  </div>
                  <div className="flex flex-col min-w-0">
                    <span className="truncate leading-tight text-[11px] sm:text-xs">{empSchedule.full_name}</span>
                    {!!subtitle && (
                      <span className="truncate text-[9px] sm:text-[10px] font-medium text-slate-400 dark:text-slate-500 leading-tight">
                        {subtitle}
                      </span>
                    )}
                  </div>
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
                    shifts={shifts}
                    status={status}
                  />
                );
              })}
            </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
});
RosterGridTable.displayName = "RosterGridTable";

export { RosterGridTable };
