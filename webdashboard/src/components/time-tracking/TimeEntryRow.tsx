import { useTranslation } from "react-i18next";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { TimeEntry } from "@/lib/api";
import { cn } from "@/lib/utils";

interface TimeEntryRowProps {
  entry: TimeEntry;
  onAdjust?: (entry: TimeEntry) => void;
  showAdjustButton?: boolean;
}

export function TimeEntryRow({
  entry,
  onAdjust,
  showAdjustButton = false
}: TimeEntryRowProps) {
  const { t } = useTranslation();
  const { formatDate, formatTime } = useDateTimeFormatter();

  return (
    <tr className="hover:bg-slate-50/50 transition-all group">
      <td className="px-8 py-5">
        <div className="flex items-center gap-3">
          <span className="text-sm font-bold text-slate-700">
            {formatDate(entry.punch_in_time)}
          </span>
          <span
            className={cn(
              "px-2 py-0.5 rounded-lg text-[8px] font-bold uppercase tracking-tighter border",
              entry.entry_type === "break"
                ? "bg-amber-50 text-amber-600 border-amber-100"
                : "bg-primary-50 text-primary-600 border-primary-100"
            )}
          >
            {entry.entry_type === "break" ? t("common.break") : t("common.work")}
          </span>
        </div>
      </td>
      <td className="px-8 py-5 text-xs font-bold text-slate-500 font-mono">
        {formatTime(entry.punch_in_time)}
      </td>
      <td className="px-8 py-5">
        {entry.punch_out_time ? (
          <span className="text-xs font-bold text-slate-500 font-mono">
            {formatTime(entry.punch_out_time)}
          </span>
        ) : (
          <span
            className={cn(
              "font-bold px-2 py-1 rounded-lg text-[9px] uppercase tracking-tighter animate-pulse",
              entry.entry_type === "break"
                ? "text-amber-600 bg-amber-50"
                : "text-primary-600 bg-primary-50"
            )}
          >
            {entry.entry_type === "break"
              ? t("time_tracking.status.on_break")
              : t("time_tracking.status.working")}
          </span>
        )}
      </td>
      {showAdjustButton && (
        <td className="px-8 py-5 text-right">
          <button
            onClick={() => onAdjust?.(entry)}
            className="text-primary-600 bg-white shadow-sm border border-slate-100 hover:bg-primary-50 px-4 py-1.5 rounded-xl transition-all font-bold text-[10px] uppercase tracking-widest active:scale-90 opacity-0 group-hover:opacity-100"
          >
            {t("common.adjust")}
          </button>
        </td>
      )}
    </tr>
  );
}
