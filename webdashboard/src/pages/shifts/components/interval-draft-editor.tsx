import { Clock } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";

import type { IntervalDraft } from "../hooks/use-shift-editor";

import BreakConfigSection from "./break-config-section";

interface IntervalDraftEditorProps {
  index: number;
  interval: IntervalDraft;
  onRemove: (key: string) => void;
  onUpdate: (key: string, patch: Partial<IntervalDraft>) => void;
  showRemove: boolean;
}

function IntervalDraftEditor({
  index,
  interval: iv,
  showRemove,
  onRemove,
  onUpdate,
}: IntervalDraftEditorProps) {
  const { t } = useTranslation();

  return (
    <div className="border border-slate-100 dark:border-slate-800 rounded-2xl p-5 space-y-4 relative">
      {/* Header */}
      <div className="flex items-center justify-between">
        <span className="text-xs font-bold text-slate-500 dark:text-slate-400">
          {t("work_segments.intervals.interval", "Khung giờ")} {index + 1}
        </span>
        {showRemove ? (
          <Button
            className="text-xs text-red-500 hover:text-red-700"
            onClick={() => { onRemove(iv.key); }}
            size="sm"
            type="button"
            variant="ghost"
          >
            ✕ {t("work_segments.intervals.remove_interval", "Xóa")}
          </Button>
        ) : null}
      </div>

      {/* Interval name */}
      <div>
        <label className="text-[10px] font-bold text-slate-400 uppercase tracking-wider">
          {t("work_segments.intervals.interval_name", "Tên khung giờ")}
        </label>
        <input
          className="w-full mt-1 px-3 py-2 border border-slate-200 dark:border-slate-700 rounded-xl text-sm bg-slate-50 dark:bg-slate-800"
          onChange={(e) => { onUpdate(iv.key, { intervalName: e.target.value }); }}
          placeholder={index === 0 ? "VD: Ca sáng, Ca chính..." : "VD: Ca chiều, Tăng ca..."}
          type="text"
          value={iv.intervalName}
        />
      </div>

      {/* Time range */}
      <div className="grid grid-cols-2 gap-3">
        <div>
          <label className="text-[10px] font-bold text-slate-400 uppercase tracking-wider">
            {t("work_segments.intervals.start_time", "Bắt đầu")}
          </label>
          <input
            className="w-full mt-1 px-3 py-2 border border-slate-200 dark:border-slate-700 rounded-xl text-sm bg-slate-50 dark:bg-slate-800"
            onChange={(e) => {
              onUpdate(iv.key, {
                startTime: e.target.value,
                isNightShift: e.target.value > iv.endTime,
              });
            }}
            type="time"
            value={iv.startTime}
          />
        </div>
        <div>
          <label className="text-[10px] font-bold text-slate-400 uppercase tracking-wider">
            {t("work_segments.intervals.end_time", "Kết thúc")}
          </label>
          <input
            className="w-full mt-1 px-3 py-2 border border-slate-200 dark:border-slate-700 rounded-xl text-sm bg-slate-50 dark:bg-slate-800"
            onChange={(e) => {
              onUpdate(iv.key, {
                endTime: e.target.value,
                isNightShift: iv.startTime > e.target.value,
              });
            }}
            type="time"
            value={iv.endTime}
          />
        </div>
      </div>

      {/* Night shift indicator */}
      {iv.startTime > iv.endTime ? (
        <div className="text-[10px] text-indigo-500 dark:text-indigo-400 bg-indigo-50 dark:bg-indigo-950/20 px-3 py-1.5 rounded-lg flex items-center gap-2">
          <Clock size={12} />
          {t("work_segments.night_shift.label", "Ca đêm")} —{" "}
          {t("work_segments.night_shift.description", "Giờ làm qua nửa đêm")}
        </div>
      ) : null}

      {/* Break Config */}
      <BreakConfigSection interval={iv} onUpdate={onUpdate} />

    </div>
  );
}

export default IntervalDraftEditor;
