import {
  CalendarDays,
  Calendar as CalendarIcon,
  ChevronLeft,
  ChevronRight,
  Copy,
  Trash,
  Zap,
  Sparkles,
  Trash2,
  Plus,
  X,
} from "lucide-react";
import { memo, useMemo } from "react";
import { useTranslation } from "react-i18next";

import { format } from "date-fns";
import { vi, enUS, zhCN, type Locale } from "date-fns/locale";

import type { useShifts } from "@/pages/shifts/hooks/use-shifts";

import { EmployeeSearch } from "@/components/common/employee-search";
import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import { Typography } from "@/components/ui/typography";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { Employee, Shift } from "@/lib/api/types";
import { readFrequentShifts, getTopFrequentIds } from "@/lib/frequent-shifts-storage";
import { cn } from "@/lib/utils";

import { formatShiftTimeRange } from "./roster-toolbar-utils";
import { getShiftAccent, type ShiftAccent } from "./shift-badge-style";

const noop = () => undefined;

const dateLocales: Partial<Record<string, Locale>> = {
  vi: vi,
  en: enUS,
  zh: zhCN,
};

// ── RosterActionBar ──────────────────────────────────────

/** Accent colours adapted for the dark primary-600 toolbar background. */
const ACCENT_ON_DARK: Record<ShiftAccent, { dot: string }> = {
  morning:   { dot: "bg-amber-300" },
  afternoon: { dot: "bg-sky-300" },
  night:     { dot: "bg-indigo-300" },
  split:     { dot: "bg-teal-300" },
};

interface QuickShiftRowProps {
  isMutating: boolean;
  onApplyShift: (shiftId: string | null) => void;
  shifts?: Shift[];
}

/** Horizontally-scrollable row of top frequent shifts (mobile touch strip; wraps on larger screens). */
export const QuickShiftRow = memo(function QuickShiftRow({
  isMutating,
  onApplyShift,
  shifts,
}: QuickShiftRowProps) {
  const { t } = useTranslation();
  const sortedShifts = useMemo(() => {
    if (!shifts || shifts.length === 0) return [];
    const freqMap = readFrequentShifts();
    const topIds = new Set(getTopFrequentIds(freqMap));
    const frequent = shifts
      .filter((s) => topIds.has(s.id))
      .sort((a, b) => (freqMap[b.id] ?? 0) - (freqMap[a.id] ?? 0));
    const rest = shifts.filter((s) => !topIds.has(s.id));
    return [...frequent, ...rest].slice(0, 3);
  }, [shifts]);

  if (sortedShifts.length === 0) return null;

  return (
    <div className="flex items-center gap-1.5 bg-white/10 p-1 rounded-xl overflow-x-auto scrollbar-hide sm:flex-wrap sm:overflow-visible">
      <span className="text-[9px] text-white/70 font-bold uppercase tracking-wider px-1.5 shrink-0">{t("shifts.quick_shift_label")}</span>
      {sortedShifts.map((shift) => {
        const accent = getShiftAccent(shift);
        const accentColors = ACCENT_ON_DARK[accent];
        const timeRange = formatShiftTimeRange(shift);
        const label = shift.report_symbol
          ? `${shift.name} (${shift.report_symbol})`
          : shift.name;
        const tooltip = timeRange ? `${label}: ${timeRange}` : label;

        return (
          <Button
            aria-label={`Assign ${shift.name} shift`}
            className={cn(
              "flex items-center gap-2 px-3 py-1.5 rounded-lg transition-all disabled:opacity-50 cursor-pointer h-auto min-w-0 shrink-0",
              "bg-white/10 hover:bg-white/20 border border-transparent hover:border-white/20",
              "active:scale-95 focus-visible:ring-2 focus-visible:ring-white/50",
              "text-white hover:text-white",
            )}
            disabled={isMutating}
            key={shift.id}
            onClick={() => { onApplyShift(shift.id); }}
            title={tooltip}
            type="button"
            variant="ghost"
          >
            <span aria-hidden="true" className={cn("w-2 h-2 rounded-full shrink-0", accentColors.dot)} />
            <span className="flex flex-col items-start leading-tight min-w-0">
              <span className="text-[11px] font-bold text-white truncate max-w-[100px]">{shift.name}</span>
              {!!timeRange && <span className="text-[9px] text-white/60 font-medium tabular-nums">{timeRange}</span>}
            </span>
          </Button>
        );
      })}
    </div>
  );
});
QuickShiftRow.displayName = "QuickShiftRow";

