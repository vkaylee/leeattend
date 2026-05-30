import { Save, Info } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { TFunction } from "i18next";

import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import { Typography } from "@/components/ui/typography";

interface CreateShiftModalProps {
  isAutoDeduct: boolean;
  isBreakOvertime: boolean;
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
  newShiftGrace: number;
  allowEarlyOt: boolean;
  maxEarlyOtMinutes: number;
  onClose: () => void;
  onCreateShift: () => void;
  setIsAutoDeduct: (value: boolean) => void;
  setIsBreakOvertime: (value: boolean) => void;
  setIsFixedBreak: (value: boolean) => void;
  setNewBreakEnd: (value: string) => void;
  setNewBreakStart: (value: string) => void;
  setNewShiftBreak: (value: number) => void;
  setNewShiftEnd: (value: string) => void;
  setNewShiftName: (value: string) => void;
  setNewShiftStart: (value: string) => void;
  setNewShiftSymbol: (value: string) => void;
  setNewShiftGrace: (value: number) => void;
  setAllowEarlyOt: (value: boolean) => void;
  setMaxEarlyOtMinutes: (value: number) => void;
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
  setIsAutoDeduct,
  isBreakOvertime,
  setIsBreakOvertime
}: {
  isAutoDeduct: boolean;
  isBreakOvertime: boolean;
  isFixedBreak: boolean;
  newBreakEnd: string;
  newBreakStart: string;
  newShiftBreak: number;
  setIsAutoDeduct: (val: boolean) => void;
  setIsBreakOvertime: (val: boolean) => void;
  setIsFixedBreak: (val: boolean) => void;
  setNewBreakEnd: (val: string) => void;
  setNewBreakStart: (val: string) => void;
  setNewShiftBreak: (val: number) => void;
  t: TFunction;
}) {
  return (
    <div className="space-y-4 bg-slate-50 dark:bg-slate-800 p-6 rounded-[32px] border border-slate-100 dark:border-slate-800">
      <div className="grid grid-cols-[110px_1fr] gap-4 items-start">
        {/* Column 1: Break Configuration Toggle */}
        <div className="space-y-2">
          <Typography className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest block" variant="h4">
            {t("shifts.modal.break_config")}
          </Typography>
          <div className="flex items-center justify-between h-[42px] bg-white dark:bg-slate-900 px-4 rounded-xl border border-slate-100/50 dark:border-slate-800/50">
            <span className="text-[10px] font-bold text-slate-500 dark:text-slate-400">{t("shifts.types.fixed")}</span>
            <button
              aria-label={t("shifts.types.fixed")}
              className={`relative inline-flex h-5 w-9 items-center rounded-full transition-colors focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2 ${isFixedBreak ? 'bg-primary-600' : 'bg-slate-200 dark:bg-slate-600'}`}
              onClick={() => { setIsFixedBreak(!isFixedBreak); }}
              type="button"
            >
              <span className={`inline-block h-3.5 w-3.5 transform rounded-full bg-white dark:bg-slate-900 transition-transform ${isFixedBreak ? 'translate-x-5' : 'translate-x-1'}`} />
            </button>
          </div>
        </div>

        {/* Column 2: Break Duration or Break Start/End */}
        {isFixedBreak ? (
          <div className="grid grid-cols-2 gap-2">
            <div className="space-y-2">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase block">{t("shifts.modal.break_start")}</label>
              <input
                className="w-full bg-white dark:bg-slate-900 border-none rounded-xl px-4 py-2.5 font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 transition-all text-xs"
                onChange={(e) => { setNewBreakStart(e.target.value); }}
                type="time"
                value={newBreakStart}
              />
            </div>
            <div className="space-y-2">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase block">{t("shifts.modal.break_end")}</label>
              <input
                className="w-full bg-white dark:bg-slate-900 border-none rounded-xl px-4 py-2.5 font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 transition-all text-xs"
                onChange={(e) => { setNewBreakEnd(e.target.value); }}
                type="time"
                value={newBreakEnd}
              />
            </div>
          </div>
        ) : (
          <div className="space-y-2">
            <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase block">
              {t("shifts.modal.break_duration")} ({t("common.minutes")})
            </label>
            <input
              className="w-full bg-white dark:bg-slate-900 border-none rounded-xl px-4 py-2.5 font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 transition-all text-xs"
              onChange={(e) => { setNewShiftBreak(parseInt(e.target.value) || 0); }}
              type="number"
              value={newShiftBreak}
            />
          </div>
        )}
      </div>

      <div className="space-y-3 pt-2">
        <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest block">
          {t("shifts.modal.break_policy_label")}
        </label>
        <div className="space-y-2">
          {/* Option 1: Auto Deduct */}
          <div className="flex items-start gap-3 p-3 bg-white dark:bg-slate-900/50 rounded-2xl border border-slate-100 dark:border-slate-800/80 cursor-pointer hover:bg-slate-100/50 dark:hover:bg-slate-800/30 transition-all">
            <input
              checked={isAutoDeduct && !isBreakOvertime ? true : false}
              className="mt-0.5 w-4 h-4 text-primary-600 border-slate-300 dark:border-slate-700 focus:ring-primary-500 cursor-pointer"
              id="break_policy_auto_deduct"
              name="break_policy"
              onChange={() => {
                setIsAutoDeduct(true);
                setIsBreakOvertime(false);
              }}
              type="radio"
              value="auto_deduct"
            />
            <div className="space-y-1">
              <label className="text-xs font-bold text-slate-700 dark:text-slate-200 cursor-pointer block" htmlFor="break_policy_auto_deduct">
                {t("shifts.modal.break_policy_auto_deduct")}
              </label>
              <span className="text-[10px] text-slate-500 dark:text-slate-400 block font-normal leading-normal">
                {t("shifts.modal.break_policy_auto_deduct_help")}
              </span>
            </div>
          </div>

          {/* Option 2: No Deduct */}
          <div className="flex items-start gap-3 p-3 bg-white dark:bg-slate-900/50 rounded-2xl border border-slate-100 dark:border-slate-800/80 cursor-pointer hover:bg-slate-100/50 dark:hover:bg-slate-800/30 transition-all">
            <input
              checked={!isAutoDeduct && !isBreakOvertime ? true : false}
              className="mt-0.5 w-4 h-4 text-primary-600 border-slate-300 dark:border-slate-700 focus:ring-primary-500 cursor-pointer"
              id="break_policy_no_deduct"
              name="break_policy"
              onChange={() => {
                setIsAutoDeduct(false);
                setIsBreakOvertime(false);
              }}
              type="radio"
              value="no_deduct"
            />
            <div className="space-y-1">
              <label className="text-xs font-bold text-slate-700 dark:text-slate-200 cursor-pointer block" htmlFor="break_policy_no_deduct">
                {t("shifts.modal.break_policy_no_deduct")}
              </label>
              <span className="text-[10px] text-slate-500 dark:text-slate-400 block font-normal leading-normal">
                {t("shifts.modal.break_policy_no_deduct_help")}
              </span>
            </div>
          </div>

          {/* Option 3: Break Overtime */}
          <div className={`flex items-start gap-3 p-3 rounded-2xl border transition-all ${
            !isFixedBreak 
              ? "bg-slate-100/50 dark:bg-slate-900/20 border-slate-200/50 dark:border-slate-800/30 opacity-60 cursor-not-allowed" 
              : "bg-white dark:bg-slate-900/50 border-slate-100 dark:border-slate-800/80 cursor-pointer hover:bg-slate-100/50 dark:hover:bg-slate-800/30"
          }`}>
            <input
              checked={isBreakOvertime ? true : false}
              className={`mt-0.5 w-4 h-4 text-primary-600 border-slate-300 dark:border-slate-700 focus:ring-primary-500 ${
                !isFixedBreak ? "cursor-not-allowed" : "cursor-pointer"
              }`}
              disabled={!isFixedBreak}
              id="break_policy_overtime"
              name="break_policy"
              onChange={() => {
                setIsAutoDeduct(false);
                setIsBreakOvertime(true);
              }}
              type="radio"
              value="overtime"
            />
            <div className="space-y-1">
              <label className={`text-xs font-bold block ${!isFixedBreak ? "text-slate-400 dark:text-slate-500 cursor-not-allowed" : "text-slate-700 dark:text-slate-200 cursor-pointer"}`} htmlFor="break_policy_overtime">
                {t("shifts.modal.break_policy_overtime")}
              </label>
              <span className="text-[10px] text-slate-500 dark:text-slate-400 block font-normal leading-normal">
                {!isFixedBreak ? (
                  <span className="text-amber-600 dark:text-amber-500 font-medium">
                    ⚠️ {t("shifts.modal.break_policy_overtime_disabled_hint")}
                  </span>
                ) : (
                  t("shifts.modal.break_policy_overtime_help")
                )}
              </span>
            </div>
          </div>
        </div>
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
  isBreakOvertime,
  setIsBreakOvertime,
  newShiftGrace,
  setNewShiftGrace,
  allowEarlyOt,
  setAllowEarlyOt,
  maxEarlyOtMinutes,
  setMaxEarlyOtMinutes,
  onCreateShift,
  isLoading,
}: CreateShiftModalProps) {
  const { t } = useTranslation();

  const isSaveDisabled = isLoading || !newShiftName || !newShiftStart || !newShiftEnd || (allowEarlyOt && (!maxEarlyOtMinutes || maxEarlyOtMinutes <= 0));

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      subtitle={t("shifts.modal.setup_subtitle")}
      title={t("shifts.modal.setup_title")}
    >
      <div className="space-y-6 pt-4 max-h-[75vh] overflow-y-auto pr-2">
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
            isBreakOvertime={isBreakOvertime}
            isFixedBreak={isFixedBreak}
            newBreakEnd={newBreakEnd}
            newBreakStart={newBreakStart}
            newShiftBreak={newShiftBreak}
            setIsAutoDeduct={setIsAutoDeduct}
            setIsBreakOvertime={setIsBreakOvertime}
            setIsFixedBreak={setIsFixedBreak}
            setNewBreakEnd={setNewBreakEnd}
            setNewBreakStart={setNewBreakStart}
            setNewShiftBreak={setNewShiftBreak}
            t={t}
          />

          {/* Grace Period (Cho phép vào sớm) */}
          <div className="space-y-2 pt-1">
            <div className="flex items-center gap-2">
              <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("shifts.grace_period")}</label>
              <div className="group relative">
                <Info className="text-slate-300 dark:text-slate-600 cursor-help" size={12} />
                <div className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 w-48 p-3 bg-slate-800 text-white text-[10px] rounded-xl opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none z-50 normal-case leading-normal font-normal">
                  {t("shifts.grace_period_desc")}
                </div>
              </div>
            </div>
            <input
              className="w-full bg-slate-50 dark:bg-slate-800 border-none rounded-2xl px-5 py-3.5 font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 transition-all text-xs"
              min="0"
              onChange={(e) => { setNewShiftGrace(parseInt(e.target.value) || 0); }}
              type="number"
              value={newShiftGrace}
            />
          </div>

          {/* Early OT Config (Tăng ca sớm) */}
          <div className="space-y-4 bg-slate-50 dark:bg-slate-800 p-6 rounded-[32px] border border-slate-100 dark:border-slate-800">
            <div className="flex items-center justify-between">
              <div className="space-y-1">
                <label className="text-xs font-bold text-slate-700 dark:text-slate-200 block">
                  {t("shifts.modal.allow_early_ot", "Cho phép tính tăng ca sớm")}
                </label>
                <span className="text-[10px] text-slate-500 dark:text-slate-400 block font-normal leading-normal">
                  {t("shifts.modal.allow_early_ot_help", "Tính thời gian làm việc trước ca là tăng ca sớm (Early OT)")}
                </span>
              </div>
              <button
                aria-label={t("shifts.modal.allow_early_ot", "Cho phép tính tăng ca sớm")}
                className={`relative inline-flex h-5 w-9 items-center rounded-full transition-colors focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2 ${allowEarlyOt ? 'bg-primary-600' : 'bg-slate-200 dark:bg-slate-600'}`}
                onClick={() => { setAllowEarlyOt(!allowEarlyOt); }}
                type="button"
              >
                <span className={`inline-block h-3.5 w-3.5 transform rounded-full bg-white dark:bg-slate-900 transition-transform ${allowEarlyOt ? 'translate-x-5' : 'translate-x-1'}`} />
              </button>
            </div>

            {allowEarlyOt && (
              <div className="space-y-2 pt-3 border-t border-slate-200/50 dark:border-slate-700/50">
                <label htmlFor="create-max-early-ot" className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest block">
                  {t("shifts.modal.max_early_ot", "Giới hạn tăng ca sớm tối đa (phút)")} <span className="text-red-500">*</span>
                </label>
                <input
                  id="create-max-early-ot"
                  className="w-full bg-white dark:bg-slate-900 border-none rounded-xl px-4 py-2.5 font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-primary-500 transition-all text-xs"
                  min="1"
                  onChange={(e) => { setMaxEarlyOtMinutes(parseInt(e.target.value) || 0); }}
                  type="number"
                  value={maxEarlyOtMinutes}
                />
                {maxEarlyOtMinutes <= 0 && (
                  <span className="text-[10px] text-red-500 block font-medium">
                    {t("shifts.modal.max_early_ot_required", "Vui lòng nhập số phút giới hạn lớn hơn 0.")}
                  </span>
                )}
              </div>
            )}
          </div>
        </div>

        <div className="flex gap-3">
          <Button className="flex-1 px-6 py-4 rounded-2xl font-bold text-xs uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-700 transition-all"
            onClick={onClose}
            variant="ghost"
          >
            {t("common.cancel")}
          </Button>
          <Button className="flex-1 bg-primary-600 hover:bg-primary-700 text-white font-bold px-6 py-4 rounded-2xl flex items-center justify-center gap-2 shadow-lg shadow-primary-100 transition-all active:scale-95 disabled:opacity-50 text-xs uppercase tracking-widest"
            disabled={isSaveDisabled}
            onClick={onCreateShift}
            variant="primary"
          >
            <Save className="w-4 h-4" /> {t("shifts.modal.save_shift_btn")}
          </Button>
        </div>
      </div>
    </Modal>
  );
}
