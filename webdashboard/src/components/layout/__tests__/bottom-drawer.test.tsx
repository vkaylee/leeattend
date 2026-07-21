import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import BottomDrawer from "@/components/layout/bottom-drawer";

// ── Mocks ─────────────────────────────────────────────────────────

const mockSetMode = vi.fn();
const mockChangeLanguage = vi.fn();
let mockMode = "light" as const;

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
  useDarkModeContext: () => ({ mode: mockMode, setMode: mockSetMode }),
}));

vi.mock("@/hooks/use-language", () => ({
  useLanguage: () => ({
    currentLanguage: "vi" as const,
    currentOption: { code: "vi", flag: "🇻🇳", name: "Vietnamese", nativeName: "Tiếng Việt" },
    changeLanguage: mockChangeLanguage,
    languages: [
      { code: "vi", flag: "🇻🇳", name: "Vietnamese", nativeName: "Tiếng Việt" },
      { code: "en", flag: "🇺🇸", name: "English", nativeName: "English" },
      { code: "zh", flag: "🇨🇳", name: "Chinese", nativeName: "中文" },
    ],
  }),
}));

vi.mock("@/hooks/use-notifications", () => ({
  useNotifications: () => ({
    notifications: [],
    unreadCount: 0,
    isLoading: false,
  }),
}));

vi.mock("@/hooks/use-focus-trap", () => ({
  useFocusTrap: () => ({ current: null }),
}));

