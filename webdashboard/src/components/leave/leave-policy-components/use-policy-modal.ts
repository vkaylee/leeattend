import { useState } from "react";

import { LeavePolicy } from "@/lib/api/types";

interface PolicyFormData {
  accrual_amount: number;
  carry_over_limit: number;
  description: string;
  frequency: string;
  leave_type_id: string;
  min_work_days_threshold: number;
  name: string;
}

const getInitialFormData = (policy: LeavePolicy | null, leaveTypes: { id: string; name: string }[]): PolicyFormData => {
  if (!policy) {
    return {
      name: "",
      description: "",
      leave_type_id: leaveTypes.length > 0 ? leaveTypes[0].id : "",
      accrual_amount: 1.0,
      frequency: "monthly",
      carry_over_limit: 5,
      min_work_days_threshold: 14
    };
  }

  const { rule_config } = policy;
  return {
    name: policy.name,
    description: policy.description ?? "",
    leave_type_id: policy.leave_type_id,
    accrual_amount: rule_config.accrual_amount,
    frequency: rule_config.frequency,
    carry_over_limit: rule_config.carry_over_limit ?? 5,
    min_work_days_threshold: rule_config.min_work_days_threshold ?? 14
  };
};

export function usePolicyModal(
  policy: LeavePolicy | null,
  leaveTypes: { id: string; name: string }[],
  onSave: (data: Partial<LeavePolicy>) => void
) {
  const [formData, setFormData] = useState<PolicyFormData>(() => getInitialFormData(policy, leaveTypes));
  const [seniorityBonus, setSeniorityBonus] = useState(policy?.rule_config.seniority_bonus ?? []);

  const addBonusMilestone = () => {
    setSeniorityBonus([...seniorityBonus, { min_years: 1, extra_days: 1.0 }]);
  };

  const removeBonusMilestone = (index: number) => {
    setSeniorityBonus(seniorityBonus.filter((_: any, i: any) => i !== index));
  };

  const updateBonusMilestone = (index: number, field: "min_years" | "extra_days", value: number) => {
    const next = [...seniorityBonus];
    next[index] = { ...next[index], [field]: value };
    setSeniorityBonus(next);
  };

  const handleSubmit = (e: React.SyntheticEvent) => {
    e.preventDefault();
    onSave({
      ...formData,
      rule_config: {
        frequency: formData.frequency as LeavePolicy["rule_config"]["frequency"],
        accrual_amount: formData.accrual_amount,
        accrual_timing: "start_of_period",
        carry_over_limit: formData.carry_over_limit,
        min_work_days_threshold: formData.min_work_days_threshold,
        seniority_bonus: [...seniorityBonus].sort((a, b) => (a.min_years ?? 0) - (b.min_years ?? 0))
      }
    });
  };

  return {
    formData,
    setFormData,
    seniorityBonus,
    addBonusMilestone,
    removeBonusMilestone,
    updateBonusMilestone,
    handleSubmit
  };
}
