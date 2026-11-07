import { render, screen, fireEvent } from "@testing-library/react";
import React from "react";
import { describe, it, expect, vi, beforeEach } from "vitest";

import AppShell from "../app-shell";

// Mock TanStack Router
vi.mock("@tanstack/react-router", () => ({
  Outlet: () => <div data-testid="mock-router-outlet">Router Content</div>,
  Link: ({
    children,
    to,
    className,
    ...props
  }: React.AnchorHTMLAttributes<HTMLAnchorElement> & { to?: string }) => (
    <a href={to} className={className} {...props}>
      {children}
    </a>
  ),
  useLocation: () => ({ pathname: "/dashboard" }),
  useNavigate: () => vi.fn(),
}));

// Mock hooks
vi.mock("@/hooks/use-theme", () => ({
  useTheme: vi.fn(),
}));

vi.mock("@/hooks/use-i18n-sync", () => ({
  useI18nSync: vi.fn(),
}));

vi.mock("@/hooks/use-workspace", () => ({
  useWorkspace: () => ({
    workspace: { id: "ws-1", name: "Acme Corp" },
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
  });

  it("renders Sidebar, Topbar, and Router Outlet viewport", () => {
    render(<AppShell />);

    expect(screen.getByTestId("sidebar-container")).toBeInTheDocument();
    expect(screen.getByTestId("topbar-container")).toBeInTheDocument();
    expect(screen.getByTestId("main-content-viewport")).toBeInTheDocument();
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

  it("opens mobile drawer when mobile hamburger button is clicked", () => {
    render(<AppShell />);

    expect(screen.queryByTestId("sidebar-mobile-drawer")).not.toBeInTheDocument();

    const mobileBtn = screen.getByTestId("topbar-mobile-menu-btn");
    fireEvent.click(mobileBtn);

    expect(screen.getByTestId("sidebar-mobile-drawer")).toBeInTheDocument();
  });
});
