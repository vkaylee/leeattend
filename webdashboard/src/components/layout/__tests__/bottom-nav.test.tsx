import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import BottomNav from "@/components/layout/bottom-nav";

// ── Mocks ─────────────────────────────────────────────────────────

const mockSetMode = vi.fn();
const mockChangeLanguage = vi.fn();
let mockUnreadCount = 0;

vi.mock("@/hooks/use-permissions", () => ({
  useMyPermissions: () => ({
    hasAnyPermission: vi.fn(() => true),
    permissions: ["employees:read", "shifts:read", "payroll:read", "ess:profile"],
    roles: [],
    managedDepartmentIds: null,
    isLoading: false,
    hasPermission: vi.fn(),
    hasRole: vi.fn(),
    hasScopedPermission: vi.fn(),
  }),
}));

vi.mock("@/hooks/use-profile", () => ({
  useProfile: () => ({
    user: { id: "x", email: "a@b.com", full_name: "Test User" },
    isLoading: false,
  }),
}));

vi.mock("@/context/use-dark-mode-context", () => ({
  useDarkModeContext: () => ({ mode: "light" as const, setMode: mockSetMode }),
}));

vi.mock("@/hooks/use-language", () => ({
  useLanguage: () => ({
    currentLanguage: "vi" as const,
    currentOption: { code: "vi", flag: "🇻🇳", name: "Vietnamese", nativeName: "Tiếng Việt" },
    changeLanguage: mockChangeLanguage,
    languages: [],
  }),
}));

vi.mock("@/hooks/use-notifications", () => ({
  useNotifications: () => ({
    notifications: [],
    unreadCount: mockUnreadCount,
    isLoading: false,
  }),
}));

let mockPathname = "/";

vi.mock("@tanstack/react-router", () => ({
  Link: ({ to, children, ...props }: { to: string; children: React.ReactNode }) => (
    <a href={to} {...props}>{children}</a>
  ),
  useLocation: () => ({ pathname: mockPathname }),
}));

// ── Wrapper ───────────────────────────────────────────────────────

const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });

function Wrapper({ children }: { children: React.ReactNode }) {
  return (
    <QueryClientProvider client={queryClient}>
      {children}
    </QueryClientProvider>
  );
}

// ── Props ─────────────────────────────────────────────────────────

const defaultProps = {
  activeGroup: null as any,
  onMoreClick: vi.fn(),
  unreadCount: 0,
};

// ── Tests ─────────────────────────────────────────────────────────

