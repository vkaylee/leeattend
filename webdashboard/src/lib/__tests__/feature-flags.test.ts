import { afterEach, describe, expect, it, vi } from "vitest";

import { fetchEffectiveFeatureFlags } from "@/lib/api/feature-flags";
import { FEATURE_FLAGS, isFeatureEnabled, loadRuntimeFeatureFlags } from "@/lib/feature-flags";

vi.mock("@/lib/api/feature-flags", () => ({
  fetchEffectiveFeatureFlags: vi.fn(),
}));

describe("FEATURE_FLAGS", () => {
  afterEach(() => {
    localStorage.clear();
    vi.clearAllMocks();
  });

  it("registers the econtracts flag with default OFF (backend not wired)", () => {
    expect(FEATURE_FLAGS.ff_econtracts).toBe(false);
  });

  it("returns the compiled default when no localStorage override exists", () => {
    expect(isFeatureEnabled("ff_econtracts")).toBe(false);
  });

  it("honors a DEV localStorage override to enable a flag", () => {
    localStorage.setItem("ff_econtracts", "true");
    expect(isFeatureEnabled("ff_econtracts")).toBe(true);
  });

  it("honors a DEV localStorage override to disable a flag", () => {
    localStorage.setItem("ff_econtracts", "false");
    expect(isFeatureEnabled("ff_econtracts")).toBe(false);
  });

  it("ignores invalid override values (anything but 'true' is off)", () => {
    localStorage.setItem("ff_econtracts", "yes");
    expect(isFeatureEnabled("ff_econtracts")).toBe(false);
  });

  it("skips protected flag loading when no auth session exists", async () => {
    await expect(loadRuntimeFeatureFlags()).resolves.toBe(false);
    expect(fetchEffectiveFeatureFlags).not.toHaveBeenCalled();
  });

  // Note: the production path (compiled default without localStorage
  // override) cannot be unit-tested here — vitest always runs with
  // import.meta.env.DEV === true (see logger.test.ts precedent). The
  // production branch is just `return FEATURE_FLAGS[flag]`, which the
  // "returns the compiled default" test above covers.
});
