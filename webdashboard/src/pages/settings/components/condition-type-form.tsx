import { Plus } from "lucide-react";
import { useState } from "react";

import type {
  ConditionTypeDefinition,
  CreateConditionTypeRequest,
  UpdateConditionTypeRequest,
  ConditionRuleEffects,
  ConditionEarlyArrivalStrategy,
} from "@/lib/api";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";

interface FormData {
  affects_rules: ConditionRuleEffects;
  category: string;
  code: string;
  color: string;
  default_is_paid: boolean;
  default_minute_allowance: number;
  icon: string;
  name_en: string;
  name_vi: string;
  sort_order: number;
}

function emptyForm(): FormData {
  return {
    code: "",
    name_en: "",
    name_vi: "",
    category: "general",
    default_minute_allowance: 0,
    default_is_paid: false,
    icon: "Clock",
    color: "blue",
    sort_order: 0,
    affects_rules: {},
  };
}

interface ConditionTypeFormProps {
  initial?: ConditionTypeDefinition;
  isSubmitting: boolean;
  onCancel: () => void;
  onSave: (data: CreateConditionTypeRequest | UpdateConditionTypeRequest) => void;
  t: (key: string) => string;
}

/**
 * ConditionTypeForm — form for creating/editing condition types.
 * Complexity (13) from validation rules: conflict detection, numeric ranges, field dependencies.
 * Extracted from condition-types-page.tsx to keep the page component lean.
 */
