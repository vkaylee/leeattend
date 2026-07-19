import { renderHook, act } from "@testing-library/react";

import { describe, it, expect } from "vitest";

import { useBonusTiers, type TierInput } from "../use-bonus-tiers";

function makeInitialTiers(): TierInput[] {
  return [
    { priority: 1, payout_amount: 150_000, rule_config: [{ metric: "late_early_count", operator: "<=", value: 2 }] },
    { priority: 2, payout_amount: 0, rule_config: [] },
  ];
}

describe("useBonusTiers", () => {
  it("initializes with given tiers", () => {
    const initial = makeInitialTiers();
    const { result } = renderHook(() => useBonusTiers(initial));
    expect(result.current.tiers).toHaveLength(2);
    expect(result.current.tiers[0].payout_amount).toBe(150_000);
  });

  it("adds a new tier with auto-incremented priority", () => {
    const { result } = renderHook(() => useBonusTiers(makeInitialTiers()));
    act(() => { result.current.handleAddTier(); });
    expect(result.current.tiers).toHaveLength(3);
    expect(result.current.tiers[2].priority).toBe(3);
    expect(result.current.tiers[2].payout_amount).toBe(0);
  });

  it("removes a tier by index", () => {
    const { result } = renderHook(() => useBonusTiers(makeInitialTiers()));
    act(() => { result.current.handleRemoveTier(0); });
    expect(result.current.tiers).toHaveLength(1);
    expect(result.current.tiers[0].priority).toBe(2);
  });

  it("updates tier payout amount", () => {
    const { result } = renderHook(() => useBonusTiers(makeInitialTiers()));
    act(() => { result.current.updateTierAmount(0, 500_000); });
    expect(result.current.tiers[0].payout_amount).toBe(500_000);
  });

  it("updates a tier rule field", () => {
    const { result } = renderHook(() => useBonusTiers(makeInitialTiers()));
    act(() => { result.current.updateTierRule(0, 0, "value", 5); });
    expect(result.current.tiers[0].rule_config[0].value).toBe(5);
  });

  it("adds a rule to a tier", () => {
    const { result } = renderHook(() => useBonusTiers(makeInitialTiers()));
    act(() => { result.current.addTierRule(1); }); // tier 2 had empty rules
    expect(result.current.tiers[1].rule_config).toHaveLength(1);
    expect(result.current.tiers[1].rule_config[0].metric).toBe("late_early_count");
  });

  it("removes a rule from a tier", () => {
    const { result } = renderHook(() => useBonusTiers(makeInitialTiers()));
    act(() => { result.current.removeTierRule(0, 0); });
    expect(result.current.tiers[0].rule_config).toHaveLength(0);
  });

  it("handles numeric string input for tier amount", () => {
    const { result } = renderHook(() => useBonusTiers(makeInitialTiers()));
    act(() => { result.current.updateTierAmount(1, "999"); });
    expect(result.current.tiers[1].payout_amount).toBe(999);
  });
});
