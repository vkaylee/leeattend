import React from "react";
import { useTranslation } from "react-i18next";
import { Clock, Info } from "lucide-react";
import Modal from "@/components/ui/Modal";
import { Shift } from "@/lib/api";
import { useDateTimeFormatter } from "@/hooks/use-datetime";

interface ShiftPickerModalProps {
  isOpen: boolean;
  onClose: () => void;
  targetDates: string[];
  shifts?: Shift[];
  onSelectShift: (shiftId: string | null) => void;
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
      isOpen={isOpen}
      onClose={onClose}
      title={t("shifts.modal.select_shift_title")}
      subtitle={t("shifts.modal.select_shift_subtitle", { count: targetDates.length })}
    >
      <div className="space-y-6 pt-4">
        <div className="grid grid-cols-1 gap-3">
          {shifts?.map(shift => (
            <button
              key={shift.id}
              onClick={() => handleSelect(shift.id)}
              className="p-5 rounded-[28px] border-2 text-left transition-all relative overflow-hidden group hover:border-primary-500 hover:bg-primary-50 dark:bg-primary-950/30 border-slate-100 dark:border-slate-800 bg-slate-50 dark:bg-slate-800/30"
            >
              <div className="font-bold text-sm uppercase tracking-tight text-slate-700 dark:text-slate-200">{shift.name}</div>
              <div className="flex flex-wrap gap-x-3 gap-y-1 mt-1.5">
                <div className="text-[10px] text-slate-500 dark:text-slate-400 font-bold flex items-center gap-1">
                  <Clock size={10} /> {formatTime(shift.start_time)} - {formatTime(shift.end_time)}
                </div>
                <div className="text-[10px] text-slate-500 dark:text-slate-400 font-bold flex items-center gap-1">
                  <Info size={10} />
                  {shift.break_start ? (
                    <span>{shift.break_start.substring(0, 5)}-{shift.break_end?.substring(0, 5)}</span>
                  ) : (
                    <span>{t("common.break")} {shift.break_duration_minutes}m</span>
                  )}
                </div>
              </div>
            </button>
          ))}

          <button
            onClick={() => handleSelect(null)}
            className="p-5 rounded-[28px] border-2 text-left transition-all relative overflow-hidden group hover:border-slate-300 dark:hover:border-slate-600 hover:bg-slate-100 dark:hover:bg-slate-700 border-slate-100 dark:border-slate-800 bg-slate-50 dark:bg-slate-800/30 border-dashed"
          >
            <div className="font-bold text-sm uppercase tracking-tight text-slate-500 dark:text-slate-400">{t("shifts.off_day")}</div>
            <div className="text-[10px] text-slate-500 dark:text-slate-400 font-bold mt-1.5">
              {t("shifts.modal.mark_as_off")}
            </div>
          </button>
        </div>

        <button
          onClick={onClose}
          className="w-full px-6 py-4 rounded-2xl font-bold text-xs uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-700 transition-all"
        >
          {t("common.cancel")}
        </button>
      </div>
    </Modal>
  );
}
