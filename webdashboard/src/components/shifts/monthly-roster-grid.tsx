import { 
  AlertCircle,
  CalendarRange,
  ChevronLeft,
  ChevronRight,
  Search,
  User
} from "lucide-react";
import React, { memo, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";

import { format, isToday as isDateToday } from "date-fns";
import { vi, enUS, zhCN, type Locale } from "date-fns/locale";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { EmployeeSchedule, Shift, EffectiveSchedule } from "@/lib/api/types";
import { useShifts } from "@/pages/shifts/hooks/use-shifts";

import { useMonthlyRoster } from "./hooks/use-monthly-roster";
import { ScheduleTracePopover } from "./schedule-trace-popover";
import { ShiftPickerModal } from "./shift-picker-modal";



const dateLocales: Partial<Record<string, Locale>> = {
  vi: vi,
  en: enUS,
  zh: zhCN,
};

// Check if weekend day (Sat/Sun)
const isWeekend = (date: Date) => {
  const day = date.getDay();
  return day === 0 || day === 6; // 0 = Sunday, 6 = Saturday
};

// Short display character for shifts
const getShiftSymbol = (shiftName?: string | null, reportSymbol?: string | null) => {
  if (reportSymbol) return reportSymbol;
  if (!shiftName) return "";
  return shiftName.substring(0, 2).toUpperCase();
};

// Get cell color styles based on priority source
const getCellStyles = (source?: string, isSelected?: boolean) => {
  const baseStyles = "transition-all relative border border-slate-100 dark:border-slate-800 text-center select-none cursor-pointer ";
  
  if (isSelected) {
    return baseStyles + "ring-2 ring-primary-500 ring-inset bg-primary-50 dark:bg-primary-950/40 z-[5]";
  }

  switch (source) {
    case "roster": // P1 (Manual Overrides)
      return baseStyles + "bg-teal-50/90 dark:bg-teal-950/40 hover:bg-teal-100 dark:hover:bg-teal-900/50";
    case "pattern": // P2 (Working Patterns)
      return baseStyles + "bg-blue-50/40 dark:bg-blue-950/20 hover:bg-blue-100/30 dark:hover:bg-blue-900/30";
    case "assignment_rule": // P3
      return baseStyles + "bg-violet-50/40 dark:bg-violet-950/20 hover:bg-violet-100/30 dark:hover:bg-violet-900/30";
    case "department_pattern": // P4
      return baseStyles + "bg-slate-50/50 dark:bg-slate-800/30 hover:bg-slate-100/40 dark:hover:bg-slate-700/30";
    case "fixed": // P5
    case "department_fixed": // P6
      return baseStyles + "bg-emerald-50/40 dark:bg-emerald-950/20 hover:bg-emerald-100/30 dark:hover:bg-emerald-900/30";
    default:
      return baseStyles + "hover:bg-slate-50 dark:hover:bg-slate-800/50";
  }
};

const getSourceIndicator = (source?: string) => {
  switch (source) {
    case "roster":
      return <span className="absolute bottom-1 right-1 w-1 h-1 rounded-full bg-teal-500" />;
    case "pattern":
      return <span className="absolute bottom-1 right-1 w-1 h-1 rounded-full bg-blue-500" />;
    case "assignment_rule":
      return <span className="absolute bottom-1 right-1 w-1 h-1 rounded-full bg-violet-500" />;
    case "department_pattern":
      return <span className="absolute bottom-1 right-1 w-1 h-1 rounded-full bg-slate-400" />;
    case "fixed":
    case "department_fixed":
      return <span className="absolute bottom-1 right-1 w-1 h-1 rounded-full bg-emerald-500" />;
    default:
      return null;
  }
};

const getHighestPrioritySource = (schedules: EffectiveSchedule[]): string | undefined => {
  const sources = schedules.map(s => s.source);
  if (sources.includes("roster")) return "roster";
  if (sources.includes("pattern")) return "pattern";
  if (sources.includes("assignment_rule")) return "assignment_rule";
  if (sources.includes("department_pattern")) return "department_pattern";
  if (sources.includes("fixed")) return "fixed";
  if (sources.includes("department_fixed")) return "department_fixed";
  return undefined;
};



interface MonthlyRosterGridProps {
  hook: ReturnType<typeof useShifts>;
  monthlyRoster: ReturnType<typeof useMonthlyRoster>;
}

// eslint-disable-next-line max-lines-per-function, complexity
export function MonthlyRosterGrid({ hook, monthlyRoster }: MonthlyRosterGridProps) {
  const { i18n, t } = useTranslation();
  const currentLocale = dateLocales[i18n.language] ?? vi;

  const gridContainerRef = useRef<HTMLDivElement | null>(null);
  const [showLeftArrow, setShowLeftArrow] = useState(false);
  const [showRightArrow, setShowRightArrow] = useState(false);

  const updateArrowVisibility = () => {
    if (gridContainerRef.current) {
      const { clientWidth, scrollLeft, scrollWidth } = gridContainerRef.current;
      setShowLeftArrow(scrollLeft > 10);
      setShowRightArrow(scrollLeft < scrollWidth - clientWidth - 10);
    }
  };

  const scrollGrid = (direction: "left" | "right") => {
    if (gridContainerRef.current) {
      const scrollAmount = 300;
      gridContainerRef.current.scrollBy({
        behavior: "smooth",
        left: direction === "left" ? -scrollAmount : scrollAmount,
      });
    }
  };

  const {
    monthDays,
    isMonthLoading,
    isMonthError,
    refetchMonthSchedules,
    shifts,
    employees,
    isEmployeesLoading,
    isEmployeesError,
    refetchEmployees,
  } = hook;

  const {
    selectedCells,
    isShiftPickerOpen,
    setIsShiftPickerOpen,
    selectedEmployeeIds,
    filteredSchedules,
    isDragging,
    handleCellMouseDown,
    handleCellMouseEnter,
    handleApplyShift,
  } = monthlyRoster;


  useEffect(() => {
    const container = gridContainerRef.current;
    if (!container) return;

    updateArrowVisibility();
    container.addEventListener("scroll", updateArrowVisibility);

    const resizeObserver = new ResizeObserver(() => {
      updateArrowVisibility();
    });
    resizeObserver.observe(container);

    return () => {
      container.removeEventListener("scroll", updateArrowVisibility);
      resizeObserver.disconnect();
    };
  }, [isMonthLoading, isEmployeesLoading, filteredSchedules.length]);

  return (
    <div className="space-y-4">

      {isShiftPickerOpen ? (
        <ShiftPickerModal
          isOpen={isShiftPickerOpen}
          onClose={() => { setIsShiftPickerOpen(false); }}
          onSelectShift={handleApplyShift}
          shifts={shifts}
          targetDates={Array.from(selectedCells).map(c => c.split(":")[1])}
        />
      ) : null}

      <div className="bg-white dark:bg-slate-900 rounded-[28px] border border-slate-100 dark:border-slate-800 shadow-sm overflow-hidden relative group/grid">
        {/* Scroll Left Button */}
        {showLeftArrow ? (
          <Button className="absolute left-4 top-1/2 -translate-y-1/2 z-30 p-3 rounded-full bg-white/90 dark:bg-slate-900/90 backdrop-blur-md border border-slate-250/60 dark:border-slate-800 shadow-premium-lg hover:bg-primary-50 dark:hover:bg-primary-950/40 text-slate-600 dark:text-slate-300 hover:text-primary-600 dark:hover:text-primary-400 hover:scale-105 active:scale-95 transition-all cursor-pointer opacity-0 group-hover/grid:opacity-100 focus:opacity-100"
            onClick={() => { scrollGrid("left"); }}
            title={t("common.previous")}
            type="button"
            variant="ghost"
          >
            <ChevronLeft size={16} />
          </Button>
        ) : null}

        {/* Scroll Right Button */}
        {showRightArrow ? (
          <Button className="absolute right-4 top-1/2 -translate-y-1/2 z-30 p-3 rounded-full bg-white/90 dark:bg-slate-900/90 backdrop-blur-md border border-slate-250/60 dark:border-slate-800 shadow-premium-lg hover:bg-primary-50 dark:hover:bg-primary-950/40 text-slate-600 dark:text-slate-300 hover:text-primary-600 dark:hover:text-primary-400 hover:scale-105 active:scale-95 transition-all cursor-pointer opacity-0 group-hover/grid:opacity-100 focus:opacity-100"
            onClick={() => { scrollGrid("right"); }}
            title={t("common.next")}
            type="button"
            variant="ghost"
          >
            <ChevronRight size={16} />
          </Button>
        ) : null}

        {isMonthError || isEmployeesError ? (
          <div className="p-12 text-center flex flex-col items-center justify-center gap-4 animate-in fade-in zoom-in duration-300">
            <div className="w-12 h-12 rounded-full bg-rose-50 dark:bg-rose-950/20 flex items-center justify-center border border-dashed border-rose-200 dark:border-rose-800/50">
              <AlertCircle className="text-rose-500" size={20} />
            </div>
            <div className="text-xs font-bold uppercase tracking-widest text-slate-700 dark:text-slate-200">
              {t("common.error")}
            </div>
            <Typography className="text-xs text-slate-450 dark:text-slate-500 max-w-xs mx-auto" variant="p">
              {t("shifts.monthly.failed_to_load")}
            </Typography>
            <Button className="mt-2 px-4 py-2 text-xs font-semibold text-white bg-primary-600 hover:bg-primary-700 active:scale-95 rounded-xl transition-all shadow-sm shadow-primary-500/10 cursor-pointer"
              onClick={() => {
                void refetchMonthSchedules();
                void refetchEmployees();
              }}
              variant="ghost"
            >
              {t("common.retry")}
            </Button>
          </div>
        ) : isMonthLoading || isEmployeesLoading || !employees ? (
          <div className="p-4 w-full">
            <div className="h-10 bg-slate-100 dark:bg-slate-800 rounded-xl mb-4 animate-pulse" />
            <div className="space-y-2">
              {Array.from({ length: 5 }).map((_, i) => (
                <div className="h-12 bg-slate-50 dark:bg-slate-800/50 rounded-xl animate-pulse" key={`skel-${i}`} />
              ))}
            </div>
          </div>
        ) : employees.length === 0 ? (
          <div className="p-12 text-center flex flex-col items-center justify-center gap-4">
            <div className="w-12 h-12 rounded-full bg-slate-50 dark:bg-slate-850 flex items-center justify-center border border-dashed border-slate-200 dark:border-slate-800">
              <CalendarRange className="text-slate-400" size={20} />
            </div>
            <div className="text-xs font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400">
              {t("shifts.monthly.all_employees")}
            </div>
            <Typography className="text-xs text-slate-400 dark:text-slate-500 max-w-xs mx-auto" variant="p">
              {t("shifts.monthly.empty_state_desc")}
            </Typography>
          </div>
        ) : selectedEmployeeIds.length === 0 ? (
          <div className="p-12 text-center flex flex-col items-center justify-center gap-4">
            <div className="w-12 h-12 rounded-full bg-slate-50 dark:bg-slate-850 flex items-center justify-center border border-dashed border-slate-200 dark:border-slate-800 animate-pulse text-slate-350 dark:text-slate-650">
              <User className="w-6 h-6 text-slate-450 dark:text-slate-500" />
            </div>
            <div className="text-xs font-bold uppercase tracking-widest text-slate-700 dark:text-slate-300">
              {t("shifts.empty_state.title")}
            </div>
            <Typography className="text-xs text-slate-450 dark:text-slate-500 max-w-xs mx-auto" variant="p">
              {t("shifts.empty_state.desc")}
            </Typography>
          </div>
        ) : filteredSchedules.length === 0 ? (
          <div className="p-12 text-center flex flex-col items-center justify-center gap-4">
            <div className="w-12 h-12 rounded-full bg-slate-50 dark:bg-slate-850 flex items-center justify-center border border-dashed border-slate-200 dark:border-slate-800">
              <Search className="text-slate-400" size={20} />
            </div>
            <div className="text-xs font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400">
              {t("common.combobox.no_results")}
            </div>
            <Typography className="text-xs text-slate-450 dark:text-slate-500 max-w-xs mx-auto" variant="p">
              {t("shifts.monthly.no_results_desc")}
            </Typography>
          </div>
        ) : (
          <RosterGridTable
            containerRef={gridContainerRef}
            currentLocale={currentLocale}
            employeesLabel={t("shifts.monthly.all_employees")}
            isDragging={isDragging}
            monthDays={monthDays}
            monthSchedules={filteredSchedules}
            onCellMouseDown={handleCellMouseDown}
            onCellMouseEnter={handleCellMouseEnter}
            selectedCells={selectedCells}
            shifts={shifts}
          />
        )}
      </div>
    </div>
  );
}



interface RosterGridTableProps {
  containerRef?: React.RefObject<HTMLDivElement | null>;
  currentLocale: Locale;
  employeesLabel: string;
  isDragging: boolean;
  monthDays: Date[];
  monthSchedules: EmployeeSchedule[];
  onCellMouseDown: (employeeId: string, dateStr: string, e: React.MouseEvent) => void;
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
            {monthDays.map(day => {
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
                  <div className={`text-[10px] font-bold uppercase tracking-tighter ${isToday ? "text-primary-600 dark:text-primary-400" : "text-slate-400 dark:text-slate-500"}`}>
                    {dayName}
                  </div>
                  <div className={`text-sm font-extrabold mt-0.5 ${
                    isToday 
                      ? "text-primary-600 dark:text-primary-400 bg-primary-100 dark:bg-primary-900/60 rounded-full w-6 h-6 flex items-center justify-center mx-auto" 
                      : isWe 
                        ? "text-amber-600 dark:text-amber-500" 
                        : "text-slate-600 dark:text-slate-300"
                  }`}>
                    {dayNum}
                  </div>
                </th>
              );
            })}
          </tr>
        </thead>
        <tbody>
          {monthSchedules.map(empSchedule => (
            <tr
              className="border-b border-slate-100 dark:border-slate-800 last:border-b-0 hover:bg-slate-50/30 dark:hover:bg-slate-850/10"
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

              {monthDays.map(day => {
                const dateStr = format(day, "yyyy-MM-dd");
                const cellKey = `${empSchedule.employee_id ?? ""}:${dateStr}`;
                const isSelected = selectedCells.has(cellKey);
                const daySchedules = empSchedule.schedules.filter((s) => s.date === dateStr);

                return (
                  <RosterCell
                    dateStr={dateStr}
                    daySchedules={daySchedules}
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

interface RosterCellProps {
  dateStr: string;
  daySchedules: EffectiveSchedule[];
  employeeId: string;
  isSelected: boolean;
  onMouseDown: (employeeId: string, dateStr: string, e: React.MouseEvent) => void;
  onMouseEnter: (employeeId: string, dateStr: string) => void;
  shifts?: Shift[];
}

const RosterCell = memo(function RosterCell({
  dateStr,
  employeeId,
  isSelected,
  onMouseDown,
  onMouseEnter,
  daySchedules,
  shifts,
}: RosterCellProps) {
  const handleMouseDown = (e: React.MouseEvent) => {
    e.preventDefault(); // Prevent text highlighting while dragging
    onMouseDown(employeeId, dateStr, e);
  };

  const handleMouseEnter = () => {
    onMouseEnter(employeeId, dateStr);
  };

  const workingSchedules = daySchedules.filter((s) => s.shift_id);
  const hasShift = workingSchedules.length > 0;
  const isHoliday = daySchedules.some((s) => s.is_holiday);
  const holidayName = daySchedules.find((s) => s.holiday_name)?.holiday_name;

  const highestSource = getHighestPrioritySource(daySchedules);
  const cellStyles = getCellStyles(highestSource, isSelected);

  const allTraces = daySchedules.flatMap((s) => s.traces ?? []);
  const hasTraces = allTraces.length > 0;

  return (
    <td
      className={cellStyles}
      onMouseDown={handleMouseDown}
      onMouseEnter={handleMouseEnter}
    >
      <div className="py-2.5 px-1 min-h-[44px] flex flex-col justify-center items-center gap-1">
        {hasShift ? (
          <div className="flex flex-col gap-1 items-center">
            {workingSchedules.slice(0, 3).map((s, idx) => {
              const shiftConfig = shifts?.find((sh) => sh.id === s.shift_id);
              const shiftSymbol = getShiftSymbol(s.shift_name, shiftConfig?.report_symbol);
              const shiftBgClass = s.source === "roster"
                ? "bg-teal-100/60 dark:bg-teal-950/60 text-teal-800 dark:text-teal-200"
                : "bg-slate-100/50 dark:bg-slate-800 text-slate-700 dark:text-slate-300";
              return (
                <span
                  className={`text-[9px] font-extrabold tracking-tight px-1.5 py-0.5 rounded-md ${shiftBgClass}`}
                  key={idx}
                  title={`${s.shift_name ?? ""} (${s.start_time?.substring(0, 5) ?? ""} - ${s.end_time?.substring(0, 5) ?? ""})`}
                >
                  {shiftSymbol}
                </span>
              );
            })}
            {workingSchedules.length > 3 && (
              <span className="text-[8px] font-bold text-slate-400 dark:text-slate-500">
                +{workingSchedules.length - 3} more
              </span>
            )}
          </div>
        ) : (
          <span className="text-[10px] text-slate-350 dark:text-slate-600 font-bold">—</span>
        )}

        {getSourceIndicator(highestSource)}

        {isHoliday ? (
          <span className="absolute top-1 left-1 text-[8px] bg-amber-100 dark:bg-amber-950 text-amber-600 rounded-full px-0.5 font-extrabold scale-90 border border-amber-200 dark:border-amber-900" title={holidayName ?? "Lễ"}>
            🎉
          </span>
        ) : null}

        {hasTraces && allTraces.length > 0 ? (
          <div className="absolute top-0.5 right-0.5 z-30 opacity-0 hover:opacity-100 group-hover:opacity-60 transition-opacity">
            <ScheduleTracePopover 
              schedules={workingSchedules.map(s => {
                const shiftConfig = shifts?.find((sh) => sh.id === s.shift_id);
                const shiftSymbol = getShiftSymbol(s.shift_name, shiftConfig?.report_symbol);
                return {
                  shiftName: s.shift_name,
                  shiftSymbol,
                  startTime: s.start_time,
                  endTime: s.end_time,
                };
              })}
              source={highestSource ?? "none"}
              traces={allTraces}
            />
          </div>
        ) : null}
      </div>
    </td>
  );
});
RosterCell.displayName = "RosterCell";
