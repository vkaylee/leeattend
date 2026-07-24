import React from "react";
import { useTranslation } from "react-i18next";

import type { IntervalDraft } from "../hooks/use-shift-editor";
import ToggleSwitch from "./toggle-switch";

function GraceAndOtSection({
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
        {t("shifts.modal.ot_grace_config", "Chính sách đi muộn & Tăng ca")}
      </span>

      <div>
        <label className="text-[10px] font-bold text-slate-400 uppercase tracking-wider">
          {t("shifts.grace_period")}
        </label>
        <input
          className="w-full mt-1 px-3 py-2 border border-slate-200 dark:border-slate-700 rounded-xl text-sm bg-slate-50 dark:bg-slate-800"
          min="0"
          onChange={(e) => {
            onUpdate(iv.key, { gracePeriodMinutes: parseInt(e.target.value) || 0 });
          }}
          type="number"
          value={iv.gracePeriodMinutes}
        />
      </div>

      <div className="flex items-center justify-between p-3 bg-slate-50 dark:bg-slate-800/40 rounded-xl">
        <div>
          <span className="text-xs font-bold text-slate-600 dark:text-slate-200 block">
            {t("shifts.modal.allow_early_ot", "Cho phép tính tăng ca sớm")}
          </span>
          <span className="text-[10px] text-slate-500 dark:text-slate-400">
            {t("shifts.modal.allow_early_ot_help", "Tính thời gian làm việc trước ca là tăng ca sớm")}
          </span>
        </div>
        <ToggleSwitch
          checked={iv.allowEarlyOt}
          onChange={(next) => { onUpdate(iv.key, { allowEarlyOt: next }); }}
        />
      </div>

      {iv.allowEarlyOt ? (
        <div>
          <label className="text-[10px] font-bold text-slate-400 uppercase tracking-wider">
            {t("shifts.modal.max_early_ot", "Giới hạn tăng ca sớm tối đa (phút)")}
          </label>
          <input
            className="w-full mt-1 px-3 py-2 border border-slate-200 dark:border-slate-700 rounded-xl text-sm bg-slate-50 dark:bg-slate-800"
            min="1"
            onChange={(e) => {
              onUpdate(iv.key, {
                maxEarlyOtMinutes: parseInt(e.target.value) || null,
              });
            }}
            type="number"
            value={iv.maxEarlyOtMinutes ?? ""}
          />
        </div>
      ) : null}
    </div>
  );
}

export default GraceAndOtSection;
