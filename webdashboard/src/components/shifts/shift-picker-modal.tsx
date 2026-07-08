import { Clock, Info } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import type { ShiftWithIntervals } from "@/pages/shifts/hooks/use-shifts-types";

import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import { Typography } from "@/components/ui/typography";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { Shift } from "@/lib/api";

interface ShiftPickerModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSelectShift: (shiftId: string | null) => void;
  shifts?: Shift[];
  targetDates: string[];
}

export function ShiftPickerModal({
  isOpen,
  onClose,
  targetDates,
  shifts,
  onSelectShift,
}: ShiftPickerModalProps) {
  const { t } = useTranslation();
  const { formatTime } = useDateTimeFormatter();

  const handleSelect = (shiftId: string | null) => {
    onSelectShift(shiftId);
    onClose();
  };

  return (
    <Modal
      backdropClassName="bg-slate-900/5 dark:bg-slate-950/10 backdrop-blur-none"
      isOpen={isOpen}
      onClose={onClose}
      subtitle={t("shifts.modal.select_shift_subtitle", { count: targetDates.length })}
      title={t("shifts.modal.select_shift_title")}
    >
      <div className="space-y-6 pt-4">
        {targetDates.length === 1 && (
          <Typography className="text-[10px] text-slate-400 dark:text-slate-500 italic text-center" variant="p">
            {t("work_segments.segments", "Tip: Assign multiple shifts to the same day for split-shift scheduling.")}
          </Typography>
        )}
        <div className="grid grid-cols-1 gap-3">
          {shifts?.map(shift => (
            <Button className="p-5 rounded-[28px] border-2 text-left transition-all relative overflow-hidden group hover:border-primary-500 hover:bg-primary-50 dark:bg-primary-950/30 border-slate-100 dark:border-slate-800 bg-slate-50 dark:bg-slate-800/30"
              key={shift.id}
              onClick={() => { handleSelect(shift.id); }}
              variant="ghost"
            >
              <div className="font-bold text-sm uppercase tracking-tight text-slate-700 dark:text-slate-200">{shift.name}</div>
              <div className="flex flex-wrap gap-x-3 gap-y-1 mt-1.5">
                {(() => {
                  const typedShift = shift as ShiftWithIntervals;
                  const iv = typedShift.intervals?.[0] as
                    | { start_time?: string; end_time?: string; break_start?: string; break_end?: string; break_duration_minutes?: number }
                    | undefined;
                  const st = iv?.start_time ?? '';
                  const et = iv?.end_time ?? '';
                  const bs = iv?.break_start;
                  const be = iv?.break_end;
                  const bd = iv?.break_duration_minutes ?? 0;
                  return (<>
                <div className="text-[10px] text-slate-500 dark:text-slate-400 font-bold flex items-center gap-1">
                  <Clock size={10} /> {st ? formatTime(st) : '—'} - {et ? formatTime(et) : '—'}
                </div>
                <div className="text-[10px] text-slate-500 dark:text-slate-400 font-bold flex items-center gap-1">
                  <Info size={10} />
                  {bs ? (
                    <span>{bs.substring(0, 5)}-{be?.substring(0, 5)}</span>
                  ) : (
                    <span>{t("common.break")} {bd}m</span>
                  )}
                </div>
                  </>);
                })()}
              </div>
            </Button>
          ))}

          <Button className="p-5 rounded-[28px] border-2 text-left transition-all relative overflow-hidden group hover:border-slate-300 dark:hover:border-slate-600 hover:bg-slate-100 dark:hover:bg-slate-700 border-slate-100 dark:border-slate-800 bg-slate-50 dark:bg-slate-800/30 border-dashed"
            onClick={() => { handleSelect(null); }}
            variant="ghost"
          >
            <div className="font-bold text-sm uppercase tracking-tight text-slate-500 dark:text-slate-400">{t("shifts.off_day")}</div>
            <div className="text-[10px] text-slate-500 dark:text-slate-400 font-bold mt-1.5">
              {t("shifts.modal.mark_as_off")}
            </div>
          </Button>
        </div>

        <Button className="w-full px-6 py-4 rounded-2xl font-bold text-xs uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-700 transition-all"
          onClick={onClose}
          variant="ghost"
        >
          {t("common.cancel")}
        </Button>
      </div>
    </Modal>
  );
}
