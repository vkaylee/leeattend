import { render, screen } from "@testing-library/react";
import React from "react";

import { vi, describe, it, expect, beforeEach } from "vitest";

import * as hooks from "../hooks/use-settings-nav";
import SettingsHubPage from "../settings-hub-page";

// ── Mocks ─────────────────────────────────────────────────────────────

vi.mock("@tanstack/react-router", () => ({
  Link: ({ children, to, className }: any) => (
    <a className={className} href={to}>
      {children}
    </a>
  ),
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, fallback?: string) => fallback ?? key,
  }),
}));

// ── Permission mock ───────────────────────────────────────────────────
let mockPermsLoading = false;

vi.mock("@/hooks/use-permissions", () => ({
  useMyPermissions: () => ({
    hasAnyPermission: vi.fn(() => true),
    permissions: [],
    roles: [],
    managedDepartmentIds: null,
    isLoading: mockPermsLoading,
    hasPermission: vi.fn(),
    hasRole: vi.fn(),
    hasScopedPermission: vi.fn(),
  }),
}));

// ── Helpers ───────────────────────────────────────────────────────────

/** A minimal SettingItem factory matching the real interface. */
function makeItem(label: string, path: string): any {
  return {
    description: `Description for ${label}`,
    icon: <span data-testid={`icon-${label}`} />,
    label,
    path,
  };
}

function makeNavState(overrides: Partial<ReturnType<typeof hooks.useSettingsNav>> = {}) {
  return {
    workspaceGroup: [],
    accessGroup: [],
    operationsGroup: [],
    systemGroup: [],
    ...overrides,
  };
}

// ── Tests ─────────────────────────────────────────────────────────────

