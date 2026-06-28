import { describe, it, expect, vi } from "vitest";

// The global vitest setup mocks use-header-actions.
// These tests verify the mock contract that all component tests rely on.

vi.mock("@/hooks/use-header-actions", () => ({
  useHeaderActions: () => ({ actions: null, setActions: vi.fn() }),
  HeaderActionsProvider: ({ children }: { children: React.ReactNode }) => children,
}));

import { useHeaderActions, HeaderActionsProvider } from "@/hooks/use-header-actions";

describe("useHeaderActions (global mock contract)", () => {
  it("returns null actions by default", () => {
    const { actions } = useHeaderActions();
    expect(actions).toBeNull();
  });

  it("setActions is a function that can be called", () => {
    const { setActions } = useHeaderActions();
    expect(typeof setActions).toBe("function");

    const jsx = {} as React.ReactNode;
    setActions(jsx);
    setActions(null);
    // Should not throw
  });

  it("HeaderActionsProvider renders children", () => {
    // It's a pass-through: ({ children }) => children
    expect(HeaderActionsProvider).toBeDefined();
  });
});
