import React from "react";
import { useTranslation } from "react-i18next";

import type { IntervalDraft } from "../hooks/use-shift-editor";

function OtAggregationSection({
  interval: iv,
  onUpdate,
}: {
  interval: IntervalDraft;
  onUpdate: (key: string, patch: Partial<IntervalDraft>) => void;
}) {
  const { t } = useTranslation();

  return (
    <div className="border-t border-slate-100 dark:border-slate-800 pt-3 space-y-2">
      <span className="text-[10px] font-bold text-slate-400 uppercase tracking-wider">
        {t("work_segments.ot_aggregation_mode.label", "Cách tính tăng ca")}
      </span>
      {(
        [
          {
            val: "aggregate" as const,
            label: t("work_segments.ot_aggregation_mode.aggregate", "Gộp (Tổng cả ngày)"),
          },
          {
            val: "per_segment" as const,
            label: t("work_segments.ot_aggregation_mode.per_segment", "Theo từng phiên"),
          },
        ] as const
      ).map((opt) => (
        <label
          className="flex items-start gap-3 p-3 rounded-xl border cursor-pointer hover:bg-slate-50 dark:hover:bg-slate-800/40 transition-colors"
          key={opt.val}
        >
          <input
            checked={iv.otAggregationMode === opt.val}
            className="mt-0.5"
            name={`otMode_${iv.key}`}
            onChange={() => { onUpdate(iv.key, { otAggregationMode: opt.val }); }}
            type="radio"
          />
          <span className="text-sm font-bold text-slate-700 dark:text-slate-250">
            {opt.label}
          </span>
        </label>
      ))}
    </div>
  );
}

export default OtAggregationSection;
