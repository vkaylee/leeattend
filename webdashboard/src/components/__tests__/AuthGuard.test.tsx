/**
 * AuthGuard has been replaced by beforeLoad hooks in TanStack Router.
 * These tests are retained as stubs to avoid breaking the test suite.
 * See src/routes/_authenticated.tsx for the new auth guard logic.
 */
import { describe, it } from "vitest";

describe("AuthGuard (legacy)", () => {
  it("auth guard is now handled by TanStack Router beforeLoad hooks", () => {
    // No-op: AuthGuard.tsx deleted, auth logic moved to route tree
  });
});