vi.mock("@tanstack/react-router", () => ({
  Link: ({ to, children, ...props }: { to: string; children: React.ReactNode }) => (
    <a href={to} {...props}>{children}</a>
  ),
  useLocation: () => ({ pathname: "/" }),
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
  isOpen: false,
  onClose: vi.fn(),
};

// ── Tests ─────────────────────────────────────────────────────────

describe("BottomDrawer", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockMode = "light";
  });

  // ── Rendering ────────────────────────────────────────────────

  it("does not render when isOpen is false", () => {
    render(<BottomDrawer {...defaultProps} isOpen={false} />, { wrapper: Wrapper });
    expect(screen.queryByTestId("bottom-drawer")).not.toBeInTheDocument();
  });

  it("renders drawer when isOpen is true", async () => {
    render(<BottomDrawer {...defaultProps} isOpen={true} />, { wrapper: Wrapper });
    await waitFor(() => {
      expect(screen.getByTestId("bottom-drawer")).toBeInTheDocument();
    });
  });

  it("has dialog role and aria-modal attribute", async () => {
    render(<BottomDrawer {...defaultProps} isOpen={true} />, { wrapper: Wrapper });
    await waitFor(() => {
      const drawer = screen.getByTestId("bottom-drawer");
      expect(drawer.getAttribute("role")).toBe("dialog");
      expect(drawer.getAttribute("aria-modal")).toBe("true");
    });
  });

  // ── Close behavior ───────────────────────────────────────────

  it("calls onClose when backdrop is clicked", async () => {
    const onClose = vi.fn();
    render(<BottomDrawer {...defaultProps} isOpen={true} onClose={onClose} />, { wrapper: Wrapper });

    await waitFor(() => {
      const drawer = screen.getByTestId("bottom-drawer");
      expect(drawer).toBeInTheDocument();
    });

    // Backdrop is the first div with fixed inset-0
    const backdrop = document.querySelector(".fixed.inset-0.z-\\[1000\\] > div:first-child");
    if (backdrop) {
      fireEvent.click(backdrop);
      expect(onClose).toHaveBeenCalled();
    }
  });

  it("calls onClose when Escape key is pressed", async () => {
    const onClose = vi.fn();
    render(<BottomDrawer {...defaultProps} isOpen={true} onClose={onClose} />, { wrapper: Wrapper });

    await waitFor(() => {
      expect(screen.getByTestId("bottom-drawer")).toBeInTheDocument();
    });

    fireEvent.keyDown(window, { key: "Escape" });
    expect(onClose).toHaveBeenCalled();
  });

  // ── Utility buttons ──────────────────────────────────────────

  it("renders theme toggle button", async () => {
    render(<BottomDrawer {...defaultProps} isOpen={true} />, { wrapper: Wrapper });
    await waitFor(() => {
      expect(screen.getByTestId("bottom-drawer-theme-toggle")).toBeInTheDocument();
    });
  });

  it("renders language toggle button", async () => {
    render(<BottomDrawer {...defaultProps} isOpen={true} />, { wrapper: Wrapper });
    await waitFor(() => {
      expect(screen.getByTestId("bottom-drawer-lang-toggle")).toBeInTheDocument();
    });
  });

  it("renders logout button", async () => {
    render(<BottomDrawer {...defaultProps} isOpen={true} />, { wrapper: Wrapper });
    await waitFor(() => {
      expect(screen.getByTestId("bottom-drawer-logout")).toBeInTheDocument();
    });
  });

  it("calls setMode when theme toggle is clicked", async () => {
    render(<BottomDrawer {...defaultProps} isOpen={true} />, { wrapper: Wrapper });
    await waitFor(() => {
      const themeBtn = screen.getByTestId("bottom-drawer-theme-toggle");
      fireEvent.click(themeBtn);
      expect(mockSetMode).toHaveBeenCalled();
    });
  });

  it("calls changeLanguage when language toggle is clicked", async () => {
    render(<BottomDrawer {...defaultProps} isOpen={true} />, { wrapper: Wrapper });
    await waitFor(() => {
      const langBtn = screen.getByTestId("bottom-drawer-lang-toggle");
      fireEvent.click(langBtn);
      expect(mockChangeLanguage).toHaveBeenCalled();
    });
  });

  // ── Navigation links ─────────────────────────────────────────

  it("renders navigation sections from menu items", async () => {
    render(<BottomDrawer {...defaultProps} isOpen={true} />, { wrapper: Wrapper });
    await waitFor(() => {
      // Should have at least some navigation links
      const links = screen.getAllByRole("link");
      expect(links.length).toBeGreaterThan(0);
    });
  });

  it("calls onClose when a navigation link is clicked", async () => {
    const onClose = vi.fn();
    render(<BottomDrawer {...defaultProps} isOpen={true} onClose={onClose} />, { wrapper: Wrapper });

    await waitFor(() => {
      const links = screen.getAllByRole("link");
      expect(links.length).toBeGreaterThan(0);

      if (links.length > 0) {
        fireEvent.click(links[0]);
        expect(onClose).toHaveBeenCalled();
      }
    });
  });

  // ── Positioning and styling ──────────────────────────────────

  it("has fixed positioning and bottom alignment", async () => {
    render(<BottomDrawer {...defaultProps} isOpen={true} />, { wrapper: Wrapper });
    await waitFor(() => {
      const drawer = screen.getByTestId("bottom-drawer");
      const container = drawer.parentElement;
      expect(container?.className).toContain("fixed");
      expect(container?.className).toContain("inset-0");
    });
  });

  it("has max-height constraint for mobile viewports", async () => {
    render(<BottomDrawer {...defaultProps} isOpen={true} />, { wrapper: Wrapper });
    await waitFor(() => {
      const drawer = screen.getByTestId("bottom-drawer");
      expect(drawer.className).toContain("max-h-");
    });
  });

  it("has rounded top corners", async () => {
    render(<BottomDrawer {...defaultProps} isOpen={true} />, { wrapper: Wrapper });
    await waitFor(() => {
      const drawer = screen.getByTestId("bottom-drawer");
      expect(drawer.className).toContain("rounded-t-");
    });
  });

  // ── Animation classes ────────────────────────────────────────

  it("has transition classes for smooth animation", async () => {
    render(<BottomDrawer {...defaultProps} isOpen={true} />, { wrapper: Wrapper });
    await waitFor(() => {
      const drawer = screen.getByTestId("bottom-drawer");
      expect(drawer.className).toContain("transition-");
    });
  });

  // ── Edge cases ───────────────────────────────────────────────

  it("does not crash when rendered with isOpen=false then isOpen=true", async () => {
    const { rerender } = render(<BottomDrawer {...defaultProps} isOpen={false} />, { wrapper: Wrapper });

    expect(() => {
      rerender(<BottomDrawer {...defaultProps} isOpen={true} />);
    }).not.toThrow();

    await waitFor(() => {
      expect(screen.getByTestId("bottom-drawer")).toBeInTheDocument();
    });
  });

  it("does not crash when onClose is undefined", async () => {
    expect(() => {
      render(<BottomDrawer isOpen={true} onClose={undefined as any} />, { wrapper: Wrapper });
    }).not.toThrow();
  });
});