interface RosterActionBarProps {
  assignLabel: string;
  cancelLabel: string;
  clearLabel: string;
  isMutating: boolean;
  onApplyShift: (shiftId: string | null) => void;
  onAssignSelected: () => void;
  onCancel: () => void;
  onClearOverrides: () => void;
  selectedCount: number;
  selectedLabel: string;
  shifts?: Shift[];
}

export const RosterActionBar = memo(function RosterActionBar({
  assignLabel,
  cancelLabel,
  clearLabel,
  isMutating,
  onAssignSelected,
  onCancel,
  onClearOverrides,
  selectedLabel,
  shifts,
  onApplyShift,
}: RosterActionBarProps) {
  return (
    <div className="absolute inset-0 z-40 bg-primary-600 text-white px-5 sm:px-6 py-3.5 sm:py-4 rounded-[28px] shadow-lg flex flex-col gap-3 animate-in fade-in duration-200">
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
        <div className="flex items-center gap-3">
          <div className="w-6 h-6 rounded-full bg-white/10 flex items-center justify-center shrink-0">
            <Sparkles className="text-white animate-pulse" size={12} />
          </div>
          <span className="text-xs font-bold uppercase tracking-widest">{selectedLabel}</span>
        </div>
        <div className="flex items-center gap-2.5 shrink-0">
          <Button className="px-4 py-2 bg-white text-primary-700 hover:bg-white/90 active:scale-95 rounded-xl text-[10px] font-extrabold uppercase tracking-widest transition-all disabled:opacity-50 cursor-pointer shadow-sm" disabled={isMutating} onClick={onAssignSelected} variant="ghost">{assignLabel}</Button>
          <Button className="p-2 border border-white/20 hover:bg-white/10 text-white rounded-xl transition-all active:scale-90 disabled:opacity-50 cursor-pointer" disabled={isMutating} onClick={onClearOverrides} title={clearLabel} variant="ghost"><Trash2 size={14} /></Button>
          <Button className="px-4 py-2 text-[10px] font-bold uppercase tracking-wider text-white/80 hover:text-white hover:bg-white/10 rounded-xl transition-all cursor-pointer" onClick={onCancel} variant="ghost">{cancelLabel}</Button>
        </div>
      </div>
      <QuickShiftRow isMutating={isMutating} onApplyShift={onApplyShift} shifts={shifts} />
    </div>
  );
});
RosterActionBar.displayName = "RosterActionBar";

// ── RosterNavHeader ─────────────────────────────────────

export function RosterNavHeader({
  hook,
  viewMode,
  isCopying,
  onCopyMonth,
}: {
  hook: ReturnType<typeof useShifts>;
  viewMode: "week" | "month";
  isCopying?: boolean;
  onCopyMonth?: () => void;
}) {
  const { i18n, t } = useTranslation();
  const { formatDate } = useDateTimeFormatter();
  const currentLocale = dateLocales[i18n.language.substring(0, 2)] ?? enUS;

  return (
    <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3">
      <div className="flex items-center gap-2">
        {viewMode === "week" ? <CalendarIcon className="text-primary-600" size={20} /> : <CalendarDays className="text-primary-600" size={20} />}
        <div className="flex flex-col">
          <Typography className="text-sm font-bold uppercase tracking-wider text-slate-700" variant="h2">
            {viewMode === "week" ? t("shifts.weekly.title") : t("shifts.monthly.title")}
          </Typography>
          {viewMode === "week" && !!hook.selectedEmployee && (
            <Typography className="text-[10px] font-bold uppercase tracking-widest text-slate-500 leading-none mt-0.5" variant="p">
              {t("shifts.current_week_roster")}
            </Typography>
          )}
        </div>
      </div>
      <div className="flex items-center justify-between sm:justify-end gap-3 w-full sm:w-auto">
        <div className="flex items-center gap-1">
          <Button
            aria-label={viewMode === "week" ? t("shifts.prev_week", { defaultValue: "Tuần trước" }) : t("common.prev_period", { defaultValue: "Kỳ trước" })}
            className="p-2 rounded-xl hover:bg-slate-100 border border-slate-100"
            onClick={() => { if (viewMode === "week") hook.handleNavigateWeek("prev"); else hook.handleNavigateMonth("prev"); }}
            type="button"
            variant="ghost"
          >
            <ChevronLeft size={16} />
          </Button>
          <span className="min-w-[140px] text-center font-bold text-xs uppercase tracking-widest text-slate-700">
            {viewMode === "week"
              ? `${formatDate(hook.weekDays[0])} - ${formatDate(hook.weekDays[6])}`
              : format(hook.currentMonth, "MMMM yyyy", { locale: currentLocale })}
          </span>
          <Button
            aria-label={viewMode === "week" ? t("shifts.next_week", { defaultValue: "Tuần tiếp theo" }) : t("common.next_period", { defaultValue: "Kỳ tiếp theo" })}
            className="p-2 rounded-xl hover:bg-slate-100 border border-slate-100"
            onClick={() => { if (viewMode === "week") hook.handleNavigateWeek("next"); else hook.handleNavigateMonth("next"); }}
            type="button"
            variant="ghost"
          >
            <ChevronRight size={16} />
          </Button>
        </div>
        {viewMode === "month" && !!onCopyMonth && (
          <Button loading={isCopying} onClick={onCopyMonth} size="sm" variant="ghost">
            <Copy size={12} />
            {t("shifts.monthly.copy")}
          </Button>
        )}
      </div>
    </div>
  );
}

