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
              {t("shifts.modal.break_mode_duration_help", "Ví dụ: tự động trừ 60 phút. Nhân viên thường chỉ cần chấm công 2 lần/ngày.")}
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
