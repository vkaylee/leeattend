import React, { useState } from "react";

import { Users, Building, Globe, Clock, Sliders, Shield } from "lucide-react";
import { useTranslation } from "react-i18next";

import { EmployeeSearch } from "@/components/common/EmployeeSearch";
import Modal from "@/components/ui/Modal";

import { Department, PayrollRuleAssignment } from "@/lib/api/types";
import { cn } from "@/lib/utils";

const SCOPE_ICONS = {
  workspace: Globe,
  department: Building,
  employee: Users,
} as const;

const SCOPE_COLORS = {
  workspace: {
    bg: "bg-amber-50 dark:bg-amber-950/40",
    text: "text-amber-600 dark:text-amber-400",
    border: "border-amber-100 dark:border-amber-800",
  },
  department: {
    bg: "bg-sky-50 dark:bg-sky-950/40",
    text: "text-sky-600 dark:text-sky-400",
    border: "border-sky-100 dark:border-sky-800",
  },
  employee: {
    bg: "bg-teal-50 dark:bg-teal-950/40",
    text: "text-teal-600 dark:text-teal-400",
    border: "border-teal-100 dark:border-teal-800",
  },
} as const;

interface Props {
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (data: Partial<PayrollRuleAssignment>) => void;
  isSubmitting: boolean;
  wsId: string;
  departments?: Department[];
  calculationMethod?: "percentage" | "fixed" | "tiered";
}