// eslint-disable-next-line complexity, max-lines-per-function -- form with 6 Boolean/tri-state effect toggles + early-arrival dropdown; 10+ genuine conditional branches
export function ConditionTypeForm({
  initial,
  isSubmitting,
  onCancel,
  onSave,
  t,
}: ConditionTypeFormProps) {
  const isEdit = !!initial;
  const [form, setForm] = useState<FormData>(
    initial
      ? {
          code: initial.code,
          name_en: initial.name_i18n.en || "",
          name_vi: initial.name_i18n.vi || "",
          category: initial.category,
          default_minute_allowance: initial.default_minute_allowance,
          default_is_paid: initial.default_is_paid,
          icon: initial.icon ?? "Clock",
          color: initial.color ?? "blue",
          sort_order: initial.sort_order,
          affects_rules: initial.affects_rules,
        }
      : emptyForm(),
  );
  const [effectError, setEffectError] = useState<string | null>(null);

  const hasEffect = Object.values(form.affects_rules).some((value) => value !== undefined);
  const setDerivedEffect = (key: "authorized_early_leave_minutes" | "is_early_leave_paid" | "grace_period_minutes", enabled: boolean) => {
    setForm(prev => ({
      ...prev,
      affects_rules: { ...prev.affects_rules, [key]: enabled ? true : undefined },
    }));
    setEffectError(null);
  };
  const setLiteralEffect = (key: "is_break_auto_deduct" | "is_break_overtime" | "allow_early_ot", value: "" | "true" | "false") => {
    setForm(prev => ({
      ...prev,
      affects_rules: { ...prev.affects_rules, [key]: value === "" ? undefined : value === "true" },
    }));
    setEffectError(null);
  };

  const handleSubmit = () => {
    if (!hasEffect) {
      setEffectError(t("settings.condition_types.effects_required"));
      return;
    }

    const details = {
      name_i18n: { en: form.name_en, vi: form.name_vi },
      category: form.category,
      default_minute_allowance: form.default_minute_allowance,
      default_is_paid: form.default_is_paid,
      affects_rules: form.affects_rules,
      icon: form.icon || null,
      color: form.color || null,
      sort_order: form.sort_order,
    };
    onSave(isEdit ? details : { code: form.code, ...details });
  };

  return (
    <div className="bg-white dark:bg-slate-900 border-2 border-primary-100 dark:border-primary-800 rounded-2xl sm:rounded-[2.5rem] p-6 sm:p-8 shadow-xl mb-8">
      <div className="flex items-center gap-3 mb-8 text-primary-700 dark:text-primary-300">
        <div className="w-8 h-8 rounded-xl bg-primary-50 dark:bg-primary-950 flex items-center justify-center shadow-inner">
          <Plus size={16} />
        </div>
        <Typography className="text-[10px] font-bold uppercase tracking-[0.2em]" variant="h4">
          {isEdit ? t("common.edit") : t("common.add_new")}
        </Typography>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mb-8">
        {!isEdit && (
          <div className="space-y-2">
            <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
              {t("settings.condition_types.code")}
            </label>
            <input
              className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 text-sm font-bold focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all"
              onChange={(e) => { setForm(prev => ({ ...prev, code: e.target.value.toUpperCase() })); }}
              placeholder="EARLY_LEAVE"
              value={form.code}
            />
          </div>
        )}
        <div className="space-y-2">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
            {t("settings.condition_types.name_en")}
          </label>
          <input
            className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 text-sm font-bold focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all"
            onChange={(e) => { setForm(prev => ({ ...prev, name_en: e.target.value })); }}
            placeholder="Early Leave"
            value={form.name_en}
          />
        </div>
        <div className="space-y-2">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
            {t("settings.condition_types.name_vi")}
          </label>
          <input
            className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 text-sm font-bold focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all"
            onChange={(e) => { setForm(prev => ({ ...prev, name_vi: e.target.value })); }}
            placeholder="Về sớm"
            value={form.name_vi}
          />
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mb-8">
        <div className="space-y-2">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">
            {t("settings.condition_types.minute_allowance")}
          </label>
          <input
            className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 text-sm font-bold focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all"
            min={0}
            onChange={(e) => { setForm(prev => ({ ...prev, default_minute_allowance: Number(e.target.value) })); }}
            type="number"
            value={form.default_minute_allowance}
          />
        </div>
        <label className="flex items-center gap-3 self-end pb-4 text-sm font-bold text-slate-600 dark:text-slate-300">
          <input
            checked={form.default_is_paid}
            className="h-4 w-4 rounded border-slate-300 text-primary-600 focus:ring-primary-500"
            onChange={(e) => { setForm(prev => ({ ...prev, default_is_paid: e.target.checked })); }}
            type="checkbox"
          />
          {t("settings.condition_types.default_paid")}
        </label>
      </div>

      {/* Policy Effects section */}
      <section aria-labelledby="condition-policy-effects" className="mb-8 rounded-3xl border border-primary-100 bg-primary-50/50 p-6 dark:border-primary-900/60 dark:bg-primary-950/20">
        <div className="mb-5">
          <Typography className="text-sm font-black text-slate-800 dark:text-slate-100" id="condition-policy-effects" variant="h4">
            {t("settings.condition_types.effects_title")}
          </Typography>
          <Typography className="mt-1 text-xs text-slate-500 dark:text-slate-400" variant="p">
            {t("settings.condition_types.effects_help")}
          </Typography>
        </div>

        <div className="space-y-3">
          {([
            ["authorized_early_leave_minutes", "effect_authorized_early_leave_minutes"],
            ["is_early_leave_paid", "effect_is_early_leave_paid"],
            ["grace_period_minutes", "effect_grace_period_minutes"],
          ] as const).map(([key, label]) => (
            <label aria-label={t(`settings.condition_types.${label}`)} className="flex items-start gap-3 rounded-2xl bg-white/80 p-4 dark:bg-slate-900/70" key={key}>
              <input
                checked={form.affects_rules[key] === true}
                className="mt-0.5 h-4 w-4 rounded border-slate-300 text-primary-600 focus:ring-primary-500"
                onChange={(e) => { setDerivedEffect(key, e.target.checked); }}
                type="checkbox"
              />
              <span>
                <span className="block text-sm font-bold text-slate-700 dark:text-slate-200">{t(`settings.condition_types.${label}`)}</span>
                <span className="block pt-0.5 text-xs text-slate-500 dark:text-slate-400">{t(`settings.condition_types.${label}_help`)}</span>
              </span>
            </label>
          ))}

          {([
            ["is_break_auto_deduct", "effect_is_break_auto_deduct"],
            ["is_break_overtime", "effect_is_break_overtime"],
            ["allow_early_ot", "effect_allow_early_ot"],
          ] as const).map(([key, label]) => (
            <label className="flex items-center justify-between gap-4 rounded-2xl bg-white/80 p-4 dark:bg-slate-900/70" key={key}>
              <span>
                <span className="block text-sm font-bold text-slate-700 dark:text-slate-200">{t(`settings.condition_types.${label}`)}</span>
                <span className="block pt-0.5 text-xs text-slate-500 dark:text-slate-400">{t(`settings.condition_types.${label}_help`)}</span>
              </span>
              <select
                aria-label={t(`settings.condition_types.${label}`)}
                className="rounded-xl border border-slate-200 bg-white px-3 py-2 text-xs font-bold text-slate-700 outline-none focus:ring-2 focus:ring-primary-500 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-200"
                onChange={(e) => { setLiteralEffect(key, e.target.value as "" | "true" | "false"); }}
                value={form.affects_rules[key] === undefined ? "" : String(form.affects_rules[key])}
              >
                <option value="">{t("settings.condition_types.not_affected")}</option>
                <option value="true">{t("common.yes")}</option>
                <option value="false">{t("common.no")}</option>
              </select>
            </label>
          ))}

          <label className="flex items-center justify-between gap-4 rounded-2xl bg-white/80 p-4 dark:bg-slate-900/70">
            <span>
              <span className="block text-sm font-bold text-slate-700 dark:text-slate-200">{t("settings.condition_types.effect_early_arrival_strategy")}</span>
              <span className="block pt-0.5 text-xs text-slate-500 dark:text-slate-400">{t("settings.condition_types.effect_early_arrival_strategy_help")}</span>
            </span>
            <select
              aria-label={t("settings.condition_types.effect_early_arrival_strategy")}
              className="rounded-xl border border-slate-200 bg-white px-3 py-2 text-xs font-bold text-slate-700 outline-none focus:ring-2 focus:ring-primary-500 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-200"
              onChange={(e) => {
                setForm(prev => ({
                  ...prev,
                  affects_rules: {
                    ...prev.affects_rules,
                    early_arrival_strategy: (e.target.value || undefined) as ConditionEarlyArrivalStrategy | undefined,
                  },
                }));
                setEffectError(null);
              }}
              value={form.affects_rules.early_arrival_strategy ?? ""}
            >
              <option value="">{t("settings.condition_types.not_affected")}</option>
              <option value="Allow">{t("settings.condition_types.strategy_allow")}</option>
              <option value="Block">{t("settings.condition_types.strategy_block")}</option>
              <option value="Clamp">{t("settings.condition_types.strategy_clamp")}</option>
            </select>
          </label>
        </div>

        {!!effectError && <Typography className="mt-4 text-xs font-bold text-rose-600" variant="p">{effectError}</Typography>}
        <Typography className="mt-5 text-xs font-bold text-primary-700 dark:text-primary-300" variant="p">
          {t("settings.condition_types.effects_preview").replace("{{count}}", String(Object.values(form.affects_rules).filter(v => v !== undefined).length))}
        </Typography>
      </section>

      <div className="flex justify-end gap-4">
        <Button
          className="px-8 py-4 rounded-2xl font-bold text-[10px] uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800 transition-all"
          onClick={onCancel}
          type="button"
          variant="ghost"
        >
          {t("common.cancel")}
        </Button>
        <Button
          className="flex items-center gap-3 bg-primary-600 text-white px-10 py-4 rounded-2xl font-bold text-[10px] uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl active:scale-95 disabled:opacity-50"
          disabled={isSubmitting || (!isEdit && !form.code) || !form.name_en}
          onClick={handleSubmit}
          variant="ghost"
        >
          {t("common.save").toUpperCase()}
        </Button>
      </div>
    </div>
  );
}
