import { Save } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { TFunction } from "i18next";

import Modal from "@/components/ui/modal";

interface CreateShiftModalProps {
  isAutoDeduct: boolean;
  isFixedBreak: boolean;
  isLoading: boolean;
  isOpen: boolean;
  newBreakEnd: string;
  newBreakStart: string;
  newShiftBreak: number;
  newShiftEnd: string;
  newShiftName: string;
  newShiftStart: string;
  newShiftSymbol: string;
  onClose: () => void;
  onCreateShift: () => void;
  setIsAutoDeduct: (value: boolean) => void;
  setIsFixedBreak: (value: boolean) => void;
  setNewBreakEnd: (value: string) => void;
  setNewBreakStart: (value: string) => void;
  setNewShiftBreak: (value: number) => void;
  setNewShiftEnd: (value: string) => void;
  setNewShiftName: (value: string) => void;
  setNewShiftStart: (value: string) => void;
  setNewShiftSymbol: (value: string) => void;
}

function BasicShiftFields({
  t,
  newShiftName,
  setNewShiftName,
  newShiftSymbol,
  setNewShiftSymbol,
  newShiftStart,
  setNewShiftStart,
  newShiftEnd,
  setNewShiftEnd
}: {
  newShiftEnd: string;
  newShiftName: string;
  newShiftStart: string;
  newShiftSymbol: string;
  setNewShiftEnd: (val: string) => void;
  setNewShiftName: (val: string) => void;
  setNewShiftStart: (val: string) => void;
  setNewShiftSymbol: (val: string) => void;
  t: TFunction;
}) {
  return (
    <>
      <div className="grid grid-cols-1 gap-4">
        <div className="space-y-2">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("shifts.table.shift_name")}</label>
          <input
            className="w-full bg-slate-50 dark:bg-slate-800 border-none rounded-2xl px-5 py-3.5 font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 transition-all"
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
            className="w-full bg-slate-50 dark:bg-slate-800 border-none rounded-2xl px-5 py-3.5 font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 transition-all"
            maxLength={10}
            onChange={(e) => { setNewShiftSymbol(e.target.value.slice(0, 10)); }}
            placeholder={t("shifts.modal.report_symbol_placeholder", "VD: S8, T8, FX...")}
            type="text"
            value={newShiftSymbol}
          />
        </div>
      </div>

      <div className="grid grid-cols-2 gap-4">
        <div className="space-y-2">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("shifts.modal.start_time")}</label>
          <input
            className="w-full bg-slate-50 dark:bg-slate-800 border-none rounded-2xl px-5 py-3.5 font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 transition-all"
            onChange={(e) => { setNewShiftStart(e.target.value); }}
            type="time"
            value={newShiftStart}
          />
        </div>
        <div className="space-y-2">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("shifts.modal.end_time")}</label>
          <input
            className="w-full bg-slate-50 dark:bg-slate-800 border-none rounded-2xl px-5 py-3.5 font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 transition-all"
            onChange={(e) => { setNewShiftEnd(e.target.value); }}
            type="time"
            value={newShiftEnd}
          />
        </div>
      </div>
    </>
  );
}

function BreakConfigPanel({
  t,
  isFixedBreak,
  setIsFixedBreak,
  newBreakStart,
  setNewBreakStart,
  newBreakEnd,
  setNewBreakEnd,
  newShiftBreak,
  setNewShiftBreak,
  isAutoDeduct,
  setIsAutoDeduct
}: {
  isAutoDeduct: boolean;
  isFixedBreak: boolean;
  newBreakEnd: string;
  newBreakStart: string;
  newShiftBreak: number;
  setIsAutoDeduct: (val: boolean) => void;
  setIsFixedBreak: (val: boolean) => void;
  setNewBreakEnd: (val: string) => void;
  setNewBreakStart: (val: string) => void;
  setNewShiftBreak: (val: number) => void;
  t: TFunction;
}) {
  return (
    <div className="space-y-4 bg-slate-50 dark:bg-slate-800 p-6 rounded-[32px] border border-slate-100 dark:border-slate-800">
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

      <div className="flex items-center gap-3 pt-2">
        <input
          checked={isAutoDeduct}
          className="w-5 h-5 rounded-lg border-slate-200 dark:border-slate-700 text-primary-600 dark:text-primary-400 focus:ring-primary-500"
          id="auto_deduct"
          onChange={(e) => { setIsAutoDeduct(e.target.checked); }}
          type="checkbox"
        />
        <label className="text-sm font-bold text-slate-700 dark:text-slate-200" htmlFor="auto_deduct">
          {t("shifts.modal.auto_deduct_label")}
        </label>
      </div>
    </div>
  );
}

export function CreateShiftModal({
  isOpen,
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
      subtitle={t("shifts.modal.setup_subtitle")}
      title={t("shifts.modal.setup_title")}
    >
      <div className="space-y-6 pt-4">
        <div className="space-y-4">
          <BasicShiftFields
            newShiftEnd={newShiftEnd}
            newShiftName={newShiftName}
            newShiftStart={newShiftStart}
            newShiftSymbol={newShiftSymbol}
            setNewShiftEnd={setNewShiftEnd}
            setNewShiftName={setNewShiftName}
            setNewShiftStart={setNewShiftStart}
            setNewShiftSymbol={setNewShiftSymbol}
            t={t}
          />

          <BreakConfigPanel
            isAutoDeduct={isAutoDeduct}
            isFixedBreak={isFixedBreak}
            newBreakEnd={newBreakEnd}
            newBreakStart={newBreakStart}
            newShiftBreak={newShiftBreak}
            setIsAutoDeduct={setIsAutoDeduct}
            setIsFixedBreak={setIsFixedBreak}
            setNewBreakEnd={setNewBreakEnd}
            setNewBreakStart={setNewBreakStart}
            setNewShiftBreak={setNewShiftBreak}
            t={t}
          />
        </div>

        <div className="flex gap-3">
          <button
            className="flex-1 px-6 py-4 rounded-2xl font-bold text-xs uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-700 transition-all"
            onClick={onClose}
          >
            {t("common.cancel")}
          </button>
          <button
            className="flex-1 bg-primary-600 hover:bg-primary-700 text-white font-bold px-6 py-4 rounded-2xl flex items-center justify-center gap-2 shadow-lg shadow-primary-100 transition-all active:scale-95 disabled:opacity-50 text-xs uppercase tracking-widest"
            disabled={isLoading}
            onClick={onCreateShift}
          >
            <Save className="w-4 h-4" /> {t("shifts.modal.save_shift_btn")}
          </button>
        </div>
      </div>
    </Modal>
  );
}
