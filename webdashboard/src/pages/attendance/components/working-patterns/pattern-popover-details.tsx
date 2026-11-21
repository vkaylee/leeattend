// Popover details content for pattern row card day timeline.
// Extracted to reduce complexity in pattern-row-card.tsx.

import { Coffee, Clock } from "lucide-react";
import { useTranslation } from "react-i18next";

import type { Shift, WorkingPatternDetail } from "@/lib/api/types";
import type { ShiftWithIntervals } from "@/pages/shifts/hooks/use-shifts-types";

import { Typography } from "@/components/ui/typography";

interface PatternPopoverDetailsProps {
  dayDetails: WorkingPatternDetail[];
  dayName: string;
  shifts?: Shift[];
}

function ShiftDetailBlock({ d, shifts }: { d: WorkingPatternDetail; shifts?: Shift[] }) {
  const { t } = useTranslation();
  const shift = shifts?.find((s) => s.id === d.shift_id);
  if (!shift) return null;

  const iv = (shift as ShiftWithIntervals).intervals?.[0] as
    | { start_time?: string; end_time?: string; break_start?: string; break_end?: string; break_duration_minutes?: number }
    | undefined;
  const st = iv?.start_time?.substring(0, 5) ?? '--';
  const et = iv?.end_time?.substring(0, 5) ?? '--';
  const bs = iv?.break_start;
  const be = iv?.break_end;
  const bd = (iv?.break_duration_minutes) ?? 0;

  return (
    <div className="space-y-2.5">
      <Typography className="font-bold text-sm text-slate-800 dark:text-slate-100 uppercase tracking-tight leading-snug" variant="h5">
        {shift.name}
      </Typography>
      <div className="flex flex-col gap-1.5 text-xs text-slate-600 dark:text-slate-350">
        <div className="flex items-center gap-2">
          <Clock className="text-primary-500 dark:text-primary-400" size={14} />
          <span className="font-semibold">{st} - {et}</span>
        </div>
        {bs && be ? (
          <div className="flex items-center gap-2">
            <Coffee className="text-amber-500 dark:text-amber-400" size={14} />
            <span>
              {t("shifts.table.break_time")}: {bs.substring(0,5)} - {be.substring(0,5)} ({bd}m)
            </span>
          </div>
        ) : null}
      </div>
    </div>
  );
}

export function PatternPopoverDetails({ dayDetails, dayName, shifts }: PatternPopoverDetailsProps) {
  const { t } = useTranslation();
  const assignedDetails = dayDetails.filter((d) => d.shift_id);
  const hasShifts = assignedDetails.length > 0;

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between border-b border-slate-100 dark:border-slate-800/60 pb-2">
        <span className="text-[10px] font-bold uppercase text-slate-400 dark:text-slate-500 tracking-wider">
          {dayName}
        </span>
      </div>
      {hasShifts ? (
        assignedDetails.map((d, idx) => (
          <ShiftDetailBlock d={d} key={idx} shifts={shifts} />
        ))
      ) : (
        <div className="py-2 text-center text-xs font-semibold text-slate-400 dark:text-slate-500 italic uppercase tracking-wider">
          {t("shifts.monthly.off_day") || "Rest Day"}
        </div>
      )}
    </div>
  );
}
