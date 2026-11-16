import { fireEvent, render, screen } from "@testing-library/react";
import React from "react";

import { beforeEach, describe, expect, it, vi } from "vitest";

import { SettingsSidebar } from "../components/settings-sidebar";
import * as hooks from "../hooks/use-settings-nav";

// ── Mocks ─────────────────────────────────────────────────────────────

let mockLocationPath = "/settings/general";

vi.mock("@tanstack/react-router", () => ({
  useLocation: () => ({ pathname: mockLocationPath, search: "", hash: "" }),
  Link: ({
    children,
    to,
    className,
    ...props
  }: React.AnchorHTMLAttributes<HTMLAnchorElement> & { to?: string }) => (
    <a className={className} href={to} {...props}>
      {children}
    </a>
  ),
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, fallback?: string) => fallback ?? key,
  }),
}));

vi.mock("@/hooks/use-permissions", () => ({
  useMyPermissions: () => ({
    hasAnyPermission: vi.fn(() => true),
    permissions: [],
    roles: [],
    managedDepartmentIds: null,
    isLoading: false,
    hasPermission: vi.fn(),
    hasRole: vi.fn(),
    hasScopedPermission: vi.fn(),
  }),
}));

// ── Helpers ───────────────────────────────────────────────────────────

function makeItem(label: string, path: string): hooks.SettingItem {
  return {
    description: `Description for ${label}`,
    icon: <span data-testid={`icon-${label}`} />,
    label,
    path,
  };
}

function makeNavState(
  overrides: Partial<hooks.SettingsNavState> = {}
) {
  return {
    organizationGroup: [],
    accessGroup: [],
    operationsGroup: [],
    systemGroup: [],
    ...overrides,
  };
}

describe("SettingsSidebar (macOS 2-column layout)", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockLocationPath = "/settings/general";
  });

  it("renders sidebar with header and admin badge", () => {
    vi.spyOn(hooks, "useSettingsNav").mockReturnValue(
      makeNavState({
        organizationGroup: [makeItem("General", "/settings/general")],
      })
    );

    render(<SettingsSidebar />);
    expect(screen.getByTestId("settings-sidebar")).toBeInTheDocument();
    expect(screen.getByText("Cài đặt")).toBeInTheDocument();
    expect(screen.getByText("Admin")).toBeInTheDocument();
  });

  it("renders back to workspace link for Focus Mode", () => {
    vi.spyOn(hooks, "useSettingsNav").mockReturnValue(
      makeNavState({
        organizationGroup: [makeItem("General", "/settings/general")],
      })
    );

    render(<SettingsSidebar />);
    const backLink = screen.getByTestId("settings-back-to-app");
    expect(backLink).toBeInTheDocument();
    expect(backLink).toHaveAttribute("href", "/");
  });
  it("renders a bounded touch-enabled scroll region for settings items", () => {
    vi.spyOn(hooks, "useSettingsNav").mockReturnValue(
      makeNavState({
        organizationGroup: [makeItem("General", "/settings/general")],
      })
    );

    render(<SettingsSidebar />);

    expect(screen.getByTestId("settings-sidebar-scroll-region")).toHaveClass(
      "min-h-0",
      "overflow-y-auto",
      "touch-pan-y"
    );
  });


  it("renders all four sections when items are populated", () => {
    vi.spyOn(hooks, "useSettingsNav").mockReturnValue(
      makeNavState({
        organizationGroup: [makeItem("General", "/settings/general")],
        accessGroup: [makeItem("Roles", "/settings/roles")],
        operationsGroup: [makeItem("Time", "/settings/time")],
        systemGroup: [makeItem("Storage", "/settings/storage")],
      })
    );

    render(<SettingsSidebar />);
    expect(screen.getByTestId("settings-section-organization")).toBeInTheDocument();
    expect(screen.getByTestId("settings-section-access")).toBeInTheDocument();
    expect(
      screen.getByTestId("settings-section-operations")
    ).toBeInTheDocument();
    expect(screen.getByTestId("settings-section-system")).toBeInTheDocument();
    expect(screen.getByText("General")).toBeInTheDocument();
    expect(screen.getByText("Roles")).toBeInTheDocument();
    expect(screen.getByText("Time")).toBeInTheDocument();
    expect(screen.getByText("Storage")).toBeInTheDocument();
  });

  it("hides empty sections when no items exist in that section", () => {
    vi.spyOn(hooks, "useSettingsNav").mockReturnValue(
      makeNavState({
        organizationGroup: [makeItem("General", "/settings/general")],
        accessGroup: [],
        operationsGroup: [],
        systemGroup: [],
      })
    );

    render(<SettingsSidebar />);
    expect(screen.getByTestId("settings-section-organization")).toBeInTheDocument();
    expect(
      screen.queryByTestId("settings-section-access")
    ).not.toBeInTheDocument();
    expect(
      screen.queryByTestId("settings-section-operations")
    ).not.toBeInTheDocument();
    expect(
      screen.queryByTestId("settings-section-system")
    ).not.toBeInTheDocument();
  });

  it("filters setting items in real-time when typing in search input", () => {
    vi.spyOn(hooks, "useSettingsNav").mockReturnValue(
      makeNavState({
        organizationGroup: [
          makeItem("Cài đặt chung", "/settings/general"),
          makeItem("Thanh toán", "/settings/billing"),
        ],
        accessGroup: [makeItem("Phân quyền", "/settings/roles")],
      })
    );

    render(<SettingsSidebar />);
    const searchInput = screen.getByTestId("settings-search-input");
    expect(screen.getByText("Cài đặt chung")).toBeInTheDocument();
    expect(screen.getByText("Thanh toán")).toBeInTheDocument();
    expect(screen.getByText("Phân quyền")).toBeInTheDocument();

    // Type "chung" without diacritics
    fireEvent.change(searchInput, { target: { value: "chung" } });

    expect(screen.getByText("Cài đặt chung")).toBeInTheDocument();
    expect(screen.queryByText("Thanh toán")).not.toBeInTheDocument();
    expect(screen.queryByText("Phân quyền")).not.toBeInTheDocument();
  });

  it("shows no results message when search matches nothing", () => {
    vi.spyOn(hooks, "useSettingsNav").mockReturnValue(
      makeNavState({
        organizationGroup: [makeItem("General", "/settings/general")],
      })
    );

    render(<SettingsSidebar />);
    const searchInput = screen.getByTestId("settings-search-input");
    fireEvent.change(searchInput, { target: { value: "NonExistentSettingXYZ" } });

    expect(screen.getByText("Không tìm thấy cài đặt phù hợp")).toBeInTheDocument();
  });
});
