import type { TFunction } from "i18next";

import {
  X,
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

import Button from "@/components/ui/button";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { Employee, EmployeeSchedule, EffectiveSchedule, Shift } from "@/lib/api/types";

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
  onClearSelection: () => void;
  onDeleteRoster: (date: string) => void;
  onOpenShiftPicker: (dates: string[]) => void;
  onToggleDaySelection: (dateString: string) => void;
  selectedDays: string[];
  selectedEmployee: Employee;
  shifts?: Shift[];
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
      {isSelected ? (
        <div className="absolute -top-1 -right-1 bg-primary-600 text-white rounded-full p-1 shadow-sm">
          <ShieldCheck size={12} />
        </div>
      ) : null}
      <div className={`text-[10px] font-bold uppercase tracking-tighter ${isToday ? 'text-primary-600 dark:text-primary-400' : 'text-slate-500 dark:text-slate-400'}`}>
        {format(day, "EEEE", { locale: currentLocale })}
      </div>
      <div className={`text-2xl font-bold mt-1 ${isToday ? 'text-primary-600 dark:text-primary-400' : 'text-slate-700 dark:text-slate-200'}`}>
        {day.getDate()}
      </div>
      {isHoliday ? (
        <div className="mt-1.5 flex items-center justify-center gap-1 text-[9px] font-bold text-amber-600 dark:text-amber-400 bg-amber-50 dark:bg-amber-950/50 px-2 py-0.5 rounded-full border border-amber-200 dark:border-amber-800 uppercase tracking-tighter">
          <PartyPopper size={9} />
          <span className="truncate max-w-[100px]">{holidayName}</span>
        </div>
      ) : null}
    </div>
  );
}

interface DayCardSourceTagsProps {
  daySchedules: EffectiveSchedule[];
  t: TFunction;
}

function DayCardSourceTags({ daySchedules, t }: DayCardSourceTagsProps) {
  const sources = Array.from(new Set(daySchedules.map((s) => s.source)));
  return (
    <div className="flex flex-wrap justify-center items-center gap-1">
      {sources.map((source) => {
        switch (source) {
          case "roster":
            return (
              <span className="text-[9px] font-bold px-2 py-0.5 rounded-full bg-teal-50 dark:bg-teal-950 text-teal-600 dark:text-teal-300 border border-teal-100 uppercase tracking-tighter flex items-center gap-1" key={source}>
                <Zap size={8} /> {t("shifts.types.flexible")}
              </span>
            );
          case "pattern":
            return (
              <span className="text-[9px] font-bold px-2 py-0.5 rounded-full bg-blue-50 dark:bg-blue-950 text-blue-600 dark:text-blue-300 border border-blue-100 dark:border-blue-800 uppercase tracking-tighter flex items-center gap-1" key={source}>
                <ShieldCheck size={8} /> {t("patterns.title")}
              </span>
            );
          case "department_pattern":
            return (
              <span className="text-[9px] font-bold px-2 py-0.5 rounded-full bg-slate-50 dark:bg-slate-800 text-slate-600 dark:text-slate-300 border border-slate-200 dark:border-slate-700 uppercase tracking-tighter flex items-center gap-1" key={source}>
                <Building2 size={8} /> {t("patterns.title")}
              </span>
            );
          case "fixed":
            return (
              <span className="text-[9px] font-bold px-2 py-0.5 rounded-full bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border border-emerald-100 dark:border-emerald-800 uppercase tracking-tighter flex items-center gap-1" key={source}>
                <ShieldCheck size={8} /> {t("shifts.types.fixed")}
              </span>
            );
          case "department_fixed":
            return (
              <span className="text-[9px] font-bold px-2 py-0.5 rounded-full bg-slate-50 dark:bg-slate-800 text-slate-500 dark:text-slate-400 border border-slate-100 dark:border-slate-800 uppercase tracking-tighter flex items-center gap-1" key={source}>
                <Building2 size={8} /> {t("common.department")}
              </span>
            );
          default:
            return null;
        }
      })}
    </div>
  );
}

