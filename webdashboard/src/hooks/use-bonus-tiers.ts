import { useState } from "react";

import type { IncentiveTier, RuleCondition } from "@/lib/api/types";

export type TierInput = Omit<IncentiveTier, "id" | "policy_id" | "rule_config"> & {
  rule_config: RuleCondition[];
};

export function useBonusTiers(initialTiers: TierInput[]) {
  const [tiers, setTiers] = useState<TierInput[]>(initialTiers);

  const handleAddTier = () => { setTiers([...tiers, { priority: tiers.length + 1, payout_amount: 0, rule_config: [] }]); };
  const handleRemoveTier = (index: number) => { setTiers(tiers.filter((_tier: TierInput, index_: number) => index_ !== index)); };
  const updateTierAmount = (index: number, value: string | number) => {
    const next = [...tiers];
    next[index].payout_amount = Number(value);
    setTiers(next);
  };
  const updateTierRule = (tierIndex: number, ruleIndex: number, field: keyof RuleCondition, value: string | number) => {
    const next = [...tiers];
    next[tierIndex].rule_config[ruleIndex] = { ...next[tierIndex].rule_config[ruleIndex], [field]: value };
    setTiers(next);
  };
  const addTierRule = (tierIndex: number) => {
    const next = [...tiers];
    next[tierIndex].rule_config.push({ metric: "late_early_count", operator: "<=", value: 0 });
    setTiers(next);
  };
  const removeTierRule = (tierIndex: number, ruleIndex: number) => {
    const next = [...tiers];
    next[tierIndex].rule_config.splice(ruleIndex, 1);
    setTiers(next);
  };

  return { tiers, handleAddTier, handleRemoveTier, updateTierAmount, updateTierRule, addTierRule, removeTierRule };
}
