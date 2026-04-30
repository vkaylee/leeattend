import React from "react";

import { format } from "date-fns";
import { vi, enUS, zhCN, type Locale } from "date-fns/locale";
import {
  ChevronLeft,
  ChevronRight,
  Copy,
  Trash,
  Calendar as CalendarIcon,
  X,
  Plus,
  Clock,
  Zap,
  ShieldCheck,
  Building2,
  PartyPopper,
} from "lucide-react";
import { useTranslation } from "react-i18next";

import { Combobox } from "@/components/ui/Combobox";

import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { Employee, EmployeeSchedule } from "@/lib/api/types";

import { ScheduleTracePopover } from "./ScheduleTracePopover";

 
const dateLocales: Record<string, Locale> = {
  vi: vi,
  en: enUS,
  zh: zhCN,
};

interface WeeklyRosterGridProps {
  selectedEmployee: Employee;
  currentDate: Date;
  onNavigateWeek: (direction: "prev" | "next") => void;
  weekDays: Date[];
  selectedDays: string[];
  onToggleDaySelection: (dateStr: string) => void;
  onClearSelection: () => void;
 
  employeeSchedule?: EmployeeSchedule;
  onCopyPreviousWeek: () => void;
  onAssignAllWeek: () => void;
  onClearWeek: () => void;
  onOpenShiftPicker: (dates: string[]) => void;
  onDeleteRoster: (date: string) => void;
 
  currentLocale?: Locale;
}

