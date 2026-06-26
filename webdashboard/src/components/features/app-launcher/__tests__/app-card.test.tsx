import { render, screen, fireEvent } from "@testing-library/react";
import React from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { describe, it, expect, vi } from "vitest";
import { LayoutDashboard } from "lucide-react";

import AppCard from "@/components/features/app-launcher/app-card";
import type { AppDefinition } from "@/lib/app-registry";

// ── Wrapper ───────────────────────────────────────────────────────

const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });

function Wrapper({ children }: { children: React.ReactNode }) {
  return (
    <QueryClientProvider client={queryClient}>
      {children}
    </QueryClientProvider>
  );
}

// ── Fixtures ──────────────────────────────────────────────────────

const dashboardApp: AppDefinition = {
  id: "dashboard",
  icon: LayoutDashboard,
  labelKey: "common.dashboard",
  descriptionKey: "dashboard.welcome",
  to: "/",
  group: "dashboard",
  testId: "app-card-dashboard",
};

const employeeApp: AppDefinition = {
  id: "employees",
  icon: LayoutDashboard,
  labelKey: "common.employees",
  descriptionKey: "dashboard.hr.hr_management_desc",
  to: "/employees",
  group: "org",
  permissions: ["employees:read"],
  testId: "app-card-employees",
};

// ── Tests ─────────────────────────────────────────────────────────

describe("AppCard", () => {
  it("renders with correct testId", () => {
    render(<AppCard app={dashboardApp} onNavigate={vi.fn()} />, { wrapper: Wrapper });

    expect(screen.getByTestId("app-card-dashboard")).toBeInTheDocument();
  });

  it("renders label text (resolved by i18n mock)", () => {
    render(<AppCard app={dashboardApp} onNavigate={vi.fn()} />, { wrapper: Wrapper });

    // i18n mock resolves keys to Vietnamese locale values
    const link = screen.getByTestId("app-card-dashboard");
    expect(link.textContent).toBeTruthy();
    expect(link.textContent).not.toBe("common.dashboard"); // Must have been resolved
  });

  it("uses correct href for navigation", () => {
    render(<AppCard app={employeeApp} onNavigate={vi.fn()} />, { wrapper: Wrapper });

    const link = screen.getByTestId("app-card-employees");
    expect(link.getAttribute("href")).toBe("/employees");
  });

  it("calls onNavigate with app id when clicked", () => {
    const onNavigate = vi.fn();

    render(<AppCard app={dashboardApp} onNavigate={onNavigate} />, { wrapper: Wrapper });

    fireEvent.click(screen.getByTestId("app-card-dashboard"));
    expect(onNavigate).toHaveBeenCalledWith("dashboard");
  });

  it("has aria-label for accessibility", () => {
    render(<AppCard app={employeeApp} onNavigate={vi.fn()} />, { wrapper: Wrapper });

    const link = screen.getByTestId("app-card-employees");
    // aria-label should be the resolved i18n value, not the key
    const ariaLabel = link.getAttribute("aria-label");
    expect(ariaLabel).toBeTruthy();
    expect(ariaLabel).not.toBe("common.employees"); // Must have been resolved
  });

  it("accepts style prop for staggered animation", () => {
    const style = { animationDelay: "60ms" };
    render(<AppCard app={dashboardApp} onNavigate={vi.fn()} style={style} />, {
      wrapper: Wrapper,
    });

    const link = screen.getByTestId("app-card-dashboard");
    expect(link.style.animationDelay).toBe("60ms");
  });

  it("renders an SVG icon", () => {
    const { container } = render(
      <AppCard app={dashboardApp} onNavigate={vi.fn()} />,
      { wrapper: Wrapper },
    );

    const svg = container.querySelector("svg");
    expect(svg).toBeInTheDocument();
  });

  it("has glass-premium class for glassmorphism", () => {
    render(<AppCard app={dashboardApp} onNavigate={vi.fn()} />, { wrapper: Wrapper });

    const link = screen.getByTestId("app-card-dashboard");
    expect(link.className).toContain("glass-premium");
  });

  it("renders a different app with different testId", () => {
    render(<AppCard app={employeeApp} onNavigate={vi.fn()} />, { wrapper: Wrapper });

    expect(screen.getByTestId("app-card-employees")).toBeInTheDocument();
    expect(screen.queryByTestId("app-card-dashboard")).not.toBeInTheDocument();
  });
});
