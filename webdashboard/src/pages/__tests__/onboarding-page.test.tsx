import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import { ROUTES } from "@/lib/routes";

import OnboardingPage from "../onboarding-page";

// Mock router
vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => vi.fn(),
}));

// Mock hooks
vi.mock("@/hooks/use-workspace", () => ({
  useWorkspaces: () => ({
    createWorkspace: vi.fn().mockResolvedValue({ id: "ws-1" }),
  }),
}));

// Mock API
vi.mock("@/lib/api", () => ({
  setCurrentWorkspaceId: vi.fn(),
}));

// Mock i18n
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}));

describe("OnboardingPage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    localStorage.clear();
  });

  it("renders correctly", () => {
    const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    render(
      <QueryClientProvider client={queryClient}>
        <OnboardingPage />
      </QueryClientProvider>
    );
    expect(screen.getByText("onboarding.welcome_title")).toBeDefined();
  });

  it("handles logout to prevent regression", () => {
    const locationMock = new URL("http://localhost/");
    const originalLocation = window.location;

    Object.defineProperty(window, "location", {
      configurable: true,
      value: {
        href: locationMock.href,
        assign: vi.fn(),
        replace: vi.fn(),
        reload: vi.fn(),
        pathname: locationMock.pathname,
        search: locationMock.search,
        hash: locationMock.hash,
        origin: locationMock.origin,
      },
    });
    
    // Set dummy localStorage to verify it gets cleared
    localStorage.setItem("test_key", "test_value");

    const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    render(
      <QueryClientProvider client={queryClient}>
        <OnboardingPage />
      </QueryClientProvider>
    );
    
    // Find the logout button
    const logoutButton = screen.getByText("common.logout");
    expect(logoutButton).toBeDefined();

    // Click logout
    fireEvent.click(logoutButton);
 

    // Verify localStorage was cleared
    expect(localStorage.getItem("test_key")).toBeNull();
    expect(localStorage.length).toBe(0);

    // Verify redirect
    expect(window.location.href).toBe(ROUTES.LOGIN);

    // Restore window.location
    Object.defineProperty(window, "location", {
      configurable: true,
      value: originalLocation,
    });
  });
});
