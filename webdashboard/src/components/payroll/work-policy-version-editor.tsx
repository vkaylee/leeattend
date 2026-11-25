/* eslint-disable max-lines-per-function -- structured 4-section policy version editor */
import { AlertTriangle, CheckCircle2, RefreshCw, Save, ShieldCheck } from "lucide-react";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import type {
  EarlyArrivalStrategy,
  MonthlyDenominator,
  OTAggregationMode,
  PaidLeaveCreditMode,
  PayMethod,
  RoundingPolicy,
  WorkPolicyValidationResponse,
  WorkPolicyVersion,
  WorkPolicyVersionFields,
} from "@/lib/api/payroll";

interface ReferenceOptions {
  overtimeVersions: { id: string; label: string }[];
  patterns: { id: string; label: string }[];
  salaryComponents: { id: string; label: string }[];
}

interface WorkPolicyVersionEditorProps {
  canManage: boolean;
  conflictMessage: string | null;
  isPublishing: boolean;
  isSaving: boolean;
  isValidating: boolean;
  onDirtyChange?: (dirty: boolean) => void;
  onPublish: (autoClosePrevious: boolean) => void;
  onReload: () => void;
  onSaveDraft: (fields: WorkPolicyVersionFields) => void;
  onValidate: () => void;
  references: ReferenceOptions;
  validation: WorkPolicyValidationResponse | null;
  version: WorkPolicyVersion;
}

function fieldsFromVersion(v: WorkPolicyVersion): WorkPolicyVersionFields {
  return {
    effective_from: v.effective_from,
    effective_to: v.effective_to ?? null,
    minutes_per_credit: v.minutes_per_credit,
    work_unit_minutes: v.work_unit_minutes,
    rounding_policy: v.rounding_policy,
    early_arrival_strategy: v.early_arrival_strategy,
    late_arrival_grace_minutes: v.late_arrival_grace_minutes,
    early_ot_enabled: v.early_ot_enabled,
    early_ot_qualification_grace_minutes: v.early_ot_qualification_grace_minutes,
    early_ot_max_minutes: v.early_ot_max_minutes ?? null,
    break_auto_deduct: v.break_auto_deduct,
    break_is_overtime: v.break_is_overtime,
    ot_aggregation_mode: v.ot_aggregation_mode,
    working_pattern_id: v.working_pattern_id ?? null,
    pay_method: v.pay_method,
    monthly_denominator: v.monthly_denominator ?? null,
    monthly_standard_days: v.monthly_standard_days ?? null,
    rate_component_definition_id: v.rate_component_definition_id ?? null,
    paid_leave_credit_mode: v.paid_leave_credit_mode,
    overtime_policy_version_id: v.overtime_policy_version_id,
  };
}

