// ── Feature flag registry ──────────────────────────────────────────
// Convention: ff_<area>_<name> (see .agent/rules/change-management.md §6).
//
// A flag defaulting to `false` hides the feature until its backend is
// wired and tested. In DEV, localStorage can override any flag for
// manual testing:
//   localStorage.setItem("ff_econtracts", "true")
// In production the compiled default is authoritative (no JS override —
// changing a flag requires a deploy, which is the intended kill-switch
// behavior).
//
// Cleanup: flags must be removed within 30 days of full rollout.

export const FEATURE_FLAGS = {
  ff_econtracts: false,
} as const;

export type FeatureFlagName = keyof typeof FEATURE_FLAGS;

export function isFeatureEnabled(flag: FeatureFlagName): boolean {
  if (import.meta.env.DEV) {
    const override = localStorage.getItem(flag);
    if (override !== null) {
      return override === "true";
    }
  }
  return FEATURE_FLAGS[flag];
}
