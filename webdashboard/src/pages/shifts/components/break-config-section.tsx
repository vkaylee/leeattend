import type { TFunction } from "i18next";
import React from "react";
import { useTranslation } from "react-i18next";

import {
  getBreakValidation,
  getIntervalPreview,
  type IntervalDraft,
} from "../hooks/use-shift-editor";

function ChoiceCard({
  checked,
  children,
  htmlFor,
  name,
  onChange,
}: {
  checked: boolean;
  children: React.ReactNode;
  htmlFor: string;
  name: string;
  onChange: () => void;
}) {
  return (
    <label
      className={`flex items-start gap-3 p-3 min-h-11 rounded-xl border cursor-pointer transition-colors focus-within:ring-2 focus-within:ring-primary-500 ${
        checked
          ? "border-primary-500 bg-primary-50/50 dark:border-primary-400 dark:bg-primary-950/20"
          : "border-slate-200 hover:bg-slate-50 dark:border-slate-700 dark:hover:bg-slate-800/60"
      }`}
      htmlFor={htmlFor}
    >
      <input checked={checked} className="mt-1" id={htmlFor} name={name} onChange={onChange} type="radio" />
      <span>{children}</span>
    </label>
  );
}

function BreakConfigSection({
  interval: iv,
  onUpdate,
}: {
  interval: IntervalDraft;
  onUpdate: (key: string, patch: Partial<IntervalDraft>) => void;
}) {
  const { t } = useTranslation();
  const modeId = `break-mode-${iv.key}`;
  const validation = getBreakValidation(iv);
  const preview = getIntervalPreview(iv);
  const hasWindow = Boolean(iv.breakStart || iv.breakEnd || iv.isFixedBreak);
  const showBreakWindow = iv.breakPunchMode === "required" ? hasWindow : iv.isFixedBreak;

  const update = (patch: Partial<IntervalDraft>) => onUpdate(iv.key, patch);
  const validationText = validation.key
    ? t(`shifts.modal.${validation.key}`)
    : null;

  return (
    <fieldset className="border-t border-slate-100 dark:border-slate-800 pt-4 space-y-4">
      <legend className="text-sm font-bold text-slate-700 dark:text-slate-200">
        {t("shifts.modal.break_config", "Break Time")}
      </legend>
      <p className="text-xs text-slate-500 dark:text-slate-400">
        {t("shifts.modal.break_config_help", "Choose how break time is captured within this work period. Break deduction and break overtime rules are configured in Work Policy.")}
      </p>

      <fieldset className="space-y-3">
        <legend className="text-sm font-semibold text-slate-700 dark:text-slate-200">
          {t("shifts.modal.break_step_record", "How do employees record breaks?")}
        </legend>
        <div aria-label={t("shifts.modal.break_capture_label", "Break punch capture")} className="grid grid-cols-1 sm:grid-cols-2 gap-3" role="radiogroup">
          <ChoiceCard
            checked={iv.breakPunchMode === "none"}
            htmlFor={`${modeId}-no-punch`}
            name={`${modeId}-capture`}
            onChange={() => update({
              breakPunchMode: "none",
              isFixedBreak: false,
              breakStart: "",
              breakEnd: "",
              breakDuration: 60,
            })}
          >
            <span className="block text-sm font-bold text-slate-700 dark:text-slate-200">
              {t("shifts.modal.break_capture_none", "No break punches")}
            </span>
            <span className="block text-xs text-slate-500 dark:text-slate-400">
              {t("shifts.modal.break_capture_none_help", "Employees only punch in and out — 2 punches per day.")}
            </span>
          </ChoiceCard>
          <ChoiceCard
            checked={iv.breakPunchMode === "required"}
            htmlFor={`${modeId}-required`}
            name={`${modeId}-capture`}
            onChange={() => update({
              breakPunchMode: "required",
              isFixedBreak: Boolean(iv.breakStart || iv.breakEnd),
              breakStart: iv.breakStart,
              breakEnd: iv.breakEnd,
            })}
          >
            <span className="block text-sm font-bold text-slate-700 dark:text-slate-200">
              {t("shifts.modal.break_capture_required", "Break punches required")}
            </span>
            <span className="block text-xs text-slate-500 dark:text-slate-400">
              {t("shifts.modal.break_capture_required_help", "Employees punch break start and end — 4 punches per day.")}
            </span>
          </ChoiceCard>
        </div>
      </fieldset>

      {iv.breakPunchMode === "none" ? (
        <div className="space-y-4 rounded-xl bg-slate-50/70 dark:bg-slate-800/40 p-3">
          <fieldset className="space-y-3">
            <legend className="text-sm font-semibold text-slate-700 dark:text-slate-200">
              {t("shifts.modal.break_schedule_label", "How is the scheduled break defined?")}
            </legend>
            <div aria-label={t("shifts.modal.break_mode", "Break configuration")} className="grid grid-cols-1 sm:grid-cols-2 gap-3" role="radiogroup">
              <ChoiceCard
                checked={iv.isFixedBreak}
                htmlFor={`${modeId}-fixed`}
                name={`${modeId}-schedule`}
                onChange={() => update({ isFixedBreak: true })}
              >
                <span className="block text-sm font-bold text-slate-700 dark:text-slate-200">
                  {t("shifts.modal.break_mode_fixed", "Fixed time")}
                </span>
                <span className="block text-xs text-slate-500 dark:text-slate-400">
                  {t("shifts.modal.break_mode_fixed_help", "Example: 12:00–13:00.")}
                </span>
              </ChoiceCard>
              <ChoiceCard
                checked={!iv.isFixedBreak}
                htmlFor={`${modeId}-duration`}
                name={`${modeId}-schedule`}
                onChange={() => update({ isFixedBreak: false, breakStart: "", breakEnd: "" })}
              >
                <span className="block text-sm font-bold text-slate-700 dark:text-slate-200">
                  {t("shifts.modal.break_mode_duration", "By duration")}
                </span>
                <span className="block text-xs text-slate-500 dark:text-slate-400">
                  {t("shifts.modal.break_mode_duration_help", "Example: 60 minutes.")}
                </span>
              </ChoiceCard>
            </div>
          </fieldset>

          {iv.isFixedBreak ? (
            <BreakWindowFields interval={iv} onUpdate={update} t={t} />
          ) : (
            <div>
              <label className="text-xs font-bold text-slate-500 dark:text-slate-400" htmlFor={`break-duration-${iv.key}`}>
                {t("shifts.modal.break_duration", "Break duration")} ({t("common.minutes", "minutes")})
              </label>
              <input
                aria-describedby={validationText ? `break-error-${iv.key}` : undefined}
                className="w-full mt-1 min-h-11 px-3 py-2 border border-slate-200 dark:border-slate-700 rounded-xl text-sm bg-white dark:bg-slate-900 focus-visible:ring-2 focus-visible:ring-primary-500"
                id={`break-duration-${iv.key}`}
                min={0}
                onChange={(event) => update({ breakDuration: Math.max(0, Number(event.target.value) || 0) })}
                type="number"
                value={iv.breakDuration}
              />
            </div>
          )}
        </div>
      ) : (
        <div className="space-y-4 rounded-xl bg-slate-50/70 dark:bg-slate-800/40 p-3">
          <p className="text-sm text-slate-600 dark:text-slate-300" role="status">
            {t("shifts.modal.break_actual_help", "Break time is deducted from the actual break punches. There is no automatic scheduled deduction.")}
          </p>
          <div className="flex items-start gap-3 text-sm text-slate-700 dark:text-slate-200">
            <input
              checked={hasWindow}
              className="mt-1 h-4 w-4 rounded border-slate-300 text-primary-600 focus:ring-primary-500"
              id={`${modeId}-allowed-window`}
              onChange={(event) => update(event.target.checked
                ? { isFixedBreak: true }
                : { isFixedBreak: false, breakStart: "", breakEnd: "" })}
              type="checkbox"
            />
            <span>
              <span className="block font-semibold">{t("shifts.modal.break_allowed_window", "Set an allowed break window")}</span>
              <span className="block text-xs text-slate-500 dark:text-slate-400">
                {t("shifts.modal.break_allowed_window_help", "Used to warn when actual break punches fall outside this window. It does not deduct scheduled time.")}
              </span>
            </span>
          </div>
          {showBreakWindow ? <BreakWindowFields interval={iv} onUpdate={update} t={t} /> : null}
        </div>
      )}

      {validationText ? (
        <p className="text-xs text-red-600 dark:text-red-400" id={`break-error-${iv.key}`} role="alert">
          {validationText}
        </p>
      ) : null}
      <p className="rounded-xl border border-primary-100 bg-primary-50/60 px-3 py-2 text-sm text-primary-800 dark:border-primary-900 dark:bg-primary-950/30 dark:text-primary-200" data-testid={`break-preview-${iv.key}`}>
        <span className="font-semibold">{t("shifts.modal.break_preview_label", "Preview")}: </span>
        {formatPreview(iv, preview, t)}
      </p>
    </fieldset>
  );
}

