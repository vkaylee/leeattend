import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import FloatingPanel from "@/components/layout/floating-panel";

// ── Mocks ─────────────────────────────────────────────────────────

const activePermissions = new Set<string>();
const mockHasAnyPermission = vi.fn((...perms: string[]) =>
  perms.length === 0 || perms.some((p) => activePermissions.has(p))
);

vi.mock("@/hooks/use-permissions", () => ({
  useMyPermissions: () => ({
    hasAnyPermission: mockHasAnyPermission,
    permissions: Array.from(activePermissions),
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
    user: { id: "x", email: "a@b.com", full_name: "Test", system_roles: [] },
    isLoading: false,
    isError: false,
    error: null,
    refetch: vi.fn(),
    updateProfile: vi.fn(),
    isUpdating: false,
  }),
}));

// Override useLocation from the global router mock
vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => vi.fn(),
  useParams: () => ({}),
  useSearch: () => ({}),
  useBlocker: () => ({ proceed: vi.fn(), reset: vi.fn(), status: "idle" }),
  createLazyFileRoute: () => (options: unknown) => ({ options }),
  createFileRoute: () => (options: unknown) => ({ options }),
  Outlet: () => null,
  Link: ({ to, children, ...props }: React.AnchorHTMLAttributes<HTMLAnchorElement> & { to?: string }) => <a href={to} {...props}>{children}</a>,
  useLocation: () => ({ pathname: "/employees", search: "", hash: "" }),
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

// ── Helpers ───────────────────────────────────────────────────────

const defaultProps = {
  group: "org" as const,
  isPinned: false,
  isOpen: true,
  onClose: vi.fn(),
  onNavigate: vi.fn(),
  onTogglePin: vi.fn(),
};

// ── Tests ─────────────────────────────────────────────────────────

