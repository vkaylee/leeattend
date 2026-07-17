import { render, screen, act } from "@testing-library/react";
import React from "react";

import { describe, expect, it, vi } from "vitest";

// ── Mocks ────────────────────────────────────────────────────────────────────
const mockSetActions = vi.fn();

vi.mock("@tanstack/react-router", () => ({
  createFileRoute: () => (opts: Record<string, unknown>) => opts,
  Outlet: () => <div data-testid="outlet" />,
}));

vi.mock("@/components/shifts/shifts-section-nav", () => ({
  ShiftsSectionNav: () => <div data-testid="shifts-section-nav" />,
}));

vi.mock("@/hooks/use-header-actions", () => ({
  useHeaderActions: () => ({ setActions: mockSetActions }),
  HeaderActionsProvider: ({ children }: { children: React.ReactNode }) => <>{children}</>,
}));

// Dynamic import so mocks are hoisted before the module loads
const { Route } = await import("../shifts");

describe("Shifts layout route", () => {
  const Component = Route.component as React.FC;

  it("injects ShiftsSectionNav into header actions on mount", () => {
    render(<Component />);

    expect(mockSetActions).toHaveBeenCalledTimes(1);

    const [[navElement]] = mockSetActions.mock.calls;
    const { container } = render(navElement);
    expect(container.querySelector('[data-testid="shifts-section-nav"]')).toBeInTheDocument();
  });

  it("renders Outlet (page content) below the section nav", () => {
    render(<Component />);

    expect(screen.getByTestId("outlet")).toBeInTheDocument();
  });

  it("cleans up header actions on unmount", () => {
    mockSetActions.mockClear();

    const { unmount } = render(<Component />);
    unmount();

    expect(mockSetActions).toHaveBeenCalledWith(null);
  });
});
