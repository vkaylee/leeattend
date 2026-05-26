import { render, screen } from "@testing-library/react";

import { vi, describe, it, expect, beforeEach } from "vitest";

import SettingsLayout from "../settings-layout";
import * as hooks from "../hooks/use-settings-nav";

vi.mock("@tanstack/react-router", () => ({
  Link: ({ children, to, className }: any) => <a className={className} href={to}>{children}</a>,
  Outlet: () => <div data-testid="settings-outlet" />,
  useLocation: () => ({ pathname: "/workspace/settings/general" }),
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

describe("SettingsLayout Page", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders the layout with mocked nav groups", () => {
    vi.spyOn(hooks, "useSettingsNav").mockReturnValue({
      workspaceGroup: [
        { label: "General", path: "/workspace/settings/general", description: "Desc", icon: <span /> }
      ],
      accessGroup: [],
      operationsGroup: [],
      systemGroup: [],
      dangerItem: { label: "Danger", path: "/workspace/settings/danger", description: "Desc", icon: <span /> }
    });

    render(<SettingsLayout />);

    expect(screen.getByText("settings.layout.workspace")).toBeDefined();
    expect(screen.getByText("General")).toBeDefined();
    expect(screen.getByText("Danger")).toBeDefined();
    expect(screen.getByTestId("settings-outlet")).toBeDefined();
  });

  it("handles empty nav groups correctly", () => {
    vi.spyOn(hooks, "useSettingsNav").mockReturnValue({
      workspaceGroup: [],
      accessGroup: [],
      operationsGroup: [],
      systemGroup: [],
      dangerItem: null
    });

    render(<SettingsLayout />);

    expect(screen.queryByText("settings.layout.workspace")).toBeNull();
    expect(screen.queryByText("Danger")).toBeNull();
  });
});
