import { History } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import { format } from "date-fns";
import { TFunction } from "i18next";

import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { TimeEntry } from "@/lib/api";

interface AdjustmentRequestModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (data: AdjustmentData) => Promise<void>;
  timeEntry?: TimeEntry;
  timezone: string;
}

export interface AdjustmentData {
  date: string;
  isBreakOvertime: boolean;
  punchIn: string;
  punchOut: string;
  reason: string;
  timeEntryId?: string;
}

function AdjustmentFormFields({ formData, setFormData, t }: { formData: AdjustmentData; setFormData: (data: AdjustmentData) => void; t: TFunction }) {
  return (
    <>
      <div className="space-y-2">
        <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">
          {t("common.work_day")}
        </label>
        <input className="w-full px-5 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-bold text-sm tracking-tight" onChange={(e) =>
            { setFormData({ ...formData, date: e.target.value }); }
          } required type="date" value={formData.date}
        />
      </div>

      <div className="grid grid-cols-2 gap-6">
        <div className="space-y-2">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">
            {t("common.punch_in")}
          </label>
          <input
            className="w-full px-5 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-mono font-bold text-sm text-center"
            onChange={(e) =>
              { setFormData({ ...formData, punchIn: e.target.value }); }
            }
            required
            type="time"
            value={formData.punchIn}
          />
        </div>
        <div className="space-y-2">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">
            {t("common.punch_out")}
          </label>
          <input
            className="w-full px-5 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-mono font-bold text-sm text-center"
            onChange={(e) =>
              { setFormData({ ...formData, punchOut: e.target.value }); }
            }
            required
            type="time"
            value={formData.punchOut}
          />
        </div>
      </div>

      <div className="flex items-center gap-4 py-2 bg-slate-50 dark:bg-slate-800/50 p-4 rounded-[1.5rem] border border-slate-100 dark:border-slate-800 shadow-inner">
        <input
          checked={formData.isBreakOvertime}
          className="w-6 h-6 text-primary-600 dark:text-primary-400 rounded-xl border-slate-200 dark:border-slate-700 focus:ring-primary-500 transition-all cursor-pointer"
          id="is_break_overtime"
          onChange={(e) =>
            { setFormData({ ...formData, isBreakOvertime: e.target.checked }); }
          }
          type="checkbox"
        />
        <label
          className="text-xs font-bold text-slate-700 dark:text-slate-200 uppercase tracking-widest cursor-pointer leading-tight"
          htmlFor="is_break_overtime"
        >
          {t("attendance_confirm.ot_label")}
        </label>
      </div>

      <div className="space-y-2">
        <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">
          {t("time_tracking.modal.reason_label")}
        </label>
        <textarea
          className="w-full px-5 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-medium text-sm h-28 resize-none"
          onChange={(e) =>
            { setFormData({ ...formData, reason: e.target.value }); }
          }
          placeholder={t("time_tracking.modal.reason_placeholder")}
          required
          value={formData.reason}
        />
      </div>
    </>
  );
}

export function AdjustmentRequestModal({
  isOpen,
  onClose,
  timeEntry,
  onSubmit,
  timezone: _timezone
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
    isBreakOvertime: timeEntry?.is_break_overtime ?? false,
    timeEntryId: timeEntry?.id
  });

  const handleSubmit = async (e: React.SyntheticEvent) => {
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
      icon={<History size={24} />}
      isOpen={isOpen}
      maxWidth="md"
      onClose={onClose}
      subtitle={t("time_tracking.modal.subtitle")}
      title={
        formData.timeEntryId
          ? t("time_tracking.modal.edit_title")
          : t("time_tracking.modal.new_title")
      }
    >
      <form className="space-y-8 pt-4" onSubmit={handleSubmit}>
        <AdjustmentFormFields formData={formData} setFormData={setFormData} t={t} />

        <div className="pt-6 grid grid-cols-2 gap-5">
          <Button className="px-8 py-4 border border-slate-100 dark:border-slate-800 rounded-2xl font-bold text-xs uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800 transition-all"
            onClick={onClose}
            type="button"
            variant="ghost"
          >
            {t("common.cancel")}
          </Button>
          <Button className="px-8 py-4 bg-slate-900 text-white rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-slate-200 dark:shadow-black/30 active:scale-95 disabled:opacity-50"
            disabled={isSubmitting}
            type="submit"
            variant="primary"
          >
            {isSubmitting ? t("common.sending").toUpperCase() : t("common.submit_request")}
          </Button>
        </div>
      </form>
    </Modal>
  );
}
