import { useSyncExternalStore } from "react";

import { fetchEffectiveFeatureFlags } from "@/lib/api/feature-flags";

// Build-time defaults remain the fail-closed fallback during bootstrap or API failure.
export const FEATURE_FLAGS = {
  ff_econtracts: false,
} as const;

export type FeatureFlagName = keyof typeof FEATURE_FLAGS;
export interface RuntimeFeatureFlag {
  key: string;
  enabled: boolean;
}

let runtimeFlags: Partial<Record<string, boolean>> = {};
let version = 0;
const listeners = new Set<() => void>();

function notify(): void {
  version += 1;
  for (const listener of listeners) listener();
}

export function subscribeFeatureFlags(listener: () => void): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

export function featureFlagsVersion(): number {
  return version;
}

export function useFeatureFlagVersion(): number {
  return useSyncExternalStore(subscribeFeatureFlags, featureFlagsVersion, featureFlagsVersion);
}

export function setRuntimeFeatureFlags(flags: RuntimeFeatureFlag[]): void {
  runtimeFlags = Object.fromEntries(flags.map(({ key, enabled }) => [key, enabled]));
  notify();
}

export function clearRuntimeFeatureFlags(): void {
  runtimeFlags = {};
  notify();
}

export function isFeatureEnabled(flag: FeatureFlagName): boolean {
  if (Object.prototype.hasOwnProperty.call(runtimeFlags, flag)) {
    return runtimeFlags[flag] === true;
  }
  if (import.meta.env.DEV) {
    const override = localStorage.getItem(flag);
    if (override !== null) return override === "true";
  }
  return FEATURE_FLAGS[flag];
}

export async function loadRuntimeFeatureFlags(): Promise<boolean> {
  try {
    setRuntimeFeatureFlags(await fetchEffectiveFeatureFlags());
    return true;
  } catch {
    clearRuntimeFeatureFlags();
    return false;
  }
}