describe("SettingsHubPage", () => {
  beforeEach(() => {
    mockPermsLoading = false;
    vi.clearAllMocks();
  });

  // ── Loading state ──────────────────────────────────────────────

  it("shows skeleton when permissions are loading", () => {
    mockPermsLoading = true;
    render(<SettingsHubPage />);
    expect(screen.getByTestId("settings-hub-skeleton")).toBeInTheDocument();
  });

  // ── Empty state ────────────────────────────────────────────────

  it("shows empty state when user has no settings access", () => {
    vi.spyOn(hooks, "useSettingsNav").mockReturnValue(makeNavState());
    render(<SettingsHubPage />);
    expect(screen.getByTestId("settings-hub-empty")).toBeInTheDocument();
  });

  it("shows empty state when all groups are empty", () => {
    vi.spyOn(hooks, "useSettingsNav").mockReturnValue(makeNavState());
    render(<SettingsHubPage />);
    expect(screen.getByTestId("settings-hub-empty")).toBeInTheDocument();
    expect(screen.queryByTestId("settings-section-workspace")).not.toBeInTheDocument();
  });


  // ── Section rendering ──────────────────────────────────────────

  it("renders the workspace section with correct test id", () => {
    vi.spyOn(hooks, "useSettingsNav").mockReturnValue(
      makeNavState({
        workspaceGroup: [
          makeItem("General", "/workspace/settings/general"),
          makeItem("Billing", "/workspace/settings/billing"),
        ],
      }),
    );
    render(<SettingsHubPage />);
    expect(screen.getByTestId("settings-section-workspace")).toBeInTheDocument();
    expect(screen.getByText("General")).toBeInTheDocument();
    expect(screen.getByText("Billing")).toBeInTheDocument();
  });

  it("renders the access section with correct test id", () => {
    vi.spyOn(hooks, "useSettingsNav").mockReturnValue(
      makeNavState({
        accessGroup: [
          makeItem("Roles", "/workspace/settings/roles"),
          makeItem("Permissions", "/workspace/settings/permissions"),
        ],
      }),
    );
    render(<SettingsHubPage />);
    expect(screen.getByTestId("settings-section-access")).toBeInTheDocument();
    expect(screen.getByText("Roles")).toBeInTheDocument();
    expect(screen.getByText("Permissions")).toBeInTheDocument();
  });

  it("renders the operations section with correct test id", () => {
    vi.spyOn(hooks, "useSettingsNav").mockReturnValue(
      makeNavState({
        operationsGroup: [
          makeItem("Time", "/workspace/settings/time"),
          makeItem("Payroll", "/workspace/settings/payroll"),
        ],
      }),
    );
    render(<SettingsHubPage />);
    expect(screen.getByTestId("settings-section-operations")).toBeInTheDocument();
    expect(screen.getByText("Time")).toBeInTheDocument();
    expect(screen.getByText("Payroll")).toBeInTheDocument();
  });

  it("renders the system section with correct test id", () => {
    vi.spyOn(hooks, "useSettingsNav").mockReturnValue(
      makeNavState({
        systemGroup: [
          makeItem("Audit Logs", "/workspace/settings/audit-logs"),
        ],
      }),
    );
    render(<SettingsHubPage />);
    expect(screen.getByTestId("settings-section-system")).toBeInTheDocument();
    expect(screen.getByText("Audit Logs")).toBeInTheDocument();
  });

  // ── Empty sections are hidden ──────────────────────────────────

  it("hides workspace section when it has no items", () => {
    vi.spyOn(hooks, "useSettingsNav").mockReturnValue(
      makeNavState({
        workspaceGroup: [],
        accessGroup: [makeItem("Roles", "/workspace/settings/roles")],
      }),
    );
    render(<SettingsHubPage />);
    expect(screen.queryByTestId("settings-section-workspace")).not.toBeInTheDocument();
    expect(screen.getByTestId("settings-section-access")).toBeInTheDocument();
  });

  it("hides all empty sections and only shows populated ones", () => {
    vi.spyOn(hooks, "useSettingsNav").mockReturnValue(
      makeNavState({
        workspaceGroup: [makeItem("General", "/workspace/settings/general")],
        accessGroup: [],
        operationsGroup: [],
      }),
    );
    render(<SettingsHubPage />);

    expect(screen.getByTestId("settings-section-workspace")).toBeInTheDocument();
    expect(screen.queryByTestId("settings-section-access")).not.toBeInTheDocument();
    expect(screen.queryByTestId("settings-section-operations")).not.toBeInTheDocument();
    expect(screen.queryByTestId("settings-section-system")).not.toBeInTheDocument();
  });


  // ── All sections together ──────────────────────────────────────

  it("renders all four sections simultaneously", () => {
    vi.spyOn(hooks, "useSettingsNav").mockReturnValue(
      makeNavState({
        workspaceGroup: [makeItem("General", "/workspace/settings/general")],
        accessGroup: [makeItem("Roles", "/workspace/settings/roles")],
        operationsGroup: [makeItem("Time", "/workspace/settings/time")],
        systemGroup: [makeItem("Audit Logs", "/workspace/settings/audit-logs")],
      }),
    );
    render(<SettingsHubPage />);

    expect(screen.getByTestId("settings-section-workspace")).toBeInTheDocument();
    expect(screen.getByTestId("settings-section-access")).toBeInTheDocument();
    expect(screen.getByTestId("settings-section-operations")).toBeInTheDocument();
    expect(screen.getByTestId("settings-section-system")).toBeInTheDocument();
    expect(screen.getByText("General")).toBeInTheDocument();
    expect(screen.getByText("Roles")).toBeInTheDocument();
    expect(screen.getByText("Time")).toBeInTheDocument();
    expect(screen.getByText("Audit Logs")).toBeInTheDocument();
  });

  // ── Header ─────────────────────────────────────────────────────

  it("renders the header with title and control panel subtitle", () => {
    vi.spyOn(hooks, "useSettingsNav").mockReturnValue(
      makeNavState({
        workspaceGroup: [makeItem("General", "/workspace/settings/general")],
      }),
    );
    render(<SettingsHubPage />);
    // The Typography component renders the i18n key as fallback
    expect(screen.getByText("settings.layout.control_panel")).toBeInTheDocument();
  });
});
