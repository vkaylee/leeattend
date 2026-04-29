import { useState } from "react";

import { format } from "date-fns";
import { History } from "lucide-react";
import { useTranslation } from "react-i18next";

import Modal from "@/components/ui/Modal";

import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { TimeEntry } from "@/lib/api";

interface AdjustmentRequestModalProps {
  isOpen: boolean;
  onClose: () => void;
  timeEntry?: TimeEntry;
  onSubmit: (data: AdjustmentData) => Promise<void>;
  timezone: string;
}

export interface AdjustmentData {
  date: string;
  punchIn: string;
  punchOut: string;
  reason: string;
  isBreakOvertime: boolean;
  timeEntryId?: string;
}

export function AdjustmentRequestModal({
  isOpen,
  onClose,
  timeEntry,
  onSubmit,
  timezone
}: AdjustmentRequestModalProps) {
  const { t } = useTranslation();
  const { formatTime24h } = useDateTimeFormatter();
  const [isSubmitting, setIsSubmitting] = useState(false);

  const [formData, setFormData] = useState<AdjustmentData>({
    date: timeEntry
      ? format(new Date(timeEntry.punch_in_time), "yyyy-MM-dd")
      : format(new Date(), "yyyy-MM-dd"),
    punchIn: timeEntry
      ? formatTime24h(timeEntry.punch_in_time)
      : "08:00",
    punchOut: timeEntry?.punch_out_time
      ? formatTime24h(timeEntry.punch_out_time)
      : "17:00",
    reason: "",
    isBreakOvertime: timeEntry?.is_break_overtime || false,
    timeEntryId: timeEntry?.id
  });

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsSubmitting(true);
    try {
      await onSubmit(formData);
      onClose();
    } catch (error) {
      console.error("Failed to submit adjustment:", error);
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title={
        formData.timeEntryId
          ? t("time_tracking.modal.edit_title")
          : t("time_tracking.modal.new_title")
      }
      subtitle={t("time_tracking.modal.subtitle")}
      icon={<History size={24} />}
      maxWidth="md"
    >
      <form onSubmit={handleSubmit} className="space-y-8 pt-4">
        <div className="space-y-2">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">
            {t("common.work_day")}
          </label>
          <input type="date" required className="w-full px-5 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-bold text-sm tracking-tight" value={formData.date} onChange={(e) =>
              setFormData({ ...formData, date: e.target.value })
            }
          />
        </div>

        <div className="grid grid-cols-2 gap-6">
          <div className="space-y-2">
            <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">
              {t("common.punch_in")}
            </label>
            <input
              type="time"
              required
              className="w-full px-5 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-mono font-bold text-sm text-center"
              value={formData.punchIn}
              onChange={(e) =>
                setFormData({ ...formData, punchIn: e.target.value })
              }
            />
          </div>
          <div className="space-y-2">
            <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">
              {t("common.punch_out")}
            </label>
            <input
              type="time"
              required
              className="w-full px-5 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-mono font-bold text-sm text-center"
              value={formData.punchOut}
              onChange={(e) =>
                setFormData({ ...formData, punchOut: e.target.value })
              }
            />
          </div>
        </div>

        <div className="flex items-center gap-4 py-2 bg-slate-50 dark:bg-slate-800/50 p-4 rounded-[1.5rem] border border-slate-100 dark:border-slate-800 shadow-inner">
          <input
            type="checkbox"
            id="is_break_overtime"
            className="w-6 h-6 text-primary-600 dark:text-primary-400 rounded-xl border-slate-200 dark:border-slate-700 focus:ring-primary-500 transition-all cursor-pointer"
            checked={formData.isBreakOvertime}
            onChange={(e) =>
              setFormData({ ...formData, isBreakOvertime: e.target.checked })
            }
          />
          <label
            htmlFor="is_break_overtime"
            className="text-xs font-bold text-slate-700 dark:text-slate-200 uppercase tracking-widest cursor-pointer leading-tight"
          >
            {t("attendance_confirm.ot_label")}
          </label>
        </div>

        <div className="space-y-2">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">
            {t("time_tracking.modal.reason_label")}
          </label>
          <textarea
            required
            placeholder={t("time_tracking.modal.reason_placeholder")}
            className="w-full px-5 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-medium text-sm h-28 resize-none"
            value={formData.reason}
            onChange={(e) =>
              setFormData({ ...formData, reason: e.target.value })
            }
          />
        </div>

        <div className="pt-6 grid grid-cols-2 gap-5">
          <button
            type="button"
            onClick={onClose}
            className="px-8 py-4 border border-slate-100 dark:border-slate-800 rounded-2xl font-bold text-xs uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800 transition-all"
          >
            {t("common.cancel")}
          </button>
          <button
            type="submit"
            disabled={isSubmitting}
            className="px-8 py-4 bg-slate-900 text-white rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-slate-200 dark:shadow-black/30 active:scale-95 disabled:opacity-50"
          >
            {isSubmitting ? t("common.sending").toUpperCase() : t("common.submit_request")}
          </button>
        </div>
      </form>
    </Modal>
  );
}
