import { CalendarRange, Calendar, Save } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { Employee } from "@/lib/api";

interface WorkingPattern {
  description?: string;
  id: string;
  name: string;
}

interface PatternAssignmentFormProps {
  endDate: string;
  isForever: boolean;
  isLoading: boolean;
  onAssign: () => void;
  patterns?: WorkingPattern[];
  selectedEmployee: Employee;
  selectedPatternId: string;
  setEndDate: (date: string) => void;
  setIsForever: (forever: boolean) => void;
  setSelectedPatternId: (id: string) => void;
  setStartDate: (date: string) => void;
  startDate: string;
}

export function PatternAssignmentForm({
  selectedEmployee,
  patterns,
  selectedPatternId,
  setSelectedPatternId,
  startDate,
  setStartDate,
  endDate,
  setEndDate,
  isForever,
  setIsForever,
  onAssign,
  isLoading,
}: PatternAssignmentFormProps) {
  const { t } = useTranslation();

  return (
    <div className="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-[40px] p-10 shadow-sm animate-in fade-in zoom-in-95 duration-300">
      <div className="flex items-center gap-6 mb-10 pb-10 border-b border-slate-50 dark:border-slate-800">
        <div className="w-20 h-20 bg-primary-600 rounded-[28px] flex items-center justify-center text-white text-3xl font-bold shadow-xl shadow-primary-100 dark:shadow-black/30">
          {selectedEmployee.full_name[0]}
        </div>
        <div>
          <h2 className="text-2xl font-bold text-slate-700 dark:text-slate-200">{selectedEmployee.full_name}</h2>
          <p className="text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 mt-1">
            {t("shifts.fixed_assignment_subtitle")}
          </p>
        </div>
      </div>
      <div className="space-y-10">
        <div className="space-y-4">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest flex items-center gap-2 ml-1">
            <CalendarRange size={12} /> {t("patterns.select_pattern")}
          </label>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {patterns?.map(pattern => (
              <button
                className={`p-5 rounded-[28px] border-2 text-left transition-all relative overflow-hidden group ${
                  selectedPatternId === pattern.id
                    ? 'border-primary-600 bg-primary-50 dark:bg-primary-950/50'
                    : 'border-slate-100 dark:border-slate-800 hover:border-slate-200 dark:hover:border-slate-700 bg-slate-50 dark:bg-slate-800/30'
                }`}
                key={pattern.id}
                onClick={() => { setSelectedPatternId(pattern.id); }}
              >
                <div className={`font-bold text-sm uppercase tracking-tight ${selectedPatternId === pattern.id ? 'text-primary-700 dark:text-primary-300' : 'text-slate-700 dark:text-slate-200'}`}>
                  {pattern.name}
                </div>
                <p className="text-[10px] text-slate-500 dark:text-slate-400 font-bold mt-1 line-clamp-1">
                  {pattern.description ?? t("common.no_description")}
                </p>
                {selectedPatternId === pattern.id && (
                  <div className="absolute top-4 right-4 w-3 h-3 bg-primary-600 rounded-full animate-pulse" />
                )}
              </button>
            ))}
          </div>
        </div>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
          <div className="space-y-3">
            <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest flex items-center gap-2 ml-1">
              <Calendar size={12} /> {t("common.effective_from")}
            </label>
            <input
              className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-2xl px-5 py-4 font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 outline-none transition-all"
              onChange={(e) => { setStartDate(e.target.value); }}
              type="date"
              value={startDate}
            />
          </div>
          <div className="space-y-3">
            <div className="flex justify-between items-center px-1">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest flex items-center gap-2">
                <Calendar size={12} /> {t("common.end_date")}
              </label>
              <button
                className={`text-[9px] font-bold px-3 py-1 rounded-full border transition-all uppercase tracking-tighter ${
                  isForever
                    ? 'bg-primary-600 text-white border-primary-600'
                    : 'bg-slate-100 dark:bg-slate-700 text-slate-500 dark:text-slate-400 border-slate-200 dark:border-slate-700'
                }`}
                onClick={() => { setIsForever(!isForever); }}
              >
                {isForever ? t("common.indefinite").toUpperCase() : t("common.definite").toUpperCase()}
              </button>
            </div>
            <input
              className={`w-full border rounded-2xl px-5 py-4 font-bold transition-all outline-none ${
                isForever
                  ? 'bg-slate-100 dark:bg-slate-700 text-slate-300 dark:text-slate-600 border-slate-50 dark:border-slate-800'
                  : 'bg-slate-50 dark:bg-slate-800 text-slate-700 dark:text-slate-200 border-slate-200 dark:border-slate-700 focus:ring-2 focus:ring-primary-500'
              }`}
              disabled={isForever}
              onChange={(e) => { setEndDate(e.target.value); }}
              type="date"
              value={endDate}
            />
          </div>
        </div>
        <button
          className="w-full bg-slate-900 text-white font-bold py-5 rounded-[28px] flex items-center justify-center gap-3 hover:bg-primary-700 transition-all active:scale-[0.98] disabled:opacity-50 shadow-2xl shadow-slate-200 dark:shadow-black/40 uppercase tracking-widest text-xs"
          disabled={isLoading}
          onClick={onAssign}
        >
          <Save size={18} /> {t("shifts.assign_btn")}
        </button>
      </div>
    </div>
  );
}
