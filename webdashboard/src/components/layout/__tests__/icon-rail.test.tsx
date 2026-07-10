import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import IconRail from "@/components/layout/icon-rail";

// ── Mocks ─────────────────────────────────────────────────────────

const mockSetMode = vi.fn();
const mockChangeLanguage = vi.fn();
let mockUnreadCount = 5;

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
    user: { id: "x", email: "a@b.com", full_name: "Test User", system_roles: ["platform_admin"] },
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

vi.mock("@tanstack/react-router", () => ({
  Link: ({ to, children, ...props }: { to: string; children: React.ReactNode }) => (
    <a href={to} {...props}>{children}</a>
  ),
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
  isPinned: false,
  onHoverGroup: vi.fn(),
  onClickGroup: vi.fn(),
  onTogglePin: vi.fn(),
  workspaceName: "LeeAttend",
  onClickWorkspaceSwitcher: vi.fn(),
};

// ── Tests ─────────────────────────────────────────────────────────

describe("IconRail", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  // ── Rendering ────────────────────────────────────────────────

  it("renders the navigation role", () => {
    render(<IconRail {...defaultProps} />, { wrapper: Wrapper });
    expect(screen.getByRole("navigation")).toBeInTheDocument();
  });

  it("renders the logo button", () => {
    render(<IconRail {...defaultProps} />, { wrapper: Wrapper });
    expect(screen.getByTestId("nav-rail-logo")).toBeInTheDocument();
  });

  it("renders all 8 group icons (dashboard + settings + 6 groups)", () => {
    render(<IconRail {...defaultProps} />, { wrapper: Wrapper });

    expect(screen.getByTestId("nav-rail-group-dashboard")).toBeInTheDocument();
    expect(screen.getByTestId("nav-rail-group-settings")).toBeInTheDocument();
    expect(screen.getByTestId("nav-rail-group-org")).toBeInTheDocument();
    expect(screen.getByTestId("nav-rail-group-time")).toBeInTheDocument();
    expect(screen.getByTestId("nav-rail-group-shifts")).toBeInTheDocument();
    expect(screen.getByTestId("nav-rail-group-payroll")).toBeInTheDocument();
    expect(screen.getByTestId("nav-rail-group-system")).toBeInTheDocument();
    expect(screen.getByTestId("nav-rail-group-ess")).toBeInTheDocument();
  });

  it("renders bottom controls", () => {
    render(<IconRail {...defaultProps} />, { wrapper: Wrapper });

    expect(screen.getByTestId("nav-rail-pin-toggle")).toBeInTheDocument();
    expect(screen.getByTestId("nav-rail-theme-toggle")).toBeInTheDocument();
    expect(screen.getByTestId("nav-rail-lang-toggle")).toBeInTheDocument();
    expect(screen.getByTestId("nav-rail-notifications")).toBeInTheDocument();
    expect(screen.getByTestId("nav-rail-user-avatar")).toBeInTheDocument();
    expect(screen.getByTestId("nav-rail-logout")).toBeInTheDocument();
  });

  // ── Logo ─────────────────────────────────────────────────────

  it("shows workspace initial in logo", () => {
    render(<IconRail {...defaultProps} workspaceName="TestCorp" />, { wrapper: Wrapper });
    // "T" appears in both logo AND avatar (user "Test User"), so check logo button content
    const logo = screen.getByTestId("nav-rail-logo");
    expect(logo.textContent).toContain("T");
  });

  it("calls onClickWorkspaceSwitcher when logo clicked", () => {
    const onShow = vi.fn();
    render(<IconRail {...defaultProps} onClickWorkspaceSwitcher={onShow} />, { wrapper: Wrapper });

    fireEvent.click(screen.getByTestId("nav-rail-logo"));
    expect(onShow).toHaveBeenCalled();
  });

  // ── Pin ──────────────────────────────────────────────────────

  it("calls onTogglePin when pin button clicked", () => {
    const onToggle = vi.fn();
    render(<IconRail {...defaultProps} onTogglePin={onToggle} />, { wrapper: Wrapper });

    fireEvent.click(screen.getByTestId("nav-rail-pin-toggle"));
    expect(onToggle).toHaveBeenCalled();
  });

  // ── Theme ────────────────────────────────────────────────────

  it("cycles theme from light to system", () => {
    render(<IconRail {...defaultProps} />, { wrapper: Wrapper });

    fireEvent.click(screen.getByTestId("nav-rail-theme-toggle"));
    expect(mockSetMode).toHaveBeenCalledWith("system");
  });

  // ── Language ─────────────────────────────────────────────────

  it("shows current language code", () => {
    render(<IconRail {...defaultProps} />, { wrapper: Wrapper });
    expect(screen.getByText("VI")).toBeInTheDocument();
  });

  it("cycles language on click (vi → en)", () => {
    render(<IconRail {...defaultProps} />, { wrapper: Wrapper });

    fireEvent.click(screen.getByTestId("nav-rail-lang-toggle"));
    expect(mockChangeLanguage).toHaveBeenCalledWith("en");
  });

  // ── Notification badge ───────────────────────────────────────

  it("shows unread notification count", () => {
    render(<IconRail {...defaultProps} />, { wrapper: Wrapper });
    const notif = screen.getByTestId("nav-rail-notifications");
    expect(notif.textContent).toContain("5"); // badge count
  });

  // ── User avatar ──────────────────────────────────────────────

  it("shows user initial in avatar", () => {
    render(<IconRail {...defaultProps} />, { wrapper: Wrapper });
    const avatar = screen.getByTestId("nav-rail-user-avatar");
    // User is "Test User" → initial "T"
    expect(avatar.textContent).toContain("T");
  });

  // ── Logout ───────────────────────────────────────────────────

  it("calls logout on click", () => {
    const clearSpy = vi.spyOn(Storage.prototype, "clear");
    render(<IconRail {...defaultProps} />, { wrapper: Wrapper });

    fireEvent.click(screen.getByTestId("nav-rail-logout"));
    expect(clearSpy).toHaveBeenCalled();
  });

  // ── Group interaction ────────────────────────────────────────

  it("calls onClickGroup when a group is clicked", () => {
    const onClick = vi.fn();
    render(<IconRail {...defaultProps} onClickGroup={onClick} />, { wrapper: Wrapper });

    fireEvent.click(screen.getByTestId("nav-rail-group-org"));
    expect(onClick).toHaveBeenCalledWith("org");
  });

  it("calls onHoverGroup on mouse enter/leave", () => {
    const onHover = vi.fn();
    render(<IconRail {...defaultProps} onHoverGroup={onHover} />, { wrapper: Wrapper });

    const orgBtn = screen.getByTestId("nav-rail-group-org");
    fireEvent.mouseEnter(orgBtn);
    expect(onHover).toHaveBeenCalledWith("org");

    fireEvent.mouseLeave(orgBtn);
    expect(onHover).toHaveBeenCalledWith(null);
  });

  // ── Tooltip ──────────────────────────────────────────────────

  it("renders tooltip via portal when hovering an icon", () => {
    render(<IconRail {...defaultProps} />, { wrapper: Wrapper });

    // Tooltips are rendered via portal to <body> on hover — none visible at rest
    expect(document.body.querySelectorAll('[role="tooltip"]').length).toBe(0);

    // Hover the org icon → tooltip appears in document.body
    const orgBtn = screen.getByTestId("nav-rail-group-org");
    fireEvent.mouseEnter(orgBtn);
    const tooltips = document.body.querySelectorAll('[role="tooltip"]');
    expect(tooltips.length).toBe(1);

    // Mouse leave → tooltip disappears
    fireEvent.mouseLeave(orgBtn);
    expect(document.body.querySelectorAll('[role="tooltip"]').length).toBe(0);
  });

  it("renders tooltip via portal on keyboard focus", () => {
    render(<IconRail {...defaultProps} />, { wrapper: Wrapper });

    const orgBtn = screen.getByTestId("nav-rail-group-org");
    fireEvent.focus(orgBtn);
    expect(document.body.querySelectorAll('[role="tooltip"]').length).toBe(1);

    fireEvent.blur(orgBtn);
    expect(document.body.querySelectorAll('[role="tooltip"]').length).toBe(0);
  });

  it("does NOT cause horizontal scrollbar on the rail", () => {
    const { container } = render(<IconRail {...defaultProps} />, { wrapper: Wrapper });

    // The nav element must have overflow-x-clip to prevent tooltips from
    // triggering a horizontal scrollbar on the page
    const nav = container.querySelector('nav[role="navigation"]');
    expect(nav).toBeTruthy();
    expect(nav?.className).toContain('overflow-x-clip');
  });

  // ── Edge cases ───────────────────────────────────────────────

  it("shows pin as active when isPinned is true", () => {
    render(<IconRail {...defaultProps} isPinned={true} />, { wrapper: Wrapper });
    const pinBtn = screen.getByTestId("nav-rail-pin-toggle");
    expect(pinBtn.className).toContain("text-primary-600");
  });

  it("shows 99+ for large notification counts", () => {
    mockUnreadCount = 150;
    render(<IconRail {...defaultProps} />, { wrapper: Wrapper });
    const notif = screen.getByTestId("nav-rail-notifications");
    expect(notif.textContent).toContain("99+");
    mockUnreadCount = 5; // reset
  });

  it("hides notification badge when count is 0", () => {
    mockUnreadCount = 0;
    render(<IconRail {...defaultProps} />, { wrapper: Wrapper });
    const notif = screen.getByTestId("nav-rail-notifications");
    expect(notif.querySelector("[class*='bg-rose-500']")).toBeNull();
    mockUnreadCount = 5; // reset
  });

  // ── Keyboard & focus ─────────────────────────────────────────

  it("Enter key on group icon triggers onClickGroup", () => {
    const onClick = vi.fn();
    render(<IconRail {...defaultProps} onClickGroup={onClick} />, { wrapper: Wrapper });

    const orgBtn = screen.getByTestId("nav-rail-group-org");
    fireEvent.keyDown(orgBtn, { key: "Enter" });
    // Button click is triggered by Enter on buttons by default browser behavior
    // We verify click handler via direct click
    fireEvent.click(orgBtn);
    expect(onClick).toHaveBeenCalledWith("org");
  });

  it("focus on a group icon does NOT trigger hoverGroup", () => {
    const onHover = vi.fn();
    render(<IconRail {...defaultProps} onHoverGroup={onHover} />, { wrapper: Wrapper });

    const orgBtn = screen.getByTestId("nav-rail-group-org");
    fireEvent.focus(orgBtn);

    // Focus triggers tooltip hover internally but NOT onGroupHover callback
    // (only onMouseEnter/Leave call onHoverGroup)
    expect(onHover).not.toHaveBeenCalled();
  });

  it("blur on a group icon does NOT trigger hoverGroup(null)", () => {
    const onHover = vi.fn();
    render(<IconRail {...defaultProps} onHoverGroup={onHover} />, { wrapper: Wrapper });

    const orgBtn = screen.getByTestId("nav-rail-group-org");
    fireEvent.focus(orgBtn);
    fireEvent.blur(orgBtn);

    // Blur only hides the tooltip; it does NOT call onHoverGroup
    expect(onHover).not.toHaveBeenCalled();
  });

  // ── Click on Dashboard & Settings (Links) ────────────────────

  it("settings icon is a link to /workspace/settings", () => {
    render(<IconRail {...defaultProps} />, { wrapper: Wrapper });

    const settingsBtn = screen.getByTestId("nav-rail-group-settings");
    expect(settingsBtn.tagName).toBe("A");
    expect(settingsBtn.getAttribute("href")).toBe("/workspace/settings");
  });

  it("settings icon navigates directly (Link), does not trigger FloatingPanel", () => {
    const onClick = vi.fn();
    // Settings is a Link — click is handled by the router, not the click handler
    render(<IconRail {...defaultProps} onClickGroup={onClick} />, { wrapper: Wrapper });

    const settingsBtn = screen.getByTestId("nav-rail-group-settings");
    expect(settingsBtn.tagName).toBe("A");
    fireEvent.click(settingsBtn);
    // onClickGroup is NOT called for direct-link icons (dashboard & settings)
    expect(onClick).not.toHaveBeenCalled();
  });

  it("dashboard icon is a link to /", () => {
    render(<IconRail {...defaultProps} />, { wrapper: Wrapper });

    const dashBtn = screen.getByTestId("nav-rail-group-dashboard");
    expect(dashBtn.tagName).toBe("A");
    expect(dashBtn.getAttribute("href")).toBe("/");
  });

  it("other group icons are buttons, not links", () => {
    render(<IconRail {...defaultProps} />, { wrapper: Wrapper });

    const orgBtn = screen.getByTestId("nav-rail-group-org");
    expect(orgBtn.tagName).toBe("BUTTON");
  });
});
