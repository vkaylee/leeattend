import { 
  Building2,
  CalendarDays,
  CalendarRange,
  ChevronDown,
  ChevronLeft,
  ChevronRight,
  Copy,
  Loader2,
  Search,
  Sparkles,
  Trash2,
  X,
  AlertCircle
} from "lucide-react";
import React, { memo, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import { format, isToday as isDateToday } from "date-fns";
import { vi, enUS, zhCN, type Locale } from "date-fns/locale";

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuLabel,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Department, EffectiveSchedule, Employee, EmployeeSchedule, Shift } from "@/lib/api/types";
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

const matchesSearch = (fullName: string, searchTerm: string): boolean => {
  const query = searchTerm.trim().toLowerCase();
  if (!query) return true;
  return fullName.toLowerCase().includes(query);
};

const matchesDepartment = (
  employeeId: string,
  selectedDeptId: string,
  employees: Employee[] | undefined,
  departments: Department[] | undefined
): boolean => {
  if (selectedDeptId === "all") return true;

  const employee = employees?.find((e) => e.id === employeeId);

  if (selectedDeptId === "none") {
    return !employee?.department_name;
  }

  const dept = departments?.find((d) => d.id === selectedDeptId);
  return !!(dept && employee?.department_name === dept.name);
};

interface MonthlyRosterGridProps {
  hook: ReturnType<typeof useShifts>;
}

// eslint-disable-next-line max-lines-per-function, complexity
export function MonthlyRosterGrid({ hook }: MonthlyRosterGridProps) {
  const { t, i18n } = useTranslation();
  const currentLocale = dateLocales[i18n.language] ?? vi;

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
    departments,
  } = hook;

  const [selectedCells, setSelectedCells] = useState<Set<string>>(new Set());
  const [isShiftPickerOpen, setIsShiftPickerOpen] = useState(false);
  const [isCopying, setIsCopying] = useState(false);
  const [isMutating, setIsMutating] = useState(false);

  const [searchTerm, setSearchTerm] = useState("");
  const [selectedDepartmentId, setSelectedDepartmentId] = useState("all");

  const monthHeaderLabel = format(currentMonth, "MMMM yyyy", { locale: currentLocale });

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
    return combinedSchedules.filter(schedule => 
      matchesSearch(schedule.full_name, searchTerm) &&
      matchesDepartment(schedule.employee_id, selectedDepartmentId, employees, departments)
    );
  }, [combinedSchedules, searchTerm, selectedDepartmentId, employees, departments]);

  const handleCellClick = (employeeId: string, dateStr: string) => {
    setSelectedCells(prev => {
      const next = new Set(prev);
      const key = `${employeeId}:${dateStr}`;
      if (next.has(key)) {
        next.delete(key);
      } else {
        next.add(key);
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
        allDepartmentsLabel={t("shifts.monthly.all_departments")}
        copyLabel={t("shifts.monthly.copy_previous_month")}
        departments={departments}
        filterDepartmentLabel={t("shifts.monthly.filter_department")}
        isCopying={isCopying}
        monthHeaderLabel={monthHeaderLabel}
        noDepartmentLabel={t("shifts.monthly.no_department")}
        onCopyMonth={handleCopyMonth}
        onDepartmentChange={setSelectedDepartmentId}
        onNavigateMonth={handleNavigateMonth}
        onSearchChange={setSearchTerm}
        searchPlaceholder={t("shifts.monthly.search_placeholder")}
        searchTerm={searchTerm}
        selectedDepartmentId={selectedDepartmentId}
        title={t("shifts.monthly.title")}
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
            monthDays={monthDays}
            monthSchedules={filteredSchedules}
            onCellClick={handleCellClick}
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
          isMutating={isMutating}
          onAssignSelected={() => { setIsShiftPickerOpen(true); }}
          onCancel={() => { setSelectedCells(new Set()); }}
          onClearOverrides={handleClearSelectedOverrides}
          selectedCount={selectedCells.size}
          selectedLabel={t("shifts.monthly.cells_selected", { count: selectedCells.size })}
        />
      )}

      {isShiftPickerOpen ? (
        <ShiftPickerModal
          isOpen={isShiftPickerOpen}
          onClose={() => { setIsShiftPickerOpen(false); }}
          onSelectShift={handleApplyShift}
          shifts={shifts}
          targetDates={Array.from(selectedCells).map(c => c.split(":")[1])}
        />
      ) : null}
    </div>
  );
}

