import React from "react";
import { useTranslation } from "react-i18next";
import { Save } from "lucide-react";
import Modal from "@/components/ui/Modal";

interface CreateShiftModalProps {
  isOpen: boolean;
  onClose: () => void;
  newShiftName: string;
  setNewShiftName: (value: string) => void;
  newShiftStart: string;
  setNewShiftStart: (value: string) => void;
  newShiftEnd: string;
  setNewShiftEnd: (value: string) => void;
  newShiftBreak: number;
  setNewShiftBreak: (value: number) => void;
  newBreakStart: string;
  setNewBreakStart: (value: string) => void;
  newBreakEnd: string;
  setNewBreakEnd: (value: string) => void;
  isFixedBreak: boolean;
  setIsFixedBreak: (value: boolean) => void;
  isAutoDeduct: boolean;
  setIsAutoDeduct: (value: boolean) => void;
  onCreateShift: () => void;
  isLoading: boolean;
}

export function CreateShiftModal({
  isOpen,
  onClose,
  newShiftName,
  setNewShiftName,
  newShiftStart,
  setNewShiftStart,
  newShiftEnd,
  setNewShiftEnd,
  newShiftBreak,
  setNewShiftBreak,
  newBreakStart,
  setNewBreakStart,
  newBreakEnd,
  setNewBreakEnd,
  isFixedBreak,
  setIsFixedBreak,
  isAutoDeduct,
  setIsAutoDeduct,
  onCreateShift,
  isLoading,
}: CreateShiftModalProps) {
  const { t } = useTranslation();

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title={t("shifts.modal.setup_title")}
      subtitle={t("shifts.modal.setup_subtitle")}
    >
      <div className="space-y-6 pt-4">
        <div className="space-y-4">
          <div className="grid grid-cols-1 gap-4">
            <div className="space-y-2">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("shifts.table.shift_name")}</label>
              <input
                type="text"
                value={newShiftName}
                onChange={(e) => setNewShiftName(e.target.value)}
                placeholder={t("shifts.modal.name_placeholder")}
                className="w-full bg-slate-50 dark:bg-slate-800 border-none rounded-2xl px-5 py-3.5 font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 transition-all"
              />
            </div>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("shifts.modal.start_time")}</label>
              <input
                type="time"
                value={newShiftStart}
                onChange={(e) => setNewShiftStart(e.target.value)}
                className="w-full bg-slate-50 dark:bg-slate-800 border-none rounded-2xl px-5 py-3.5 font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 transition-all"
              />
            </div>
            <div className="space-y-2">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("shifts.modal.end_time")}</label>
              <input
                type="time"
                value={newShiftEnd}
                onChange={(e) => setNewShiftEnd(e.target.value)}
                className="w-full bg-slate-50 dark:bg-slate-800 border-none rounded-2xl px-5 py-3.5 font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 transition-all"
              />
            </div>
          </div>

          <div className="space-y-4 bg-slate-50 dark:bg-slate-800 p-6 rounded-[32px] border border-slate-100 dark:border-slate-800">
            <div className="flex items-center justify-between">
              <h4 className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("shifts.modal.break_config")}</h4>
              <div className="flex items-center gap-2">
                <span className="text-[10px] font-bold text-slate-500 dark:text-slate-400">{t("shifts.types.fixed")}</span>
                <button
                  type="button"
                  onClick={() => setIsFixedBreak(!isFixedBreak)}
                  className={`relative inline-flex h-5 w-9 items-center rounded-full transition-colors ${isFixedBreak ? 'bg-primary-600' : 'bg-slate-200 dark:bg-slate-600'}`}
                >
                  <span className={`inline-block h-3.5 w-3.5 transform rounded-full bg-white dark:bg-slate-900 transition-transform ${isFixedBreak ? 'translate-x-5' : 'translate-x-1'}`} />
                </button>
              </div>
            </div>

            {isFixedBreak ? (
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase">{t("shifts.modal.break_start")}</label>
                  <input
                    type="time"
                    value={newBreakStart}
                    onChange={(e) => setNewBreakStart(e.target.value)}
                    className="w-full bg-white dark:bg-slate-900 border-none rounded-xl px-4 py-2.5 font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 transition-all"
                  />
                </div>
                <div className="space-y-2">
                  <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase">{t("shifts.modal.break_end")}</label>
                  <input
                    type="time"
                    value={newBreakEnd}
                    onChange={(e) => setNewBreakEnd(e.target.value)}
                    className="w-full bg-white dark:bg-slate-900 border-none rounded-xl px-4 py-2.5 font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 transition-all"
                  />
                </div>
              </div>
            ) : (
              <div className="space-y-2">
                <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase">{t("shifts.modal.break_duration")} ({t("common.minutes")})</label>
                <input
                  type="number"
                  value={newShiftBreak}
                  onChange={(e) => setNewShiftBreak(parseInt(e.target.value) || 0)}
                  className="w-full bg-white dark:bg-slate-900 border-none rounded-xl px-4 py-2.5 font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 transition-all"
                />
              </div>
            )}

            <div className="flex items-center gap-3 pt-2">
              <input
                type="checkbox"
                id="auto_deduct"
                checked={isAutoDeduct}
                onChange={(e) => setIsAutoDeduct(e.target.checked)}
                className="w-5 h-5 rounded-lg border-slate-200 dark:border-slate-700 text-primary-600 dark:text-primary-400 focus:ring-primary-500"
              />
              <label htmlFor="auto_deduct" className="text-sm font-bold text-slate-700 dark:text-slate-200">
                {t("shifts.modal.auto_deduct_label")}
              </label>
            </div>
          </div>
        </div>

        <div className="flex gap-3">
          <button
            onClick={onClose}
            className="flex-1 px-6 py-4 rounded-2xl font-bold text-xs uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-700 transition-all"
          >
            {t("common.cancel")}
          </button>
          <button
            onClick={onCreateShift}
            disabled={isLoading}
            className="flex-1 bg-primary-600 hover:bg-primary-700 text-white font-bold px-6 py-4 rounded-2xl flex items-center justify-center gap-2 shadow-lg shadow-primary-100 transition-all active:scale-95 disabled:opacity-50 text-xs uppercase tracking-widest"
          >
            <Save className="w-4 h-4" /> {t("shifts.modal.save_shift_btn")}
          </button>
        </div>
      </div>
    </Modal>
  );
}
