import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Tags, Plus, Pencil, Trash2, ShieldAlert, Clock, Heart, Baby, Ban } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { Typography } from "@/components/ui/typography";
import { useConfirm } from "@/hooks/use-confirm";
import {
  fetchConditionTypes,
  createConditionType,
  updateConditionType,
  deleteConditionType,
  getCurrentWorkspaceId,
  type ConditionTypeDefinition,
  type CreateConditionTypeRequest,
  type UpdateConditionTypeRequest,
  type ConditionRuleEffects,
  type ConditionEarlyArrivalStrategy,
} from "@/lib/api";
import { cn } from "@/lib/utils";

const LUCIDE_ICONS: Record<string, React.ReactNode> = {
  Clock: <Clock size={18} />,
  Baby: <Baby size={18} />,
  Heart: <Heart size={18} />,
  ShieldAlert: <ShieldAlert size={18} />,
  Ban: <Ban size={18} />,
};

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

const CATEGORY_LABELS: Record<string, string> = {
  working_hours: "Working Hours",
  leave: "Leave",
  general: "General",
};

// ── Form Component ─────────────────────────────────────────────────

function ConditionTypeForm({
  initial,
  onSave,
  onCancel,
  isSubmitting,
  t,
}: {
  initial?: ConditionTypeDefinition;
  onSave: (data: CreateConditionTypeRequest | UpdateConditionTypeRequest) => void;
  onCancel: () => void;
  isSubmitting: boolean;
  t: (key: string) => string;
}) {
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
    setForm({
      ...form,
      affects_rules: {
        ...form.affects_rules,
        [key]: enabled ? true : undefined,
      },
    });
    setEffectError(null);
  };
  const setLiteralEffect = (key: "is_break_auto_deduct" | "is_break_overtime" | "allow_early_ot", value: "" | "true" | "false") => {
    setForm({
      ...form,
      affects_rules: {
        ...form.affects_rules,
        [key]: value === "" ? undefined : value === "true",
      },
    });
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
              onChange={(e) => { setForm({ ...form, code: e.target.value.toUpperCase() }); }}
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
            onChange={(e) => { setForm({ ...form, name_en: e.target.value }); }}
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
            onChange={(e) => { setForm({ ...form, name_vi: e.target.value }); }}
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
            onChange={(e) => { setForm({ ...form, default_minute_allowance: Number(e.target.value) }); }}
            type="number"
            value={form.default_minute_allowance}
          />
        </div>
        <label className="flex items-center gap-3 self-end pb-4 text-sm font-bold text-slate-600 dark:text-slate-300">
          <input
            checked={form.default_is_paid}
            className="h-4 w-4 rounded border-slate-300 text-primary-600 focus:ring-primary-500"
            onChange={(e) => { setForm({ ...form, default_is_paid: e.target.checked }); }}
            type="checkbox"
          />
          {t("settings.condition_types.default_paid")}
        </label>
      </div>

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
                setForm({
                  ...form,
                  affects_rules: {
                    ...form.affects_rules,
                    early_arrival_strategy: (e.target.value || undefined) as ConditionEarlyArrivalStrategy | undefined,
                  },
                });
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
          {t("settings.condition_types.effects_preview").replace("{{count}}", String(Object.values(form.affects_rules).filter((value) => value !== undefined).length))}
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

// ── Main Page Component ────────────────────────────────────────────

export default function ConditionTypesPage() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const { confirm, ConfirmationDialog } = useConfirm();
  const wsId = getCurrentWorkspaceId();
  const [isAdding, setIsAdding] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);

  const { data: types = [], isLoading } = useQuery({
    queryKey: ["condition-types", wsId],
    queryFn: () => (wsId ? fetchConditionTypes(wsId) : Promise.resolve([])),
    enabled: !!wsId,
  });

  const createMutation = useMutation({
    mutationFn: (req: CreateConditionTypeRequest) => {
      if (!wsId) throw new Error("No workspace");
      return createConditionType(wsId, req);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["condition-types", wsId] });
      setIsAdding(false);
    },
  });

  const updateMutation = useMutation({
    mutationFn: ({ id, req }: { id: string; req: UpdateConditionTypeRequest }) => {
      if (!wsId) throw new Error("No workspace");
      return updateConditionType(wsId, id, req);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["condition-types", wsId] });
      setEditingId(null);
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => {
      if (!wsId) throw new Error("No workspace");
      return deleteConditionType(wsId, id);
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["condition-types", wsId] });
    },
  });

  const handleDelete = async (item: ConditionTypeDefinition) => {
    const ok = await confirm({
      title: t("common.delete_confirm"),
      message: t("settings.condition_types.confirm_delete", {
        defaultValue: "Delete this condition type? Existing conditions using it will not be affected.",
      }),
      variant: "danger",
    });
    if (ok) deleteMutation.mutate(item.id);
  };

  if (isLoading || !wsId) {
    return (
      <div className="space-y-4" data-testid="condition-types-skeleton">
        <Skeleton className="h-8 w-48" />
        {[1, 2, 3].map((i) => (
          <Skeleton className="h-24 w-full rounded-2xl" key={i} />
        ))}
      </div>
    );
  }

  return (
    <div className="space-y-8 animate-in fade-in duration-500">
      {/* Header */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-6">
        <div className="flex items-center gap-4">
          <div className="w-12 h-12 rounded-2xl bg-primary-50 dark:bg-primary-950 flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-sm border border-primary-100 dark:border-primary-800/50">
            <Tags size={22} />
          </div>
          <div>
            <Typography className="text-xl font-black text-slate-800 dark:text-slate-100" variant="h2">
              {t("settings.categories.condition_types", { defaultValue: "Condition Types" })}
            </Typography>
            <Typography className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mt-0.5" variant="p">
              {t("settings.condition_types.subtitle", { defaultValue: "Configure working condition types available for employees" })}
            </Typography>
          </div>
        </div>
        {!isAdding && (
          <Button
            className="flex items-center justify-center gap-2 bg-primary-600 text-white px-6 py-2.5 rounded-2xl font-bold text-[10px] uppercase tracking-widest hover:bg-primary-700 transition-all shadow-lg active:scale-95"
            onClick={() => { setIsAdding(true); setEditingId(null); }}
            variant="ghost"
          >
            <Plus size={14} /> {t("common.add_new").toUpperCase()}
          </Button>
        )}
      </div>

      {/* Add Form */}
      {isAdding ? (
        <ConditionTypeForm
          isSubmitting={createMutation.isPending}
          onCancel={() => { setIsAdding(false); }}
          onSave={(data) => { createMutation.mutate(data as CreateConditionTypeRequest); }}
          t={(k) => t(k)}
        />
      ) : null}

      {/* Type Cards */}
      <div className="grid grid-cols-1 gap-4">
        {types.length === 0 ? (
          <div className="bg-slate-50 dark:bg-slate-800/50 border border-dashed border-slate-200 dark:border-slate-700 rounded-[2rem] p-16 text-center">
            <Typography className="text-sm font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest" variant="h4">
              {t("settings.condition_types.no_types", { defaultValue: "No condition types defined" })}
            </Typography>
          </div>
        ) : (
          types.map((item) => (
            <div
              className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[2rem] p-6 hover:shadow-premium-xl transition-all group"
              key={item.id}
            >
              {editingId === item.id ? (
                <ConditionTypeForm
                  initial={item}
                  isSubmitting={updateMutation.isPending}
                  onCancel={() => { setEditingId(null); }}
                  onSave={(data) => { updateMutation.mutate({ id: item.id, req: data }); }}
                  t={(k) => t(k)}
                />
              ) : (
                <div className="flex items-start justify-between">
                  <div className="flex gap-5">
                    <div className="w-12 h-12 rounded-2xl bg-slate-50 dark:bg-slate-800 flex items-center justify-center text-slate-500 dark:text-slate-400 shadow-inner">
                      {LUCIDE_ICONS[item.icon ?? ""] ?? <Clock size={18} />}
                    </div>
                    <div>
                      <div className="flex items-center gap-3 mb-1">
                        <Typography className="font-bold text-slate-700 dark:text-slate-200 text-base tracking-tight" variant="h4">
                          {item.name_i18n.en || item.code}
                        </Typography>
                        <span className="text-[9px] font-bold uppercase tracking-widest text-slate-400 bg-slate-100 dark:bg-slate-800 px-2 py-0.5 rounded-full">
                          {item.code}
                        </span>
                        {item.is_system ? (
                          <span className="text-[9px] font-bold uppercase tracking-widest text-primary-600 bg-primary-50 dark:bg-primary-950 px-2 py-0.5 rounded-full">
                            SYSTEM
                          </span>
                        ) : null}
                        <span className={cn(
                          "px-2.5 py-0.5 rounded-full text-[9px] font-bold uppercase tracking-widest border",
                          item.default_is_paid
                            ? "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800"
                            : "bg-slate-50 dark:bg-slate-800 text-slate-500 dark:text-slate-400 border-slate-100 dark:border-slate-800",
                        )}>
                          {item.default_is_paid ? "PAID" : "UNPAID"}
                        </span>
                      </div>
                      <div className="flex items-center gap-3 text-[10px] text-slate-500 dark:text-slate-400 font-bold uppercase tracking-widest mt-1">
                        <span>{CATEGORY_LABELS[item.category] ?? item.category}</span>
                        <span>·</span>
                        <span>{item.default_minute_allowance}m default</span>
                      </div>
                    </div>
                  </div>
                  <div className="flex gap-2 opacity-0 group-hover:opacity-100 transition-all -translate-y-2 group-hover:translate-y-0">
                    <Button
                      className="p-3 rounded-2xl bg-slate-50 dark:bg-slate-800 text-slate-500 hover:bg-primary-50 hover:text-primary-600 transition-all shadow-sm"
                      onClick={() => { setEditingId(item.id); setIsAdding(false); }}
                      variant="ghost"
                    >
                      <Pencil size={16} />
                    </Button>
                    {!item.is_system && (
                      <Button
                        className="p-3 rounded-2xl bg-rose-50 dark:bg-rose-950/30 text-rose-600 dark:text-rose-400 hover:bg-rose-600 hover:text-white transition-all shadow-sm"
                        onClick={() => handleDelete(item)}
                        variant="ghost"
                      >
                        <Trash2 size={16} />
                      </Button>
                    )}
                  </div>
                </div>
              )}
            </div>
          ))
        )}
      </div>

      <ConfirmationDialog />
    </div>
  );
}
