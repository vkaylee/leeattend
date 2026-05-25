import * as Popover from "@radix-ui/react-popover";
import { 
  AlertCircle,
  CalendarDays,
  CalendarRange,
  ChevronLeft,
  ChevronRight,
  Clock,
  Copy,
  Loader2,
  Search,
  Sparkles,
  Trash2,
  User
} from "lucide-react";
import React, { memo, useMemo, useState, useEffect } from "react";
import { useTranslation } from "react-i18next";

import { format, isToday as isDateToday } from "date-fns";
import { vi, enUS, zhCN, type Locale } from "date-fns/locale";

import { EmployeeSearch } from "@/components/common/employee-search";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { EffectiveSchedule, Employee, EmployeeSchedule, Shift } from "@/lib/api/types";
import { useShifts } from "@/pages/shifts/hooks/use-shifts";

import { ScheduleTracePopover } from "./schedule-trace-popover";


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
    return baseStyles + "ring-2 ring-primary-500 ring-inset bg-primary-50 dark:bg-primary-950/40 z-10";
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
    const shiftConfig = shifts?.find(s => s.id === shiftId);
    shiftSymbol = getShiftSymbol(daySchedule.shift_name, shiftConfig?.report_symbol);
  }

  const source = daySchedule.source;
  const shiftBgClass = source === "roster"
    ? "bg-teal-100/60 dark:bg-teal-950/60 text-teal-800 dark:text-teal-200"
    : "bg-slate-100/50 dark:bg-slate-800 text-slate-700 dark:text-slate-300";

  const traces = daySchedule.traces;
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
  const { formatTime } = useDateTimeFormatter();

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

  const monthHeaderLabel = format(currentMonth, "MMMM yyyy", { locale: currentLocale });

  const selectedEmployeesMeta = useMemo(() => {
    if (!employees) return [];
    return employees.filter(emp => selectedEmployeeIds.includes(emp.id));
  }, [employees, selectedEmployeeIds]);

  const combinedSchedules = useMemo(() => {
    if (!employees) return [];
    return employees.map(emp => {
      const existing = monthSchedules?.find(s => s.employee_id === emp.id);
      if (existing) {
        return existing;
      }
      return {
        employee_id: emp.id,
        full_name: emp.full_name,
        schedules: []
      };
    });
  }, [employees, monthSchedules]);

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

    const startEmpIdx = filteredSchedules.findIndex(s => s.employee_id === startEmpId);
    const endEmpIdx = filteredSchedules.findIndex(s => s.employee_id === employeeId);

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

  return (
    <div className="space-y-4">
      <RosterToolbar
        copyLabel={t("shifts.monthly.copy_previous_month")}
        isCopying={isCopying}
        monthHeaderLabel={monthHeaderLabel}
        onCopyMonth={handleCopyMonth}
        onNavigateMonth={handleNavigateMonth}
        onSelectedEmployeesChange={setSelectedEmployeeIds}
        searchPlaceholder={t("shifts.monthly.search_placeholder")}
        selectedEmployeeIds={selectedEmployeeIds}
        selectedEmployeesMeta={selectedEmployeesMeta}
        title={t("shifts.monthly.title")}
        workspaceId={hook.wsId}
      />

      <div className="bg-white dark:bg-slate-900 rounded-[28px] border border-slate-100 dark:border-slate-800 shadow-sm overflow-hidden">
        {isMonthError || isEmployeesError ? (
          <div className="p-12 text-center flex flex-col items-center justify-center gap-4 animate-in fade-in zoom-in duration-300">
            <div className="w-12 h-12 rounded-full bg-rose-50 dark:bg-rose-950/20 flex items-center justify-center border border-dashed border-rose-200 dark:border-rose-800/50">
              <AlertCircle className="text-rose-500" size={20} />
            </div>
            <div className="text-xs font-bold uppercase tracking-widest text-slate-700 dark:text-slate-200">
              {t("common.error")}
            </div>
            <p className="text-xs text-slate-450 dark:text-slate-500 max-w-xs mx-auto">
              {t("shifts.monthly.failed_to_load")}
            </p>
            <button
              className="mt-2 px-4 py-2 text-xs font-semibold text-white bg-primary-600 hover:bg-primary-700 active:scale-95 rounded-xl transition-all shadow-sm shadow-primary-500/10 cursor-pointer"
              onClick={() => {
                void refetchMonthSchedules();
                void refetchEmployees();
              }}
            >
              {t("common.retry")}
            </button>
          </div>
        ) : isMonthLoading || isEmployeesLoading || !employees ? (
          <div className="p-12 flex flex-col items-center justify-center gap-3">
            <Loader2 className="animate-spin text-primary-600" size={32} />
            <span className="text-xs text-slate-500 dark:text-slate-400 font-bold uppercase tracking-widest animate-pulse">
              {t("common.loading")}...
            </span>
          </div>
        ) : employees.length === 0 ? (
          <div className="p-12 text-center flex flex-col items-center justify-center gap-4">
            <div className="w-12 h-12 rounded-full bg-slate-50 dark:bg-slate-850 flex items-center justify-center border border-dashed border-slate-200 dark:border-slate-800">
              <CalendarRange className="text-slate-400" size={20} />
            </div>
            <div className="text-xs font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400">
              {t("shifts.monthly.all_employees")}
            </div>
            <p className="text-xs text-slate-400 dark:text-slate-500 max-w-xs mx-auto">
              {t("shifts.monthly.empty_state_desc")}
            </p>
          </div>
        ) : selectedEmployeeIds.length === 0 ? (
          <div className="p-12 text-center flex flex-col items-center justify-center gap-4">
            <div className="w-12 h-12 rounded-full bg-slate-50 dark:bg-slate-850 flex items-center justify-center border border-dashed border-slate-200 dark:border-slate-800 animate-pulse text-slate-350 dark:text-slate-650">
              <User className="w-6 h-6 text-slate-450 dark:text-slate-500" />
            </div>
            <div className="text-xs font-bold uppercase tracking-widest text-slate-700 dark:text-slate-300">
              {t("shifts.empty_state.title")}
            </div>
            <p className="text-xs text-slate-450 dark:text-slate-500 max-w-xs mx-auto">
              {t("shifts.empty_state.desc")}
            </p>
          </div>
        ) : filteredSchedules.length === 0 ? (
          <div className="p-12 text-center flex flex-col items-center justify-center gap-4">
            <div className="w-12 h-12 rounded-full bg-slate-50 dark:bg-slate-850 flex items-center justify-center border border-dashed border-slate-200 dark:border-slate-800">
              <Search className="text-slate-400" size={20} />
            </div>
            <div className="text-xs font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400">
              {t("common.combobox.no_results")}
            </div>
            <p className="text-xs text-slate-400 dark:text-slate-500 max-w-xs mx-auto">
              {t("shifts.monthly.no_results_desc")}
            </p>
          </div>
        ) : (
          <RosterGridTable
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

      {selectedCells.size > 0 && (
        <RosterActionBar
          assignLabel={t("shifts.monthly.assign_selected")}
          cancelLabel={t("common.cancel")}
          clearLabel={t("shifts.monthly.clear_month")}
          formatTime={formatTime}
          handleApplyShift={handleApplyShift}
          isMutating={isMutating}
          isShiftPickerOpen={isShiftPickerOpen}
          onCancel={() => { setSelectedCells(new Set()); }}
          onClearOverrides={handleClearSelectedOverrides}
          selectedCount={selectedCells.size}
          selectedLabel={t("shifts.monthly.cells_selected", { count: selectedCells.size })}
          setIsShiftPickerOpen={setIsShiftPickerOpen}
          shifts={shifts}
        />
      )}
    </div>
  );
}

interface RosterToolbarProps {
  copyLabel: string;
  isCopying: boolean;
  monthHeaderLabel: string;
  onCopyMonth: () => void;
  onNavigateMonth: (dir: "prev" | "next") => void;
  onSelectedEmployeesChange: (ids: string[]) => void;
  searchPlaceholder: string;
  selectedEmployeeIds: string[];
  selectedEmployeesMeta: Employee[];
  title: string;
  workspaceId: string;
}

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

  return (
    <div className="flex flex-col gap-4 bg-white dark:bg-slate-900 p-5 rounded-[28px] border border-slate-100 dark:border-slate-800 shadow-sm">
      {/* Row 1: Header, Navigation & Copy Action */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3">
        <div className="flex items-center gap-2">
          <CalendarDays className="text-primary-600 dark:text-primary-400" size={20} />
          <h2 className="text-sm font-bold uppercase tracking-wider text-slate-700 dark:text-slate-200">
            {title}
          </h2>
        </div>

        <div className="flex items-center justify-between sm:justify-end gap-3 w-full sm:w-auto">
          <div className="flex items-center gap-1">
            <button
              className="p-2 rounded-xl hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors border border-slate-100 dark:border-slate-800"
              onClick={() => { onNavigateMonth("prev"); }}
              type="button"
            >
              <ChevronLeft size={16} />
            </button>
            <span className="min-w-[140px] text-center font-bold text-xs uppercase tracking-widest text-slate-700 dark:text-slate-200">
              {monthHeaderLabel}
            </span>
            <button
              className="p-2 rounded-xl hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors border border-slate-100 dark:border-slate-800"
              onClick={() => { onNavigateMonth("next"); }}
              type="button"
            >
              <ChevronRight size={16} />
            </button>
          </div>

          <button
            className="flex items-center gap-1.5 px-3 py-2 text-xs font-bold uppercase tracking-wider border border-slate-200 dark:border-slate-800 rounded-xl hover:bg-slate-50 dark:hover:bg-slate-800/80 transition-colors text-slate-600 dark:text-slate-300 disabled:opacity-50"
            disabled={isCopying}
            onClick={onCopyMonth}
            type="button"
          >
            {isCopying ? <Loader2 className="animate-spin" size={12} /> : <Copy size={12} />}
            <span>{copyLabel}</span>
          </button>
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
            onSelectedEmployeesChange(ids);
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
                    onSelectedEmployeesChange(selectedEmployeeIds.filter((id) => id !== emp.id));
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
  formatTime: (time: string | null | undefined) => string;
  handleApplyShift: (shiftId: string | null) => void;
  isMutating: boolean;
  isShiftPickerOpen: boolean;
  onCancel: () => void;
  onClearOverrides: () => void;
  selectedCount: number;
  selectedLabel: string;
  setIsShiftPickerOpen: (open: boolean) => void;
  shifts?: Shift[];
}

const RosterActionBar = memo(function RosterActionBar({
  assignLabel,
  cancelLabel,
  clearLabel,
  formatTime,
  handleApplyShift,
  isMutating,
  isShiftPickerOpen,
  onCancel,
  onClearOverrides,
  selectedLabel,
  setIsShiftPickerOpen,
  shifts,
}: RosterActionBarProps) {
  const { t } = useTranslation();

  return (
    <div className="fixed bottom-6 left-1/2 transform -translate-x-1/2 z-40 animate-in slide-in-from-bottom-8 duration-300">
      <div className="backdrop-blur-md bg-white/95 dark:bg-slate-900/95 border border-slate-200 dark:border-slate-800 shadow-[0_20px_50px_rgba(0,0,0,0.15)] rounded-3xl px-6 py-4 flex items-center justify-between gap-6 min-w-[340px] md:min-w-[450px]">
        <div className="flex items-center gap-2">
          <div className="w-5 h-5 rounded-full bg-primary-100 dark:bg-primary-950 flex items-center justify-center">
            <Sparkles className="text-primary-600 dark:text-primary-400" size={10} />
          </div>
          <span className="text-xs font-bold text-slate-700 dark:text-slate-200">
            {selectedLabel}
          </span>
        </div>

        <div className="flex items-center gap-2">
          <Popover.Root onOpenChange={setIsShiftPickerOpen} open={isShiftPickerOpen}>
            <Popover.Trigger asChild>
              <button
                className="px-3 py-2 bg-primary-600 hover:bg-primary-700 text-white rounded-xl text-[10px] font-extrabold uppercase tracking-widest transition-all disabled:opacity-50 cursor-pointer"
                disabled={isMutating}
              >
                {assignLabel}
              </button>
            </Popover.Trigger>
            <Popover.Portal>
              <Popover.Content
                className="z-50 w-72 bg-white dark:bg-slate-900 rounded-[2rem] shadow-premium-2xl border border-slate-100 dark:border-slate-800 p-5 animate-in fade-in zoom-in-95 duration-200 flex flex-col gap-4"
                onClick={(e) => { e.stopPropagation(); }}
                side="top"
                sideOffset={12}
              >
                <div className="text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest border-b border-slate-50 dark:border-slate-800 pb-2">
                  {t("shifts.modal.select_shift_title")}
                </div>
                <div className="grid grid-cols-1 gap-2 max-h-[240px] overflow-y-auto custom-scrollbar pr-1">
                  {shifts?.map(shift => (
                    <button
                      className="p-3.5 rounded-2xl border text-left transition-all relative overflow-hidden group hover:border-primary-500 hover:bg-primary-50 dark:bg-primary-950/30 border-slate-100 dark:border-slate-800 bg-slate-50 dark:bg-slate-800/30 cursor-pointer"
                      key={shift.id}
                      onClick={() => {
                        handleApplyShift(shift.id);
                        setIsShiftPickerOpen(false);
                      }}
                    >
                      <div className="font-bold text-xs uppercase tracking-tight text-slate-700 dark:text-slate-200">{shift.name}</div>
                      <div className="flex items-center gap-1.5 mt-1 text-[10px] text-slate-500 dark:text-slate-400 font-bold">
                        <Clock size={10} /> {formatTime(shift.start_time)} - {formatTime(shift.end_time)}
                      </div>
                    </button>
                  ))}

                  <button
                    className="p-3.5 rounded-2xl border border-dashed text-left transition-all hover:border-slate-300 dark:hover:border-slate-600 hover:bg-slate-100 dark:hover:bg-slate-700 border-slate-100 dark:border-slate-800 bg-slate-50 dark:bg-slate-800/30 cursor-pointer"
                    onClick={() => {
                      handleApplyShift(null);
                      setIsShiftPickerOpen(false);
                    }}
                  >
                    <div className="font-bold text-xs uppercase tracking-tight text-slate-500 dark:text-slate-400">{t("shifts.off_day")}</div>
                    <div className="text-[10px] text-slate-500 dark:text-slate-400 font-bold mt-1">
                      {t("shifts.modal.mark_as_off")}
                    </div>
                  </button>
                </div>
              </Popover.Content>
            </Popover.Portal>
          </Popover.Root>

          <button
            className="p-2 border border-slate-200 dark:border-slate-800 hover:bg-rose-50 dark:hover:bg-rose-950/40 text-rose-600 dark:text-rose-400 rounded-xl transition-colors disabled:opacity-50 cursor-pointer"
            disabled={isMutating}
            onClick={onClearOverrides}
            title={clearLabel}
          >
            <Trash2 size={14} />
          </button>

          <button
            className="px-3 py-2 text-[10px] font-bold uppercase tracking-wider text-slate-500 hover:bg-slate-100 dark:hover:bg-slate-800 rounded-xl transition-colors cursor-pointer"
            onClick={onCancel}
          >
            {cancelLabel}
          </button>
        </div>
      </div>
    </div>
  );
});
RosterActionBar.displayName = "RosterActionBar";

interface RosterGridTableProps {
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
    <div className={`overflow-x-auto relative max-w-full roster-scrollbar ${isDragging ? "select-none" : ""}`}>
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
                const daySchedule = empSchedule.schedules.find(s => s.date === dateStr);
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
          <button 
            className="absolute top-0.5 right-0.5 opacity-0 hover:opacity-100 group-hover:opacity-60 transition-opacity bg-transparent border-none p-0 cursor-default"
            onClick={handleTraceClick}
            type="button"
          >
            <ScheduleTracePopover 
              source={source ?? "none"} 
              traces={traces} 
            />
          </button>
        ) : null}
      </div>
    </td>
  );
});
RosterCell.displayName = "RosterCell";
