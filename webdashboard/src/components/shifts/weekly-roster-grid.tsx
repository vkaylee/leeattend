import type { TFunction } from "i18next";

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
import React from "react";
import { useTranslation } from "react-i18next";

import { format } from "date-fns";
import { vi, enUS, zhCN, type Locale } from "date-fns/locale";

import { Combobox } from "@/components/ui/combobox";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { Employee, EmployeeSchedule, EffectiveSchedule } from "@/lib/api/types";

import { ScheduleTracePopover } from "./schedule-trace-popover";

const dateLocales: Partial<Record<string, Locale>> = {
  vi: vi,
  en: enUS,
  zh: zhCN,
};

interface WeeklyRosterGridProps {
  currentDate: Date;
  currentLocale?: Locale;
  employeeSchedule?: EmployeeSchedule;
  onAssignAllWeek: () => void;
  onClearSelection: () => void;
  onClearWeek: () => void;
  onCopyPreviousWeek: () => void;
 
  onDeleteRoster: (date: string) => void;
  onNavigateWeek: (direction: "prev" | "next") => void;
  onOpenShiftPicker: (dates: string[]) => void;
  onToggleDaySelection: (dateString: string) => void;
  selectedDays: string[];
  selectedEmployee: Employee;
 
  weekDays: Date[];
}

interface DayCardHeaderProps {
  currentLocale: Locale;
  day: Date;
  holidayName?: string | null;
  isHoliday: boolean;
  isSelected: boolean;
  isToday: boolean;
}

function DayCardHeader({ day, currentLocale, isToday, isSelected, isHoliday, holidayName }: DayCardHeaderProps) {
  return (
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
  );
}

interface DayCardSourceTagsProps {
  effectiveSchedule?: EffectiveSchedule;
  source?: string;
  t: TFunction;
}

function DayCardSourceTags({ source, effectiveSchedule, t }: DayCardSourceTagsProps) {
  return (
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
      {!!effectiveSchedule && <ScheduleTracePopover source={effectiveSchedule.source} traces={effectiveSchedule.traces} />}
    </div>
  );
}

interface DayCardShiftDetailsProps {
  dateString: string;
  effectiveSchedule?: EffectiveSchedule;
  formatTime: (time: string | null | undefined) => string;
  onDeleteRoster: (date: string) => void;
  source?: string;
  t: TFunction;
}

function DayCardShiftDetails({ effectiveSchedule, source, formatTime, onDeleteRoster, dateString, t }: DayCardShiftDetailsProps) {
  if (!effectiveSchedule?.shift_id) {
    return (
      <div className="bg-slate-50 dark:bg-slate-800/50 border border-slate-100 dark:border-slate-800 border-dashed p-6 rounded-[24px] text-[10px] text-slate-500 dark:text-slate-400 font-bold text-center uppercase tracking-widest flex flex-col items-center justify-center gap-2">
        {t("shifts.off_day")}
      </div>
    );
  }

  return (
    <div
      className={`p-4 rounded-[24px] relative group/item animate-in zoom-in-95 border ${source === 'roster' ? 'bg-teal-50 dark:bg-teal-950/50 border-teal-100' : 'bg-primary-50 dark:bg-primary-950 border-blue-100 dark:border-blue-800'}`}
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
          className="absolute -top-2 -right-2 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-full p-1.5 shadow-md opacity-0 group-hover/item:opacity-100 transition-opacity hover:bg-rose-50 dark:bg-rose-950 hover:border-rose-200 group/btn"
          onClick={(e) => {
            e.stopPropagation();
            onDeleteRoster(dateString);
          }}
        >
          <X className="text-slate-500 dark:text-slate-400 group-hover/btn:text-rose-500" size={12} />
        </button>
      )}
    </div>
  );
}

interface WeeklyRosterDayCardProps {
  currentLocale: Locale;
  dateString: string;
  day: Date;
  effectiveSchedule?: EffectiveSchedule;
  formatTime: (time: string | null | undefined) => string;
  isHoliday: boolean;
  isSelected: boolean;
  isToday: boolean;
  onDeleteRoster: (date: string) => void;
  onToggleDaySelection: (dateString: string) => void;
  t: TFunction;
}

function WeeklyRosterDayCard({
  currentLocale,
  dateString,
  day,
  effectiveSchedule,
  formatTime,
  isHoliday,
  isSelected,
  isToday,
  onDeleteRoster,
  onToggleDaySelection,
  t
}: WeeklyRosterDayCardProps) {
  const holidayName = effectiveSchedule?.holiday_name;
  const source = effectiveSchedule?.source;

  return (
    <div
      className={`bg-white dark:bg-slate-900 border p-4 rounded-[32px] flex flex-col min-h-[350px] transition-all relative group cursor-pointer ${isSelected
          ? 'border-primary-500 ring-4 ring-primary-50 shadow-md'
          : isHoliday
            ? 'border-amber-300 dark:border-amber-700 ring-4 ring-amber-50 dark:ring-amber-950 shadow-lg'
            : isToday
              ? 'border-primary-300 ring-4 ring-primary-50 shadow-lg'
              : 'border-slate-100 dark:border-slate-800 hover:border-slate-200 dark:hover:border-slate-700 shadow-sm'
        }`}
      onClick={() => { onToggleDaySelection(dateString); }}
      onKeyDown={(e) => { if(e.key === 'Enter' || e.key === ' ') { e.preventDefault(); onToggleDaySelection(dateString); } }}
      role="button"
      tabIndex={0}
    >
      <DayCardHeader currentLocale={currentLocale} day={day} holidayName={holidayName} isHoliday={isHoliday} isSelected={isSelected} isToday={isToday} />
      <div className="flex-1 space-y-4">
        <DayCardSourceTags effectiveSchedule={effectiveSchedule} source={source} t={t} />
        <DayCardShiftDetails dateString={dateString} effectiveSchedule={effectiveSchedule} formatTime={formatTime} onDeleteRoster={onDeleteRoster} source={source} t={t} />
      </div>
    </div>
  );
}

