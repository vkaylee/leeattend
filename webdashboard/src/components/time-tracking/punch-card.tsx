import { Clock, Layers } from "lucide-react";
import { useTranslation } from "react-i18next";

import type { WorkSegment } from "@/lib/api/types";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { TimeEntry } from "@/lib/api";
import { maybeNull } from "@/types/api-overrides";

import { HoldToPunchButton } from "./hold-to-punch-button";

interface PunchCardProps {
  activeEntry?: TimeEntry;
  isPunching: boolean;
  onPunch: () => Promise<void>;
  onRequestAdjustment: () => void;
  /** Today's scheduled work segments, sorted by start time */
  todaySegments?: WorkSegment[];
  userName: string;
}

export function PunchCard({
  activeEntry,
  userName,
  onPunch,
  onRequestAdjustment,
  isPunching,
  todaySegments,
}: PunchCardProps) {
  const { t } = useTranslation();
  const { formatTime } = useDateTimeFormatter();

  return (
    <div className="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-3xl sm:rounded-[40px] p-6 sm:p-10 shadow-2xl shadow-slate-200 dark:shadow-black/40 flex flex-col items-center text-center relative overflow-hidden group">
      <div className="absolute top-0 right-0 p-6 opacity-5 group-hover:opacity-10 transition-opacity">
        <Clock className="-rotate-12" size={120} />
      </div>

      <div className={`w-24 h-24 rounded-[2rem] flex items-center justify-center mb-8 transition-all duration-700 shadow-inner ${
        activeEntry
          ? 'bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-400 animate-pulse'
          : 'bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 shadow-emerald-50'
      }`}>
        <Clock className="w-10 h-10" />
      </div>

      <div className="mb-10 relative z-10">
        <Typography className="text-2xl font-bold text-slate-700 dark:text-slate-200 tracking-tight uppercase leading-tight" variant="h2">
          {userName}
        </Typography>
        <div className="mt-3 flex items-center justify-center gap-2">
          <span className={`w-2 h-2 rounded-full ${
            activeEntry ? 'bg-amber-50 dark:bg-amber-950 animate-ping' : 'bg-emerald-50 dark:bg-emerald-950'
          }`} />
          <Typography className="text-slate-500 dark:text-slate-400 text-xs font-bold uppercase tracking-widest" variant="p">
            {activeEntry
              ? t("time_tracking.status.working")
              : t("time_tracking.status.off_duty")
            }
          </Typography>
        </div>
      </div>

      {!!todaySegments && todaySegments.length > 1 && (
        <div className="w-full mb-6 relative z-10">
          <div className="flex items-center gap-2 text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-wider mb-2">
            <Layers size={12} />
            {todaySegments.length} {t("work_segments.segments", "Segments")}
          </div>
          <div className="space-y-1.5">
            {todaySegments.map((seg, i) => {
              const isActive = activeEntry?.segment_id === seg.id;
              const segmentKey = maybeNull(seg.id) ?? i;
              return (
                <div
                  className={`flex items-center justify-between px-3 py-1.5 rounded-xl text-xs transition-colors ${
                    isActive
                      ? "bg-primary-50 dark:bg-primary-950/50 text-primary-700 dark:text-primary-300 font-bold"
                      : "bg-slate-50 dark:bg-slate-800/50 text-slate-500 dark:text-slate-400"
                  }`}
                  key={segmentKey}
                >
                  <span>
                    {seg.template_id ? `Segment ${i + 1}` : t("work_segments.segment", "Segment")} {i + 1}
                  </span>
                  <span className="font-mono text-[10px]">
                    {maybeNull(seg.scheduled_start)?.substring(0, 5)} – {maybeNull(seg.scheduled_end)?.substring(0, 5)}
                  </span>
                </div>
              );
            })}
          </div>
        </div>
      )}

      {activeEntry ? (
        <div className="w-full space-y-6 relative z-10">
          <div className="bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 p-6 rounded-3xl shadow-inner">
            <div className="text-slate-500 dark:text-slate-400 font-bold uppercase text-[9px] tracking-[0.2em] mb-2">
              {t("time_tracking.status.shift_start_at")}
            </div>
            <div className="font-bold text-4xl text-slate-800 dark:text-slate-100 tracking-tighter font-mono">
              {formatTime(activeEntry.punch_in_time)}
            </div>
          </div>

          <HoldToPunchButton
            isActive={true}
            isPunching={isPunching}
            label={t("time_tracking.actions.hold_to_punch", "Giữ để Chấm công")}
            onPunch={onPunch}
          />
        </div>
      ) : (
        <HoldToPunchButton
          isActive={false}
          isPunching={isPunching}
          label={t("time_tracking.actions.hold_to_punch", "Giữ để Chấm công")}
          onPunch={onPunch}
        />
      )}

      <Button className="mt-8 text-primary-600 dark:text-primary-400 text-[10px] font-bold uppercase tracking-widest hover:text-primary-800 transition-colors py-2"
        onClick={onRequestAdjustment}
        variant="ghost"
      >
        {t("time_tracking.forgot_punch")}
      </Button>
    </div>
  );
}