interface RosterToolbarProps {
  allDepartmentsLabel: string;
  copyLabel: string;
  departments: Department[] | undefined;
  filterDepartmentLabel: string;
  isCopying: boolean;
  monthHeaderLabel: string;
  noDepartmentLabel: string;
  onCopyMonth: () => void;
  onDepartmentChange: (val: string) => void;
  onNavigateMonth: (dir: "prev" | "next") => void;
  onSearchChange: (val: string) => void;
  searchPlaceholder: string;
  searchTerm: string;
  selectedDepartmentId: string;
  title: string;
}

// eslint-disable-next-line max-lines-per-function
const RosterToolbar = memo(function RosterToolbar({
  copyLabel,
  isCopying,
  monthHeaderLabel,
  onCopyMonth,
  onNavigateMonth,
  title,
  searchTerm,
  onSearchChange,
  selectedDepartmentId,
  onDepartmentChange,
  departments,
  searchPlaceholder,
  filterDepartmentLabel,
  allDepartmentsLabel,
  noDepartmentLabel,
}: RosterToolbarProps) {
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
            >
              <ChevronLeft size={16} />
            </button>
            <span className="min-w-[140px] text-center font-bold text-xs uppercase tracking-widest text-slate-700 dark:text-slate-200">
              {monthHeaderLabel}
            </span>
            <button
              className="p-2 rounded-xl hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors border border-slate-100 dark:border-slate-800"
              onClick={() => { onNavigateMonth("next"); }}
            >
              <ChevronRight size={16} />
            </button>
          </div>

          <button
            className="flex items-center gap-1.5 px-3 py-2 text-xs font-bold uppercase tracking-wider border border-slate-200 dark:border-slate-800 rounded-xl hover:bg-slate-50 dark:hover:bg-slate-800/80 transition-colors text-slate-600 dark:text-slate-300 disabled:opacity-50"
            disabled={isCopying}
            onClick={onCopyMonth}
          >
            {isCopying ? <Loader2 className="animate-spin" size={12} /> : <Copy size={12} />}
            <span>{copyLabel}</span>
          </button>
        </div>
      </div>

      {/* Separator line */}
      <div className="h-px bg-slate-100 dark:bg-slate-800 w-full" />

      {/* Row 2: Search Input & Department Dropdown filter */}
      <div className="flex flex-col sm:flex-row gap-3 w-full">
        {/* Search Bar */}
        <div className="relative flex-1">
          <Search className="absolute left-3.5 top-1/2 -translate-y-1/2 text-slate-400 dark:text-slate-500" size={16} />
          <input
            className="w-full pl-10 pr-9 py-2.5 bg-slate-50 dark:bg-slate-950 border border-slate-200/80 dark:border-slate-800 rounded-xl text-xs font-medium placeholder-slate-400 dark:placeholder-slate-500 text-slate-700 dark:text-slate-200 focus:outline-none focus:ring-2 focus:ring-primary-500/20 focus:border-primary-500 transition-all"
            onChange={(e) => { onSearchChange(e.target.value); }}
            placeholder={searchPlaceholder}
            type="text"
            value={searchTerm}
          />
          {!!searchTerm && (
            <button
              className="absolute right-3 top-1/2 -translate-y-1/2 p-0.5 rounded-full hover:bg-slate-200 dark:hover:bg-slate-800 text-slate-400 dark:text-slate-500 transition-colors"
              onClick={() => { onSearchChange(""); }}
            >
              <X size={14} />
            </button>
          )}
        </div>

        {/* Department Filter */}
        <div className="sm:w-64">
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <button className="flex items-center justify-between w-full px-3.5 py-2.5 bg-slate-50 dark:bg-slate-950 border border-slate-200/80 dark:border-slate-800 rounded-xl text-xs font-bold text-slate-600 dark:text-slate-350 hover:bg-slate-100/50 dark:hover:bg-slate-900/50 focus:outline-none focus:ring-2 focus:ring-primary-500/20 transition-all uppercase tracking-wider">
                <div className="flex items-center gap-2 truncate">
                  <Building2 className="text-slate-400 dark:text-slate-500 shrink-0" size={15} />
                  <span className="truncate">
                    {selectedDepartmentId === "all"
                      ? allDepartmentsLabel
                      : selectedDepartmentId === "none"
                        ? noDepartmentLabel
                        : departments?.find((d) => d.id === selectedDepartmentId)?.name ?? allDepartmentsLabel}
                  </span>
                </div>
                <ChevronDown className="text-slate-400 shrink-0 ml-1" size={14} />
              </button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" className="w-56">
              <DropdownMenuLabel className="text-[10px] font-extrabold uppercase tracking-widest text-slate-400 dark:text-slate-500">
                {filterDepartmentLabel}
              </DropdownMenuLabel>
              <DropdownMenuRadioGroup onValueChange={onDepartmentChange} value={selectedDepartmentId}>
                <DropdownMenuRadioItem className="text-xs" value="all">
                  {allDepartmentsLabel}
                </DropdownMenuRadioItem>
                <DropdownMenuRadioItem className="text-xs" value="none">
                  {noDepartmentLabel}
                </DropdownMenuRadioItem>
                {departments?.map((dept) => (
                  <DropdownMenuRadioItem className="text-xs" key={dept.id} value={dept.id}>
                    {dept.name}
                  </DropdownMenuRadioItem>
                ))}
              </DropdownMenuRadioGroup>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
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
          <button
            className="px-3 py-2 bg-primary-600 hover:bg-primary-700 text-white rounded-xl text-[10px] font-extrabold uppercase tracking-widest transition-all disabled:opacity-50"
            disabled={isMutating}
            onClick={onAssignSelected}
          >
            {assignLabel}
          </button>

          <button
            className="p-2 border border-slate-200 dark:border-slate-800 hover:bg-rose-50 dark:hover:bg-rose-950/40 text-rose-600 dark:text-rose-400 rounded-xl transition-colors disabled:opacity-50"
            disabled={isMutating}
            onClick={onClearOverrides}
            title={clearLabel}
          >
            <Trash2 size={14} />
          </button>

          <button
            className="px-3 py-2 text-[10px] font-bold uppercase tracking-wider text-slate-500 hover:bg-slate-100 dark:hover:bg-slate-800 rounded-xl transition-colors"
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
  monthDays: Date[];
  monthSchedules: EmployeeSchedule[];
  onCellClick: (employeeId: string, dateStr: string) => void;
  selectedCells: Set<string>;
  shifts?: Shift[];
}

const RosterGridTable = memo(function RosterGridTable({
  currentLocale,
  employeesLabel,
  monthDays,
  monthSchedules,
  onCellClick,
  selectedCells,
  shifts,
}: RosterGridTableProps) {
  return (
    <div className="overflow-x-auto relative max-w-full">
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
                    onClick={onCellClick}
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
  onClick: (employeeId: string, dateStr: string) => void;
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
  onClick,
  shiftBgClass,
  shiftSymbol,
  source,
  traces,
}: RosterCellProps) {
  const handleCellClick = () => {
    onClick(employeeId, dateStr);
  };

  const handleTraceClick = (e: React.MouseEvent) => {
    e.stopPropagation();
  };

  return (
    <td
      className={cellStyles}
      onClick={handleCellClick}
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
