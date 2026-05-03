import { Clock } from "lucide-react";
import React from "react";

interface DateRangeFieldsProps {
  effectiveFrom: string;
  effectiveTo: string;
  setEffectiveFrom: (value: string) => void;
  setEffectiveTo: (value: string) => void;
  t: (key: string, defaultValue?: string) => string;
}

export function DateRangeFields({
  effectiveFrom,
  setEffectiveFrom,
  effectiveTo,
  setEffectiveTo,
  t
}: DateRangeFieldsProps) {
  return (
    <div className="grid grid-cols-2 gap-4">
      <div className="space-y-3">
        <label
          className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1 flex items-center gap-1.5"
          htmlFor="effective-from"
        >
          <Clock size={10} /> {t("payroll.effective_from", "Effective From")}
        </label>
        <input
          className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-2xl px-5 py-3.5 font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all shadow-sm"
          id="effective-from"
          onChange={(e) => { setEffectiveFrom(e.target.value); }}
          required
          type="date"
          value={effectiveFrom}
        />
      </div>
      <div className="space-y-3">
        <label
          className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1 flex items-center gap-1.5"
          htmlFor="effective-to"
        >
          <Clock size={10} /> {t("payroll.effective_to", "Effective To")}
          <span className="opacity-40 normal-case tracking-normal">
            ({t("common.optional", "optional")})
          </span>
        </label>
        <input
          className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-2xl px-5 py-3.5 font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all shadow-sm"
          id="effective-to"
          onChange={(e) => { setEffectiveTo(e.target.value); }}
          type="date"
          value={effectiveTo}
        />
      </div>
    </div>
  );
}
