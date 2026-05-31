import { 
  AlertCircle,
  CalendarDays,
  CalendarRange,
  ChevronLeft,
  ChevronRight,
  Copy,
  Loader2,
  Search,
  Sparkles,
  Trash2,
  User
} from "lucide-react";
import React, { memo, useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";

import { format, isToday as isDateToday } from "date-fns";
import { vi, enUS, zhCN, type Locale } from "date-fns/locale";

import { EmployeeSearch } from "@/components/common/employee-search";
import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { EffectiveSchedule, Employee, EmployeeSchedule, Shift } from "@/lib/api/types";
import { useShifts } from "@/pages/shifts/hooks/use-shifts";

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

interface RosterCellData {
  cellStyles: string;
  hasShift: boolean;
  hasTraces: boolean;
  holidayName?: string | null;
  isHoliday: boolean;
  shiftBgClass: string;
  shiftSymbol: string;
  source?: string;
  traces?: EffectiveSchedule["traces"];
}

const computeCellData = (
  daySchedule: EffectiveSchedule | undefined,
  isSelected: boolean,
  shifts?: Shift[]
): RosterCellData => {
  if (!daySchedule) {
    return {
      cellStyles: getCellStyles(undefined, isSelected),
      hasShift: false,
      hasTraces: false,
      isHoliday: false,
      shiftBgClass: "bg-slate-100/50 dark:bg-slate-800 text-slate-700 dark:text-slate-300",
      shiftSymbol: "",
    };
  }

  const shiftId = daySchedule.shift_id;
  const hasShift = !!shiftId;
  const isHoliday = daySchedule.is_holiday ?? false;
  const holidayName = daySchedule.holiday_name;
  
  let shiftSymbol = "";
  if (hasShift) {
    const shiftConfig = shifts?.find((s: any) => s.id === shiftId);
    shiftSymbol = getShiftSymbol(daySchedule.shift_name, shiftConfig?.report_symbol);
  }

  const source = daySchedule.source;
  const shiftBgClass = source === "roster"
    ? "bg-teal-100/60 dark:bg-teal-950/60 text-teal-800 dark:text-teal-200"
    : "bg-slate-100/50 dark:bg-slate-800 text-slate-700 dark:text-slate-300";

  const traces = daySchedule.traces || [];
  const hasTraces = traces.length > 0;

  return {
    cellStyles: getCellStyles(source, isSelected),
    hasShift,
    hasTraces,
    holidayName,
    isHoliday,
    shiftBgClass,
    shiftSymbol,
    source,
    traces,
  };
};



interface MonthlyRosterGridProps {
  hook: ReturnType<typeof useShifts>;
}

// eslint-disable-next-line max-lines-per-function, complexity
export function MonthlyRosterGrid({ hook }: MonthlyRosterGridProps) {
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
    currentMonth,
    handleNavigateMonth,
    monthSchedules,
    isMonthLoading,
    isMonthError,
    refetchMonthSchedules,
    shifts,
    handleMonthBulkAssign,
    handleMonthDeleteRoster,
    handleCopyPreviousMonth,
    employees,
    isEmployeesLoading,
    isEmployeesError,
    refetchEmployees,
  } = hook;

  const [selectedCells, setSelectedCells] = useState<Set<string>>(new Set());
  const [isShiftPickerOpen, setIsShiftPickerOpen] = useState(false);
  const [isCopying, setIsCopying] = useState(false);
  const [isMutating, setIsMutating] = useState(false);

  const [selectedEmployeeIds, setSelectedEmployeeIds] = useState<string[]>([]);
  const [selectedEmployees, setSelectedEmployees] = useState<Employee[]>([]);

  const monthHeaderLabel = format(currentMonth, "MMMM yyyy", { locale: currentLocale });

  const selectedEmployeesMeta = useMemo(() => {
    const list = [...selectedEmployees];
    if (employees) {
      const empMap = new Map(employees.map(e => [e.id, e]));
      selectedEmployeeIds.forEach(id => {
        if (!list.some(e => e.id === id)) {
          const emp = empMap.get(id);
          if (emp) list.push(emp);
        }
      });
    }
    return list;
  }, [employees, selectedEmployeeIds, selectedEmployees]);

  const combinedSchedules = useMemo(() => {
    if (!employees) return [];
    const list = [...employees];
    selectedEmployees.forEach(selectedEmp => {
      if (!list.some(e => e.id === selectedEmp.id)) {
        list.push(selectedEmp);
      }
    });
    return list.map(emp => {
      const existing = monthSchedules?.find((s: any) => s.employee_id === emp.id);
      if (existing) {
        return existing;
      }
      return {
        employee_id: emp.id,
        full_name: emp.full_name,
        schedules: []
      };
    });
  }, [employees, monthSchedules, selectedEmployees]);

  const filteredSchedules = useMemo(() => {
    if (selectedEmployeeIds.length === 0) return [];
    return combinedSchedules.filter(schedule => 
      selectedEmployeeIds.includes(schedule.employee_id)
    );
  }, [combinedSchedules, selectedEmployeeIds]);

  const [dragStartCell, setDragStartCell] = useState<string | null>(null);
  const [isDragging, setIsDragging] = useState(false);
  const [dragSnapshot, setDragSnapshot] = useState<Set<string>>(new Set());

  useEffect(() => {
    const handleGlobalMouseUp = () => {
      setIsDragging(false);
      setDragStartCell(null);
    };

    if (isDragging) {
      window.addEventListener("mouseup", handleGlobalMouseUp);
      return () => {
        window.removeEventListener("mouseup", handleGlobalMouseUp);
      };
    }
  }, [isDragging]);

  const handleCellMouseDown = (employeeId: string, dateStr: string, e: React.MouseEvent) => {
    if (e.button !== 0) return; // Only left click
    const key = `${employeeId}:${dateStr}`;
    setIsDragging(true);
    setDragStartCell(key);
    
    setSelectedCells(prev => {
      const next = new Set(prev);
      if (next.has(key)) {
        next.delete(key);
      } else {
        next.add(key);
      }
      setDragSnapshot(new Set(next));
      return next;
    });
  };

  const handleCellMouseEnter = (employeeId: string, dateStr: string) => {
    if (!isDragging || !dragStartCell) return;

    const [startEmpId, startDateStr] = dragStartCell.split(":");

    const startEmpIdx = filteredSchedules.findIndex((s: any) => s.employee_id === startEmpId);
    const endEmpIdx = filteredSchedules.findIndex((s: any) => s.employee_id === employeeId);

    if (startEmpIdx === -1 || endEmpIdx === -1) return;

    const startDayIdx = monthDays.findIndex(d => format(d, "yyyy-MM-dd") === startDateStr);
    const endDayIdx = monthDays.findIndex(d => format(d, "yyyy-MM-dd") === dateStr);

    if (startDayIdx === -1 || endDayIdx === -1) return;

    const minEmpIdx = Math.min(startEmpIdx, endEmpIdx);
    const maxEmpIdx = Math.max(startEmpIdx, endEmpIdx);
    const minDayIdx = Math.min(startDayIdx, endDayIdx);
    const maxDayIdx = Math.max(startDayIdx, endDayIdx);

    setSelectedCells(() => {
      const next = new Set(dragSnapshot);
      for (let i = minEmpIdx; i <= maxEmpIdx; i++) {
        const empId = filteredSchedules[i].employee_id;
        for (let j = minDayIdx; j <= maxDayIdx; j++) {
          const dStr = format(monthDays[j], "yyyy-MM-dd");
          next.add(`${empId}:${dStr}`);
        }
      }
      return next;
    });
  };

  const handleApplyShift = async (shiftId: string | null) => {
    setIsMutating(true);
    try {
      const groups = Array.from(selectedCells).reduce<Record<string, string[] | undefined>>((acc, key) => {
        const [empId, d] = key.split(":");
        const list = acc[empId];
        if (list) {
          list.push(d);
        } else {
          acc[empId] = [d];
        }
        return acc;
      }, {});
      const assignments = Object.entries(groups).map(([employeeId, dates]) => ({
        employeeId,
        dates: dates ?? [],
      }));
      await handleMonthBulkAssign(assignments, shiftId);
      setSelectedCells(new Set());
    } finally {
      setIsMutating(false);
    }
  };

  const handleClearSelectedOverrides = async () => {
    setIsMutating(true);
    try {
      const groups = Array.from(selectedCells).reduce<Record<string, string[] | undefined>>((acc, key) => {
        const [empId, d] = key.split(":");
        const list = acc[empId];
        if (list) {
          list.push(d);
        } else {
          acc[empId] = [d];
        }
        return acc;
      }, {});
      const assignments = Object.entries(groups).map(([employeeId, dates]) => ({
        employeeId,
        dates: dates ?? [],
      }));
      await handleMonthDeleteRoster(assignments);
      setSelectedCells(new Set());
    } finally {
      setIsMutating(false);
    }
  };

  const handleCopyMonth = async () => {
    setIsCopying(true);
    try {
      await handleCopyPreviousMonth();
    } finally {
      setIsCopying(false);
    }
  };

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
      <div className="relative">
        <RosterToolbar
          copyLabel={t("shifts.monthly.copy_previous_month")}
          isCopying={isCopying}
          monthHeaderLabel={monthHeaderLabel}
          onCopyMonth={handleCopyMonth}
          onNavigateMonth={handleNavigateMonth}
          onSelectedEmployeesChange={(ids, emps) => {
            setSelectedEmployeeIds(ids);
            if (emps) setSelectedEmployees(emps);
          }}
          searchPlaceholder={t("shifts.monthly.search_placeholder")}
          selectedEmployeeIds={selectedEmployeeIds}
          selectedEmployeesMeta={selectedEmployeesMeta}
          title={t("shifts.monthly.title")}
          workspaceId={hook.wsId}
        />

        {selectedCells.size > 0 ? (
          <RosterActionBar
            assignLabel={t("shifts.monthly.assign_selected")}
            cancelLabel={t("common.cancel")}
            clearLabel={t("shifts.monthly.clear_month")}
            isMutating={isMutating}
            onAssignSelected={() => { setIsShiftPickerOpen(true); }}
            onCancel={() => { setSelectedCells(new Set()); }}
            onClearOverrides={handleClearSelectedOverrides}
            selectedCount={selectedCells.size}
            selectedLabel={t("shifts.monthly.cells_selected", { count: selectedCells.size })}
          />
        ) : null}
      </div>

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

interface RosterToolbarProps {
  copyLabel: string;
  isCopying: boolean;
  monthHeaderLabel: string;
  onCopyMonth: () => void;
  onNavigateMonth: (dir: "prev" | "next") => void;
  onSelectedEmployeesChange: (ids: string[], selectedEmps?: Employee[]) => void;
  searchPlaceholder: string;
  selectedEmployeeIds: string[];
  selectedEmployeesMeta: Employee[];
  title: string;
  workspaceId: string;
}

// eslint-disable-next-line max-lines-per-function
const RosterToolbar = memo(function RosterToolbar({
  copyLabel,
  isCopying,
  monthHeaderLabel,
  onCopyMonth,
  onNavigateMonth,
  title,
  searchPlaceholder,
  workspaceId,
  selectedEmployeeIds,
  onSelectedEmployeesChange,
  selectedEmployeesMeta,
}: RosterToolbarProps) {
  const { t } = useTranslation();
  const selectedMetaRef = useRef<Employee[]>(selectedEmployeesMeta);

  useEffect(() => {
    selectedMetaRef.current = selectedEmployeesMeta;
  }, [selectedEmployeesMeta]);

  return (
    <div className="flex flex-col gap-4 bg-white dark:bg-slate-900 p-5 rounded-[28px] border border-slate-100 dark:border-slate-800 shadow-sm">
      {/* Row 1: Header, Navigation & Copy Action */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3">
        <div className="flex items-center gap-2">
          <CalendarDays className="text-primary-600 dark:text-primary-400" size={20} />
          <Typography className="text-sm font-bold uppercase tracking-wider text-slate-700 dark:text-slate-200" variant="h2">
            {title}
          </Typography>
        </div>

        <div className="flex items-center justify-between sm:justify-end gap-3 w-full sm:w-auto">
          <div className="flex items-center gap-1">
            <Button className="p-2 rounded-xl hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors border border-slate-100 dark:border-slate-800"
              onClick={() => { onNavigateMonth("prev"); }}
              type="button"
              variant="ghost"
            >
              <ChevronLeft size={16} />
            </Button>
            <span className="min-w-[140px] text-center font-bold text-xs uppercase tracking-widest text-slate-700 dark:text-slate-200">
              {monthHeaderLabel}
            </span>
            <Button className="p-2 rounded-xl hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors border border-slate-100 dark:border-slate-800"
              onClick={() => { onNavigateMonth("next"); }}
              type="button"
              variant="ghost"
            >
              <ChevronRight size={16} />
            </Button>
          </div>

          <Button className="flex items-center gap-1.5 px-3 py-2 text-xs font-bold uppercase tracking-wider border border-slate-200 dark:border-slate-800 rounded-xl hover:bg-slate-50 dark:hover:bg-slate-800/80 transition-colors text-slate-600 dark:text-slate-300 disabled:opacity-50"
            disabled={isCopying}
            onClick={onCopyMonth}
            type="button"
            variant="ghost"
          >
            {isCopying ? <Loader2 className="animate-spin" size={12} /> : <Copy size={12} />}
            <span>{copyLabel}</span>
          </Button>
        </div>
      </div>

      {/* Separator line */}
      <div className="h-px bg-slate-100 dark:bg-slate-800 w-full" />

      {/* Row 2: Multi-select Search & Checklist */}
      <div className="flex flex-col gap-3.5 w-full animate-in fade-in duration-200">
        {/* Multi-select search bar */}
        <EmployeeSearch
          clearOnEscape
          hideChips
          multiple
          onChange={(val) => {
            const ids = Array.isArray(val) ? val : (val ? [val] : []);
            selectedMetaRef.current = selectedMetaRef.current.filter(e => ids.includes(e.id));
            onSelectedEmployeesChange(ids, selectedMetaRef.current);
          }}
          onSelectEmployee={(emp) => {
            if (!selectedMetaRef.current.some(e => e.id === emp.id)) {
              selectedMetaRef.current = [...selectedMetaRef.current, emp];
            }
          }}
          placeholder={searchPlaceholder}
          value={selectedEmployeeIds}
          workspaceId={workspaceId}
        />

        {/* Selected Checklist (Stick) */}
        {selectedEmployeesMeta.length > 0 && (
          <div className="flex flex-wrap gap-2.5 p-3.5 bg-slate-50 dark:bg-slate-950/40 rounded-2xl border border-slate-200/50 dark:border-slate-800/50 animate-in fade-in slide-in-from-top-2 duration-300">
            <div className="w-full text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest mb-1">
              {t("shifts.monthly.selected_employees", "Nhân sự đang chọn ({{count}})", { count: selectedEmployeesMeta.length })}
            </div>
            {selectedEmployeesMeta.map((emp) => (
              <label
                className="flex items-center gap-2 px-3 py-1.5 bg-white dark:bg-slate-900 border border-slate-250/60 dark:border-slate-800 rounded-xl text-xs font-bold text-slate-700 dark:text-slate-200 shadow-sm hover:border-primary-500/50 cursor-pointer select-none transition-all"
                key={emp.id}
              >
                <input
                  checked={selectedEmployeeIds.includes(emp.id)}
                  className="rounded border-slate-350 dark:border-slate-700 text-primary-600 focus:ring-primary-500 cursor-pointer h-3.5 w-3.5"
                  onChange={() => {
                    const newIds = selectedEmployeeIds.filter((id) => id !== emp.id);
                    selectedMetaRef.current = selectedMetaRef.current.filter(e => e.id !== emp.id);
                    onSelectedEmployeesChange(newIds, selectedMetaRef.current);
                  }}
                  type="checkbox"
                />
                <span>{emp.full_name}</span>
              </label>
            ))}
          </div>
        )}
      </div>
    </div>
  );
});

RosterToolbar.displayName = "RosterToolbar";

interface RosterActionBarProps {
  assignLabel: string;
  cancelLabel: string;
  clearLabel: string;
  isMutating: boolean;
  onAssignSelected: () => void;
  onCancel: () => void;
  onClearOverrides: () => void;
  selectedCount: number;
  selectedLabel: string;
}

const RosterActionBar = memo(function RosterActionBar({
  assignLabel,
  cancelLabel,
  clearLabel,
  isMutating,
  onAssignSelected,
  onCancel,
  onClearOverrides,
  selectedLabel,
}: RosterActionBarProps) {
  return (
    <div className="absolute inset-0 z-40 bg-primary-600 text-white px-6 py-4 rounded-[28px] shadow-lg flex flex-col sm:flex-row sm:items-center justify-between gap-4 animate-in fade-in duration-200">
      <div className="flex items-center gap-3">
        <div className="w-6 h-6 rounded-full bg-white/10 flex items-center justify-center">
          <Sparkles className="text-white animate-pulse" size={12} />
        </div>
        <span className="text-xs font-bold uppercase tracking-widest">
          {selectedLabel}
        </span>
      </div>

      <div className="flex items-center gap-2.5">
        <Button className="px-4 py-2 bg-white text-primary-700 hover:bg-white/90 active:scale-95 rounded-xl text-[10px] font-extrabold uppercase tracking-widest transition-all disabled:opacity-50 cursor-pointer shadow-sm"
          disabled={isMutating}
          onClick={onAssignSelected}
          variant="ghost"
        >
          {assignLabel}
        </Button>

        <Button className="p-2 border border-white/20 hover:bg-white/10 text-white rounded-xl transition-all active:scale-90 disabled:opacity-50 cursor-pointer"
          disabled={isMutating}
          onClick={onClearOverrides}
          title={clearLabel}
          variant="ghost"
        >
          <Trash2 size={14} />
        </Button>

        <Button className="px-4 py-2 text-[10px] font-bold uppercase tracking-wider text-white/80 hover:text-white hover:bg-white/10 rounded-xl transition-all cursor-pointer"
          onClick={onCancel}
          variant="ghost"
        >
          {cancelLabel}
        </Button>
      </div>
    </div>
  );
});
RosterActionBar.displayName = "RosterActionBar";

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

// eslint-disable-next-line max-lines-per-function
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
              key={empSchedule.employee_id}
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
                const cellKey = `${empSchedule.employee_id}:${dateStr}`;
                const isSelected = selectedCells.has(cellKey);
                const daySchedule = empSchedule.schedules.find((s: any) => s.date === dateStr);
                const cellData = computeCellData(daySchedule, isSelected, shifts);

                return (
                  <RosterCell
                    cellStyles={cellData.cellStyles}
                    dateStr={dateStr}
                    employeeId={empSchedule.employee_id}
                    hasShift={cellData.hasShift}
                    hasTraces={cellData.hasTraces}
                    holidayName={cellData.holidayName}
                    isHoliday={cellData.isHoliday}
                    isSelected={isSelected}
                    key={dateStr}
                    onMouseDown={onCellMouseDown}
                    onMouseEnter={onCellMouseEnter}
                    shiftBgClass={cellData.shiftBgClass}
                    shiftSymbol={cellData.shiftSymbol}
                    source={cellData.source}
                    traces={cellData.traces}
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
  cellStyles: string;
  dateStr: string;
  employeeId: string;
  hasShift: boolean;
  hasTraces: boolean;
  holidayName?: string | null;
  isHoliday: boolean;
  isSelected: boolean;
  onMouseDown: (employeeId: string, dateStr: string, e: React.MouseEvent) => void;
  onMouseEnter: (employeeId: string, dateStr: string) => void;
  shiftBgClass: string;
  shiftSymbol: string;
  source?: string;
  traces?: EffectiveSchedule["traces"];
}

const RosterCell = memo(function RosterCell({
  cellStyles,
  dateStr,
  employeeId,
  hasShift,
  hasTraces,
  holidayName,
  isHoliday,
  onMouseDown,
  onMouseEnter,
  shiftBgClass,
  shiftSymbol,
  source,
  traces,
}: RosterCellProps) {
  const handleMouseDown = (e: React.MouseEvent) => {
    e.preventDefault(); // Prevent text highlighting while dragging
    onMouseDown(employeeId, dateStr, e);
  };

  const handleMouseEnter = () => {
    onMouseEnter(employeeId, dateStr);
  };

  const handleTraceClick = (e: React.MouseEvent) => {
    e.stopPropagation();
  };

  return (
    <td
      className={cellStyles}
      onMouseDown={handleMouseDown}
      onMouseEnter={handleMouseEnter}
    >
      <div className="py-2.5 px-1 min-h-[44px] flex flex-col justify-center items-center">
        {hasShift ? (
          <span className={`text-[10px] font-extrabold tracking-tight px-1.5 py-0.5 rounded-md ${shiftBgClass}`}>
            {shiftSymbol}
          </span>
        ) : (
          <span className="text-[10px] text-slate-350 dark:text-slate-600 font-bold">—</span>
        )}

        {getSourceIndicator(source)}

        {isHoliday ? (
          <span className="absolute top-1 left-1 text-[8px] bg-amber-100 dark:bg-amber-950 text-amber-600 rounded-full px-0.5 font-extrabold scale-90 border border-amber-200 dark:border-amber-900" title={holidayName ?? "Lễ"}>
            🎉
          </span>
        ) : null}

        {hasTraces && traces ? (
          <Button className="absolute top-0.5 right-0.5 opacity-0 hover:opacity-100 group-hover:opacity-60 transition-opacity bg-transparent border-none p-0 cursor-default" 
            onClick={handleTraceClick}
            type="button"
            variant="ghost"
          >
            <ScheduleTracePopover 
              source={source ?? "none"} 
              traces={traces} 
            />
          </Button>
        ) : null}
      </div>
    </td>
  );
});
RosterCell.displayName = "RosterCell";