export function AssignmentFormModal({
  isOpen,
  onClose,
  onSubmit,
  isSubmitting,
  wsId,
  departments,
  calculationMethod,
}: Props) {
  const { t } = useTranslation();

  const [assigneeType, setAssigneeType] = useState<
    "workspace" | "department" | "employee"
  >("employee");
  const [assigneeId, setAssigneeId] = useState<string>("");
  const [effectiveFrom, setEffectiveFrom] = useState(
    () => new Date().toISOString().split("T")[0]
  );
  const [effectiveTo, setEffectiveTo] = useState<string>("");
  const [hasOverride, setHasOverride] = useState(false);
  const [overrideRate, setOverrideRate] = useState<string>("");
  const [overrideAmount, setOverrideAmount] = useState<string>("");
  const [overrideBase, setOverrideBase] = useState<string>("gross");

  const handleClose = () => {
    onClose();
    setAssigneeType("employee");
    setAssigneeId("");
    setEffectiveFrom(new Date().toISOString().split("T")[0]);
    setEffectiveTo("");
    setHasOverride(false);
    setOverrideRate("");
    setOverrideAmount("");
    setOverrideBase("gross");
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    let overrideConfig: Record<string, unknown> | undefined;
    if (hasOverride) {
      if (calculationMethod === "percentage" && overrideRate) {
        overrideConfig = { rate: parseFloat(overrideRate), base: overrideBase };
      } else if (calculationMethod === "fixed" && overrideAmount) {
        overrideConfig = { amount: parseFloat(overrideAmount) };
      }
    }
    onSubmit({
      assignee_type: assigneeType,
      assignee_id: assigneeType === "workspace" ? undefined : assigneeId || undefined,
      effective_from: effectiveFrom,
      effective_to: effectiveTo || undefined,
      override_config: overrideConfig,
    });
  };

  return (
    <Modal
      isOpen={isOpen}
      onClose={handleClose}
      title={t("payroll.assign_rule", "Assign Rule")}
      subtitle={t(
        "payroll.assign_rule_desc",
        "Define who this rule applies to and when"
      )}
      icon={<Shield size={24} />}
      maxWidth="lg"
    >
      <form
        onSubmit={handleSubmit}
        className="space-y-8 pt-2 animate-in fade-in duration-300"
      >
        {/* Scope selector */}
        <div className="space-y-3">
          <div className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1 mb-1">
            {t("payroll.assignment_scope", "Assignment Scope")}
          </div>
          <div className="grid grid-cols-3 gap-3">
            {(["employee", "department", "workspace"] as const).map((scope) => {
              const Icon = SCOPE_ICONS[scope];
              const colors = SCOPE_COLORS[scope];
              const isActive = assigneeType === scope;
              return (
                <button
                  key={scope}
                  type="button"
                  onClick={() => {
                    setAssigneeType(scope);
                    setAssigneeId("");
                  }}
                  className={cn(
                    "flex flex-col items-center gap-2.5 p-5 rounded-2xl border-2 transition-all active:scale-[0.97]",
                    isActive
                      ? `${colors.bg} ${colors.border} shadow-lg`
                      : "bg-slate-50 dark:bg-slate-800/50 border-transparent hover:border-slate-200 dark:hover:border-slate-700 hover:bg-white dark:hover:bg-slate-900"
                  )}
                >
                  <Icon
                    size={22}
                    className={
                      isActive ? colors.text : "text-slate-400 dark:text-slate-500"
                    }
                  />
                  <span
                    className={cn(
                      "text-[10px] font-bold uppercase tracking-widest",
                      isActive
                        ? "text-slate-700 dark:text-slate-200"
                        : "text-slate-500 dark:text-slate-400"
                    )}
                  >
                    {t(`payroll.scope_${scope}`, scope)}
                  </span>
                </button>
              );
            })}
          </div>
        </div>

        {/* Assignee picker */}
        {assigneeType === "employee" && (
          <div className="space-y-3">
            <div className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1 mb-1">
              {t("payroll.select_employee", "Select Employee")}
            </div>
            <EmployeeSearch
              workspaceId={wsId}
              value={assigneeId}
              onChange={(val) => setAssigneeId(Array.isArray(val) ? val[0] : val)}
              placeholder={t("payroll.search_employees", "Search employees…")}
            />
          </div>
        )}

        {assigneeType === "department" && (
          <div className="space-y-3">
            <label
              htmlFor="dept-select"
              className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1"
            >
              {t("payroll.select_department", "Select Department")}
            </label>
            <select
              id="dept-select"
              required
              value={assigneeId}
              onChange={(e) => setAssigneeId(e.target.value)}
              className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-2xl px-5 py-3.5 font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-sky-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all uppercase tracking-tight shadow-sm"
            >
              <option value="">
                {t(
                  "payroll.select_department_placeholder",
                  "— Select department —"
                )}
              </option>
              {departments?.map((d) => (
                <option key={d.id} value={d.id}>
                  {d.name}
                </option>
              ))}
            </select>
          </div>
        )}

        {assigneeType === "workspace" && (
          <div className="bg-amber-50 dark:bg-amber-950/30 border border-amber-100 dark:border-amber-800 rounded-2xl p-5 flex items-center gap-4">
            <Globe
              size={20}
              className="text-amber-600 dark:text-amber-400 shrink-0"
            />
            <p className="text-xs font-bold text-amber-800 dark:text-amber-200 uppercase tracking-tight">
              {t(
                "payroll.workspace_scope_info",
                "This rule will apply to all employees in the workspace"
              )}
            </p>
          </div>
        )}

        {/* Date range */}
        <div className="grid grid-cols-2 gap-4">
          <div className="space-y-3">
            <label
              htmlFor="effective-from"
              className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1 flex items-center gap-1.5"
            >
              <Clock size={10} /> {t("payroll.effective_from", "Effective From")}
            </label>
            <input
              id="effective-from"
              type="date"
              required
              value={effectiveFrom}
              onChange={(e) => setEffectiveFrom(e.target.value)}
              className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-2xl px-5 py-3.5 font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all shadow-sm"
            />
          </div>
          <div className="space-y-3">
            <label
              htmlFor="effective-to"
              className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1 flex items-center gap-1.5"
            >
              <Clock size={10} /> {t("payroll.effective_to", "Effective To")}
              <span className="opacity-40 normal-case tracking-normal">
                ({t("common.optional", "optional")})
              </span>
            </label>
            <input
              id="effective-to"
              type="date"
              value={effectiveTo}
              onChange={(e) => setEffectiveTo(e.target.value)}
              className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-2xl px-5 py-3.5 font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all shadow-sm"
            />
          </div>
        </div>

        {/* Override config toggle */}
        {(calculationMethod === "percentage" ||
          calculationMethod === "fixed") && (
          <div className="space-y-4">
            {/* eslint-disable-next-line jsx-a11y/label-has-associated-control */}
            <label
              htmlFor="override-config"
              className={cn(
                "flex items-center gap-4 p-5 rounded-2xl cursor-pointer transition-all border-2 active:scale-[0.98]",
                hasOverride
                  ? "bg-orange-50 dark:bg-orange-950/30 border-orange-100 dark:border-orange-800 shadow-lg"
                  : "bg-slate-50 dark:bg-slate-800/50 border-transparent hover:border-slate-200 dark:hover:border-slate-700"
              )}
            >
              <input
                id="override-config"
                type="checkbox"
                checked={hasOverride}
                onChange={(e) => setHasOverride(e.target.checked)}
                className="w-5 h-5 rounded-lg text-orange-600 focus:ring-orange-500 border-slate-300 dark:border-slate-600 cursor-pointer"
              />
              <div>
                <p className="font-bold text-sm text-slate-700 dark:text-slate-200 uppercase tracking-tight flex items-center gap-2">
                  <Sliders size={14} className="opacity-50" />{" "}
                  {t("payroll.override_config", "Override Default Config")}
                </p>
                <p className="text-[10px] text-slate-500 dark:text-slate-400 uppercase tracking-widest mt-1 opacity-70">
                  {t(
                    "payroll.override_config_desc",
                    "Apply a different rate or amount for this specific assignment"
                  )}
                </p>
              </div>
            </label>

            {hasOverride && (
              <div className="bg-orange-50 dark:bg-orange-950/20 rounded-2xl p-5 border border-orange-100 dark:border-orange-800 space-y-4 animate-in slide-in-from-top-2 duration-200">
                {calculationMethod === "percentage" && (
                  <div className="grid grid-cols-2 gap-4">
                    <div className="space-y-2">
                      <label
                        htmlFor="override-rate"
                        className="text-[10px] font-bold text-orange-700 dark:text-orange-300 uppercase tracking-widest"
                      >
                        {t("payroll.override_rate", "Override Rate (%)")}
                      </label>
                      <input
                        id="override-rate"
                        type="number"
                        step="0.01"
                        value={overrideRate}
                        onChange={(e) => setOverrideRate(e.target.value)}
                        placeholder="e.g. 10.5"
                        className="w-full bg-white dark:bg-slate-900 border border-orange-200 dark:border-orange-800 rounded-xl px-4 py-3 font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-orange-500/20 outline-none"
                      />
                    </div>
                    <div className="space-y-2">
                      <label
                        htmlFor="override-base"
                        className="text-[10px] font-bold text-orange-700 dark:text-orange-300 uppercase tracking-widest"
                      >
                        {t("payroll.override_base", "Override Base")}
                      </label>
                      <select
                        id="override-base"
                        value={overrideBase}
                        onChange={(e) => setOverrideBase(e.target.value)}
                        className="w-full bg-white dark:bg-slate-900 border border-orange-200 dark:border-orange-800 rounded-xl px-4 py-3 font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-orange-500/20 outline-none uppercase tracking-tight"
                      >
                        <option value="gross">
                          {t("payroll.base_gross", "Gross Salary")}
                        </option>
                        <option value="gross_salary">
                          {t("payroll.base_gross_alt", "Gross Salary (alt)")}
                        </option>
                        <option value="taxable_income">
                          {t("payroll.base_taxable", "Taxable Income")}
                        </option>
                      </select>
                    </div>
                  </div>
                )}
                {calculationMethod === "fixed" && (
                  <div className="space-y-2">
                    <label
                      htmlFor="override-amount"
                      className="text-[10px] font-bold text-orange-700 dark:text-orange-300 uppercase tracking-widest"
                    >
                      {t("payroll.override_amount", "Override Amount (₫)")}
                    </label>
                    <input
                      id="override-amount"
                      type="number"
                      value={overrideAmount}
                      onChange={(e) => setOverrideAmount(e.target.value)}
                      placeholder="e.g. 500000"
                      className="w-full bg-white dark:bg-slate-900 border border-orange-200 dark:border-orange-800 rounded-xl px-4 py-3 font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-orange-500/20 outline-none"
                    />
                  </div>
                )}
              </div>
            )}
          </div>
        )}

        {/* Actions */}
        <div className="flex gap-4 pt-2 border-t border-slate-50 dark:border-slate-800">
          <button
            type="button"
            onClick={handleClose}
            className="flex-1 px-6 py-3.5 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 text-slate-500 dark:text-slate-400 rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-slate-50 dark:hover:bg-slate-800 transition-all active:scale-95"
          >
            {t("common.cancel", "Cancel")}
          </button>
          <button
            type="submit"
            disabled={
              isSubmitting ||
              (assigneeType !== "workspace" && !assigneeId)
            }
            className="flex-[1.5] bg-teal-600 text-white rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-teal-700 transition-all shadow-xl shadow-teal-100 dark:shadow-black/30 active:scale-95 disabled:opacity-50 disabled:cursor-not-allowed py-3.5"
          >
            {isSubmitting
              ? t("common.saving", "Saving") + "…"
              : t("payroll.assign_rule", "Assign Rule")}
          </button>
        </div>
      </form>
    </Modal>
  );
}
