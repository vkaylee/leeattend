import { Save, Zap, Info } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Modal from "@/components/ui/Modal";
import { Shift } from "@/lib/api";

interface EditShiftModalProps {
  correctionEffectiveDate: string;
  editingShift: Shift | null;
  isAutoDeduct: boolean;
  isCorrection: boolean;
  isFixedBreak: boolean;
  isLoading: boolean;
  newBreakEnd: string;
  newBreakStart: string;
  newShiftBreak: number;
  newShiftEnd: string;
  newShiftGrace: number;
  newShiftName: string;
  newShiftStart: string;
  newShiftSymbol: string;
  onClose: () => void;
  onUpdateShift: () => void;
  setCorrectionEffectiveDate: (value: string) => void;
  setIsAutoDeduct: (value: boolean) => void;
  setIsCorrection: (value: boolean) => void;
  setIsFixedBreak: (value: boolean) => void;
  setNewBreakEnd: (value: string) => void;
  setNewBreakStart: (value: string) => void;
  setNewShiftBreak: (value: number) => void;
  setNewShiftEnd: (value: string) => void;
  setNewShiftGrace: (value: number) => void;
  setNewShiftName: (value: string) => void;
  setNewShiftStart: (value: string) => void;
  setNewShiftSymbol: (value: string) => void;
}

