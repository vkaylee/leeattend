import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen } from "@testing-library/react";
import React from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

let mockLocationPath = "/";

vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => vi.fn(),
  useLocation: () => ({ pathname: mockLocationPath, search: "", hash: "" }),
  Link: ({
    to,
    children,
    ...props
  }: {
    to: string;
    children: React.ReactNode;
    [key: string]: unknown;
  }) => (
    <a href={to} {...props}>
      {children}
    </a>
  ),
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, defaultVal?: string) => defaultVal ?? key,
  }),
}));

let mockUnreadCount = 0;
vi.mock("@/hooks/use-notifications", () => ({
  useNotifications: () => ({
    notifications: [],
    unreadCount: mockUnreadCount,
    isLoading: false,
  }),
}));

vi.mock("@/hooks/use-profile", () => ({
  useProfile: () => ({
    user: { full_name: "Tony Stark", email: "tony@stark.com" },
    isLoading: false,
  }),
}));

vi.mock("@/context/use-dark-mode-context", () => ({
  useDarkModeContext: () => ({
    mode: "light",
    setMode: vi.fn(),
    resolved: "light",
    isDark: false,
  }),
}));

vi.mock("@/components/ui/language-switcher", () => ({
  LanguageSwitcher: () => <div data-testid="language-switcher-mock" />,
}));

import FloatingDock from "../floating-dock";

const createTestQueryClient = () =>
  new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });

function renderFloatingDock(onOpenLauncher = vi.fn()) {
  return render(
    <QueryClientProvider client={createTestQueryClient()}>
      <FloatingDock onOpenLauncher={onOpenLauncher} />
    </QueryClientProvider>
  );
}

describe("FloatingDock Component", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockLocationPath = "/";
    mockUnreadCount = 0;
  });

  it("renders navigation container with accessible role and label", () => {
    renderFloatingDock();
    const nav = screen.getByRole("navigation", { name: "Navigation" });
    expect(nav).toBeInTheDocument();
    expect(screen.getByTestId("floating-dock")).toBeInTheDocument();
  });

  it("renders DSM 7 Launcher button and triggers onOpenLauncher when clicked", () => {
    const onOpen = vi.fn();
    renderFloatingDock(onOpen);

    const launcherBtn = screen.getByTestId("dock-dsm-launcher-btn");
    expect(launcherBtn).toBeInTheDocument();
    fireEvent.click(launcherBtn);
    expect(onOpen).toHaveBeenCalledTimes(1);
  });

  it("renders all core application shortcut icons", () => {
    renderFloatingDock();

    expect(screen.getByTestId("dock-app-dashboard")).toBeInTheDocument();
    expect(screen.getByTestId("dock-app-org")).toBeInTheDocument();
    expect(screen.getByTestId("dock-app-time")).toBeInTheDocument();
    expect(screen.getByTestId("dock-app-leave")).toBeInTheDocument();
    expect(screen.getByTestId("dock-app-payroll")).toBeInTheDocument();
    expect(screen.getByTestId("dock-app-settings")).toBeInTheDocument();
  });

  it("highlights settings app icon when on /workspace/settings route", () => {
    mockLocationPath = "/workspace/settings/general";
    renderFloatingDock();

    const settingsApp = screen.getByTestId("dock-app-settings");
    expect(settingsApp.className).toContain("bg-primary-50");
  });
  it("renders unread notifications badge when unreadCount > 0", () => {
    mockUnreadCount = 7;
    renderFloatingDock();

    const notif = screen.getByTestId("dock-notifications");
    expect(notif).toBeInTheDocument();
    expect(notif.textContent).toContain("7");
  });

  it("renders Profile link and Logout button complying with SaaS test contracts", () => {
    const clearSpy = vi.spyOn(Storage.prototype, "clear");
    renderFloatingDock();

    const profileLink = screen.getByRole("link", { name: "Profile" });
    expect(profileLink).toBeInTheDocument();
    expect(profileLink.getAttribute("href")).toBe("/ess/profile");

    const logoutBtn = screen.getByRole("button", { name: "Logout" });
    expect(logoutBtn).toBeInTheDocument();
    expect(logoutBtn.getAttribute("aria-label")).toBe("Logout");

    fireEvent.click(logoutBtn);
    expect(clearSpy).toHaveBeenCalled();
  });

  it("displays user initial in avatar", () => {
    renderFloatingDock();
    const avatar = screen.getByTestId("dock-user-avatar");
    expect(avatar.textContent).toContain("T");
  });
});