export function WeeklyRosterGrid({
  selectedEmployee,

  onNavigateWeek,
  weekDays,
  selectedDays,
  onToggleDaySelection,
  onClearSelection,
  employeeSchedule,
  onCopyPreviousWeek,
  onAssignAllWeek,
  onClearWeek,
  onOpenShiftPicker,
  onDeleteRoster,
}: WeeklyRosterGridProps) {
  const { t, i18n } = useTranslation();
  const currentLocale = dateLocales[i18n.language.substring(0, 2)] || enUS;
  const { formatDate, formatTime } = useDateTimeFormatter();

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between bg-white dark:bg-slate-900 p-4 rounded-[32px] border border-slate-200 dark:border-slate-700 shadow-sm">
        <div className="flex items-center gap-4">
          <div className="w-12 h-12 bg-primary-600 rounded-2xl flex items-center justify-center text-white text-lg font-bold shadow-lg shadow-primary-100">
            {selectedEmployee.full_name[0]}
          </div>
          <div>
            <h2 className="text-lg font-bold text-slate-700 dark:text-slate-200 leading-tight">{selectedEmployee.full_name}</h2>
            <p className="text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400">{t("shifts.current_week_roster")}</p>
          </div>
        </div>

        <div className="flex items-center gap-3">
          <div className="w-56">
            <Combobox
              options={[
                { value: "copy-prev", label: t("shifts.actions.copy_previous"), icon: <Copy className="w-4 h-4" /> },
                { value: "assign-all", label: t("shifts.actions.assign_all_week"), icon: <CalendarIcon className="w-4 h-4" /> },
                { value: "clear-week", label: t("shifts.actions.clear_week"), icon: <Trash className="w-4 h-4" /> },
              ]}
              onChange={(val) => {
                if (val === "copy-prev") onCopyPreviousWeek();
                else if (val === "clear-week") onClearWeek();
                else if (val === "assign-all") onAssignAllWeek();
              }}
              placeholder={t("common.quick_actions")}
              className="h-10 bg-slate-100 dark:bg-slate-700 border-slate-200 dark:border-slate-700 text-[10px] py-2 px-3 rounded-xl uppercase tracking-tighter"
            />
          </div>
          <button
            onClick={() => onNavigateWeek("prev")}
            className="p-2.5 hover:bg-slate-100 dark:hover:bg-slate-700 border border-slate-100 dark:border-slate-800 rounded-xl transition-all"
          >
            <ChevronLeft size={18} />
          </button>
          <div className="px-4 py-2 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-xl text-xs font-bold text-slate-600 dark:text-slate-300 uppercase tracking-tighter">
            {formatDate(weekDays[0])} - {formatDate(weekDays[6])}
          </div>
          <button
            onClick={() => onNavigateWeek("next")}
            className="p-2.5 hover:bg-slate-100 dark:hover:bg-slate-700 border border-slate-100 dark:border-slate-800 rounded-xl transition-all"
          >
            <ChevronRight size={18} />
          </button>
        </div>
      </div>

      {selectedDays.length > 0 && (
        <div className="bg-primary-600 text-white p-4 rounded-[24px] flex items-center justify-between shadow-xl shadow-primary-100 dark:shadow-black/30 animate-in slide-in-from-top-4 duration-300">
          <div className="flex items-center gap-4">
            <span className="text-xs font-bold uppercase tracking-widest">
              {t("shifts.days_selected", { count: selectedDays.length })}
            </span>
            <div className="h-4 w-px bg-white dark:bg-slate-900/20" />
            <button
              onClick={() => onOpenShiftPicker(selectedDays)}
              className="bg-white dark:bg-slate-900/10 border border-white/20 rounded-xl px-4 py-2 text-[10px] font-bold uppercase tracking-widest hover:bg-white dark:hover:bg-slate-900/20 transition-all flex items-center gap-2"
            >
              <Plus size={14} /> {t("shifts.assign_selected_btn")}
            </button>
          </div>
          <button onClick={onClearSelection} className="hover:bg-white dark:hover:bg-slate-900/10 p-2 rounded-xl transition-all">
            <X size={18} />
          </button>
        </div>
      )}

      <div className="grid grid-cols-1 md:grid-cols-7 gap-4">
        {weekDays.map((day, idx) => {
          const dateStr = format(day, 'yyyy-MM-dd');
          const isSelected = selectedDays.includes(dateStr);
 
          const effectiveSchedule = employeeSchedule?.schedules.find((s) => s.date === dateStr);
          const isToday = day.toDateString() === new Date().toDateString();
 
          const source = effectiveSchedule?.source;
          const isHoliday = effectiveSchedule?.is_holiday === true;
          const holidayName = effectiveSchedule?.holiday_name;

          return (
 
 
            <div
              key={idx}
              onClick={() => onToggleDaySelection(dateStr)}
              onKeyDown={(e) => { if(e.key === 'Enter' || e.key === ' ') { e.preventDefault(); onToggleDaySelection(dateStr); } }}
              role="button"
              tabIndex={0}
              className={`bg-white dark:bg-slate-900 border p-4 rounded-[32px] flex flex-col min-h-[350px] transition-all relative group cursor-pointer ${isSelected
                  ? 'border-primary-500 ring-4 ring-primary-50 shadow-md'
                  : isHoliday
                    ? 'border-amber-300 dark:border-amber-700 ring-4 ring-amber-50 dark:ring-amber-950 shadow-lg'
                    : isToday
                      ? 'border-primary-300 ring-4 ring-primary-50 shadow-lg'
                      : 'border-slate-100 dark:border-slate-800 hover:border-slate-200 dark:hover:border-slate-700 shadow-sm'
                }`}
            >
              <div className="text-center mb-5 pb-5 border-b border-slate-50 dark:border-slate-800 relative">
                {isSelected && (
                  <div className="absolute -top-1 -right-1 bg-primary-600 text-white rounded-full p-1 shadow-sm">
                    <ShieldCheck size={12} />
                  </div>
                )}
                <div className={`text-[10px] font-bold uppercase tracking-tighter ${isToday ? 'text-primary-600 dark:text-primary-400' : 'text-slate-500 dark:text-slate-400'}`}>
                  {format(day, "EEEE", { locale: currentLocale })}
                </div>
                <div className={`text-2xl font-bold mt-1 ${isToday ? 'text-primary-600 dark:text-primary-400' : 'text-slate-700 dark:text-slate-200'}`}>
                  {day.getDate()}
                </div>
                {isHoliday && (
                  <div className="mt-1.5 flex items-center justify-center gap-1 text-[9px] font-bold text-amber-600 dark:text-amber-400 bg-amber-50 dark:bg-amber-950/50 px-2 py-0.5 rounded-full border border-amber-200 dark:border-amber-800 uppercase tracking-tighter">
                    <PartyPopper size={9} />
                    <span className="truncate max-w-[100px]">{holidayName}</span>
                  </div>
                )}
              </div>
              <div className="flex-1 space-y-4">
                <div className="flex justify-center items-center gap-1">
                  {source === "roster" && (
                    <span className="text-[9px] font-bold px-2 py-0.5 rounded-full bg-teal-50 dark:bg-teal-950 text-teal-600 dark:text-teal-300 border border-teal-100 uppercase tracking-tighter flex items-center gap-1">
                      <Zap size={8} /> {t("shifts.types.flexible")}
                    </span>
                  )}
                  {source === "pattern" && (
                    <span className="text-[9px] font-bold px-2 py-0.5 rounded-full bg-blue-50 dark:bg-blue-950 text-blue-600 dark:text-blue-300 border border-blue-100 dark:border-blue-800 uppercase tracking-tighter flex items-center gap-1">
                      <ShieldCheck size={8} /> {t("patterns.title")}
                    </span>
                  )}
                  {source === "department_pattern" && (
                    <span className="text-[9px] font-bold px-2 py-0.5 rounded-full bg-slate-50 dark:bg-slate-800 text-slate-600 dark:text-slate-300 border border-slate-200 dark:border-slate-700 uppercase tracking-tighter flex items-center gap-1">
                      <Building2 size={8} /> {t("patterns.title")}
                    </span>
                  )}
                  {source === "fixed" && (
                    <span className="text-[9px] font-bold px-2 py-0.5 rounded-full bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border border-emerald-100 dark:border-emerald-800 uppercase tracking-tighter flex items-center gap-1">
                      <ShieldCheck size={8} /> {t("shifts.types.fixed")}
                    </span>
                  )}
                  {source === "department_fixed" && (
                    <span className="text-[9px] font-bold px-2 py-0.5 rounded-full bg-slate-50 dark:bg-slate-800 text-slate-500 dark:text-slate-400 border border-slate-100 dark:border-slate-800 uppercase tracking-tighter flex items-center gap-1">
                      <Building2 size={8} /> {t("common.department")}
                    </span>
                  )}
                  {effectiveSchedule && (
                    <ScheduleTracePopover traces={effectiveSchedule.traces || []} source={effectiveSchedule.source} />
                  )}
                </div>
                {effectiveSchedule?.shift_id ? (
                  <div
                    className={`p-4 rounded-[24px] relative group/item animate-in zoom-in-95 border ${source === 'roster' ? 'bg-teal-50 dark:bg-teal-950/50 border-teal-100' : 'bg-primary-50 dark:bg-primary-950 border-blue-100 dark:border-blue-800'
                      }`}
                  >
                    <div className={`font-bold text-xs mb-1 uppercase tracking-tight leading-tight ${source === 'roster' ? 'text-teal-700 dark:text-teal-300' : 'text-blue-700 dark:text-blue-300'}`}>
                      {effectiveSchedule.shift_name}
                    </div>
                    <div className={`flex items-center gap-1.5 text-[10px] font-bold ${source === 'roster' ? 'text-teal-500' : 'text-blue-500'}`}>
                      {formatTime(effectiveSchedule.start_time)} - {formatTime(effectiveSchedule.end_time)}
                    </div>
                    {effectiveSchedule.break_duration_minutes !== null && (
                      <div className="mt-2 pt-2 border-t border-black/5 flex items-center gap-1.5 text-[9px] font-bold uppercase tracking-tighter text-slate-500 dark:text-slate-400">
                        <Clock size={8} />
                        {effectiveSchedule.break_start ? (
                          <span>
                            {t("common.break")}: {effectiveSchedule.break_start.substring(0, 5)} - {effectiveSchedule.break_end?.substring(0, 5)}
                          </span>
                        ) : (
                          <span>{t("common.break")}: {effectiveSchedule.break_duration_minutes}m</span>
                        )}
                      </div>
                    )}
                    {source === 'roster' && (
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          onDeleteRoster(dateStr);
                        }}
                        className="absolute -top-2 -right-2 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-full p-1.5 shadow-md opacity-0 group-hover/item:opacity-100 transition-opacity hover:bg-rose-50 dark:bg-rose-950 hover:border-rose-200 group/btn"
                      >
                        <X size={12} className="text-slate-500 dark:text-slate-400 group-hover/btn:text-rose-500" />
                      </button>
                    )}
                  </div>
                ) : (
                  <div className="bg-slate-50 dark:bg-slate-800/50 border border-slate-100 dark:border-slate-800 border-dashed p-6 rounded-[24px] text-[10px] text-slate-500 dark:text-slate-400 font-bold text-center uppercase tracking-widest flex flex-col items-center justify-center gap-2">
                    {t("shifts.off_day")}
                  </div>
                )}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
