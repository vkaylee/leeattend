import { Clock, Settings2 } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";

import {
  getBreakValidation,
  intervalAbsoluteMinutes,
  type IntervalDraft,
} from "../hooks/use-shift-editor";

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
  const [showAdvanced, setShowAdvanced] = useState(Boolean(iv.intervalName));
  const breakValidation = getBreakValidation(iv);
  const isInvalidFixedBreak = Boolean(breakValidation.key);
  const intervalRange = intervalAbsoluteMinutes(iv);

  return (
    <fieldset className="border border-slate-100 dark:border-slate-800 rounded-2xl p-5 space-y-4 relative">
      <legend className="sr-only">
        {t("work_segments.intervals.period_number", "Khoảng làm việc {{number}}", { number: index + 1 })}
      </legend>
      <div className="flex items-center justify-between gap-3">
        <span className="text-sm font-bold text-slate-700 dark:text-slate-200">
          {index === 0
            ? t("work_segments.intervals.primary_period", "Khoảng làm việc chính")
            : t("work_segments.intervals.period_number", "Khoảng làm việc {{number}}", { number: index + 1 })}
        </span>
        {showRemove ? (
          <Button
            aria-label={t("work_segments.intervals.remove_interval", "Xóa khoảng làm việc")}
            className="text-xs text-red-500 hover:text-red-700 min-h-11"
            onClick={() => { onRemove(iv.key); }}
            size="sm"
            type="button"
            variant="ghost"
          >
            {t("work_segments.intervals.remove_interval", "Xóa khoảng làm việc")}
          </Button>
        ) : null}
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
        <div>
          <label className="text-xs font-bold text-slate-500 dark:text-slate-400" htmlFor={`interval-start-${iv.key}`}>
            {t("work_segments.intervals.start_time", "Giờ bắt đầu")}
          </label>
          <input
            className="w-full mt-1 min-h-11 px-3 py-2 border border-slate-200 dark:border-slate-700 rounded-xl text-sm bg-slate-50 dark:bg-slate-800"
            id={`interval-start-${iv.key}`}
            onChange={(e) => {
              const nextStart = e.target.value;
              const nextEndDayOffset = iv.endDayOffset - iv.startDayOffset <= 1
                ? (nextStart > iv.endTime ? iv.startDayOffset + 1 : iv.startDayOffset)
                : iv.endDayOffset;
              onUpdate(iv.key, {
                endDayOffset: nextEndDayOffset,
                isNightShift: nextStart > iv.endTime,
                startTime: nextStart,
              });
            }}
            type="time"
            value={iv.startTime}
          />
        </div>
        <div>
          <label className="text-xs font-bold text-slate-500 dark:text-slate-400" htmlFor={`interval-end-${iv.key}`}>
            {t("work_segments.intervals.end_time", "Giờ kết thúc")}
          </label>
          <input
            className="w-full mt-1 min-h-11 px-3 py-2 border border-slate-200 dark:border-slate-700 rounded-xl text-sm bg-slate-50 dark:bg-slate-800"
            id={`interval-end-${iv.key}`}
            onChange={(e) => {
              const nextEnd = e.target.value;
              const nextEndDayOffset = iv.endDayOffset === iv.startDayOffset + 1 && iv.startTime <= nextEnd
                ? iv.startDayOffset
                : iv.endDayOffset;
              onUpdate(iv.key, {
                endDayOffset: nextEndDayOffset,
                endTime: nextEnd,
                isNightShift: iv.startTime > nextEnd,
              });
            }}
            type="time"
            value={iv.endTime}
          />
        </div>
      </div>

      {iv.startDayOffset < iv.endDayOffset || iv.startTime > iv.endTime ? (
        <div className="text-xs text-indigo-500 dark:text-indigo-400 bg-indigo-50 dark:bg-indigo-950/20 px-3 py-2 rounded-lg flex items-center gap-2" role="status">
          <Clock aria-hidden="true" size={14} />
          {t("work_segments.night_shift.description", "Shift crosses midnight — hours attributed to start day")}
        </div>
      ) : null}

      <BreakConfigSection interval={iv} onUpdate={onUpdate} />

      <div>
        <Button
          aria-expanded={showAdvanced}
          className="text-xs text-slate-500 dark:text-slate-400 px-0"
          onClick={() => { setShowAdvanced((current) => !current); }}
          type="button"
          variant="ghost"
        >
          <Settings2 aria-hidden="true" size={14} />
          {t("work_segments.intervals.advanced", "Tùy chọn nâng cao")}
        </Button>
        {showAdvanced ? (
          <div className="mt-3">
            <label className="text-xs font-bold text-slate-500 dark:text-slate-400" htmlFor={`interval-name-${iv.key}`}>
              {t("work_segments.intervals.interval_name", "Tên khoảng làm việc")} ({t("common.optional", "không bắt buộc")})
            </label>
            <input
              className="w-full mt-1 min-h-11 px-3 py-2 border border-slate-200 dark:border-slate-700 rounded-xl text-sm bg-slate-50 dark:bg-slate-800"
              id={`interval-name-${iv.key}`}
              onChange={(e) => { onUpdate(iv.key, { intervalName: e.target.value }); }}
              placeholder={t("work_segments.intervals.name_placeholder", "Ví dụ: Ca sáng")}
              type="text"
              value={iv.intervalName}
            />
          </div>
        ) : null}
      </div>

      {isInvalidFixedBreak ? (
        <p className="text-xs text-red-600 dark:text-red-400" role="alert">
          {t(`shifts.modal.${breakValidation.key}`, "Please check the work and break times.")}
        </p>
      ) : null}
      {intervalRange && intervalRange.end <= intervalRange.start ? (
        <p className="text-xs text-red-600 dark:text-red-400" role="alert">
          {t("shifts.messages.interval_order", "Work period end must be after its start.")}
        </p>
      ) : null}
    </fieldset>
  );
}

export default IntervalDraftEditor;
