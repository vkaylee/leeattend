import { Clock, Shield } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { useAssignmentForm } from "./hooks/use-assignment-form";

import Modal from "@/components/ui/modal";
import { Department, PayrollRuleAssignment } from "@/lib/api/types";

import { AssigneePicker } from "./assignee-picker";
import { OverrideConfigSection } from "./override-config-section";
import { ScopeSelector, type ScopeType } from "./scope-selector";

interface Props {
  calculationMethod?: "percentage" | "fixed" | "tiered";
  departments?: Department[];
  isOpen: boolean;
  isSubmitting: boolean;
  onClose: () => void;
  onSubmit: (data: Partial<PayrollRuleAssignment>) => void;
  wsId: string;
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

  const {
    assigneeType,
    setAssigneeType,
    assigneeId,
    setAssigneeId,
    effectiveFrom,
    setEffectiveFrom,
    effectiveTo,
    setEffectiveTo,
    hasOverride,
    setHasOverride,
    overrideRate,
    setOverrideRate,
    overrideAmount,
    setOverrideAmount,
    overrideBase,
    setOverrideBase,
    handleClose,
    handleSubmit,
  } = useAssignmentForm(calculationMethod, onClose, onSubmit);

  return (
    <Modal
      icon={<Shield size={24} />}
      isOpen={isOpen}
      maxWidth="lg"
      onClose={handleClose}
      subtitle={t(
        "payroll.assign_rule_desc",
        "Define who this rule applies to and when"
      )}
      title={t("payroll.assign_rule", "Assign Rule")}
    >
      <form
        className="space-y-8 pt-2 animate-in fade-in duration-300"
        onSubmit={handleSubmit}
      >
        <ScopeSelector
          assigneeType={assigneeType}
          setAssigneeId={setAssigneeId}
          setAssigneeType={setAssigneeType}
        />

        <AssigneePicker
          assigneeId={assigneeId}
          assigneeType={assigneeType}
          departments={departments}
          setAssigneeId={setAssigneeId}
          wsId={wsId}
        />

        {/* Date range */}
        <div className="grid grid-cols-2 gap-4">
          <div className="space-y-3">
            <label
              className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1 flex items-center gap-1.5"
              htmlFor="effective-from"
            >
              <Clock size={10} /> {t("payroll.effective_from", "Effective From")}
            </label>
            <input
              className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-2xl px-5 py-3.5 font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all shadow-sm"
              id="effective-from"
              onChange={(e) => { setEffectiveFrom(e.target.value); }}
              required
              type="date"
              value={effectiveFrom}
            />
          </div>
          <div className="space-y-3">
            <label
              className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1 flex items-center gap-1.5"
              htmlFor="effective-to"
            >
              <Clock size={10} /> {t("payroll.effective_to", "Effective To")}
              <span className="opacity-40 normal-case tracking-normal">
                ({t("common.optional", "optional")})
              </span>
            </label>
            <input
              className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-2xl px-5 py-3.5 font-bold text-sm text-slate-700 dark:text-slate-200 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all shadow-sm"
              id="effective-to"
              onChange={(e) => { setEffectiveTo(e.target.value); }}
              type="date"
              value={effectiveTo}
            />
          </div>
        </div>

        <OverrideConfigSection
          calculationMethod={calculationMethod}
          hasOverride={hasOverride}
          overrideAmount={overrideAmount}
          overrideBase={overrideBase}
          overrideRate={overrideRate}
          setHasOverride={setHasOverride}
          setOverrideAmount={setOverrideAmount}
          setOverrideBase={setOverrideBase}
          setOverrideRate={setOverrideRate}
        />

        {/* Actions */}
        <div className="flex gap-4 pt-2 border-t border-slate-50 dark:border-slate-800">
          <button
            className="flex-1 px-6 py-3.5 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 text-slate-500 dark:text-slate-400 rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-slate-50 dark:hover:bg-slate-800 transition-all active:scale-95"
            onClick={handleClose}
            type="button"
          >
            {t("common.cancel", "Cancel")}
          </button>
          <button
            className="flex-[1.5] bg-teal-600 text-white rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-teal-700 transition-all shadow-xl shadow-teal-100 dark:shadow-black/30 active:scale-95 disabled:opacity-50 disabled:cursor-not-allowed py-3.5"
            disabled={
              isSubmitting ||
              (assigneeType !== "workspace" && !assigneeId)
            }
            type="submit"
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

