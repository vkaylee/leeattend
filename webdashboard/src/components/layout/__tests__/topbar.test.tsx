import { render, screen, fireEvent } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import { Topbar } from "../topbar";

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
  useLocation: () => ({ pathname: "/employees" }),
}));

// Mock dark mode
const mockSetMode = vi.fn();
vi.mock("@/context/use-dark-mode-context", () => ({
  useDarkModeContext: () => ({
    isDark: false,
    setMode: mockSetMode,
  }),
}));

// Mock notifications
vi.mock("@/hooks/use-notifications", () => ({
  useNotifications: () => ({
    unreadCount: 5,
  }),
}));

// Mock profile
vi.mock("@/hooks/use-profile", () => ({
  useProfile: () => ({
    user: { id: "user-1", full_name: "John Doe", email: "john@example.com" },
  }),
}));

// Mock language switcher
vi.mock("@/components/ui/language-switcher", () => ({
  LanguageSwitcher: ({ className }: { className?: string; compact?: boolean }) => (
    <div className={className} data-testid="mock-lang-switcher">
      VI
    </div>
  ),
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

describe("Topbar Layout Component", () => {
  const onOpenSearch = vi.fn();
  const onOpenMobileMenu = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders breadcrumb with organization name and page title", () => {
    render(
      <Topbar
        onOpenMobileMenu={onOpenMobileMenu}
        onOpenSearch={onOpenSearch}
        organizationName="Acme Corp"
      />
    );

    const breadcrumb = screen.getByTestId("topbar-breadcrumb");
    expect(breadcrumb).toBeInTheDocument();
    expect(screen.getByText("Acme Corp")).toBeInTheDocument();
    expect(screen.getByText("Nhân viên")).toBeInTheDocument();
  });

  it("calls onOpenSearch when desktop search bar is clicked", () => {
    render(
      <Topbar
        onOpenMobileMenu={onOpenMobileMenu}
        onOpenSearch={onOpenSearch}
        organizationName="Acme Corp"
      />
    );

    const searchBtn = screen.getByTestId("topbar-search-trigger");
    expect(searchBtn).toBeInTheDocument();
    fireEvent.click(searchBtn);
    expect(onOpenSearch).toHaveBeenCalledTimes(1);
  });

  it("calls onOpenMobileMenu when mobile hamburger button is clicked", () => {
    render(
      <Topbar
        onOpenMobileMenu={onOpenMobileMenu}
        onOpenSearch={onOpenSearch}
        organizationName="Acme Corp"
      />
    );

    const mobileBtn = screen.getByTestId("topbar-mobile-menu-btn");
    expect(mobileBtn).toBeInTheDocument();
    expect(mobileBtn).toHaveClass("xl:hidden");
    fireEvent.click(mobileBtn);
    expect(onOpenMobileMenu).toHaveBeenCalledTimes(1);
  });

  it("renders unread notifications badge", () => {
    render(
      <Topbar
        onOpenMobileMenu={onOpenMobileMenu}
        onOpenSearch={onOpenSearch}
        organizationName="Acme Corp"
      />
    );

    expect(screen.getByText("5")).toBeInTheDocument();
  });

  it("toggles dark mode when theme button is clicked", () => {
    render(
      <Topbar
        onOpenMobileMenu={onOpenMobileMenu}
        onOpenSearch={onOpenSearch}
        organizationName="Acme Corp"
      />
    );

    const themeBtn = screen.getByTestId("topbar-theme-toggle");
    fireEvent.click(themeBtn);
    expect(mockSetMode).toHaveBeenCalledWith("dark");
  });

  it("renders back button instead of mobile hamburger when isSettings is true", () => {
    render(
      <Topbar
        isSettings={true}
        onOpenMobileMenu={onOpenMobileMenu}
        onOpenSearch={onOpenSearch}
        organizationName="Acme Corp"
      />
    );

    const backBtn = screen.getByTestId("topbar-back-btn");
    expect(backBtn).toBeInTheDocument();
    expect(backBtn).toHaveAttribute("href", "/");
    expect(backBtn).toHaveClass("xl:hidden");
    expect(screen.queryByTestId("topbar-mobile-menu-btn")).not.toBeInTheDocument();
  });
});
