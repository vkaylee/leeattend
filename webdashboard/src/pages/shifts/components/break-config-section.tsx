import React from "react";
import { useTranslation } from "react-i18next";

import type { IntervalDraft } from "../hooks/use-shift-editor";

function BreakConfigSection({
  interval: iv,
  onUpdate,
}: {
  interval: IntervalDraft;
  onUpdate: (key: string, patch: Partial<IntervalDraft>) => void;
}) {
  const { t } = useTranslation();
  const modeId = `break-mode-${iv.key}`;

  return (
    <fieldset className="border-t border-slate-100 dark:border-slate-800 pt-4 space-y-3">
      <legend className="text-sm font-bold text-slate-700 dark:text-slate-200">
        {t("shifts.modal.break_config", "Nghỉ giữa ca")}
      </legend>
      <div className="text-xs text-slate-500 dark:text-slate-400">
        {t("shifts.modal.break_config_help", "Chọn cách hệ thống tính giờ nghỉ trong khoảng làm việc này.")}
      </div>

      <div aria-label={t("shifts.modal.break_capture_label", "Cách ghi nhận giờ nghỉ")} className="grid grid-cols-1 sm:grid-cols-2 gap-3" role="radiogroup">
        <label aria-label={t("shifts.modal.break_capture_none", "Không cần chấm công nghỉ")} className="flex items-start gap-3 p-3 min-h-11 rounded-xl border border-slate-200 dark:border-slate-700 cursor-pointer" htmlFor={`${modeId}-no-punch`}>
          <input checked={iv.breakPunchMode === "none"} className="mt-1" id={`${modeId}-no-punch`} name={`${modeId}-capture`} onChange={() => { onUpdate(iv.key, { breakPunchMode: "none", breakCreditMode: iv.isAutoDeduct ? "scheduled" : "none" }); }} type="radio" />
          <span><span className="block text-sm font-bold text-slate-700 dark:text-slate-200">{t("shifts.modal.break_capture_none", "Không cần chấm công nghỉ")}</span><span className="block text-xs text-slate-500 dark:text-slate-400">{t("shifts.modal.break_capture_none_help", "Hệ thống tính theo chính sách giờ nghỉ.")}</span></span>
        </label>
        <label aria-label={t("shifts.modal.break_capture_required", "Bắt buộc chấm công nghỉ")} className="flex items-start gap-3 p-3 min-h-11 rounded-xl border border-slate-200 dark:border-slate-700 cursor-pointer" htmlFor={`${modeId}-required`}>
          <input checked={iv.breakPunchMode === "required"} className="mt-1" id={`${modeId}-required`} name={`${modeId}-capture`} onChange={() => { onUpdate(iv.key, { breakPunchMode: "required", breakCreditMode: "actual", isFixedBreak: true }); }} type="radio" />
          <span><span className="block text-sm font-bold text-slate-700 dark:text-slate-200">{t("shifts.modal.break_capture_required", "Bắt buộc chấm công nghỉ")}</span><span className="block text-xs text-slate-500 dark:text-slate-400">{t("shifts.modal.break_capture_required_help", "Nhân viên bấm bắt đầu và kết thúc giờ nghỉ.")}</span></span>
        </label>
      </div>

      <div aria-label={t("shifts.modal.break_policy_label", "Chính sách giờ nghỉ")} className="grid grid-cols-1 sm:grid-cols-2 gap-3" role="radiogroup">
        <label aria-label={t("shifts.modal.break_policy_auto_deduct", "Tự động trừ giờ nghỉ")} className="flex items-start gap-3 p-3 min-h-11 rounded-xl border border-slate-200 dark:border-slate-700 cursor-pointer hover:bg-slate-50 dark:hover:bg-slate-800/60" htmlFor={`${modeId}-auto-deduct`}>
          <input
            checked={iv.isAutoDeduct}
            className="mt-1"
            id={`${modeId}-auto-deduct`}
            name={`${modeId}-policy`}
            onChange={() => { onUpdate(iv.key, { isAutoDeduct: true }); }}
            type="radio"
          />
          <span>
            <span className="block text-sm font-bold text-slate-700 dark:text-slate-200">{t("shifts.modal.break_policy_auto_deduct", "Tự động trừ giờ nghỉ")}</span>
            <span className="block text-xs text-slate-500 dark:text-slate-400">{t("shifts.modal.break_policy_auto_deduct_help", "Không cần chấm công khi bắt đầu hoặc kết thúc nghỉ.")}</span>
          </span>
        </label>
        <label aria-label={t("shifts.modal.break_policy_no_deduct", "Không tự động trừ")} className="flex items-start gap-3 p-3 min-h-11 rounded-xl border border-slate-200 dark:border-slate-700 cursor-pointer hover:bg-slate-50 dark:hover:bg-slate-800/60" htmlFor={`${modeId}-paid-break`}>
          <input
            checked={!iv.isAutoDeduct}
            className="mt-1"
            id={`${modeId}-paid-break`}
            name={`${modeId}-policy`}
            onChange={() => { onUpdate(iv.key, { isAutoDeduct: false }); }}
            type="radio"
          />
          <span>
            <span className="block text-sm font-bold text-slate-700 dark:text-slate-200">{t("shifts.modal.break_policy_no_deduct", "Không tự động trừ")}</span>
            <span className="block text-xs text-slate-500 dark:text-slate-400">{t("shifts.modal.break_policy_no_deduct_help", "Khoảng nghỉ vẫn được tính vào thời gian hiện diện.")}</span>
          </span>
        </label>
      </div>

      <div aria-label={t("shifts.modal.break_mode", "Cách cấu hình giờ nghỉ")} className="grid grid-cols-1 sm:grid-cols-2 gap-3" role="radiogroup">
        <label aria-label={t("shifts.modal.break_mode_duration", "Theo thời lượng")} className="flex items-start gap-3 p-3 min-h-11 rounded-xl border border-slate-200 dark:border-slate-700 cursor-pointer hover:bg-slate-50 dark:hover:bg-slate-800/60" htmlFor={`${modeId}-duration`}>
          <input
            checked={!iv.isFixedBreak}
            className="mt-1"
            id={`${modeId}-duration`}
            name={modeId}
            onChange={() => { onUpdate(iv.key, { isFixedBreak: false, breakStart: "", breakEnd: "" }); }}
            type="radio"
          />
          <span>
            <span className="block text-sm font-bold text-slate-700 dark:text-slate-200">
              {t("shifts.modal.break_mode_duration", "Theo thời lượng")}
            </span>
            <span className="block text-xs text-slate-500 dark:text-slate-400">
              {t("shifts.modal.break_mode_duration_help", "Ví dụ: 60 phút. Cách trừ giờ nghỉ tuân theo chính sách phía trên.")}
            </span>
          </span>
        </label>
        <label aria-label={t("shifts.modal.break_mode_fixed", "Theo giờ cố định")} className="flex items-start gap-3 p-3 min-h-11 rounded-xl border border-slate-200 dark:border-slate-700 cursor-pointer hover:bg-slate-50 dark:hover:bg-slate-800/60" htmlFor={`${modeId}-fixed`}>

          <input
            checked={iv.isFixedBreak}
            className="mt-1"
            id={`${modeId}-fixed`}
            name={modeId}
            onChange={() => { onUpdate(iv.key, { isFixedBreak: true }); }}
            type="radio"
          />
          <span>
            <span className="block text-sm font-bold text-slate-700 dark:text-slate-200">
              {t("shifts.modal.break_mode_fixed", "Theo giờ cố định")}
            </span>
            <span className="block text-xs text-slate-500 dark:text-slate-400">
              {t("shifts.modal.break_mode_fixed_help", "Ví dụ: nghỉ từ 12:00 đến 13:00.")}
            </span>
          </span>
        </label>
      </div>

      {iv.isFixedBreak ? (
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <div>
            <label className="text-xs font-bold text-slate-500 dark:text-slate-400" htmlFor={`break-start-${iv.key}`}>
              <span>{t("shifts.modal.break_start", "Bắt đầu nghỉ")}</span>
            </label>
            <input
              aria-label={t("shifts.modal.break_start", "Bắt đầu nghỉ")}
              className="w-full mt-1 min-h-11 px-3 py-2 border border-slate-200 dark:border-slate-700 rounded-xl text-sm bg-slate-50 dark:bg-slate-800"
              id={`break-start-${iv.key}`}
              onChange={(e) => { onUpdate(iv.key, { breakStart: e.target.value }); }}
              required
              type="time"
              value={iv.breakStart}
            />
          </div>
          <div>
            <label className="text-xs font-bold text-slate-500 dark:text-slate-400" htmlFor={`break-end-${iv.key}`}>
              <span>{t("shifts.modal.break_end", "Kết thúc nghỉ")}</span>
            </label>
            <input
              aria-label={t("shifts.modal.break_end", "Kết thúc nghỉ")}
              className="w-full mt-1 min-h-11 px-3 py-2 border border-slate-200 dark:border-slate-700 rounded-xl text-sm bg-slate-50 dark:bg-slate-800"
              id={`break-end-${iv.key}`}
              onChange={(e) => { onUpdate(iv.key, { breakEnd: e.target.value }); }}
              required
              type="time"
              value={iv.breakEnd}
            />
          </div>
        </div>
      ) : (
        <div>
          <label className="text-xs font-bold text-slate-500 dark:text-slate-400" htmlFor={`break-duration-${iv.key}`}>
            {t("shifts.modal.break_duration", "Thời lượng nghỉ")} ({t("common.minutes", "phút")})
          </label>
          <input
            className="w-full mt-1 min-h-11 px-3 py-2 border border-slate-200 dark:border-slate-700 rounded-xl text-sm bg-slate-50 dark:bg-slate-800"
            id={`break-duration-${iv.key}`}
            min={0}
            onChange={(e) => { onUpdate(iv.key, { breakDuration: Math.max(0, Number(e.target.value) || 0) }); }}
            type="number"
            value={iv.breakDuration}
          />
        </div>
      )}
    </fieldset>
  );
}

export default BreakConfigSection;
