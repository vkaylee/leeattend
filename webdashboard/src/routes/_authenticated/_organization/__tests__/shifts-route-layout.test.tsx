import { render, screen } from "@testing-library/react";
import React from "react";

import { describe, expect, it, vi } from "vitest";

// ── Mocks ────────────────────────────────────────────────────────────────────
const mockSetActions = vi.fn();
const mockFetchMyPermissions = vi.fn();

vi.mock("@tanstack/react-router", () => ({
  createFileRoute: () => (opts: Record<string, unknown>) => opts,
  Outlet: () => <div data-testid="outlet" />,
  redirect: (opts: Record<string, unknown>) => ({ isRedirect: true, ...opts }),
}));

vi.mock("@/lib/api", () => ({
  fetchMyPermissions: () => mockFetchMyPermissions(),
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
const { Route: ReportsRoute } = await import("../reports");
const { Route: AdjustmentsRoute } = await import("../attendance-adjustments");
const { Route: AttendanceLogsRoute } = await import("../attendance-logs");

describe("Shifts layout route", () => {
  const Component = Route.component as React.FC;
  const beforeLoad = Route.beforeLoad as () => Promise<void>;

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

  it("redirects employee without shift/roster permissions from /shifts", async () => {
    mockFetchMyPermissions.mockResolvedValueOnce({ permissions: ["ess:clock", "schedules:read"] });
    await expect(beforeLoad()).rejects.toMatchObject({ isRedirect: true, to: "/" });

    mockFetchMyPermissions.mockResolvedValueOnce({ permissions: ["shifts:read"] });
    await expect(beforeLoad()).resolves.toBeUndefined();
  });

  it("guards /reports, /attendance-adjustments, and /attendance-logs by permission", async () => {
    const reportsGuard = ReportsRoute.beforeLoad as () => Promise<void>;
    const adjustmentsGuard = AdjustmentsRoute.beforeLoad as () => Promise<void>;
    const logsGuard = AttendanceLogsRoute.beforeLoad as () => Promise<void>;

    mockFetchMyPermissions.mockResolvedValueOnce({ permissions: ["departments:read"] });
    await expect(reportsGuard()).rejects.toMatchObject({ isRedirect: true, to: "/" });
    mockFetchMyPermissions.mockResolvedValueOnce({ permissions: ["reports:read"] });
    await expect(reportsGuard()).resolves.toBeUndefined();

    mockFetchMyPermissions.mockResolvedValueOnce({ permissions: ["departments:read"] });
    await expect(adjustmentsGuard()).rejects.toMatchObject({ isRedirect: true, to: "/" });
    mockFetchMyPermissions.mockResolvedValueOnce({ permissions: ["attendance:adjustments:approve"] });
    await expect(adjustmentsGuard()).resolves.toBeUndefined();

    mockFetchMyPermissions.mockResolvedValueOnce({ permissions: ["departments:read"] });
    await expect(logsGuard()).rejects.toMatchObject({ isRedirect: true, to: "/" });
    mockFetchMyPermissions.mockResolvedValueOnce({ permissions: ["attendance:logs:read"] });
    await expect(logsGuard()).resolves.toBeUndefined();
  });
});
