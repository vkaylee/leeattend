import {
  CalendarDays,
  ChevronDown,
  ChevronUp,
  Clock,
  Trash2,
} from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import StatusBadge from "@/components/ui/status-badge";
import { Shift, WorkingPattern, WorkingPatternDetail } from "@/lib/api/types";
import { cn } from "@/lib/utils";

interface PatternCardProps {
  expandedPatternId: string | null;
  onDelete: (pattern: WorkingPattern) => void;
  onToggleExpand: (id: string) => void;
  pattern: WorkingPattern;
  patternDetails?: WorkingPatternDetail[];
  shifts?: Shift[];
}

function DayDetailRow({ day, dayDetails, shifts }: { day: string, dayDetails: WorkingPatternDetail[], shifts?: Shift[] }) {
  const { t } = useTranslation();
  const hasShifts = dayDetails.some((d) => d.shift_id);
  return (
    <div className={cn("flex items-center gap-3 px-4 py-2 rounded-xl border transition-all", hasShifts ? "bg-white dark:bg-slate-800/50 border-primary-100 dark:border-primary-800/50" : "bg-slate-50/50 dark:bg-slate-800/20 border-slate-100 dark:border-slate-800 opacity-50")}>
      <span className={cn("w-10 flex-shrink-0 text-[9px] font-bold uppercase tracking-widest", hasShifts ? "text-slate-700 dark:text-slate-200" : "text-slate-400 dark:text-slate-500")}>{day}</span>
      <div className="flex-1 flex items-center gap-1.5 flex-wrap">
        {hasShifts ? (
          dayDetails.filter((d) => d.shift_id).map((d, i) => (
            <div className="flex items-center gap-1 px-2 py-1 rounded-lg bg-primary-50 dark:bg-primary-950 border border-primary-100 dark:border-primary-800" key={i}>
              <Clock className="text-primary-500 dark:text-primary-400" size={8} />
              <span className="text-[8px] font-bold text-primary-600 dark:text-primary-400 uppercase tracking-tight">{shifts?.find((s) => s.id === d.shift_id)?.name ?? "—"}</span>
            </div>
          ))
        ) : (
          <span className="text-[9px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest italic">{t("status_off")}</span>
        )}
      </div>
    </div>
  );
}

export function PatternCard({ expandedPatternId, onDelete, onToggleExpand, pattern, patternDetails, shifts }: PatternCardProps) {
  const { t } = useTranslation();
  const daysOfWeekShort = [t("days.sunday_short"), t("days.monday_short"), t("days.tuesday_short"), t("days.wednesday_short"), t("days.thursday_short"), t("days.friday_short"), t("days.saturday_short")];
  const isExpanded = expandedPatternId === pattern.id;

  return (
    <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[2.5rem] p-10 shadow-sm hover:shadow-2xl hover:shadow-primary-100/10 hover:scale-[1.02] transition-all group relative overflow-hidden flex flex-col">
      <div className="absolute top-0 right-0 p-8 flex gap-3 translate-y-4 opacity-0 group-hover:translate-y-0 group-hover:opacity-100 transition-all duration-500">
        <button className="p-3 bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 hover:text-rose-600 dark:text-rose-300 rounded-xl shadow-md border border-slate-100 dark:border-slate-800 transition-all active:scale-90" onClick={() => { onDelete(pattern); }} title={t("common.delete")} type="button"><Trash2 size={18} /></button>
      </div>
      <div className="flex items-center gap-5 mb-8">
        <div className="w-16 h-16 bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-400 rounded-[1.5rem] flex items-center justify-center shadow-inner border border-primary-100 dark:border-primary-800/50 group-hover:scale-110 group-hover:rotate-3 transition-all duration-500"><CalendarDays size={32} /></div>
        <div><h3 className="font-bold text-2xl text-slate-700 dark:text-slate-200 leading-none uppercase tracking-tight mb-2">{pattern.name}</h3><StatusBadge variant={pattern.is_active ? "success" : "neutral"}>{pattern.is_active ? t("status_active_caps") : t("status_off")}</StatusBadge></div>
      </div>
      <p className="text-slate-500 dark:text-slate-400 text-xs mb-10 line-clamp-2 min-h-[3rem] font-bold uppercase tracking-tight opacity-60 leading-relaxed italic">{pattern.description ?? t("common.no_description")}</p>
      <button className="mt-auto flex items-center justify-between w-full p-5 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl hover:bg-white dark:hover:bg-slate-900 hover:border-primary-100 dark:border-primary-800 transition-all group/btn" onClick={() => { onToggleExpand(pattern.id); }} type="button"><span className="text-[10px] font-bold text-slate-500 dark:text-slate-400 group-hover/btn:text-primary-600 dark:text-primary-400 uppercase tracking-widest transition-colors">{t("patterns.view_details").toUpperCase()}</span>{isExpanded ? <ChevronUp className="text-slate-300 dark:text-slate-600" size={16} /> : <ChevronDown className="text-slate-300 dark:text-slate-600" size={16} />}</button>
      {isExpanded ? (
        <div className="mt-6 pt-6 border-t border-slate-100 dark:border-slate-800 animate-in slide-in-from-top-4 duration-500">
          <div className="flex flex-col gap-2">
            {daysOfWeekShort.map((day, index) => (
              <DayDetailRow day={day} dayDetails={patternDetails?.filter((d) => d.day_of_week === index) ?? []} key={index} shifts={shifts} />
            ))}
          </div>
        </div>
      ) : null}
    </div>
  );
}
