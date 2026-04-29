import React from "react";

import { Save, Zap, Info } from "lucide-react";
import { useTranslation } from "react-i18next";

import Modal from "@/components/ui/Modal";

import { Shift } from "@/lib/api";

interface EditShiftModalProps {
  editingShift: Shift | null;
  onClose: () => void;
  newShiftName: string;
  setNewShiftName: (value: string) => void;
  newShiftSymbol: string;
  setNewShiftSymbol: (value: string) => void;
  newShiftStart: string;
  setNewShiftStart: (value: string) => void;
  newShiftEnd: string;
  setNewShiftEnd: (value: string) => void;
  newShiftBreak: number;
  setNewShiftBreak: (value: number) => void;
  newShiftGrace: number;
  setNewShiftGrace: (value: number) => void;
  newBreakStart: string;
  setNewBreakStart: (value: string) => void;
  newBreakEnd: string;
  setNewBreakEnd: (value: string) => void;
  isFixedBreak: boolean;
  setIsFixedBreak: (value: boolean) => void;
  isAutoDeduct: boolean;
  setIsAutoDeduct: (value: boolean) => void;
  isCorrection: boolean;
  setIsCorrection: (value: boolean) => void;
  correctionEffectiveDate: string;
  setCorrectionEffectiveDate: (value: string) => void;
  onUpdateShift: () => void;
  isLoading: boolean;
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
      title={t("shifts.modal.setup_title")}
      subtitle={t("shifts.modal.edit_subtitle")}
    >
      <div className="space-y-4 pt-2">
        <div className="space-y-3">
          <div className="space-y-2">
            <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("shifts.table.shift_name")}</label>
            <input
              type="text"
              value={newShiftName}
              onChange={(e) => setNewShiftName(e.target.value)}
              placeholder={t("shifts.modal.name_placeholder")}
              className="w-full bg-slate-50 dark:bg-slate-800 border-none rounded-xl px-4 py-2.5 font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 transition-all text-sm"
            />
          </div>

          <div className="space-y-2">
            <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
              {t("shifts.modal.report_symbol", "Ký hiệu báo cáo")}
              <span className="ml-1 normal-case font-normal text-slate-400">(tối đa 10 ký tự)</span>
            </label>
            <input
              type="text"
              value={newShiftSymbol}
              onChange={(e) => setNewShiftSymbol(e.target.value.slice(0, 10))}
              placeholder={t("shifts.modal.report_symbol_placeholder", "VD: S8, T8, FX...")}
              maxLength={10}
              className="w-full bg-slate-50 dark:bg-slate-800 border-none rounded-xl px-4 py-2.5 font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 transition-all text-sm"
            />
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("shifts.modal.start_time")}</label>
              <input
                type="time"
                value={newShiftStart}
                onChange={(e) => setNewShiftStart(e.target.value)}
                className="w-full bg-slate-50 dark:bg-slate-800 border-none rounded-xl px-4 py-2.5 font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 transition-all text-sm"
              />
            </div>
            <div className="space-y-2">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("shifts.modal.end_time")}</label>
              <input
                type="time"
                value={newShiftEnd}
                onChange={(e) => setNewShiftEnd(e.target.value)}
                className="w-full bg-slate-50 dark:bg-slate-800 border-none rounded-xl px-4 py-2.5 font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 transition-all text-sm"
              />
            </div>
          </div>

          <div className="space-y-3 bg-slate-50 dark:bg-slate-800 p-4 rounded-2xl border border-slate-100 dark:border-slate-800">
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

            <div className="flex items-center gap-3 pt-1">
              <input
                type="checkbox"
                id="auto_deduct_edit"
                checked={isAutoDeduct}
                onChange={(e) => setIsAutoDeduct(e.target.checked)}
                className="w-4 h-4 rounded-md border-slate-200 dark:border-slate-700 text-primary-600 dark:text-primary-400 focus:ring-primary-500"
              />
              <label htmlFor="auto_deduct_edit" className="text-sm font-bold text-slate-700 dark:text-slate-200">
                {t("shifts.modal.auto_deduct_label")}
              </label>
            </div>

            {/* Grace Period */}
            <div className="space-y-2 pt-1">
              <div className="flex items-center gap-2">
                <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase">{t("shifts.grace_period")}</label>
                <div className="group relative">
                  <Info size={12} className="text-slate-300 dark:text-slate-600 cursor-help" />
                  <div className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 w-48 p-3 bg-slate-800 text-white text-[10px] rounded-xl opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none z-50">
                    {t("shifts.grace_period_desc")}
                  </div>
                </div>
              </div>
              <input
                type="number"
                min="0"
                value={newShiftGrace}
                onChange={(e) => setNewShiftGrace(parseInt(e.target.value) || 0)}
                className="w-full bg-white dark:bg-slate-900 border-none rounded-xl px-4 py-2.5 font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 transition-all"
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
                type="checkbox"
                id="is_correction"
                checked={isCorrection}
                onChange={(e) => setIsCorrection(e.target.checked)}
                className="w-4 h-4 rounded-md border-amber-200 text-amber-600 dark:text-amber-300 focus:ring-amber-500"
              />
              <label htmlFor="is_correction" className="text-xs font-bold text-amber-800 uppercase tracking-widest">
                {t("shifts.modal.correction_warning")}
              </label>
            </div>

            {!isCorrection && (
              <div className="space-y-2 animate-in fade-in slide-in-from-top-2 duration-300">
                <label className="text-[10px] font-bold text-amber-700 dark:text-amber-300 uppercase tracking-widest">{t("shifts.modal.effective_date_label")}</label>
                <input
                  type="date"
                  value={correctionEffectiveDate}
                  onChange={(e) => setCorrectionEffectiveDate(e.target.value)}
                  className="w-full bg-white dark:bg-slate-900 border-amber-200 rounded-xl px-4 py-2.5 font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-amber-500 transition-all text-sm"
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
            onClick={onClose}
            className="flex-1 px-6 py-3 rounded-2xl font-bold text-xs uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-700 transition-all"
          >
            {t("common.cancel")}
          </button>
          <button
            onClick={onUpdateShift}
            disabled={isLoading}
            className="flex-1 bg-primary-600 hover:bg-primary-700 text-white font-bold px-6 py-3 rounded-2xl flex items-center justify-center gap-2 shadow-lg shadow-primary-100 transition-all active:scale-95 disabled:opacity-50 text-xs uppercase tracking-widest"
          >
            <Save className="w-4 h-4" /> {t("shifts.modal.update_config_btn")}
          </button>
        </div>
      </div>
    </Modal>
  );
}
