import { render, screen, fireEvent } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import { Sidebar } from "../sidebar";

// Mock TanStack Router
vi.mock("@tanstack/react-router", () => ({
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
  useLocation: () => ({ pathname: "/dashboard" }),
}));

vi.mock("@/hooks/use-organization", () => ({
  useOrganization: () => ({
    organization: { name: "Acme Corp" },
  }),
}));

// Mock permissions and nav menu
vi.mock("@/hooks/use-permissions", () => ({
  useMyPermissions: () => ({
    hasAnyPermission: () => true,
    hasPermission: () => true,
  }),
}));

// Mock i18n
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, defaultVal?: unknown) => {
      if (typeof defaultVal === "string") return defaultVal;
      return key;
    },
    i18n: { language: "vi" },
  }),
}));

describe("Sidebar Layout Component", () => {
  const onToggleCollapse = vi.fn();
  const onMobileClose = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders expanded sidebar with brand name and navigation groups", () => {
    render(
      <Sidebar
        isCollapsed={false}
        onToggleCollapse={onToggleCollapse}
      />
    );

    const sidebar = screen.getByTestId("sidebar-container");
    expect(sidebar).toBeInTheDocument();
    expect(sidebar).toHaveClass("w-64", "lg:flex");
    expect(screen.getByText("LeeAttend")).toBeInTheDocument();
    expect(screen.getByText("Acme Corp")).toBeInTheDocument();
    expect(screen.getByTestId("sidebar-link-dashboard")).toBeInTheDocument();
  });

  it("renders collapsed sidebar with w-16 and icon tooltips", () => {
    render(
      <Sidebar
        isCollapsed={true}
        onToggleCollapse={onToggleCollapse}
      />
    );

    const sidebar = screen.getByTestId("sidebar-container");
    expect(sidebar).toBeInTheDocument();
    expect(sidebar).toHaveClass("w-16");
    expect(screen.queryByText("LeeAttend")).not.toBeInTheDocument();
  });

  it("calls onToggleCollapse when collapse button is clicked", () => {
    render(
      <Sidebar
        isCollapsed={false}
        onToggleCollapse={onToggleCollapse}
      />
    );

    const toggleBtn = screen.getByTestId("sidebar-toggle-btn");
    fireEvent.click(toggleBtn);
    expect(onToggleCollapse).toHaveBeenCalledTimes(1);
  });

  it("toggles collapse state with Cmd+B keyboard shortcut", () => {
    render(
      <Sidebar
        isCollapsed={false}
        onToggleCollapse={onToggleCollapse}
      />
    );

    fireEvent.keyDown(window, { key: "b", metaKey: true });
    expect(onToggleCollapse).toHaveBeenCalledTimes(1);

    fireEvent.keyDown(window, { key: "b", ctrlKey: true });
    expect(onToggleCollapse).toHaveBeenCalledTimes(2);
  });

  it("renders mobile drawer when isMobileOpen is true and closes on backdrop click", () => {
    render(
      <Sidebar
        isCollapsed={false}
        isMobileOpen={true}
        onMobileClose={onMobileClose}
        onToggleCollapse={onToggleCollapse}
      />
    );

    expect(screen.getByTestId("sidebar-mobile-drawer")).toHaveClass("lg:hidden");
    const drawer = screen.getByTestId("sidebar-mobile-drawer");
    expect(drawer).toBeInTheDocument();

    const backdrop = screen.getByTestId("sidebar-mobile-backdrop");
    fireEvent.click(backdrop);
    expect(onMobileClose).toHaveBeenCalledTimes(1);
  });

  it("closes the mobile drawer on Escape and restores focus to its trigger", () => {
    const trigger = document.createElement("button");
    document.body.appendChild(trigger);
    trigger.focus();

    const { rerender } = render(
      <Sidebar
        isCollapsed={false}
        isMobileOpen={true}
        onMobileClose={onMobileClose}
        onToggleCollapse={onToggleCollapse}
      />
    );

    expect(document.activeElement).toHaveAttribute("data-testid", "sidebar-logo");
    fireEvent.keyDown(document, { key: "Escape" });
    expect(onMobileClose).toHaveBeenCalledTimes(1);

    rerender(
      <Sidebar
        isCollapsed={false}
        isMobileOpen={false}
        onMobileClose={onMobileClose}
        onToggleCollapse={onToggleCollapse}
      />
    );
    expect(document.activeElement).toBe(trigger);
    trigger.remove();
  });
});