export function WorkPolicyVersionEditor({
  canManage,
  conflictMessage,
  isPublishing,
  isSaving,
  isValidating,
  onDirtyChange,
  onPublish,
  onReload,
  onSaveDraft,
  onValidate,
  references,
  validation,
  version,
}: WorkPolicyVersionEditorProps) {
  const { t } = useTranslation();
  const isDraft = version.status === "draft";
  const readOnly = !isDraft || !canManage;

  const [form, setForm] = useState<WorkPolicyVersionFields>(() => fieldsFromVersion(version));
  const [autoClosePrevious, setAutoClosePrevious] = useState(true);
  const [isDirty, setIsDirty] = useState(false);

  useEffect(() => {
    setForm(fieldsFromVersion(version));
    setIsDirty(false);
    onDirtyChange?.(false);
  }, [version.id, version.row_version, onDirtyChange, version]);

  const updateField = <K extends keyof WorkPolicyVersionFields>(key: K, value: WorkPolicyVersionFields[K]) => {
    setForm((prev) => ({ ...prev, [key]: value }));
    setIsDirty(true);
    onDirtyChange?.(true);
  };

  const handlePayMethodChange = (nextMethod: PayMethod) => {
    setForm((prev) => {
      if (nextMethod === "monthly") {
        return {
          ...prev,
          pay_method: nextMethod,
          monthly_denominator: prev.monthly_denominator ?? "fixed",
          monthly_standard_days: (prev.monthly_denominator ?? "fixed") === "fixed" ? (prev.monthly_standard_days ?? "22") : null,
          rate_component_definition_id: null,
        };
      }
      return {
        ...prev,
        pay_method: nextMethod,
        monthly_denominator: null,
        monthly_standard_days: null,
        rate_component_definition_id: prev.rate_component_definition_id ?? references.salaryComponents[0]?.id ?? null,
      };
    });
    setIsDirty(true);
    onDirtyChange?.(true);
  };

  const handleDenominatorChange = (denom: MonthlyDenominator) => {
    setForm((prev) => ({
      ...prev,
      monthly_denominator: denom,
      monthly_standard_days: denom === "fixed" ? (prev.monthly_standard_days ?? "22") : null,
    }));
    setIsDirty(true);
    onDirtyChange?.(true);
  };

  const hasBlockers = Boolean(validation && !validation.valid);

  return (
    <div className="space-y-6 rounded-2xl border border-slate-200 bg-white p-6 dark:border-slate-800 dark:bg-slate-900" data-testid="work-policy-version-editor">
      <div className="flex flex-wrap items-center justify-between gap-3 border-b border-slate-100 pb-4 dark:border-slate-800">
        <div>
          <div className="flex items-center gap-2">
            <Typography className="text-base font-bold text-slate-800 dark:text-slate-100" variant="h3">
              {version.group_name} · v{version.version_no}
            </Typography>
            <span
              className={`rounded-full px-2.5 py-0.5 text-xs font-semibold ${
                version.status === "published"
                  ? "bg-emerald-50 text-emerald-700 dark:bg-emerald-950/60 dark:text-emerald-300"
                  : version.status === "draft"
                    ? "bg-amber-50 text-amber-700 dark:bg-amber-950/60 dark:text-amber-300"
                    : "bg-slate-100 text-slate-600 dark:bg-slate-800 dark:text-slate-300"
              }`}
              data-testid="work-policy-version-status"
            >
              {version.status}
            </span>
            <span className="font-mono text-xs text-slate-400">rev {version.row_version}</span>
          </div>
          <p className="mt-0.5 text-xs text-slate-500 dark:text-slate-400">
            {isDraft
              ? t("payroll.work_policies.draft_editable_hint", "Editable draft. Validate and publish to make immutable and active.")
              : t("payroll.work_policies.published_readonly_hint", "Published versions are immutable. Clone into a new draft to change rules.")}
          </p>
        </div>

        <div className="flex flex-wrap items-center gap-2">
          <Button data-testid="validate-work-policy-btn" disabled={isValidating} onClick={onValidate} size="sm" variant="outline">
            <ShieldCheck className="mr-1.5 h-3.5 w-3.5" />
            {t("payroll.work_policies.validate", "Validate")}
          </Button>
          {!readOnly ? (
            <>
              <Button
                data-testid="save-work-policy-draft-btn"
                disabled={isSaving || !isDirty}
                onClick={() => { onSaveDraft(form); }}
                size="sm"
                variant="outline"
              >
                <Save className="mr-1.5 h-3.5 w-3.5" />
                {t("payroll.work_policies.save_draft", "Save Draft")}
              </Button>
              <Button
                data-testid="publish-work-policy-btn"
                disabled={isPublishing || isDirty || hasBlockers}
                onClick={() => { onPublish(autoClosePrevious); }}
                size="sm"
                variant="primary"
              >
                <CheckCircle2 className="mr-1.5 h-3.5 w-3.5" />
                {t("payroll.work_policies.publish", "Publish")}
              </Button>
            </>
          ) : null}
        </div>
      </div>

      {conflictMessage ? (
        <div className="flex flex-wrap items-center justify-between gap-3 rounded-xl border border-rose-200 bg-rose-50 p-3 text-xs text-rose-800 dark:border-rose-800 dark:bg-rose-950/40 dark:text-rose-200" data-testid="work-policy-conflict-banner" role="alert">
          <div className="flex items-center gap-2">
            <AlertTriangle className="h-4 w-4 shrink-0" />
            <span>{conflictMessage}</span>
          </div>
          <Button onClick={onReload} size="sm" variant="outline">
            <RefreshCw className="mr-1 h-3.5 w-3.5" />
            {t("common.refresh", "Reload latest")}
          </Button>
        </div>
      ) : null}

      {/* Section 1: Effective Window */}
      <fieldset className="space-y-3" disabled={readOnly}>
        <legend className="text-xs font-bold uppercase tracking-wider text-slate-500">
          1. {t("payroll.work_policies.section_effective", "Effective Window")}
        </legend>
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
          <label className="text-xs font-medium text-slate-700 dark:text-slate-300">
            {t("payroll.work_policies.effective_from", "Effective From")}
            <input
              className="mt-1 w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-sm disabled:opacity-60 dark:border-slate-700 dark:bg-slate-950"
              data-testid="wp-effective-from"
              onChange={(e) => { updateField("effective_from", e.target.value); }}
              type="date"
              value={form.effective_from}
            />
          </label>
          <label className="text-xs font-medium text-slate-700 dark:text-slate-300">
            {t("payroll.work_policies.effective_to", "Effective To (Exclusive)")}
            <input
              className="mt-1 w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-sm disabled:opacity-60 dark:border-slate-700 dark:bg-slate-950"
              data-testid="wp-effective-to"
              onChange={(e) => { updateField("effective_to", e.target.value || null); }}
              type="date"
              value={form.effective_to ?? ""}
            />
          </label>
          {isDraft && canManage ? (
            <label className="flex items-center gap-2 pt-5 text-xs text-slate-600 dark:text-slate-300">
              <input checked={autoClosePrevious} onChange={(e) => { setAutoClosePrevious(e.target.checked); }} type="checkbox" />
              {t("payroll.work_policies.auto_close_previous", "Auto-close previous published version on publish")}
            </label>
          ) : null}
        </div>
      </fieldset>

      {/* Section 2: Attendance & Work Credit */}
      <fieldset className="space-y-3 border-t border-slate-100 pt-4 dark:border-slate-800" disabled={readOnly}>
        <legend className="text-xs font-bold uppercase tracking-wider text-slate-500">
          2. {t("payroll.work_policies.section_attendance", "Attendance & Work Credit")}
        </legend>
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
          <label className="text-xs font-medium text-slate-700 dark:text-slate-300">
            {t("payroll.work_policies.minutes_per_credit", "Minutes per Credit")}
            <input
              className="mt-1 w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-sm disabled:opacity-60 dark:border-slate-700 dark:bg-slate-950"
              data-testid="wp-minutes-per-credit"
              min={1}
              onChange={(e) => { updateField("minutes_per_credit", Number(e.target.value) || 0); }}
              type="number"
              value={form.minutes_per_credit}
            />
          </label>
          <label className="text-xs font-medium text-slate-700 dark:text-slate-300">
            {t("payroll.work_policies.work_unit_minutes", "Work Unit (min)")}
            <input
              className="mt-1 w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-sm disabled:opacity-60 dark:border-slate-700 dark:bg-slate-950"
              min={1}
              onChange={(e) => { updateField("work_unit_minutes", Number(e.target.value) || 1); }}
              type="number"
              value={form.work_unit_minutes}
            />
          </label>
          <label className="text-xs font-medium text-slate-700 dark:text-slate-300">
            {t("payroll.work_policies.rounding", "Rounding Policy")}
            <select
              className="mt-1 w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-sm disabled:opacity-60 dark:border-slate-700 dark:bg-slate-950"
              onChange={(e) => { updateField("rounding_policy", e.target.value as RoundingPolicy); }}
              value={form.rounding_policy}
            >
              <option value="none">none</option>
              <option value="floor">floor</option>
              <option value="round">round</option>
              <option value="ceil">ceil</option>
            </select>
          </label>
          <label className="text-xs font-medium text-slate-700 dark:text-slate-300">
            {t("payroll.work_policies.early_arrival", "Early Arrival Strategy")}
            <select
              className="mt-1 w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-sm disabled:opacity-60 dark:border-slate-700 dark:bg-slate-950"
              onChange={(e) => { updateField("early_arrival_strategy", e.target.value as EarlyArrivalStrategy); }}
              value={form.early_arrival_strategy}
            >
              <option value="clamp">clamp</option>
              <option value="allow">allow</option>
              <option value="block">block</option>
            </select>
          </label>
          <label className="text-xs font-medium text-slate-700 dark:text-slate-300">
            {t("payroll.work_policies.late_grace", "Late Grace (min)")}
            <input
              className="mt-1 w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-sm disabled:opacity-60 dark:border-slate-700 dark:bg-slate-950"
              min={0}
              onChange={(e) => { updateField("late_arrival_grace_minutes", Number(e.target.value) || 0); }}
              type="number"
              value={form.late_arrival_grace_minutes}
            />
          </label>
          <label className="text-xs font-medium text-slate-700 dark:text-slate-300">
            {t("payroll.work_policies.ot_aggregation", "OT Aggregation")}
            <select
              className="mt-1 w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-sm disabled:opacity-60 dark:border-slate-700 dark:bg-slate-950"
              onChange={(e) => { updateField("ot_aggregation_mode", e.target.value as OTAggregationMode); }}
              value={form.ot_aggregation_mode}
            >
              <option value="aggregate">aggregate</option>
              <option value="per_segment">per_segment</option>
            </select>
          </label>
        </div>

        <div className="flex flex-wrap gap-6 pt-1 text-xs text-slate-700 dark:text-slate-300">
          <label className="flex items-center gap-2">
            <input checked={form.break_auto_deduct} onChange={(e) => { updateField("break_auto_deduct", e.target.checked); }} type="checkbox" />
            {t("payroll.work_policies.break_auto_deduct", "Auto-deduct break")}
          </label>
          <label className="flex items-center gap-2">
            <input checked={form.break_is_overtime} onChange={(e) => { updateField("break_is_overtime", e.target.checked); }} type="checkbox" />
            {t("payroll.work_policies.break_is_overtime", "Worked break counts as OT")}
          </label>
          <label className="flex items-center gap-2">
            <input checked={form.early_ot_enabled} onChange={(e) => { updateField("early_ot_enabled", e.target.checked); }} type="checkbox" />
            {t("payroll.work_policies.early_ot_enabled", "Enable early OT")}
          </label>
        </div>

        {form.early_ot_enabled ? (
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
            <label className="text-xs font-medium text-slate-700 dark:text-slate-300">
              {t("payroll.work_policies.early_ot_grace", "Early OT Qualification Grace (min)")}
              <input
                className="mt-1 w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-sm disabled:opacity-60 dark:border-slate-700 dark:bg-slate-950"
                min={0}
                onChange={(e) => { updateField("early_ot_qualification_grace_minutes", Number(e.target.value) || 0); }}
                type="number"
                value={form.early_ot_qualification_grace_minutes}
              />
            </label>
            <label className="text-xs font-medium text-slate-700 dark:text-slate-300">
              {t("payroll.work_policies.early_ot_max", "Early OT Max (min, optional)")}
              <input
                className="mt-1 w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-sm disabled:opacity-60 dark:border-slate-700 dark:bg-slate-950"
                min={0}
                onChange={(e) => { updateField("early_ot_max_minutes", e.target.value === "" ? null : Number(e.target.value)); }}
                type="number"
                value={form.early_ot_max_minutes ?? ""}
              />
            </label>
          </div>
        ) : null}
      </fieldset>

      {/* Section 3: Pay Basis */}
      <fieldset className="space-y-3 border-t border-slate-100 pt-4 dark:border-slate-800" disabled={readOnly}>
        <legend className="text-xs font-bold uppercase tracking-wider text-slate-500">
          3. {t("payroll.work_policies.section_pay_basis", "Pay Basis & Leave Credit")}
        </legend>
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
          <label className="text-xs font-medium text-slate-700 dark:text-slate-300">
            {t("payroll.work_policies.pay_method", "Pay Method")}
            <select
              className="mt-1 w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-sm disabled:opacity-60 dark:border-slate-700 dark:bg-slate-950"
              data-testid="wp-pay-method"
              onChange={(e) => { handlePayMethodChange(e.target.value as PayMethod); }}
              value={form.pay_method}
            >
              <option value="monthly">monthly</option>
              <option value="hourly">hourly</option>
              <option value="shift">shift</option>
            </select>
          </label>

          {form.pay_method === "monthly" ? (
            <label className="text-xs font-medium text-slate-700 dark:text-slate-300">
              {t("payroll.work_policies.monthly_denominator", "Monthly Denominator")}
              <select
                className="mt-1 w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-sm disabled:opacity-60 dark:border-slate-700 dark:bg-slate-950"
                data-testid="wp-monthly-denominator"
                onChange={(e) => { handleDenominatorChange(e.target.value as MonthlyDenominator); }}
                value={form.monthly_denominator ?? "fixed"}
              >
                <option value="fixed">fixed</option>
                <option value="schedule">schedule</option>
              </select>
            </label>
          ) : null}

          {form.pay_method === "monthly" && form.monthly_denominator === "fixed" ? (
            <label className="text-xs font-medium text-slate-700 dark:text-slate-300">
              {t("payroll.work_policies.monthly_standard_days", "Standard Days / Month")}
              <input
                className="mt-1 w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-sm disabled:opacity-60 dark:border-slate-700 dark:bg-slate-950"
                data-testid="wp-monthly-standard-days"
                onChange={(e) => { updateField("monthly_standard_days", e.target.value); }}
                step="0.5"
                type="number"
                value={form.monthly_standard_days ?? ""}
              />
            </label>
          ) : null}

          {form.pay_method !== "monthly" ? (
            <label className="text-xs font-medium text-slate-700 dark:text-slate-300">
              {t("payroll.work_policies.rate_component", "Rate Salary Component")}
              <select
                className="mt-1 w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-sm disabled:opacity-60 dark:border-slate-700 dark:bg-slate-950"
                data-testid="wp-rate-component"
                onChange={(e) => { updateField("rate_component_definition_id", e.target.value || null); }}
                value={form.rate_component_definition_id ?? ""}
              >
                <option value="">--</option>
                {references.salaryComponents.map((c) => (
                  <option key={c.id} value={c.id}>{c.label}</option>
                ))}
              </select>
            </label>
          ) : null}

          <label className="text-xs font-medium text-slate-700 dark:text-slate-300">
            {t("payroll.work_policies.paid_leave_credit_mode", "Paid Leave Credit")}
            <select
              className="mt-1 w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-sm disabled:opacity-60 dark:border-slate-700 dark:bg-slate-950"
              onChange={(e) => { updateField("paid_leave_credit_mode", e.target.value as PaidLeaveCreditMode); }}
              value={form.paid_leave_credit_mode}
            >
              <option value="standard_day">standard_day</option>
              <option value="scheduled_minutes">scheduled_minutes</option>
              <option value="unpaid">unpaid</option>
            </select>
          </label>
        </div>
      </fieldset>

      {/* Section 4: Linked References */}
      <fieldset className="space-y-3 border-t border-slate-100 pt-4 dark:border-slate-800" disabled={readOnly}>
        <legend className="text-xs font-bold uppercase tracking-wider text-slate-500">
          4. {t("payroll.work_policies.section_references", "Overtime Policy & Working Pattern")}
        </legend>
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
          <label className="text-xs font-medium text-slate-700 dark:text-slate-300">
            {t("payroll.work_policies.overtime_policy_version", "Published Overtime Policy Version")}
            <select
              className="mt-1 w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-sm disabled:opacity-60 dark:border-slate-700 dark:bg-slate-950"
              data-testid="wp-ot-version"
              onChange={(e) => { updateField("overtime_policy_version_id", e.target.value); }}
              value={form.overtime_policy_version_id}
            >
              <option value="">--</option>
              {references.overtimeVersions.map((ov) => (
                <option key={ov.id} value={ov.id}>{ov.label}</option>
              ))}
            </select>
          </label>
          <label className="text-xs font-medium text-slate-700 dark:text-slate-300">
            {t("payroll.work_policies.working_pattern", "Default Working Pattern (Optional)")}
            <select
              className="mt-1 w-full rounded-xl border border-slate-200 bg-white px-3 py-2 text-sm disabled:opacity-60 dark:border-slate-700 dark:bg-slate-950"
              onChange={(e) => { updateField("working_pattern_id", e.target.value || null); }}
              value={form.working_pattern_id ?? ""}
            >
              <option value="">--</option>
              {references.patterns.map((p) => (
                <option key={p.id} value={p.id}>{p.label}</option>
              ))}
            </select>
          </label>
        </div>
      </fieldset>

      {validation ? (
        <div className="space-y-2 border-t border-slate-100 pt-4 dark:border-slate-800" data-testid="work-policy-validation-results">
          {validation.valid ? (
            <div className="rounded-xl border border-emerald-200 bg-emerald-50 p-3 text-xs font-semibold text-emerald-700 dark:border-emerald-800 dark:bg-emerald-950/40 dark:text-emerald-300">
              {t("payroll.work_policies.validation_pass", "Validation passed with 0 blockers.")}
            </div>
          ) : null}
          {(validation.blockers ?? []).map((b, idx) => (
            <div className="rounded-xl border border-rose-200 bg-rose-50 p-3 text-xs text-rose-700 dark:border-rose-800 dark:bg-rose-950/40 dark:text-rose-200" key={`${b.code}-${idx}`}>
              <span className="font-mono font-bold">[{b.code}]</span> {b.message}
            </div>
          ))}
          {(validation.warnings ?? []).map((w, idx) => (
            <div className="rounded-xl border border-amber-200 bg-amber-50 p-3 text-xs text-amber-800 dark:border-amber-800 dark:bg-amber-950/40 dark:text-amber-200" key={`${w.code}-${idx}`}>
              <span className="font-mono font-bold">[{w.code}]</span> {w.message}
            </div>
          ))}
        </div>
      ) : null}
    </div>
  );
}
