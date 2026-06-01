import { render, screen } from "@testing-library/react";
import React from "react";

import { vi, describe, it, expect, beforeEach } from "vitest";

import * as hooks from "../hooks/use-settings-nav";
import SettingsLayout from "../settings-layout";

const mockUseLocation = vi.fn();
vi.mock("@tanstack/react-router", () => ({
  Link: ({ children, to, className }: any) => <a className={className} href={to}>{children}</a>,
  Outlet: () => <div data-testid="settings-outlet" />,
  useLocation: () => mockUseLocation(),
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

describe("SettingsLayout Page", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUseLocation.mockReturnValue({ pathname: "/workspace/settings/general" });
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

  it("hides the sidebar and expands layout to full width on workflow editor paths", () => {
    // 1. Mock the route as a new workflow builder
    mockUseLocation.mockReturnValue({ pathname: "/workspace/settings/workflows/new" });
    
    vi.spyOn(hooks, "useSettingsNav").mockReturnValue({
      workspaceGroup: [
        { label: "General", path: "/workspace/settings/general", description: "Desc", icon: <span /> }
      ],
      accessGroup: [],
      operationsGroup: [],
      systemGroup: [],
      dangerItem: null
    });

    const { container } = render(<SettingsLayout />);

    // 2. Sidebar should be completely hidden, meaning workspace settings heading is not rendered
    expect(screen.queryByText("settings.layout.workspace")).toBeNull();
    
    // 3. Outlet container should still render inside the layout
    expect(screen.getByTestId("settings-outlet")).toBeDefined();

    // 4. Outer container must NOT have flex/gap class layout to allow full-width expansion
    const outerContainer = container.firstChild as HTMLElement;
    expect(outerContainer.className).not.toContain("flex-col");
    expect(outerContainer.className).not.toContain("lg:flex-row");
    expect(outerContainer.className).not.toContain("gap-8");
  });

  it("hides the sidebar and expands layout to full width on workflow edit paths", () => {
    // 1. Mock the route as an edit workflow builder
    mockUseLocation.mockReturnValue({ pathname: "/workspace/settings/workflows/wf-123/edit" });
    
    vi.spyOn(hooks, "useSettingsNav").mockReturnValue({
      workspaceGroup: [
        { label: "General", path: "/workspace/settings/general", description: "Desc", icon: <span /> }
      ],
      accessGroup: [],
      operationsGroup: [],
      systemGroup: [],
      dangerItem: null
    });

    const { container } = render(<SettingsLayout />);

    // 2. Sidebar should be completely hidden
    expect(screen.queryByText("settings.layout.workspace")).toBeNull();
    
    // 3. Outlet container should render
    expect(screen.getByTestId("settings-outlet")).toBeDefined();

    // 4. Outer container must NOT have flex/gap class layout
    const outerContainer = container.firstChild as HTMLElement;
    expect(outerContainer.className).not.toContain("flex-col");
    expect(outerContainer.className).not.toContain("lg:flex-row");
    expect(outerContainer.className).not.toContain("gap-8");
  });
});