interface DayCardShiftDetailsProps {
  dateString: string;
  daySchedules: EffectiveSchedule[];
  formatTime: (time: string | null | undefined) => string;
  onDeleteRoster: (date: string) => void;
  shifts?: Shift[];
  t: TFunction;
}

const getShiftSymbol = (shiftName?: string | null, reportSymbol?: string | null) => {
  if (reportSymbol) return reportSymbol;
  if (!shiftName) return "";
  return shiftName.substring(0, 2).toUpperCase();
};

function DayCardShiftDetails({ daySchedules, formatTime, onDeleteRoster, dateString, t, shifts }: DayCardShiftDetailsProps) {
  const workingSchedules = daySchedules.filter((s) => s.shift_id);

  if (workingSchedules.length === 0) {
    return (
      <div className="bg-slate-50 dark:bg-slate-800/50 border border-slate-100 dark:border-slate-800 border-dashed p-6 rounded-[24px] text-[10px] text-slate-500 dark:text-slate-400 font-bold text-center uppercase tracking-widest flex flex-col items-center justify-center gap-2">
        {t("shifts.off_day")}
      </div>
    );
  }

  return (
    <div className="space-y-3">
      {workingSchedules.length > 1 && (
        <div className="text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-wider text-center">
          {workingSchedules.length} {t("work_segments.segments", "segments")}
        </div>
      )}
      {workingSchedules.map((s, idx) => {
        const shiftConfig = shifts?.find((sh) => sh.id === s.shift_id);
        const shiftSymbol = getShiftSymbol(s.shift_name, shiftConfig?.report_symbol);
        return (
          <div
            className={`p-4 rounded-[24px] relative group/item animate-in zoom-in-95 border ${s.source === 'roster' ? 'bg-teal-50 dark:bg-teal-950/50 border-teal-100' : 'bg-primary-50 dark:bg-primary-950 border-blue-100 dark:border-blue-800'}`}
            key={idx}
          >
            <div className="flex justify-between items-start gap-2">
              <div className={`font-bold text-xs mb-1 uppercase tracking-tight leading-tight ${s.source === 'roster' ? 'text-teal-700 dark:text-teal-300' : 'text-blue-700 dark:text-blue-300'}`}>
                {s.shift_name}
              </div>
              {!!s.traces && s.traces.length > 0 && (
                <div className="shrink-0">
                  <ScheduleTracePopover 
                    endTime={s.end_time} 
                    shiftName={s.shift_name} 
                    shiftSymbol={shiftSymbol}
                    source={s.source}
                    startTime={s.start_time}
                    traces={s.traces}
                  />
                </div>
              )}
            </div>
            <div className={`flex items-center gap-1.5 text-[10px] font-bold ${s.source === 'roster' ? 'text-teal-500' : 'text-blue-500'}`}>
              {formatTime(s.start_time)} - {formatTime(s.end_time)}
            </div>
            {s.break_duration_minutes !== null && (
              <div className="mt-2 pt-2 border-t border-black/5 flex items-center gap-1.5 text-[9px] font-bold uppercase tracking-tighter text-slate-500 dark:text-slate-400">
                <Clock size={8} />
                {s.break_start ? (
                  <span>
                    {t("common.break")}: {s.break_start.substring(0, 5)} - {s.break_end?.substring(0, 5)}
                  </span>
                ) : (
                  <span>{t("common.break")}: {s.break_duration_minutes}m</span>
                )}
              </div>
            )}
            {s.source === 'roster' && (
              <Button className="absolute -top-2 -right-2 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-full p-1.5 shadow-md opacity-0 group-hover/item:opacity-100 transition-opacity hover:bg-rose-50 dark:bg-rose-950 hover:border-rose-200 group/btn"
                onClick={(e: React.MouseEvent) => {
                  e.stopPropagation();
                  onDeleteRoster(dateString);
                }}
                variant="ghost"
              >
                <X className="text-slate-500 dark:text-slate-400 group-hover/btn:text-rose-500" size={12} />
              </Button>
            )}
          </div>
        );
      })}
    </div>
  );
}