export function WeeklyRosterGridHeader({
  selectedEmployee,
  t,
  onCopyPreviousWeek,
  onClearWeek,
  onAssignAllWeek,
  onNavigateWeek,
  formatDate,
  weekDays
}: {
  formatDate: (date: string | Date | undefined) => string;
  onAssignAllWeek: () => void;
  onClearWeek: () => void;
  onCopyPreviousWeek: () => void;
  onNavigateWeek: (direction: "prev" | "next") => void;
  selectedEmployee: Employee;
  t: TFunction;
  weekDays: Date[];
}) {
  return (
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
            className="h-10 bg-slate-100 dark:bg-slate-700 border-slate-200 dark:border-slate-700 text-[10px] py-2 px-3 rounded-xl uppercase tracking-tighter"
            onChange={(value) => {
              if (value === "copy-prev") onCopyPreviousWeek();
              else if (value === "clear-week") onClearWeek();
              else if (value === "assign-all") onAssignAllWeek();
            }}
            options={[
              { value: "copy-prev", label: t("shifts.actions.copy_previous"), icon: <Copy className="w-4 h-4" /> },
              { value: "assign-all", label: t("shifts.actions.assign_all_week"), icon: <CalendarIcon className="w-4 h-4" /> },
              { value: "clear-week", label: t("shifts.actions.clear_week"), icon: <Trash className="w-4 h-4" /> },
            ]}
            placeholder={t("common.quick_actions")}
          />
        </div>
        <button
          className="p-2.5 hover:bg-slate-100 dark:hover:bg-slate-700 border border-slate-100 dark:border-slate-800 rounded-xl transition-all"
          onClick={() => { onNavigateWeek("prev"); }}
        >
          <ChevronLeft size={18} />
        </button>
        <div className="px-4 py-2 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-xl text-xs font-bold text-slate-600 dark:text-slate-300 uppercase tracking-tighter">
          {formatDate(weekDays[0])} - {formatDate(weekDays[6])}
        </div>
        <button
          className="p-2.5 hover:bg-slate-100 dark:hover:bg-slate-700 border border-slate-100 dark:border-slate-800 rounded-xl transition-all"
          onClick={() => { onNavigateWeek("next"); }}
        >
          <ChevronRight size={18} />
        </button>
      </div>
    </div>
  );
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
  const currentLocale = dateLocales[i18n.language.substring(0, 2)] ?? enUS;
  const { formatDate, formatTime } = useDateTimeFormatter();

  return (
    <div className="space-y-6">
      <WeeklyRosterGridHeader
        formatDate={formatDate}
        onAssignAllWeek={onAssignAllWeek}
        onClearWeek={onClearWeek}
        onCopyPreviousWeek={onCopyPreviousWeek}
        onNavigateWeek={onNavigateWeek}
        selectedEmployee={selectedEmployee}
        t={t}
        weekDays={weekDays}
      />

      {selectedDays.length > 0 && (
        <div className="bg-primary-600 text-white p-4 rounded-[24px] flex items-center justify-between shadow-xl shadow-primary-100 dark:shadow-black/30 animate-in slide-in-from-top-4 duration-300">
          <div className="flex items-center gap-4">
            <span className="text-xs font-bold uppercase tracking-widest">
              {t("shifts.days_selected", { count: selectedDays.length })}
            </span>
            <div className="h-4 w-px bg-white dark:bg-slate-900/20" />
            <button
              className="bg-white dark:bg-slate-900/10 border border-white/20 rounded-xl px-4 py-2 text-[10px] font-bold uppercase tracking-widest hover:bg-white dark:hover:bg-slate-900/20 transition-all flex items-center gap-2"
              onClick={() => { onOpenShiftPicker(selectedDays); }}
            >
              <Plus size={14} /> {t("shifts.assign_selected_btn")}
            </button>
          </div>
          <button className="hover:bg-white dark:hover:bg-slate-900/10 p-2 rounded-xl transition-all" onClick={onClearSelection}>
            <X size={18} />
          </button>
        </div>
      )}

      <div className="grid grid-cols-1 md:grid-cols-7 gap-4">
        {weekDays.map((day, index) => {
          const dateString = format(day, 'yyyy-MM-dd');
          const isSelected = selectedDays.includes(dateString);
 
          const effectiveSchedule = employeeSchedule?.schedules.find((s) => s.date === dateString);
          const isToday = day.toDateString() === new Date().toDateString();
 
          const isHoliday = effectiveSchedule?.is_holiday === true;

          return (
            <WeeklyRosterDayCard
              currentLocale={currentLocale}
              dateString={dateString}
              day={day}
              effectiveSchedule={effectiveSchedule}
              formatTime={formatTime}
              isHoliday={isHoliday}
              isSelected={isSelected}
              isToday={isToday}
              key={index}
              onDeleteRoster={onDeleteRoster}
              onToggleDaySelection={onToggleDaySelection}
              t={t}
            />
          );
        })}
      </div>
    </div>
  );
}
