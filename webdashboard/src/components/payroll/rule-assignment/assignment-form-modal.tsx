import { Shield } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Modal from "@/components/ui/modal";
import { Department, PayrollRuleAssignment } from "@/lib/api/types";

import { AssigneePicker } from "./assignee-picker";
import { DateRangeFields } from "./components/date-range-fields";
import { ModalActions } from "./components/modal-actions";
import { useAssignmentForm } from "./hooks/use-assignment-form";
import { OverrideConfigSection } from "./override-config-section";
import { ScopeSelector } from "./scope-selector";

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

        <DateRangeFields 
          effectiveFrom={effectiveFrom}
          effectiveTo={effectiveTo}
          setEffectiveFrom={setEffectiveFrom}
          setEffectiveTo={setEffectiveTo}
          t={t as (key: string, defaultValue?: string) => string}
        />

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

        <ModalActions 
          isDisabled={isSubmitting || (assigneeType !== "workspace" && !assigneeId)}
          isSubmitting={isSubmitting}
          onClose={handleClose}
          submitLabel={t("payroll.assign_rule", "Assign Rule")}
          t={t as (key: string, defaultValue?: string) => string}
        />
      </form>
    </Modal>
  );
}
