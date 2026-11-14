import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import AppShell from "../app-shell";

// Mock TanStack Router
let mockPathname = "/dashboard";
vi.mock("@tanstack/react-router", () => ({
  Outlet: () => <div data-testid="mock-router-outlet">Router Content</div>,
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
  useLocation: () => ({ pathname: mockPathname }),
  useNavigate: () => vi.fn(),
}));

// Mock hooks
vi.mock("@/hooks/use-theme", () => ({
  useTheme: vi.fn(),
}));

vi.mock("@/hooks/use-i18n-sync", () => ({
  useI18nSync: vi.fn(),
}));

vi.mock("@/hooks/use-organization", () => ({
  useOrganization: () => ({
    organization: { name: "Acme Corp" },
  }),
}));

vi.mock("@/hooks/use-permissions", () => ({
  useMyPermissions: () => ({
    hasAnyPermission: () => true,
    hasPermission: () => true,
  }),
}));

vi.mock("@/context/use-dark-mode-context", () => ({
  useDarkModeContext: () => ({
    isDark: false,
    setMode: vi.fn(),
  }),
}));

vi.mock("@/hooks/use-notifications", () => ({
  useNotifications: () => ({
    unreadCount: 0,
  }),
}));

vi.mock("@/hooks/use-profile", () => ({
  useProfile: () => ({
    user: { id: "user-1", full_name: "Admin", email: "admin@example.com" },
  }),
}));

vi.mock("@/components/ui/language-switcher", () => ({
  LanguageSwitcher: () => <div>LANG</div>,
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, defaultVal?: unknown) => {
      if (defaultVal && typeof defaultVal === "object" && "defaultValue" in defaultVal) {
        const val = defaultVal.defaultValue;
        if (typeof val === "string") return val;
      }
      if (typeof defaultVal === "string") return defaultVal;
      return key;
    },
    i18n: { language: "vi" },
  }),
}));

describe("AppShell Component", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    localStorage.clear();
    mockPathname = "/dashboard";
  });

  it("renders Sidebar, Topbar, and Router Outlet viewport with responsive padding gutters", () => {
    render(<AppShell />);

    expect(screen.getByTestId("sidebar-container")).toBeInTheDocument();
    expect(screen.getByTestId("topbar-container")).toBeInTheDocument();
    const viewport = screen.getByTestId("main-content-viewport");
    expect(viewport).toBeInTheDocument();
    expect(viewport).toHaveClass("p-4", "md:p-6", "lg:p-8");
    expect(screen.getByTestId("mock-router-outlet")).toBeInTheDocument();
  });

  it("opens CommandPalette when Cmd+K shortcut is pressed", () => {
    render(<AppShell />);

    expect(screen.queryByTestId("command-palette-modal")).not.toBeInTheDocument();

    fireEvent.keyDown(window, { key: "k", metaKey: true });
    expect(screen.getByTestId("command-palette-modal")).toBeInTheDocument();

    fireEvent.keyDown(window, { key: "Escape" });
    expect(screen.queryByTestId("command-palette-modal")).not.toBeInTheDocument();
  });

  it("opens command palette from the global open event", async () => {
    render(<AppShell />);

    window.dispatchEvent(new CustomEvent("open-command-palette"));
    await waitFor(() => {
      expect(screen.getByTestId("command-palette-modal")).toBeInTheDocument();
    });
  });

  it("opens mobile drawer when mobile hamburger button is clicked", () => {
    render(<AppShell />);

    expect(screen.queryByTestId("sidebar-mobile-drawer")).not.toBeInTheDocument();

    const mobileBtn = screen.getByTestId("topbar-mobile-menu-btn");
    fireEvent.click(mobileBtn);

    expect(screen.getByTestId("sidebar-mobile-drawer")).toBeInTheDocument();
  });

  it("hides main Sidebar in Focus Mode when navigating to /settings", () => {
    mockPathname = "/settings/general";
    render(<AppShell />);

    expect(screen.queryByTestId("sidebar-container")).not.toBeInTheDocument();
    expect(screen.getByTestId("topbar-container")).toBeInTheDocument();
    expect(screen.getByTestId("main-content-viewport")).toHaveClass("overflow-hidden");
  });
});
