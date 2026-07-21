import { Clock, Info } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";

import type { IntervalDraft } from "../hooks/use-shift-editor";

interface IntervalEditorCardProps {
  intervalDrafts: IntervalDraft[];
  onAddInterval: () => void;
  onRemoveInterval: (key: string) => void;
  onUpdateInterval: (key: string, patch: Partial<IntervalDraft>) => void;
}

export function IntervalEditorCard({
  intervalDrafts,
  onAddInterval,
  onRemoveInterval,
  onUpdateInterval,
}: IntervalEditorCardProps) {
  const { t } = useTranslation();

  return (
    <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-2xl sm:rounded-[2.5rem] p-5 sm:p-10 shadow-sm space-y-6">
      <div className="flex items-center justify-between border-b border-slate-50 dark:border-slate-800/80 pb-4">
        <Typography
          className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em]"
          variant="h3"
        >
          {t("work_segments.intervals.title", "Khung giờ trong ca").toUpperCase()}
        </Typography>
        <Button
          className="text-xs"
          onClick={onAddInterval}
          size="sm"
          type="button"
          variant="outline"
        >
          + {t("work_segments.intervals.add_interval", "Thêm khung giờ")}
        </Button>
      </div>

      {intervalDrafts.length === 0 ? (
        <div className="text-center py-8 text-slate-400 dark:text-slate-500 text-sm italic">
          <Info className="mx-auto mb-2 opacity-40" size={24} />
          {t(
            "work_segments.intervals.no_intervals",
            "Chưa có khung giờ. Hệ thống dùng giờ bắt đầu/kết thúc của ca.",
          )}
        </div>
      ) : (
        <div className="space-y-6">
          {intervalDrafts.map((iv, idx) => (
            <IntervalDraftEditor
              index={idx}
              interval={iv}
              key={iv.key}
              onRemove={onRemoveInterval}
              onUpdate={onUpdateInterval}
              showRemove={intervalDrafts.length > 1}
            />
          ))}
        </div>
      )}
    </div>
  );
}

// ── Single interval editor ──────────────────────────────────────────────────

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

      {/* Grace period & Early OT */}
      <GraceAndOtSection interval={iv} onUpdate={onUpdate} />

      {/* OT Aggregation Mode */}
      <OtAggregationSection interval={iv} onUpdate={onUpdate} />
    </div>
  );
}

// ── Sub-sections ────────────────────────────────────────────────────────────

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

function GraceAndOtSection({
  interval: iv,
  onUpdate,
}: {
  interval: IntervalDraft;
  onUpdate: (key: string, patch: Partial<IntervalDraft>) => void;
  // eslint-disable-next-line max-lines
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

// ── Reusable toggle switch ──────────────────────────────────────────────────

function ToggleSwitch({
  checked,
  onChange,
}: {
  checked: boolean;
  onChange: (next: boolean) => void;
}) {
  return (
    <div
      aria-checked={checked}
      className={`relative inline-flex h-5 w-9 items-center rounded-full transition-colors focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2 cursor-pointer ${
        checked ? "bg-primary-600" : "bg-slate-200 dark:bg-slate-700"
      }`}
      onClick={() => { onChange(!checked); }}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          onChange(!checked);
        }
      }}
      role="switch"
      tabIndex={0}
    >
      <span
        className={`inline-block h-3.5 w-3.5 transform rounded-full bg-white dark:bg-slate-900 transition-transform ${
          checked ? "translate-x-5" : "translate-x-1"
        }`}
      />
    </div>
  );
}
