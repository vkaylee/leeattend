import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import OnboardingPage from "../OnboardingPage";
import React from "react";
import { ROUTES } from "@/lib/routes";

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
    render(<OnboardingPage />);
    expect(screen.getByText("onboarding.welcome_title")).toBeDefined();
  });

  it("handles logout to prevent regression", async () => {
    // Mock window.location
    const originalLocation = window.location;
    delete (window as any).location;
    window.location = { ...originalLocation, href: "http://localhost/" } as any;
    
    // Set dummy localStorage to verify it gets cleared
    localStorage.setItem("test_key", "test_value");

    render(<OnboardingPage />);
    
    // Find the logout button
    const logoutBtn = screen.getByText("common.logout");
    expect(logoutBtn).toBeDefined();

    // Click logout
    fireEvent.click(logoutBtn);

    // Verify localStorage was cleared
    expect(localStorage.getItem("test_key")).toBeNull();
    expect(localStorage.length).toBe(0);

    // Verify redirect
    expect(window.location.href).toBe(ROUTES.LOGIN);

    // Restore window.location
    (window as any).location = originalLocation;
  });
});
