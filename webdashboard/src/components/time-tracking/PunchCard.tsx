import { Clock } from "lucide-react";
import { useTranslation } from "react-i18next";

import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { TimeEntry } from "@/lib/api";

interface PunchCardProps {
  activeEntry?: TimeEntry;
  isOnBreak: boolean;
  isPunching: boolean;
  onPunchIn: (type: "work" | "break") => Promise<void>;
  onPunchOut: () => Promise<void>;
  onRequestAdjustment: () => void;
  userName: string;
}

export function PunchCard({
  activeEntry,
  isOnBreak,
  userName,
  onPunchIn,
  onPunchOut,
  onRequestAdjustment,
  isPunching
}: PunchCardProps) {
  const { t } = useTranslation();
  const { formatTime } = useDateTimeFormatter();

  return (
    <div className="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-[40px] p-10 shadow-2xl shadow-slate-200 dark:shadow-black/40 flex flex-col items-center text-center relative overflow-hidden group">
      <div className="absolute top-0 right-0 p-6 opacity-5 group-hover:opacity-10 transition-opacity">
        <Clock className="-rotate-12" size={120} />
      </div>

      <div className={`w-24 h-24 rounded-[2rem] flex items-center justify-center mb-8 transition-all duration-700 shadow-inner ${
        activeEntry
          ? (isOnBreak ? 'bg-amber-50 dark:bg-amber-950 text-amber-600 dark:text-amber-300 animate-pulse' : 'bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-400 animate-pulse')
          : 'bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 shadow-emerald-50'
      }`}>
        <Clock className="w-10 h-10" />
      </div>

      <div className="mb-10 relative z-10">
        <h2 className="text-2xl font-bold text-slate-700 dark:text-slate-200 tracking-tight uppercase leading-tight">
          {userName}
        </h2>
        <div className="mt-3 flex items-center justify-center gap-2">
          <span className={`w-2 h-2 rounded-full ${
            activeEntry ? 'bg-amber-50 dark:bg-amber-950 animate-ping' : 'bg-emerald-50 dark:bg-emerald-950'
          }`} />
          <p className="text-slate-500 dark:text-slate-400 text-xs font-bold uppercase tracking-widest">
            {activeEntry
              ? (isOnBreak ? t("time_tracking.status.on_break") : t("time_tracking.status.working"))
              : t("time_tracking.status.off_duty")
            }
          </p>
        </div>
      </div>

      {activeEntry ? (
        <div className="w-full space-y-6 relative z-10">
          <div className="bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 p-6 rounded-3xl shadow-inner">
            <div className="text-slate-500 dark:text-slate-400 font-bold uppercase text-[9px] tracking-[0.2em] mb-2">
              {isOnBreak ? t("time_tracking.status.break_start_at") : t("time_tracking.status.shift_start_at")}
            </div>
            <div className="font-bold text-4xl text-slate-800 dark:text-slate-100 tracking-tighter font-mono">
              {formatTime(activeEntry.punch_in_time)}
            </div>
          </div>

          <div className="flex flex-col gap-4">
            {isOnBreak ? (
              <button
                className="w-full bg-emerald-600 text-white py-5 rounded-2xl font-bold uppercase tracking-widest text-[10px] hover:bg-emerald-700 hover:scale-[1.02] active:scale-95 transition-all shadow-lg shadow-emerald-100 disabled:opacity-50"
                disabled={isPunching}
                onClick={() => onPunchOut()}
              >
                {t("time_tracking.actions.return_to_work")}
              </button>
            ) : (
              <>
                <button
                  className="w-full bg-amber-50 dark:bg-amber-950 text-white py-5 rounded-2xl font-bold uppercase tracking-widest text-[10px] hover:bg-amber-600 hover:scale-[1.02] active:scale-95 transition-all shadow-lg shadow-amber-100 disabled:opacity-50"
                  disabled={isPunching}
                  onClick={() => onPunchIn("break")}
                >
                  {t("time_tracking.actions.start_break")}
                </button>
                <button
                  className="w-full bg-rose-600 text-white py-5 rounded-2xl font-bold uppercase tracking-widest text-[10px] hover:bg-rose-700 hover:scale-[1.02] active:scale-95 transition-all shadow-lg shadow-rose-100 disabled:opacity-50"
                  disabled={isPunching}
                  onClick={() => onPunchOut()}
                >
                  {t("time_tracking.actions.punch_out")}
                </button>
              </>
            )}
          </div>
        </div>
      ) : (
        <button
          className="w-full bg-primary-600 text-white py-6 rounded-[1.5rem] font-bold uppercase tracking-widest text-xs hover:bg-primary-700 hover:scale-[1.02] active:scale-95 transition-all shadow-2xl shadow-primary-100 relative z-10 disabled:opacity-50"
          disabled={isPunching}
          onClick={() => onPunchIn("work")}
        >
          {t("time_tracking.actions.start_work")}
        </button>
      )}

      <button
        className="mt-8 text-primary-600 dark:text-primary-400 text-[10px] font-bold uppercase tracking-widest hover:text-primary-800 transition-colors py-2"
        onClick={onRequestAdjustment}
      >
        {t("time_tracking.forgot_punch")}
      </button>
    </div>
  );
}
