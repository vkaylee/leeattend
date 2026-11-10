import { render, screen, fireEvent } from "@testing-library/react";
import React from "react";
import { describe, it, expect, vi } from "vitest";

import Button from "@/components/ui/button";
import { Route } from "@/routes/_authenticated/_organization/settings";

interface MockRouteOptions {
  options: {
    component: React.ComponentType;
  };
}

vi.mock("@tanstack/react-router", () => ({
  createFileRoute: () => (config: Record<string, unknown>) => ({
    options: config,
    useLocation: () => ({ pathname: "/settings/general" }),
  }),
  useLocation: () => ({ pathname: "/settings/general" }),
  Outlet: () => <div data-testid="settings-outlet">Settings Outlet Content</div>,
  Link: ({ children, to }: { children: React.ReactNode; to: string }) => <a href={to}>{children}</a>,
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, fallback?: string) => fallback ?? key,
  }),
}));

vi.mock("@/pages/settings/components/settings-sidebar", () => ({
  SettingsSidebar: ({ onItemClick }: { onItemClick?: () => void }) => (
    <div data-testid="mock-settings-sidebar">
      Mock Sidebar
      <Button data-testid="mock-sidebar-link" onClick={onItemClick}>
        Link
      </Button>
    </div>
  ),
}));

describe("SettingsPageLayout (Apple HIG 2-column & Drill-down)", () => {
  it("renders sidebar, detail pane, and outlet", () => {
    const routeWithComponent = Route as unknown as MockRouteOptions;
    const Component = routeWithComponent.options.component;
    render(<Component />);

    expect(screen.getByTestId("mock-settings-sidebar")).toBeInTheDocument();
    expect(screen.getByTestId("settings-outlet")).toBeInTheDocument();
  });
  it("gives the settings sidebar a bounded full-height viewport for mouse and touch scrolling", () => {
    const routeWithComponent = Route as unknown as MockRouteOptions;
    const Component = routeWithComponent.options.component;
    render(<Component />);

    const sidebarPanel = screen.getByTestId("settings-sidebar-panel");
    expect(sidebarPanel).toHaveClass("min-h-0", "h-full");
  });


  it("toggles mobile menu button when clicked", () => {
    const routeWithComponent = Route as unknown as MockRouteOptions;
    const Component = routeWithComponent.options.component;
    render(<Component />);

    const menuToggle = screen.getByRole("button", { name: /danh mục/i });
    expect(menuToggle).toBeInTheDocument();
    expect(menuToggle).toHaveAttribute("aria-expanded", "false");

    fireEvent.click(menuToggle);
    const closeToggle = screen.getByRole("button", { name: /đóng/i });
    expect(closeToggle).toBeInTheDocument();
    expect(closeToggle).toHaveAttribute("aria-expanded", "true");
  });

  it("closes mobile menu when onItemClick is called from SettingsSidebar", () => {
    const routeWithComponent = Route as unknown as MockRouteOptions;
    const Component = routeWithComponent.options.component;
    render(<Component />);

    const menuToggle = screen.getByRole("button", { name: /danh mục/i });
    fireEvent.click(menuToggle);
    expect(screen.getByRole("button", { name: /đóng/i })).toBeInTheDocument();

    const sidebarLink = screen.getByTestId("mock-sidebar-link");
    fireEvent.click(sidebarLink);

    expect(screen.getByRole("button", { name: /danh mục/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /danh mục/i })).toHaveAttribute("aria-expanded", "false");
  });
});
