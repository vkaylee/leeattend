/**
 * WorkspaceGuard has been replaced by beforeLoad hooks in TanStack Router.
 * These tests are retained as stubs to avoid breaking the test suite.
 * See src/routes/_authenticated/_workspace.tsx for the new workspace guard logic.
 */
import { describe, it } from "vitest";

describe("WorkspaceGuard (legacy)", () => {
  it("workspace guard is now handled by TanStack Router beforeLoad hooks", () => {
    // No-op: WorkspaceGuard.tsx deleted, logic moved to route tree
  });
});
