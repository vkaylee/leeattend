import { useState } from "react";

import type { PayrollRuleAssignment } from "@/lib/api/types";

import type { FixedConfig, PercentageConfig } from "../../payroll-rule-config";
import type { ScopeType } from "../scope-selector";

export function useAssignmentForm(
  calculationMethod: "percentage" | "fixed" | "tiered" | undefined,
  onClose: () => void,
  onSubmit: (data: Partial<PayrollRuleAssignment>) => void
) {
  const [assigneeType, setAssigneeType] = useState<ScopeType>("employee");
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

  const handleSubmit = (e: React.SyntheticEvent) => {
    e.preventDefault();
    let overrideConfig: PercentageConfig | FixedConfig | undefined;
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

  return {
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
  };
}
