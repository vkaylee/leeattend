/* eslint-disable @typescript-eslint/no-empty-function, @typescript-eslint/prefer-nullish-coalescing, max-lines */
import { 
  CalendarDays,
  Calendar as CalendarIcon,
  ChevronLeft,
  ChevronRight,
  Copy,
  Loader2,
  Trash,
  Zap,
  Sparkles,
  Trash2,
  Plus,
  Undo2,
  Users,
  X,
} from "lucide-react";
import { memo, useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";

import { format } from "date-fns";
import { vi, enUS, zhCN, type Locale } from "date-fns/locale";

import { EmployeeSearch } from "@/components/common/employee-search";
import { BulkEmployeePickerModal } from "@/components/shifts/bulk-employee-picker-modal";
import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import { Typography } from "@/components/ui/typography";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { Employee, Shift } from "@/lib/api/types";
import { useShifts } from "@/pages/shifts/hooks/use-shifts";

const dateLocales: Partial<Record<string, Locale>> = {
  vi: vi,
  en: enUS,
  zh: zhCN,
};

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

const RosterActionBar = memo(function RosterActionBar({
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
        {/* Quick Assign Presets */}
        {!!shifts && shifts.length > 0 && (
          <div className="hidden md:flex items-center gap-1.5 mr-2 bg-white/10 p-1 rounded-xl">
            <span className="text-[9px] text-white/70 font-bold uppercase tracking-wider px-1.5">Quick:</span>
            {shifts.slice(0, 3).map((shift) => (
              <Button
                className="px-2.5 py-1 bg-white/20 hover:bg-white text-white hover:text-primary-700 rounded-lg text-[9px] font-extrabold uppercase tracking-wider transition-all disabled:opacity-50 cursor-pointer border border-transparent hover:border-white/10"
                disabled={isMutating}
                key={shift.id}
                onClick={() => { onApplyShift(shift.id); }}
                variant="ghost"
              >
                {shift.report_symbol || shift.name.substring(0, 2).toUpperCase()}
              </Button>
            ))}
          </div>
        )}

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

interface RosterToolbarProps {
  hook: ReturnType<typeof useShifts>;
  // For month view
  isCopying?: boolean;
  isMutating?: boolean;
  onApplyShift?: (shiftId: string | null) => void;
  onAssignSelected?: () => void;
  onCancelSelection?: () => void;
  onClearOverrides?: () => void;
  onClearWeekSelection?: () => void;
  onCopyMonth?: () => void;
  onOpenWeekShiftPicker?: (dates: string[]) => void;
  // Month action bar overlays
  selectedCells?: Set<string>;
  // Week action bar overlays
  selectedDays?: string[];
  selectedEmployeesMeta?: Employee[];
  viewMode: "week" | "month";
}

function RosterNavHeader({
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
          <Button className="p-2 rounded-xl hover:bg-slate-100 border border-slate-100" onClick={() => { if (viewMode === "week") hook.handleNavigateWeek("prev"); else hook.handleNavigateMonth("prev"); }} type="button" variant="ghost">
            <ChevronLeft size={16} />
          </Button>
          <span className="min-w-[140px] text-center font-bold text-xs uppercase tracking-widest text-slate-700">
            {viewMode === "week"
              ? `${formatDate(hook.weekDays[0])} - ${formatDate(hook.weekDays[6])}`
              : format(hook.currentMonth, "MMMM yyyy", { locale: currentLocale })}
          </span>
          <Button className="p-2 rounded-xl hover:bg-slate-100 border border-slate-100" onClick={() => { if (viewMode === "week") hook.handleNavigateWeek("next"); else hook.handleNavigateMonth("next"); }} type="button" variant="ghost">
            <ChevronRight size={16} />
          </Button>
        </div>
        {viewMode === "month" && (
          <Button className="flex items-center gap-1.5 px-3 py-2 text-xs font-bold uppercase tracking-wider border border-slate-200 rounded-xl hover:bg-slate-50 text-slate-600 disabled:opacity-50" disabled={isCopying} onClick={onCopyMonth} type="button" variant="ghost">
            {isCopying ? <Loader2 className="animate-spin" size={12} /> : <Copy size={12} />}
            <span>{t("shifts.monthly.copy_previous_month")}</span>
          </Button>
        )}
      </div>
    </div>
  );
}

function MonthViewControls({
  hook,
  selectedEmployeesMeta,
}: {
  hook: ReturnType<typeof useShifts>;
  selectedEmployeesMeta?: Employee[];
}) {
  const { t } = useTranslation();
  const selectedMetaRef = useRef<Employee[]>(selectedEmployeesMeta || []);
  useEffect(() => { selectedMetaRef.current = selectedEmployeesMeta || []; }, [selectedEmployeesMeta]);

  // Snapshot of the last cleared selection, so a single "clear all" can be
  // undone before the user starts building a new group. Auto-expires so the
  // undo affordance doesn't linger.
  const [undoSnapshot, setUndoSnapshot] = useState<{ ids: string[]; meta: Employee[] } | null>(null);
  const undoTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  const clearUndoTimer = useCallback(() => {
    if (undoTimer.current) {
      clearTimeout(undoTimer.current);
      undoTimer.current = null;
    }
  }, []);

  useEffect(() => clearUndoTimer, [clearUndoTimer]);

  const handleClearAll = useCallback(() => {
    const ids = [...hook.selectedEmployeeIds];
    const meta = [...selectedMetaRef.current];
    if (ids.length === 0) return;
    setUndoSnapshot({ ids, meta });
    selectedMetaRef.current = [];
    hook.setSelectedEmployeeIds([]);
    hook.setSelectedEmployees([]);
    clearUndoTimer();
    undoTimer.current = setTimeout(() => { setUndoSnapshot(null); }, 6000);
  }, [hook, clearUndoTimer]);

  const handleUndo = useCallback(() => {
    if (!undoSnapshot) return;
    selectedMetaRef.current = undoSnapshot.meta;
    hook.setSelectedEmployeeIds(undoSnapshot.ids);
    hook.setSelectedEmployees(undoSnapshot.meta);
    setUndoSnapshot(null);
    clearUndoTimer();
  }, [undoSnapshot, hook, clearUndoTimer]);

  const [isBulkOpen, setIsBulkOpen] = useState(false);

  const handleBulkCommit = useCallback(
    (ids: string[], meta: Employee[]) => {
      selectedMetaRef.current = meta;
      hook.setSelectedEmployeeIds(ids);
      hook.setSelectedEmployees(meta);
    },
    [hook],
  );

  return (
    <>
      <div className="flex items-center gap-2 w-full md:max-w-2xl">
        <div className="flex-1 min-w-0">
          <EmployeeSearch
            clearOnEscape
            data-testid="monthly-employee-search"
            departments={hook.departments}
            hideChips
            multiple
            onChange={(val) => {
              const ids = Array.isArray(val) ? val : (val ? [val] : []);
              const filteredMeta = selectedMetaRef.current.filter(e => ids.includes(e.id));
              selectedMetaRef.current = filteredMeta;
              hook.setSelectedEmployeeIds(ids);
              hook.setSelectedEmployees(filteredMeta);
            }}
            onSelectEmployee={(emp) => {
              if (!selectedMetaRef.current.some(e => e.id === emp.id)) {
                selectedMetaRef.current = [...selectedMetaRef.current, emp];
                hook.setSelectedEmployees(selectedMetaRef.current);
              }
            }}
            placeholder={t("shifts.monthly.search_placeholder")}
            value={hook.selectedEmployeeIds}
            workspaceId={hook.wsId}
          />
        </div>
        <Button
          className="flex h-[var(--ui-height-md)] shrink-0 items-center gap-2 rounded-2xl border border-slate-100 dark:border-slate-700 bg-slate-50 dark:bg-slate-800 px-4 text-sm font-bold uppercase tracking-tight text-slate-600 dark:text-slate-300 hover:border-primary-200 dark:hover:border-primary-800 shadow-inner transition-all cursor-pointer"
          data-testid="monthly-bulk-select"
          onClick={() => { setIsBulkOpen(true); }}
          type="button"
          variant="ghost"
        >
          <Users size={16} />
          <span className="hidden sm:inline">{t("shifts.monthly.bulk_select")}</span>
        </Button>
      </div>
      {isBulkOpen ? (
        <BulkEmployeePickerModal
          departments={hook.departments}
          initialIds={hook.selectedEmployeeIds}
          initialMeta={selectedEmployeesMeta ?? []}
          isOpen={isBulkOpen}
          onClose={() => { setIsBulkOpen(false); }}
          onCommit={handleBulkCommit}
          wsId={hook.wsId}
        />
      ) : null}
      {!!selectedEmployeesMeta && selectedEmployeesMeta.length > 0 && (
        <EmployeeChecklist employees={selectedEmployeesMeta} hook={hook} onClearAll={handleClearAll} />
      )}
      {!!undoSnapshot && (
        <div className="flex items-center justify-between gap-3 px-3.5 py-2.5 bg-slate-800 dark:bg-slate-700 text-white rounded-2xl animate-in fade-in slide-in-from-top-2 duration-300">
          <span className="text-xs font-bold uppercase tracking-widest">
            {t("shifts.monthly.cleared_selection", { count: undoSnapshot.ids.length })}
          </span>
          <Button
            className="flex items-center gap-1.5 px-3 py-1.5 bg-white/10 hover:bg-white/20 text-white rounded-xl text-[10px] font-extrabold uppercase tracking-widest transition-all active:scale-95 cursor-pointer"
            onClick={handleUndo}
            variant="ghost"
          >
            <Undo2 size={14} /> {t("shifts.monthly.undo")}
          </Button>
        </div>
      )}
    </>
  );
}

function EmployeeChecklist({
  hook,
  employees,
  onClearAll,
}: {
  hook: ReturnType<typeof useShifts>;
  employees: Employee[];
  onClearAll: () => void;
}) {
  const { t } = useTranslation();
  return (
    <div className="flex flex-wrap gap-2.5 p-3.5 bg-slate-50 dark:bg-slate-950/40 rounded-2xl border border-slate-200/50 animate-in fade-in slide-in-from-top-2 duration-300">
      <div className="w-full flex items-center justify-between mb-1">
        <span className="text-[10px] font-bold text-slate-400 uppercase tracking-widest">
          {t("shifts.monthly.selected_employees", { count: employees.length })}
        </span>
        <Button
          className="flex items-center gap-1.5 px-2.5 py-1 text-[10px] font-extrabold uppercase tracking-widest text-slate-500 hover:text-rose-600 hover:bg-rose-50 dark:hover:bg-rose-950/40 rounded-lg transition-all cursor-pointer"
          onClick={onClearAll}
          type="button"
          variant="ghost"
        >
          <X size={12} /> {t("shifts.monthly.clear_all_employees")}
        </Button>
      </div>
      {employees.map((emp) => (
        <label className="flex items-center gap-2 px-3 py-1.5 bg-white border border-slate-200 rounded-xl text-xs font-bold text-slate-700 shadow-sm hover:border-primary-500/50 cursor-pointer select-none transition-all" key={emp.id}>
          <input
            checked={hook.selectedEmployeeIds.includes(emp.id)}
            className="rounded border-slate-300 text-primary-600 focus:ring-primary-500 cursor-pointer h-3.5 w-3.5"
            onChange={() => {
              hook.setSelectedEmployeeIds(hook.selectedEmployeeIds.filter((id) => id !== emp.id));
              hook.setSelectedEmployees(employees.filter(e => e.id !== emp.id));
            }}
            type="checkbox"
          />
          <span>{emp.full_name}</span>
        </label>
      ))}
    </div>
  );
}

function WeekViewControls({ hook }: { hook: ReturnType<typeof useShifts> }) {
  const { t } = useTranslation();

  const onCopyPreviousWeek = () => void hook.handleCopyPreviousWeek();
  const onClearWeek = () => { hook.handleAssignAllWeek(null); };
  const onAssignAllWeek = () => {
    hook.setTargetDates(hook.weekDays.map((d) => format(d, "yyyy-MM-dd")));
    hook.setIsShiftPickerOpen(true);
  };
  const onAssignPattern = () => { hook.setIsPatternModalOpen(true); };

  return (
    <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
      <div className="w-full md:max-w-md">
        <EmployeeSearch
          clearOnEscape
          data-testid="employee-select"
          departments={hook.departments}
          keyboardShortcut={["/", "Enter"]}
          onChange={(id) => { if (!id) hook.setSelectedEmployee(null); }}
          onSelectEmployee={(emp) => { hook.setSelectedEmployee(emp); }}
          placeholder={t("shifts.empty_state.title")}
          value={hook.selectedEmployee?.id ?? ""}
          workspaceId={hook.wsId}
        />
      </div>
      {!!hook.selectedEmployee && (
        <div className="w-full md:w-56">
          <Combobox
            className="h-10 bg-slate-100 dark:bg-slate-700 border-slate-200 text-[10px] py-2 px-3 rounded-xl uppercase tracking-tighter"
            onChange={(value) => {
              if (value === "copy-prev") onCopyPreviousWeek();
              else if (value === "clear-week") onClearWeek();
              else if (value === "assign-all") onAssignAllWeek();
              else if (value === "assign-pattern") onAssignPattern();
            }}
            options={[
              { value: "copy-prev", label: t("shifts.actions.copy_previous"), icon: <Copy className="w-4 h-4" /> },
              { value: "assign-pattern", label: t("patterns.assign_btn"), icon: <Zap className="w-4 h-4" /> },
              { value: "assign-all", label: t("shifts.actions.assign_all_week"), icon: <CalendarIcon className="w-4 h-4" /> },
              { value: "clear-week", label: t("shifts.actions.clear_week"), icon: <Trash className="w-4 h-4" /> },
            ]}
            placeholder={t("common.quick_actions")}
          />
        </div>
      )}
    </div>
  );
}

function WeekActionBar({
  selectedDays,
  onOpenWeekShiftPicker,
  onClearWeekSelection,
}: {
  selectedDays?: string[];
  onOpenWeekShiftPicker?: (dates: string[]) => void;
  onClearWeekSelection?: () => void;
}) {
  const { t } = useTranslation();
  return (
    <div className="absolute inset-0 z-40 bg-primary-600 text-white px-6 py-4 rounded-[28px] flex items-center justify-between shadow-xl animate-in fade-in duration-300">
      <div className="flex items-center gap-4">
        <span className="text-xs font-bold uppercase tracking-widest">{t("shifts.days_selected", { count: selectedDays?.length ?? 0 })}</span>
        <div className="h-4 w-px bg-white/20" />
        <Button className="bg-white/10 border border-white/20 rounded-xl px-4 py-2 text-[10px] font-bold uppercase tracking-widest hover:bg-white/20 transition-all flex items-center gap-2 cursor-pointer"
          onClick={() => { if (selectedDays && onOpenWeekShiftPicker) onOpenWeekShiftPicker(selectedDays); }} variant="ghost">
          <Plus size={14} /> {t("shifts.assign_selected_btn")}
        </Button>
      </div>
      <Button className="hover:bg-white/10 p-2 rounded-xl transition-all cursor-pointer" onClick={onClearWeekSelection} variant="ghost">
        <X size={18} />
      </Button>
    </div>
  );
}

function MonthOverlay({
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
      assignLabel={t("shifts.monthly.assign_selected")}
      cancelLabel={t("common.cancel")}
      clearLabel={t("shifts.monthly.clear_month")}
      isMutating={isMutating ?? false}
      onApplyShift={onApplyShift ?? (() => {})}
      onAssignSelected={onAssignSelected ?? (() => {})}
      onCancel={onCancelSelection ?? (() => {})}
      onClearOverrides={onClearOverrides ?? (() => {})}
      selectedCount={selectedCells.size}
      selectedLabel={t("shifts.monthly.cells_selected", { count: selectedCells.size })}
      shifts={hook.shifts}
    />
  );
}

export const RosterToolbar = memo(function RosterToolbar(props: RosterToolbarProps) {
  const { hook, viewMode, selectedCells, selectedDays } = props;

  return (
    <div className="relative">
      <div className="flex flex-col gap-4 bg-white dark:bg-slate-900 p-5 rounded-[28px] border border-slate-100 dark:border-slate-800 shadow-sm">
        <RosterNavHeader hook={hook} isCopying={props.isCopying} onCopyMonth={props.onCopyMonth} viewMode={viewMode} />
        <div className="h-px bg-slate-100 dark:bg-slate-800 w-full" />
        <div className="flex flex-col gap-3.5 w-full animate-in fade-in duration-200">
          {viewMode === "month" ? <MonthViewControls hook={hook} selectedEmployeesMeta={props.selectedEmployeesMeta} /> : <WeekViewControls hook={hook} />}
        </div>
      </div>

      {viewMode === "month" && <MonthOverlay hook={hook} isMutating={props.isMutating} onApplyShift={props.onApplyShift} onAssignSelected={props.onAssignSelected} onCancelSelection={props.onCancelSelection} onClearOverrides={props.onClearOverrides} selectedCells={selectedCells} />}
      {viewMode === "week" && !!selectedDays && selectedDays.length > 0 && <WeekActionBar onClearWeekSelection={props.onClearWeekSelection} onOpenWeekShiftPicker={props.onOpenWeekShiftPicker} selectedDays={selectedDays} />}
    </div>
  );
});

RosterToolbar.displayName = "RosterToolbar";