describe("BottomNav", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUnreadCount = 0;
    localStorage.setItem("ff_ess", "true");
  });

  // ── Rendering ────────────────────────────────────────────────

  it("renders the navigation role", () => {
    render(<BottomNav {...defaultProps} />, { wrapper: Wrapper });
    expect(screen.getByRole("navigation")).toBeInTheDocument();
  });

  it("renders all 5 bottom nav items", () => {
    render(<BottomNav {...defaultProps} />, { wrapper: Wrapper });

    expect(screen.getByTestId("bottom-nav-dashboard")).toBeInTheDocument();
    expect(screen.getByTestId("bottom-nav-time")).toBeInTheDocument();
    expect(screen.getByTestId("bottom-nav-org")).toBeInTheDocument();
    expect(screen.getByTestId("bottom-nav-ess")).toBeInTheDocument();
    expect(screen.getByTestId("bottom-nav-more")).toBeInTheDocument();
  });

  it("has lg:hidden class for mobile-only display", () => {
    const { container } = render(<BottomNav {...defaultProps} />, { wrapper: Wrapper });
    const nav = container.querySelector('nav[role="navigation"]');
    expect(nav?.className).toContain("lg:hidden");
  });

  // ── Active state ─────────────────────────────────────────────

  it("marks dashboard as active when activeGroup is 'dashboard'", () => {
    render(<BottomNav {...defaultProps} activeGroup="dashboard" />, { wrapper: Wrapper });
    const dashBtn = screen.getByTestId("bottom-nav-dashboard");
    const icon = dashBtn.querySelector("svg");
    expect(icon?.className.baseVal ?? icon?.className).toContain("text-primary-600");
  });

  it("marks time as active when activeGroup is 'time'", () => {
    render(<BottomNav {...defaultProps} activeGroup="time" />, { wrapper: Wrapper });
    const timeBtn = screen.getByTestId("bottom-nav-time");
    const icon = timeBtn.querySelector("svg");
    expect(icon?.className.baseVal ?? icon?.className).toContain("text-primary-600");
  });

  it("marks org as active when activeGroup is 'org'", () => {
    render(<BottomNav {...defaultProps} activeGroup="org" />, { wrapper: Wrapper });
    const orgBtn = screen.getByTestId("bottom-nav-org");
    const icon = orgBtn.querySelector("svg");
    expect(icon?.className.baseVal ?? icon?.className).toContain("text-primary-600");
  });

  it("marks ess as active when activeGroup is 'ess'", () => {
    render(<BottomNav {...defaultProps} activeGroup="ess" />, { wrapper: Wrapper });
    const essBtn = screen.getByTestId("bottom-nav-ess");
    const icon = essBtn.querySelector("svg");
    expect(icon?.className.baseVal ?? icon?.className).toContain("text-primary-600");
  });

  it("shows active indicator pill when item is active", () => {
    render(<BottomNav {...defaultProps} activeGroup="dashboard" />, { wrapper: Wrapper });
    const dashBtn = screen.getByTestId("bottom-nav-dashboard");
    const pill = dashBtn.querySelector("span.bg-primary-500");
    expect(pill).toBeInTheDocument();
  });

  it("does not show active indicator pill when item is not active", () => {
    mockPathname = "/some-unknown-path";
    render(<BottomNav {...defaultProps} activeGroup={null} />, { wrapper: Wrapper });
    const dashBtn = screen.getByTestId("bottom-nav-dashboard");
    const pill = dashBtn.querySelector("span.bg-primary-500");
    expect(pill).not.toBeInTheDocument();
    mockPathname = "/";
  });

  // ── More button ──────────────────────────────────────────────

  it("calls onMoreClick when More button is clicked", () => {
    const onMore = vi.fn();
    render(<BottomNav {...defaultProps} onMoreClick={onMore} />, { wrapper: Wrapper });

    fireEvent.click(screen.getByTestId("bottom-nav-more"));
    expect(onMore).toHaveBeenCalled();
  });

  it("More button is a button element, not a link", () => {
    render(<BottomNav {...defaultProps} />, { wrapper: Wrapper });
    const moreBtn = screen.getByTestId("bottom-nav-more");
    expect(moreBtn.tagName).toBe("BUTTON");
  });

  // ── Links ────────────────────────────────────────────────────

  it("dashboard link points to /", () => {
    render(<BottomNav {...defaultProps} />, { wrapper: Wrapper });
    const dashBtn = screen.getByTestId("bottom-nav-dashboard");
    expect(dashBtn.tagName).toBe("A");
    expect(dashBtn.getAttribute("href")).toBe("/");
  });

  it("time link points to first time group item", () => {
    render(<BottomNav {...defaultProps} />, { wrapper: Wrapper });
    const timeBtn = screen.getByTestId("bottom-nav-time");
    expect(timeBtn.tagName).toBe("A");
    // First item in time group is attendance-logs
    expect(timeBtn.getAttribute("href")).toBe("/attendance-logs");
  });

  it("org link points to first org group item", () => {
    render(<BottomNav {...defaultProps} />, { wrapper: Wrapper });
    const orgBtn = screen.getByTestId("bottom-nav-org");
    expect(orgBtn.tagName).toBe("A");
    // First item in org group is employees
    expect(orgBtn.getAttribute("href")).toBe("/employees");
  });

  it("ess link points to first ess group item", () => {
    render(<BottomNav {...defaultProps} />, { wrapper: Wrapper });
    const essBtn = screen.getByTestId("bottom-nav-ess");
    expect(essBtn.tagName).toBe("A");
    // First item in ess group is ess/profile
    expect(essBtn.getAttribute("href")).toBe("/ess/profile");
  });

  // ── Safe area padding ────────────────────────────────────────

  it("has pb-safe class for notch device padding", () => {
    const { container } = render(<BottomNav {...defaultProps} />, { wrapper: Wrapper });
    const nav = container.querySelector('nav[role="navigation"]');
    expect(nav?.className).toContain("pb-safe");
  });

  // ── Fixed bottom positioning ─────────────────────────────────

  it("has fixed bottom positioning classes", () => {
    const { container } = render(<BottomNav {...defaultProps} />, { wrapper: Wrapper });
    const nav = container.querySelector('nav[role="navigation"]');
    expect(nav?.className).toContain("fixed");
    expect(nav?.className).toContain("bottom-0");
  });

  // ── Height and touch targets ─────────────────────────────────

  it("has h-16 class for proper touch target height", () => {
    const { container } = render(<BottomNav {...defaultProps} />, { wrapper: Wrapper });
    const nav = container.querySelector('nav[role="navigation"]');
    const innerDiv = nav?.querySelector("div");
    expect(innerDiv?.className).toContain("h-16");
  });

  // ── Edge cases ───────────────────────────────────────────────

  it("does not crash when activeGroup is null", () => {
    expect(() => {
      render(<BottomNav {...defaultProps} activeGroup={null} />, { wrapper: Wrapper });
    }).not.toThrow();
  });

  it("does not crash when activeGroup is undefined", () => {
    expect(() => {
      render(<BottomNav {...defaultProps} activeGroup={undefined as any} />, { wrapper: Wrapper });
    }).not.toThrow();
  });
});