// ── EmployeeChecklist ───────────────────────────────────

export function EmployeeChecklist({
  hook, employees, onClearAll,
}: {
  hook: ReturnType<typeof useShifts>;
  employees: Employee[];
  onClearAll: () => void;
}) {
  const { t } = useTranslation();
  return (
    <div className="flex flex-wrap gap-2.5 p-3.5 bg-slate-50 dark:bg-slate-950/40 rounded-2xl border border-slate-200/50 animate-in fade-in slide-in-from-top-2 duration-300">
      <div className="w-full flex items-center justify-between mb-1">
        <span className="text-[10px] font-bold text-slate-400 uppercase tracking-widest">{t("shifts.monthly.selected_employees", { count: employees.length })}</span>
        <Button className="flex items-center gap-1.5 px-2.5 py-1 text-[10px] font-extrabold uppercase tracking-widest text-slate-500 hover:text-rose-600 hover:bg-rose-50 dark:hover:bg-rose-950/40 rounded-lg transition-all cursor-pointer" onClick={onClearAll} type="button" variant="ghost"><X size={12} /> {t("shifts.monthly.clear_all_employees")}</Button>
      </div>
      {employees.map((emp) => (
        <label className="flex items-center gap-2 px-3 py-1.5 bg-white border border-slate-200 rounded-xl text-xs font-bold text-slate-700 shadow-sm hover:border-primary-500/50 cursor-pointer select-none transition-all" key={emp.id}>
          <input checked={hook.selectedEmployeeIds.includes(emp.id)} className="rounded border-slate-300 text-primary-600 focus:ring-primary-500 cursor-pointer h-3.5 w-3.5"
            onChange={() => { hook.setSelectedEmployeeIds(hook.selectedEmployeeIds.filter((id) => id !== emp.id)); hook.setSelectedEmployees(employees.filter(e => e.id !== emp.id)); }}
            type="checkbox" />
          <span>{emp.full_name}</span>
        </label>
      ))}
    </div>
  );
}

// ── WeekViewControls ────────────────────────────────────

