import React from "react";
import { useTranslation } from "react-i18next";
import { format } from "date-fns";
import { vi, enUS, zhCN } from "date-fns/locale";
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
} from "lucide-react";
import { Combobox } from "@/components/ui/Combobox";
import { Employee } from "@/lib/api";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { ScheduleTracePopover } from "./ScheduleTracePopover";

const dateLocales: Record<string, any> = {
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
  employeeSchedule?: any;
  onCopyPreviousWeek: () => void;
  onAssignAllWeek: () => void;
  onClearWeek: () => void;
  onOpenShiftPicker: (dates: string[]) => void;
  onDeleteRoster: (date: string) => void;
  currentLocale?: any;
}

export function WeeklyRosterGrid({
  selectedEmployee,
  currentDate,
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
      <div className="flex items-center justify-between bg-white p-4 rounded-[32px] border border-slate-200 shadow-sm">
        <div className="flex items-center gap-4">
          <div className="w-12 h-12 bg-primary-600 rounded-2xl flex items-center justify-center text-white text-lg font-bold shadow-lg shadow-primary-100">
            {selectedEmployee.full_name[0]}
          </div>
          <div>
            <h2 className="text-lg font-bold text-slate-700 leading-tight">{selectedEmployee.full_name}</h2>
            <p className="text-[10px] font-bold uppercase tracking-widest text-slate-500">{t("shifts.current_week_roster")}</p>
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
              className="h-10 bg-slate-100 border-slate-200 text-[10px] py-2 px-3 rounded-xl uppercase tracking-tighter"
            />
          </div>
          <button
            onClick={() => onNavigateWeek("prev")}
            className="p-2.5 hover:bg-slate-100 border border-slate-100 rounded-xl transition-all"
          >
            <ChevronLeft size={18} />
          </button>
          <div className="px-4 py-2 bg-slate-50 border border-slate-200 rounded-xl text-xs font-bold text-slate-600 uppercase tracking-tighter">
            {formatDate(weekDays[0])} - {formatDate(weekDays[6])}
          </div>
          <button
            onClick={() => onNavigateWeek("next")}
            className="p-2.5 hover:bg-slate-100 border border-slate-100 rounded-xl transition-all"
          >
            <ChevronRight size={18} />
          </button>
        </div>
      </div>

      {selectedDays.length > 0 && (
        <div className="bg-primary-600 text-white p-4 rounded-[24px] flex items-center justify-between shadow-xl shadow-primary-100 animate-in slide-in-from-top-4 duration-300">
          <div className="flex items-center gap-4">
            <span className="text-xs font-bold uppercase tracking-widest">
              {t("shifts.days_selected", { count: selectedDays.length })}
            </span>
            <div className="h-4 w-px bg-white/20" />
            <button
              onClick={() => onOpenShiftPicker(selectedDays)}
              className="bg-white/10 border border-white/20 rounded-xl px-4 py-2 text-[10px] font-bold uppercase tracking-widest hover:bg-white/20 transition-all flex items-center gap-2"
            >
              <Plus size={14} /> {t("shifts.assign_selected_btn")}
            </button>
          </div>
          <button onClick={onClearSelection} className="hover:bg-white/10 p-2 rounded-xl transition-all">
            <X size={18} />
          </button>
        </div>
      )}

      <div className="grid grid-cols-1 md:grid-cols-7 gap-4">
        {weekDays.map((day, idx) => {
          const dateStr = format(day, 'yyyy-MM-dd');
          const isSelected = selectedDays.includes(dateStr);
          const effectiveSchedule = employeeSchedule?.schedules.find((s: any) => s.date === dateStr);
          const isToday = day.toDateString() === new Date().toDateString();
          const source = effectiveSchedule?.source as any;

          return (
            <div
              key={idx}
              onClick={() => onToggleDaySelection(dateStr)}
              className={`bg-white border p-4 rounded-[32px] flex flex-col min-h-[350px] transition-all relative group cursor-pointer ${
                isSelected
                  ? 'border-primary-500 ring-4 ring-primary-50 shadow-md'
                  : isToday
                  ? 'border-primary-300 ring-4 ring-primary-50 shadow-lg'
                  : 'border-slate-100 hover:border-slate-200 shadow-sm'
              }`}
            >
              <div className="text-center mb-5 pb-5 border-b border-slate-50 relative">
                {isSelected && (
                  <div className="absolute -top-1 -right-1 bg-primary-600 text-white rounded-full p-1 shadow-sm">
                    <ShieldCheck size={12} />
                  </div>
                )}
                <div className={`text-[10px] font-bold uppercase tracking-tighter ${isToday ? 'text-primary-600' : 'text-slate-500'}`}>
                  {format(day, "EEEE", { locale: currentLocale })}
                </div>
                <div className={`text-2xl font-bold mt-1 ${isToday ? 'text-primary-600' : 'text-slate-700'}`}>
                  {day.getDate()}
                </div>
              </div>
              <div className="flex-1 space-y-4">
                <div className="flex justify-center items-center gap-1">
                  {source === "roster" && (
                    <span className="text-[9px] font-bold px-2 py-0.5 rounded-full bg-purple-50 text-purple-600 border border-purple-100 uppercase tracking-tighter flex items-center gap-1">
                      <Zap size={8} /> {t("shifts.types.flexible")}
                    </span>
                  )}
                  {source === "pattern" && (
                    <span className="text-[9px] font-bold px-2 py-0.5 rounded-full bg-blue-50 text-blue-600 border border-blue-100 uppercase tracking-tighter flex items-center gap-1">
                      <ShieldCheck size={8} /> {t("patterns.title")}
                    </span>
                  )}
                  {source === "department_pattern" && (
                    <span className="text-[9px] font-bold px-2 py-0.5 rounded-full bg-slate-50 text-slate-600 border border-slate-200 uppercase tracking-tighter flex items-center gap-1">
                      <Building2 size={8} /> {t("patterns.title")}
                    </span>
                  )}
                  {source === "fixed" && (
                    <span className="text-[9px] font-bold px-2 py-0.5 rounded-full bg-emerald-50 text-emerald-600 border border-emerald-100 uppercase tracking-tighter flex items-center gap-1">
                      <ShieldCheck size={8} /> {t("shifts.types.fixed")}
                    </span>
                  )}
                  {source && (source === "department" || source === "department_fixed") && (
                    <span className="text-[9px] font-bold px-2 py-0.5 rounded-full bg-slate-50 text-slate-500 border border-slate-100 uppercase tracking-tighter flex items-center gap-1">
                      <Building2 size={8} /> {t("common.department")}
                    </span>
                  )}
                  {effectiveSchedule && (
                    <ScheduleTracePopover traces={effectiveSchedule.traces || []} source={effectiveSchedule.source} />
                  )}
                </div>
                {effectiveSchedule?.shift_id ? (
                  <div
                    className={`p-4 rounded-[24px] relative group/item animate-in zoom-in-95 border ${
                      source === 'roster' ? 'bg-purple-50/50 border-purple-100' : 'bg-primary-50 border-blue-100'
                    }`}
                  >
                    <div className={`font-bold text-xs mb-1 uppercase tracking-tight leading-tight ${source === 'roster' ? 'text-purple-700' : 'text-blue-700'}`}>
                      {effectiveSchedule.shift_name}
                    </div>
                    <div className={`flex items-center gap-1.5 text-[10px] font-bold ${source === 'roster' ? 'text-purple-500' : 'text-blue-500'}`}>
                      {formatTime(effectiveSchedule.start_time)} - {formatTime(effectiveSchedule.end_time)}
                    </div>
                    {effectiveSchedule.break_duration_minutes !== null && (
                      <div className="mt-2 pt-2 border-t border-black/5 flex items-center gap-1.5 text-[9px] font-bold uppercase tracking-tighter text-slate-500">
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
                        className="absolute -top-2 -right-2 bg-white border border-slate-200 rounded-full p-1.5 shadow-md opacity-0 group-hover/item:opacity-100 transition-opacity hover:bg-rose-50 hover:border-rose-200 group/btn"
                      >
                        <X size={12} className="text-slate-500 group-hover/btn:text-rose-500" />
                      </button>
                    )}
                  </div>
                ) : (
                  <div className="bg-slate-50/50 border border-slate-100 border-dashed p-6 rounded-[24px] text-[10px] text-slate-500 font-bold text-center uppercase tracking-widest flex flex-col items-center justify-center gap-2">
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
