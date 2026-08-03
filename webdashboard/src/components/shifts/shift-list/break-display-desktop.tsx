import { Clock, Zap } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

interface BreakDisplayDesktopProps {
  endTime: string | null;
  interval: { break_start?: string | null; break_end?: string | null; break_duration_minutes?: number } | undefined;
  isAutoDeduct: boolean;
  startTime: string | null;
}

/** Desktop break column: fixed break times or simple duration. */
export function BreakDisplayDesktop({ interval, isAutoDeduct }: BreakDisplayDesktopProps) {
  const { t } = useTranslation();
  const breakStart = interval?.break_start ?? null;
  const breakEnd = interval?.break_end ?? null;
  const breakDuration = interval?.break_duration_minutes ?? 0;
  const hasFixedBreak = !!(breakStart && breakEnd);

  return (
    <td className="px-8 py-6 text-center">
      {hasFixedBreak ? (
        <div className="flex flex-col items-center gap-1.5">
          <div className="flex items-center gap-2 px-3 py-1 bg-slate-50 dark:bg-slate-800 rounded-lg border border-slate-100 dark:border-slate-800 shadow-inner">
            <span className="text-[10px] text-slate-600 dark:text-slate-300 font-bold font-mono">
              {breakStart.substring(0, 5)}
            </span>
            <span className="text-slate-300 dark:text-slate-600 font-bold">-</span>
            <span className="text-[10px] text-slate-600 dark:text-slate-300 font-bold font-mono">
              {breakEnd.substring(0, 5)}
            </span>
          </div>
          {isAutoDeduct ? (
            <div className="text-[8px] font-bold text-blue-500 uppercase tracking-widest flex items-center gap-1 opacity-80">
              <Zap className="fill-blue-500" size={10} /> {t("shifts.auto_deduct")}
            </div>
          ) : null}
        </div>
      ) : (
        <div className="flex items-center justify-center gap-2 font-bold text-[10px] text-slate-500 dark:text-slate-400 uppercase tracking-widest">
          <Clock className="text-slate-200 dark:text-slate-700" size={14} />
          {breakDuration} {t("common.minutes")}
        </div>
      )}
    </td>
  );
}