export function WeekViewControls({ hook }: { hook: ReturnType<typeof useShifts> }) {
  const { t } = useTranslation();
  return (
    <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
      <div className="w-full md:max-w-md">
        <EmployeeSearch clearOnEscape data-testid="employee-select" departments={hook.departments} keyboardShortcut={["/", "Enter"]}
          onChange={(id) => { if (!id) hook.setSelectedEmployee(null); }}
          onSelectEmployee={(emp) => { hook.setSelectedEmployee(emp); }}
          placeholder={t("shifts.empty_state.title")} value={hook.selectedEmployee?.id ?? ""}  />
      </div>
      {!!hook.selectedEmployee && (
        <div className="w-full md:w-56">
          <Combobox className="h-10 bg-slate-100 dark:bg-slate-700 border-slate-200 text-[10px] py-2 px-3 rounded-xl uppercase tracking-tighter"
            onChange={(value) => {
              if (value === "copy-prev") void hook.handleCopyPreviousWeek();
              else if (value === "clear-week") hook.handleAssignAllWeek(null);
              else if (value === "assign-all") { hook.setTargetDates(hook.weekDays.map((d) => format(d, "yyyy-MM-dd"))); hook.setIsShiftPickerOpen(true); }
              else if (value === "assign-pattern") hook.setIsPatternModalOpen(true);
            }}
            options={[
              { value: "copy-prev", label: t("shifts.actions.copy_previous"), icon: <Copy className="w-4 h-4" /> },
              { value: "assign-pattern", label: t("patterns.assign_btn"), icon: <Zap className="w-4 h-4" /> },
              { value: "assign-all", label: t("shifts.actions.assign_all_week"), icon: <CalendarIcon className="w-4 h-4" /> },
              { value: "clear-week", label: t("shifts.actions.clear_week"), icon: <Trash className="w-4 h-4" /> },
            ]}
            placeholder={t("common.quick_actions")} />
        </div>
      )}
    </div>
  );
}

// ── WeekActionBar + MonthOverlay ────────────────────────

export function WeekActionBar({
  selectedDays, onOpenWeekShiftPicker, onClearWeekSelection, shifts, onApplyShift, isMutating,
}: {
  selectedDays?: string[];
  onOpenWeekShiftPicker?: (dates: string[]) => void;
  onClearWeekSelection?: () => void;
  shifts?: Shift[];
  onApplyShift?: (shiftId: string | null) => void;
  isMutating?: boolean;
}) {
  const { t } = useTranslation();
  return (
    <div className="absolute inset-0 z-40 bg-primary-600 text-white px-5 sm:px-6 py-3.5 sm:py-4 rounded-[28px] flex flex-col gap-3 shadow-xl animate-in fade-in duration-300">
      <div className="flex items-center justify-between gap-3">
        <div className="flex items-center gap-4">
          <span className="text-xs font-bold uppercase tracking-widest">{t("shifts.days_selected", { count: selectedDays?.length ?? 0 })}</span>
          <div className="h-4 w-px bg-white/20" />
          <Button className="bg-white/10 border border-white/20 rounded-xl px-4 py-2 text-[10px] font-bold uppercase tracking-widest hover:bg-white/20 transition-all flex items-center gap-2 cursor-pointer"
            onClick={() => { if (selectedDays && onOpenWeekShiftPicker) onOpenWeekShiftPicker(selectedDays); }} variant="ghost"><Plus size={14} /> {t("shifts.assign_selected_btn")}</Button>
        </div>
        <Button className="hover:bg-white/10 p-2 rounded-xl transition-all cursor-pointer shrink-0" onClick={onClearWeekSelection} variant="ghost"><X size={18} /></Button>
      </div>
      <QuickShiftRow isMutating={isMutating ?? false} onApplyShift={onApplyShift ?? noop} shifts={shifts} />
    </div>
  );
}

export function MonthOverlay({
  hook, selectedCells, isMutating, onCancelSelection, onAssignSelected, onClearOverrides, onApplyShift,
}: {
  hook: ReturnType<typeof useShifts>;
  selectedCells?: Set<string>;
  isMutating?: boolean;
  onCancelSelection?: () => void;
  onAssignSelected?: () => void;
  onClearOverrides?: () => void;
  onApplyShift?: (shiftId: string | null) => void;
}) {
  const { t } = useTranslation();
  if (!selectedCells || selectedCells.size === 0) return null;
  return (
    <RosterActionBar
      assignLabel={t("shifts.monthly.assign_selected")} cancelLabel={t("common.cancel")} clearLabel={t("shifts.monthly.clear_month")}
      isMutating={isMutating ?? false} onApplyShift={onApplyShift ?? noop} onAssignSelected={onAssignSelected ?? noop}
      onCancel={onCancelSelection ?? noop} onClearOverrides={onClearOverrides ?? noop}
      selectedCount={selectedCells.size} selectedLabel={t("shifts.monthly.cells_selected", { count: selectedCells.size })} shifts={hook.shifts}
    />
  );
}