describe("FloatingPanel", () => {
  beforeEach(() => {
    activePermissions.clear();
    vi.clearAllMocks();
  });

  it("renders nothing when isOpen is false", () => {
    const { container } = render(
      <FloatingPanel {...defaultProps} isOpen={false} />,
      { wrapper: Wrapper },
    );
    expect(container.innerHTML).toBe("");
  });

  it("renders nothing when group is null", () => {
    const { container } = render(
      <FloatingPanel {...defaultProps} group={null} />,
      { wrapper: Wrapper },
    );
    expect(container.innerHTML).toBe("");
  });

  it("renders group name header when open", () => {
    activePermissions.add("employees:read");
    activePermissions.add("members:read");
    activePermissions.add("departments:read");

    render(<FloatingPanel {...defaultProps} />, { wrapper: Wrapper });

    // Group name should appear
    expect(screen.getByRole("menu")).toBeInTheDocument();
    expect(screen.getByRole("heading")).toBeInTheDocument();
  });

  it("uses the lg breakpoint for the desktop panel", () => {
    activePermissions.add("employees:read");
    render(<FloatingPanel {...defaultProps} />, { wrapper: Wrapper });
    expect(screen.getByRole("complementary").className).toContain("hidden lg:flex");
  });

  it("renders sub-items for visible permissions", () => {
    activePermissions.add("employees:read");
    activePermissions.add("members:read");
    activePermissions.add("departments:read");

    render(<FloatingPanel {...defaultProps} />, { wrapper: Wrapper });

    // These should exist because we have the permissions
    expect(screen.getByTestId("sidebar-link-employees")).toBeInTheDocument();
    expect(screen.getByTestId("sidebar-link-departments")).toBeInTheDocument();
    expect(screen.getByTestId("sidebar-link-people")).toBeInTheDocument();
  });

  it("hides sub-items user has no permission for", () => {
    activePermissions.add("employees:read");
    // members:read and departments:read NOT granted

    render(<FloatingPanel {...defaultProps} />, { wrapper: Wrapper });

    expect(screen.getByTestId("sidebar-link-employees")).toBeInTheDocument();
    expect(screen.queryByTestId("sidebar-link-departments")).not.toBeInTheDocument();
    expect(screen.queryByTestId("sidebar-link-people")).not.toBeInTheDocument();
  });

  it("calls onClose when Escape pressed (not pinned)", () => {
    activePermissions.add("employees:read");
    const onClose = vi.fn();

    render(<FloatingPanel {...defaultProps} onClose={onClose} />, {
      wrapper: Wrapper,
    });

    fireEvent.keyDown(document, { key: "Escape" });
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("does not call onClose on Escape when pinned", () => {
    activePermissions.add("employees:read");
    const onClose = vi.fn();

    render(<FloatingPanel {...defaultProps} isPinned={true} onClose={onClose} />, {
      wrapper: Wrapper,
    });

    fireEvent.keyDown(document, { key: "Escape" });
    expect(onClose).not.toHaveBeenCalled();
  });

  it("calls onClose when clicking outside", () => {
    activePermissions.add("employees:read");
    vi.useFakeTimers();
    const onClose = vi.fn();

    render(<FloatingPanel {...defaultProps} onClose={onClose} />, {
      wrapper: Wrapper,
    });

    // Flush the setTimeout(0) that registers the outside-click listener
    vi.advanceTimersByTime(0);

    // Click on document body (outside panel)
    fireEvent.mouseDown(document.body);
    expect(onClose).toHaveBeenCalled();

    vi.useRealTimers();
  });

  it("calls onNavigate when a link is clicked", () => {
    activePermissions.add("employees:read");
    const onNavigate = vi.fn();

    render(<FloatingPanel {...defaultProps} onNavigate={onNavigate} />, {
      wrapper: Wrapper,
    });

    fireEvent.click(screen.getByTestId("sidebar-link-employees"));
    expect(onNavigate).toHaveBeenCalledTimes(1);
  });

  it("calls onTogglePin when pin button is clicked", () => {
    activePermissions.add("employees:read");
    const onTogglePin = vi.fn();

    render(<FloatingPanel {...defaultProps} onTogglePin={onTogglePin} />, {
      wrapper: Wrapper,
    });

    const pinButton = screen.getByTestId("nav-panel-pin-org");
    fireEvent.click(pinButton);
    expect(onTogglePin).toHaveBeenCalledTimes(1);
  });

  it("highlights active route item", () => {
    activePermissions.add("employees:read");

    render(<FloatingPanel {...defaultProps} />, { wrapper: Wrapper });

    const link = screen.getByTestId("sidebar-link-employees");
    expect(link.getAttribute("aria-current")).toBe("page");
  });

  it("shows close button when not pinned", () => {
    activePermissions.add("employees:read");

    render(<FloatingPanel {...defaultProps} isPinned={false} />, { wrapper: Wrapper });

    expect(screen.getByTestId("nav-panel-close-org")).toBeInTheDocument();
  });

  it("hides close button when pinned", () => {
    activePermissions.add("employees:read");

    render(<FloatingPanel {...defaultProps} isPinned={true} />, { wrapper: Wrapper });

    expect(screen.queryByTestId("nav-panel-close-org")).not.toBeInTheDocument();
  });

  it("calls onMouseEnter and onMouseLeave", () => {
    activePermissions.add("employees:read");
    const onMouseEnter = vi.fn();
    const onMouseLeave = vi.fn();

    render(
      <FloatingPanel {...defaultProps} onMouseEnter={onMouseEnter} onMouseLeave={onMouseLeave} />,
      { wrapper: Wrapper },
    );

    const panel = screen.getByRole("menu");
    fireEvent.mouseEnter(panel);
    expect(onMouseEnter).toHaveBeenCalled();

    fireEvent.mouseLeave(panel);
    expect(onMouseLeave).toHaveBeenCalled();
  });

  // ── Focus trap restore ───────────────────────────────────────

  it("closing panel does not trigger focus on rail icons", () => {
    activePermissions.add("employees:read");
    const onClose = vi.fn();

    const { rerender } = render(
      <FloatingPanel {...defaultProps} onClose={onClose} />,
      { wrapper: Wrapper },
    );

    expect(screen.getByTestId("sidebar-link-employees")).toBeInTheDocument();

    // Simulate closing the panel (e.g., by hovering Dashboard icon)
    rerender(
      <FloatingPanel {...defaultProps} isOpen={false} onClose={onClose} />,
    );

    // Panel should be gone — focus trap should have cleaned up
    // without triggering any hover on rail icons
    expect(screen.queryByTestId("sidebar-link-employees")).not.toBeInTheDocument();
  });
});