interface WeeklyRosterDayCardProps {
  currentLocale: Locale;
  dateString: string;
  day: Date;
  daySchedules: EffectiveSchedule[];
  formatTime: (time: string | null | undefined) => string;
  isHoliday: boolean;
  isSelected: boolean;
  isToday: boolean;
  onDeleteRoster: (date: string) => void;
  onToggleDaySelection: (dateString: string) => void;
  shifts?: Shift[];
  t: TFunction;
}

function WeeklyRosterDayCard({
  currentLocale,
  dateString,
  day,
  daySchedules,
  formatTime,
  isHoliday,
  isSelected,
  isToday,
  onDeleteRoster,
  onToggleDaySelection,
  t,
  shifts
}: WeeklyRosterDayCardProps) {
  const holidayName = daySchedules.find((s) => s.holiday_name)?.holiday_name;

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
        <DayCardSourceTags daySchedules={daySchedules} t={t} />
        <DayCardShiftDetails dateString={dateString} daySchedules={daySchedules} formatTime={formatTime} onDeleteRoster={onDeleteRoster} shifts={shifts} t={t} />
      </div>
    </div>
  );
}



export function WeeklyRosterGrid({
  selectedEmployee,
  weekDays,
  selectedDays,
  onToggleDaySelection,
  onClearSelection,
  employeeSchedule,
  onOpenShiftPicker,
  onDeleteRoster,
  shifts,
}: WeeklyRosterGridProps) {
  const { t, i18n } = useTranslation();
  const currentLocale = dateLocales[i18n.language.substring(0, 2)] ?? enUS;
  const { formatDate, formatTime } = useDateTimeFormatter();
  React.useEffect(() => {
    if (selectedDays.length === 0) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      // Ignore if user is typing in inputs or editing content
      const activeEl = document.activeElement;
      if (
        activeEl &&
        (activeEl.tagName === "INPUT" ||
          activeEl.tagName === "TEXTAREA" ||
          activeEl.getAttribute("contenteditable") === "true")
      ) {
        return;
      }

      if (e.key === "a" || e.key === "A") {
        e.preventDefault();
        onOpenShiftPicker(selectedDays);
      } else if (e.key === "Delete" || e.key === "Backspace") {
        e.preventDefault();
        selectedDays.forEach((dateStr) => {
          const schedule = employeeSchedule?.schedules.find((s) => s.date === dateStr);
          if (schedule?.source === "roster") {
            onDeleteRoster(dateStr);
          }
        });
        onClearSelection();
      } else if (e.key === "Escape") {
        e.preventDefault();
        onClearSelection();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [selectedDays, onOpenShiftPicker, onDeleteRoster, onClearSelection, employeeSchedule]);

  return (
    <div className="space-y-6">

      <div className="grid grid-cols-1 md:grid-cols-7 gap-4">
        {weekDays.map((day, index) => {
          const dateString = format(day, 'yyyy-MM-dd');
          const isSelected = selectedDays.includes(dateString);
 
          const daySchedules = employeeSchedule?.schedules.filter((s) => s.date === dateString) ?? [];
          const isToday = day.toDateString() === new Date().toDateString();
 
          const isHoliday = daySchedules.some((s) => s.is_holiday);

          return (
            <WeeklyRosterDayCard
              currentLocale={currentLocale}
              dateString={dateString}
              day={day}
              daySchedules={daySchedules}
              formatTime={formatTime}
              isHoliday={isHoliday}
              isSelected={isSelected}
              isToday={isToday}
              key={index}
              onDeleteRoster={onDeleteRoster}
              onToggleDaySelection={onToggleDaySelection}
              shifts={shifts}
              t={t}
            />
          );
        })}
      </div>
    </div>
  );
}
