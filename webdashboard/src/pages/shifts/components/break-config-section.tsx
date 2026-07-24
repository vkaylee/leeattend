import React from "react";
import { useTranslation } from "react-i18next";

import type { IntervalDraft } from "../hooks/use-shift-editor";
import ToggleSwitch from "./toggle-switch";

function BreakConfigSection({
  interval: iv,
  onUpdate,
}: {
  interval: IntervalDraft;
  onUpdate: (key: string, patch: Partial<IntervalDraft>) => void;
}) {
  const { t } = useTranslation();

  return (
    <div className="border-t border-slate-100 dark:border-slate-800 pt-3 space-y-3">
      <span className="text-[10px] font-bold text-slate-400 uppercase tracking-wider">
        {t("shifts.modal.break_config", "Thiết lập giờ nghỉ")}
      </span>

      {/* Fixed Break Toggle */}
      <div className="flex items-center justify-between p-3 bg-slate-50 dark:bg-slate-800/40 rounded-xl">
        <span className="text-xs font-bold text-slate-600 dark:text-slate-350">
          {t("shifts.types.fixed", "Giờ nghỉ cố định")}
        </span>
        <ToggleSwitch
          checked={iv.isFixedBreak}
          onChange={(next) => {
            onUpdate(iv.key, {
              isFixedBreak: next,
              isBreakOvertime: next ? iv.isBreakOvertime : false,
              isAutoDeduct: next ? iv.isAutoDeduct : true,
            });
          }}
        />
      </div>

      {iv.isFixedBreak ? (
        <div className="grid grid-cols-2 gap-3 animate-in fade-in duration-350">
          <div>
            <label className="text-[10px] font-bold text-slate-400 uppercase">
              {t("shifts.modal.break_start")}
            </label>
            <input
              className="w-full mt-1 px-3 py-2 border border-slate-200 dark:border-slate-700 rounded-xl text-sm bg-slate-50 dark:bg-slate-800"
              onChange={(e) => { onUpdate(iv.key, { breakStart: e.target.value }); }}
              type="time"
              value={iv.breakStart}
            />
          </div>
          <div>
            <label className="text-[10px] font-bold text-slate-400 uppercase">
              {t("shifts.modal.break_end")}
            </label>
            <input
              className="w-full mt-1 px-3 py-2 border border-slate-200 dark:border-slate-700 rounded-xl text-sm bg-slate-50 dark:bg-slate-800"
              onChange={(e) => { onUpdate(iv.key, { breakEnd: e.target.value }); }}
              type="time"
              value={iv.breakEnd}
            />
          </div>
        </div>
      ) : (
        <div className="animate-in fade-in duration-350">
          <label className="text-[10px] font-bold text-slate-400 uppercase block">
            {t("shifts.modal.break_duration")} ({t("common.minutes")})
          </label>
          <input
            className="w-full mt-1 px-3 py-2 border border-slate-200 dark:border-slate-700 rounded-xl text-sm bg-slate-50 dark:bg-slate-800"
            onChange={(e) => {
              onUpdate(iv.key, { breakDuration: parseInt(e.target.value) || 0 });
            }}
            type="number"
            value={iv.breakDuration}
          />
        </div>
      )}

      {/* Break Policy */}
      <div className="space-y-2">
        <span className="text-[10px] font-bold text-slate-400 uppercase tracking-wider block">
          {t("shifts.modal.break_policy_label")}
        </span>
        {(
          [
            {
              val: { isAutoDeduct: true, isBreakOvertime: false },
              label: t("shifts.modal.break_policy_auto_deduct"),
              help: t("shifts.modal.break_policy_auto_deduct_help"),
            },
            {
              val: { isAutoDeduct: false, isBreakOvertime: false },
              label: t("shifts.modal.break_policy_no_deduct"),
              help: t("shifts.modal.break_policy_no_deduct_help"),
            },
            {
              val: { isAutoDeduct: false, isBreakOvertime: true },
              label: t("shifts.modal.break_policy_overtime"),
              help: iv.isFixedBreak
                ? t("shifts.modal.break_policy_overtime_help")
                : t("shifts.modal.break_policy_overtime_disabled_hint"),
              disabled: !iv.isFixedBreak,
            },
          ] as const
        ).map((opt, oi) => (
          <label
            aria-label={`${opt.label} ${opt.help}`}
            className={`flex items-start gap-3 p-3 rounded-xl border transition-all ${
              "disabled" in opt && opt.disabled
                ? "bg-slate-100/50 dark:bg-slate-900/20 border-slate-200/50 opacity-60 cursor-not-allowed"
                : "bg-slate-50 dark:bg-slate-800/40 border-slate-100 dark:border-slate-800/80 cursor-pointer hover:bg-slate-100/50"
            }`}
            htmlFor={`break_policy_${iv.key}_${oi}`}
            key={oi}
          >
            <input
              checked={
                iv.isAutoDeduct === opt.val.isAutoDeduct &&
                iv.isBreakOvertime === opt.val.isBreakOvertime
              }
              className="mt-0.5 w-4 h-4 text-primary-600 cursor-pointer"
              disabled={"disabled" in opt && opt.disabled}
              id={`break_policy_${iv.key}_${oi}`}
              name={`break_policy_${iv.key}`}
              onChange={() => { onUpdate(iv.key, opt.val); }}
              type="radio"
            />
            <div>
              <span className="text-xs font-bold text-slate-700 dark:text-slate-200 block">
                {opt.label}
              </span>
              <span className="text-[10px] text-slate-500 dark:text-slate-400 block">
                {opt.help}
              </span>
            </div>
          </label>
        ))}
      </div>
    </div>
  );
}

export default BreakConfigSection;