export function EditShiftModal({
  editingShift,
  onClose,
  newShiftName,
  setNewShiftName,
  newShiftSymbol,
  setNewShiftSymbol,
  newShiftStart,
  setNewShiftStart,
  newShiftEnd,
  setNewShiftEnd,
  newShiftBreak,
  setNewShiftBreak,
  newShiftGrace,
  setNewShiftGrace,
  newBreakStart,
  setNewBreakStart,
  newBreakEnd,
  setNewBreakEnd,
  isFixedBreak,
  setIsFixedBreak,
  isAutoDeduct,
  setIsAutoDeduct,
  isCorrection,
  setIsCorrection,
  correctionEffectiveDate,
  setCorrectionEffectiveDate,
  onUpdateShift,
  isLoading,
}: EditShiftModalProps) {
  const { t } = useTranslation();

  return (
    <Modal
      isOpen={!!editingShift}
      onClose={onClose}
      subtitle={t("shifts.modal.edit_subtitle")}
      title={t("shifts.modal.setup_title")}
    >
      <div className="space-y-4 pt-2">
        <div className="space-y-3">
          <div className="space-y-2">
            <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("shifts.table.shift_name")}</label>
            <input
              className="w-full bg-slate-50 dark:bg-slate-800 border-none rounded-xl px-4 py-2.5 font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 transition-all text-sm"
              onChange={(e) => { setNewShiftName(e.target.value); }}
              placeholder={t("shifts.modal.name_placeholder")}
              type="text"
              value={newShiftName}
            />
          </div>

          <div className="space-y-2">
            <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
              {t("shifts.modal.report_symbol", "Ký hiệu báo cáo")}
              <span className="ml-1 normal-case font-normal text-slate-400">(tối đa 10 ký tự)</span>
            </label>
            <input
              className="w-full bg-slate-50 dark:bg-slate-800 border-none rounded-xl px-4 py-2.5 font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 transition-all text-sm"
              maxLength={10}
              onChange={(e) => { setNewShiftSymbol(e.target.value.slice(0, 10)); }}
              placeholder={t("shifts.modal.report_symbol_placeholder", "VD: S8, T8, FX...")}
              type="text"
              value={newShiftSymbol}
            />
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("shifts.modal.start_time")}</label>
              <input
                className="w-full bg-slate-50 dark:bg-slate-800 border-none rounded-xl px-4 py-2.5 font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 transition-all text-sm"
                onChange={(e) => { setNewShiftStart(e.target.value); }}
                type="time"
                value={newShiftStart}
              />
            </div>
            <div className="space-y-2">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("shifts.modal.end_time")}</label>
              <input
                className="w-full bg-slate-50 dark:bg-slate-800 border-none rounded-xl px-4 py-2.5 font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 transition-all text-sm"
                onChange={(e) => { setNewShiftEnd(e.target.value); }}
                type="time"
                value={newShiftEnd}
              />
            </div>
          </div>

          <div className="space-y-3 bg-slate-50 dark:bg-slate-800 p-4 rounded-2xl border border-slate-100 dark:border-slate-800">
            <div className="flex items-center justify-between">
              <h4 className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("shifts.modal.break_config")}</h4>
              <div className="flex items-center gap-2">
                <span className="text-[10px] font-bold text-slate-500 dark:text-slate-400">{t("shifts.types.fixed")}</span>
                <button
                  className={`relative inline-flex h-5 w-9 items-center rounded-full transition-colors ${isFixedBreak ? 'bg-primary-600' : 'bg-slate-200 dark:bg-slate-600'}`}
                  onClick={() => { setIsFixedBreak(!isFixedBreak); }}
                  type="button"
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
                    className="w-full bg-white dark:bg-slate-900 border-none rounded-xl px-4 py-2.5 font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 transition-all"
                    onChange={(e) => { setNewBreakStart(e.target.value); }}
                    type="time"
                    value={newBreakStart}
                  />
                </div>
                <div className="space-y-2">
                  <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase">{t("shifts.modal.break_end")}</label>
                  <input
                    className="w-full bg-white dark:bg-slate-900 border-none rounded-xl px-4 py-2.5 font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 transition-all"
                    onChange={(e) => { setNewBreakEnd(e.target.value); }}
                    type="time"
                    value={newBreakEnd}
                  />
                </div>
              </div>
            ) : (
              <div className="space-y-2">
                <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase">{t("shifts.modal.break_duration")} ({t("common.minutes")})</label>
                <input
                  className="w-full bg-white dark:bg-slate-900 border-none rounded-xl px-4 py-2.5 font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 transition-all"
                  onChange={(e) => { setNewShiftBreak(parseInt(e.target.value) || 0); }}
                  type="number"
                  value={newShiftBreak}
                />
              </div>
            )}

            <div className="flex items-center gap-3 pt-1">
              <input
                checked={isAutoDeduct}
                className="w-4 h-4 rounded-md border-slate-200 dark:border-slate-700 text-primary-600 dark:text-primary-400 focus:ring-primary-500"
                id="auto_deduct_edit"
                onChange={(e) => { setIsAutoDeduct(e.target.checked); }}
                type="checkbox"
              />
              <label className="text-sm font-bold text-slate-700 dark:text-slate-200" htmlFor="auto_deduct_edit">
                {t("shifts.modal.auto_deduct_label")}
              </label>
            </div>

            {/* Grace Period */}
            <div className="space-y-2 pt-1">
              <div className="flex items-center gap-2">
                <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase">{t("shifts.grace_period")}</label>
                <div className="group relative">
                  <Info className="text-slate-300 dark:text-slate-600 cursor-help" size={12} />
                  <div className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 w-48 p-3 bg-slate-800 text-white text-[10px] rounded-xl opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none z-50">
                    {t("shifts.grace_period_desc")}
                  </div>
                </div>
              </div>
              <input
                className="w-full bg-white dark:bg-slate-900 border-none rounded-xl px-4 py-2.5 font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 transition-all"
                min="0"
                onChange={(e) => { setNewShiftGrace(parseInt(e.target.value) || 0); }}
                type="number"
                value={newShiftGrace}
              />
            </div>
          </div>

          <div className="bg-amber-50 dark:bg-amber-950 p-4 rounded-2xl border border-amber-100 dark:border-amber-800 space-y-3">
            <div className="flex items-center gap-3">
              <div className="bg-amber-100 dark:bg-amber-900 p-2 rounded-xl text-amber-600 dark:text-amber-300">
                <Zap size={16} />
              </div>
              <div>
                <h4 className="text-xs font-bold text-amber-900 uppercase tracking-tight leading-none">{t("shifts.modal.correction_label")}</h4>
                <p className="text-[10px] font-bold text-amber-700 dark:text-amber-300 leading-tight">{t("shifts.modal.correction_desc")}</p>
              </div>
            </div>

            <div className="flex items-center gap-3 bg-white dark:bg-slate-900/50 p-2.5 rounded-xl">
              <input
                checked={isCorrection}
                className="w-4 h-4 rounded-md border-amber-200 text-amber-600 dark:text-amber-300 focus:ring-amber-500"
                id="is_correction"
                onChange={(e) => { setIsCorrection(e.target.checked); }}
                type="checkbox"
              />
              <label className="text-xs font-bold text-amber-800 uppercase tracking-widest" htmlFor="is_correction">
                {t("shifts.modal.correction_warning")}
              </label>
            </div>

            {!isCorrection && (
              <div className="space-y-2 animate-in fade-in slide-in-from-top-2 duration-300">
                <label className="text-[10px] font-bold text-amber-700 dark:text-amber-300 uppercase tracking-widest">{t("shifts.modal.effective_date_label")}</label>
                <input
                  className="w-full bg-white dark:bg-slate-900 border-amber-200 rounded-xl px-4 py-2.5 font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-amber-500 transition-all text-sm"
                  onChange={(e) => { setCorrectionEffectiveDate(e.target.value); }}
                  type="date"
                  value={correctionEffectiveDate}
                />
                <p className="text-[9px] font-bold text-amber-600 dark:text-amber-300 leading-tight italic px-1">
                  {t("shifts.modal.effective_date_desc")}
                </p>
              </div>
            )}
          </div>
        </div>

        <div className="flex gap-3">
          <button
            className="flex-1 px-6 py-3 rounded-2xl font-bold text-xs uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-700 transition-all"
            onClick={onClose}
          >
            {t("common.cancel")}
          </button>
          <button
            className="flex-1 bg-primary-600 hover:bg-primary-700 text-white font-bold px-6 py-3 rounded-2xl flex items-center justify-center gap-2 shadow-lg shadow-primary-100 transition-all active:scale-95 disabled:opacity-50 text-xs uppercase tracking-widest"
            disabled={isLoading}
            onClick={onUpdateShift}
          >
            <Save className="w-4 h-4" /> {t("shifts.modal.update_config_btn")}
          </button>
        </div>
      </div>
    </Modal>
  );
}