function BreakWindowFields({
  interval: iv,
  onUpdate,
  t,
}: {
  interval: IntervalDraft;
  onUpdate: (patch: Partial<IntervalDraft>) => void;
  t: TFunction;
}) {
  return (
    <div>
      <p className="text-xs font-semibold text-slate-600 dark:text-slate-300">
        {t("shifts.modal.break_window_label", "Break window")}
      </p>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 mt-2">
        <TimeField id={`break-start-${iv.key}`} label={t("shifts.modal.break_start", "Break Start")} value={iv.breakStart} onChange={(breakStart) => onUpdate({ breakStart })} />
        <TimeField id={`break-end-${iv.key}`} label={t("shifts.modal.break_end", "Break End")} value={iv.breakEnd} onChange={(breakEnd) => onUpdate({ breakEnd })} />
      </div>
      <p className="mt-1 text-xs text-slate-500 dark:text-slate-400">
        {t("shifts.modal.break_window_hint", "Use 24-hour time within this work period.")}
      </p>
    </div>
  );
}

function TimeField({ id, label, value, onChange }: { id: string; label: string; value: string; onChange: (value: string) => void }) {
  return (
    <div>
      <label className="text-xs font-bold text-slate-500 dark:text-slate-400" htmlFor={id}>{label}</label>
      <input
        aria-label={label}
        className="w-full mt-1 min-h-11 px-3 py-2 border border-slate-200 dark:border-slate-700 rounded-xl text-sm bg-white dark:bg-slate-900 focus-visible:ring-2 focus-visible:ring-primary-500"
        id={id}
        onChange={(event) => onChange(event.target.value)}
        type="time"
        value={value}
      />
    </div>
  );
}

function formatPreview(
  interval: IntervalDraft,
  preview: ReturnType<typeof getIntervalPreview>,
  t: TFunction,
) {
  const range = `${interval.startTime || "--:--"}–${interval.endTime || "--:--"}`;
  if (!preview) return t("shifts.modal.break_preview_invalid", "Complete valid work and break times to see the result.");
  if (preview.breakLabel === "actual") return `${range} · ${t("shifts.modal.break_preview_actual", "actual break time")}`;
  const breakMinutes = preview.breakMinutes ?? 0;
  const hours = Math.floor(preview.workMinutes / 60);
  const minutes = preview.workMinutes % 60;
  const duration = `${hours}h${minutes.toString().padStart(2, "0")}`;
  const breakLabel = t("shifts.modal.break_preview_deducted", "break {{minutes}} min", { minutes: breakMinutes });
  return `${range} · ${breakLabel} → ${duration} ${t("shifts.modal.break_preview_working", "working hours")}`;
}

export default BreakConfigSection;
