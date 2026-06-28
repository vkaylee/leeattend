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
  X,
} from "lucide-react";
import React, { memo } from "react";
import { useTranslation } from "react-i18next";

import { format } from "date-fns";
import { vi, enUS, zhCN, type Locale } from "date-fns/locale";

import { EmployeeSearch } from "@/components/common/employee-search";
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

export const RosterToolbar = memo(function RosterToolbar({
  hook,
  viewMode,
  isCopying,
  onCopyMonth,
  selectedEmployeesMeta,
  selectedCells,
  onCancelSelection,
  onAssignSelected,
  onClearOverrides,
  onApplyShift,
  isMutating,
  selectedDays,
  onClearWeekSelection,
  onOpenWeekShiftPicker,
}: RosterToolbarProps) {
  const { i18n, t } = useTranslation();
  const currentLocale = dateLocales[i18n.language.substring(0, 2)] ?? enUS;

  const selectedMetaRef = React.useRef<Employee[]>(selectedEmployeesMeta || []);

  React.useEffect(() => {
    selectedMetaRef.current = selectedEmployeesMeta || [];
  }, [selectedEmployeesMeta]);
  const { formatDate } = useDateTimeFormatter();

  const onCopyPreviousWeek = () => {
    void hook.handleCopyPreviousWeek();
  };

  const onClearWeek = () => {
    hook.handleAssignAllWeek(null);
  };

  const onAssignAllWeek = () => {
    hook.setTargetDates(hook.weekDays.map((d) => format(d, "yyyy-MM-dd")));
    hook.setIsShiftPickerOpen(true);
  };

  const onAssignPattern = () => {
    hook.setIsPatternModalOpen(true);
  };

  return (
    <div className="relative">
      <div className="flex flex-col gap-4 bg-white dark:bg-slate-900 p-5 rounded-[28px] border border-slate-100 dark:border-slate-800 shadow-sm">
        {/* Row 1: Header, Navigation & View-specific Actions */}
        <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3">
          <div className="flex items-center gap-2">
            {viewMode === "week" ? (
              <CalendarIcon className="text-primary-600 dark:text-primary-400" size={20} />
            ) : (
              <CalendarDays className="text-primary-600 dark:text-primary-400" size={20} />
            )}
            <div className="flex flex-col">
              <Typography className="text-sm font-bold uppercase tracking-wider text-slate-700 dark:text-slate-200" variant="h2">
                {viewMode === "week" 
                  ? t("shifts.weekly.title", "Lịch tuần") 
                  : t("shifts.monthly.title", "Lịch tháng")}
              </Typography>
              {viewMode === "week" && !!hook.selectedEmployee && (
                <Typography className="text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 leading-none mt-0.5" variant="p">
                  {t("shifts.current_week_roster")}
                </Typography>
              )}
            </div>
          </div>

          <div className="flex items-center justify-between sm:justify-end gap-3 w-full sm:w-auto">
            <div className="flex items-center gap-1">
              <Button className="p-2 rounded-xl hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors border border-slate-100 dark:border-slate-800"
                onClick={() => {
                  if (viewMode === "week") {
                    hook.handleNavigateWeek("prev");
                  } else {
                    hook.handleNavigateMonth("prev");
                  }
                }}
                type="button"
                variant="ghost"
              >
                <ChevronLeft size={16} />
              </Button>
              <span className="min-w-[140px] text-center font-bold text-xs uppercase tracking-widest text-slate-700 dark:text-slate-200">
                {viewMode === "week"
                  ? `${formatDate(hook.weekDays[0])} - ${formatDate(hook.weekDays[6])}`
                  : format(hook.currentMonth, "MMMM yyyy", { locale: currentLocale })}
              </span>
              <Button className="p-2 rounded-xl hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors border border-slate-100 dark:border-slate-800"
                onClick={() => {
                  if (viewMode === "week") {
                    hook.handleNavigateWeek("next");
                  } else {
                    hook.handleNavigateMonth("next");
                  }
                }}
                type="button"
                variant="ghost"
              >
                <ChevronRight size={16} />
              </Button>
            </div>

            {viewMode === "month" && (
              <Button className="flex items-center gap-1.5 px-3 py-2 text-xs font-bold uppercase tracking-wider border border-slate-200 dark:border-slate-800 rounded-xl hover:bg-slate-50 dark:hover:bg-slate-800/80 transition-colors text-slate-600 dark:text-slate-300 disabled:opacity-50"
                disabled={isCopying}
                onClick={onCopyMonth}
                type="button"
                variant="ghost"
              >
                {isCopying ? <Loader2 className="animate-spin" size={12} /> : <Copy size={12} />}
                <span>{t("shifts.monthly.copy_previous_month", "Sao chép tháng trước")}</span>
              </Button>
            )}
          </div>
        </div>

        {/* Separator line */}
        <div className="h-px bg-slate-100 dark:bg-slate-800 w-full" />

        {/* Row 2: Search Input & Selection-specific controls */}
        <div className="flex flex-col gap-3.5 w-full animate-in fade-in duration-200">
          {viewMode === "month" ? (
            <>
              <div className="w-full md:max-w-md">
                <EmployeeSearch
                  clearOnEscape
                  data-testid="monthly-employee-search"
                  hideChips
                  multiple
                  onChange={(val) => {
                    const ids = Array.isArray(val) ? val : (val ? [val] : []);
                    const currentMeta = selectedMetaRef.current;
                    const filteredMeta = currentMeta.filter(e => ids.includes(e.id));
                    selectedMetaRef.current = filteredMeta;
                    hook.setSelectedEmployeeIds(ids);
                    hook.setSelectedEmployees(filteredMeta);
                  }}
                  onSelectEmployee={(emp) => {
                    const currentMeta = selectedMetaRef.current;
                    if (!currentMeta.some(e => e.id === emp.id)) {
                      const newMeta = [...currentMeta, emp];
                      selectedMetaRef.current = newMeta;
                      hook.setSelectedEmployees(newMeta);
                    }
                  }}
                  placeholder={t("shifts.monthly.search_placeholder", "Tìm kiếm nhân sự...")}
                  value={hook.selectedEmployeeIds}
                  workspaceId={hook.wsId}
                />
              </div>

              {/* Selected Checklist (Stick) */}
              {!!selectedEmployeesMeta && selectedEmployeesMeta.length > 0 && (
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
                        checked={hook.selectedEmployeeIds.includes(emp.id)}
                        className="rounded border-slate-350 dark:border-slate-700 text-primary-600 focus:ring-primary-500 cursor-pointer h-3.5 w-3.5"
                        onChange={() => {
                          const newIds = hook.selectedEmployeeIds.filter((id) => id !== emp.id);
                          const newMeta = selectedEmployeesMeta.filter(e => e.id !== emp.id);
                          hook.setSelectedEmployeeIds(newIds);
                          hook.setSelectedEmployees(newMeta);
                        }}
                        type="checkbox"
                      />
                      <span>{emp.full_name}</span>
                    </label>
                  ))}
                </div>
              )}
            </>
          ) : (
            <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
              {/* Single-select search bar for Week view */}
              <div className="w-full md:max-w-md">
                <EmployeeSearch
                  clearOnEscape
                  data-testid="employee-select"
                  keyboardShortcut={["/", "Enter"]}
                  onChange={(id) => {
                    if (!id) hook.setSelectedEmployee(null);
                  }}
                  onSelectEmployee={(emp) => {
                    hook.setSelectedEmployee(emp);
                  }}
                  placeholder={t("shifts.empty_state.title", "Tìm kiếm nhân sự...")}
                  value={hook.selectedEmployee?.id ?? ""}
                  workspaceId={hook.wsId}
                />
              </div>

              {/* Quick Actions Dropdown for Week view when employee is selected */}
              {!!hook.selectedEmployee && (
                <div className="w-full md:w-56">
                  <Combobox
                    className="h-10 bg-slate-100 dark:bg-slate-700 border-slate-200 dark:border-slate-700 text-[10px] py-2 px-3 rounded-xl uppercase tracking-tighter"
                    onChange={(value) => {
                      if (value === "copy-prev") onCopyPreviousWeek();
                      else if (value === "clear-week") onClearWeek();
                      else if (value === "assign-all") onAssignAllWeek();
                      else if (value === "assign-pattern") onAssignPattern();
                    }}
                    options={[
                      { value: "copy-prev", label: t("shifts.actions.copy_previous", "Sao chép tuần trước"), icon: <Copy className="w-4 h-4" /> },
                      { value: "assign-pattern", label: t("patterns.assign_btn", "Gán ca tuần tự (Pattern)"), icon: <Zap className="w-4 h-4" /> },
                      { value: "assign-all", label: t("shifts.actions.assign_all_week", "Gán cả tuần"), icon: <CalendarIcon className="w-4 h-4" /> },
                      { value: "clear-week", label: t("shifts.actions.clear_week", "Xóa lịch tuần này"), icon: <Trash className="w-4 h-4" /> },
                    ]}
                    placeholder={t("common.quick_actions", "Thao tác nhanh")}
                  />
                </div>
              )}
            </div>
          )}
        </div>
      </div>

      {/* Month Selection Action Bar Overlay */}
      {viewMode === "month" && !!selectedCells && selectedCells.size > 0 && (
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
      )}

      {/* Week Selection Action Bar Overlay */}
      {viewMode === "week" && !!selectedDays && selectedDays.length > 0 && (
        <div className="absolute inset-0 z-40 bg-primary-600 text-white px-6 py-4 rounded-[28px] flex items-center justify-between shadow-xl shadow-primary-100 dark:shadow-black/30 animate-in fade-in duration-300">
          <div className="flex items-center gap-4">
            <span className="text-xs font-bold uppercase tracking-widest">
              {t("shifts.days_selected", { count: selectedDays.length })}
            </span>
            <div className="h-4 w-px bg-white dark:bg-slate-900/20" />
            <Button className="bg-white dark:bg-slate-900/10 border border-white/20 rounded-xl px-4 py-2 text-[10px] font-bold uppercase tracking-widest hover:bg-white dark:hover:bg-slate-900/20 transition-all flex items-center gap-2 cursor-pointer"
              onClick={() => { if (onOpenWeekShiftPicker) onOpenWeekShiftPicker(selectedDays); }}
              variant="ghost"
            >
              <Plus size={14} /> {t("shifts.assign_selected_btn")}
            </Button>
          </div>
          <Button className="hover:bg-white dark:hover:bg-slate-900/10 p-2 rounded-xl transition-all cursor-pointer" onClick={onClearWeekSelection} variant="ghost">
            <X size={18} />
          </Button>
        </div>
      )}
    </div>
  );
});

RosterToolbar.displayName = "RosterToolbar";
