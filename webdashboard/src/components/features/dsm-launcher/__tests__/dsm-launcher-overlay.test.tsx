import { render, screen, fireEvent } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import React from "react";
import { describe, it, expect, vi, beforeEach } from "vitest";

import { DsmLauncherOverlay } from "../dsm-launcher-overlay";

const mockNavigate = vi.fn();
const mockRecordAccess = vi.fn();

vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => mockNavigate,
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, opts?: { defaultValue?: string }) => opts?.defaultValue ?? key,
  }),
}));

vi.mock("@/hooks/use-permissions", () => ({
  useMyPermissions: () => ({
    hasAnyPermission: vi.fn(() => true),
    isLoading: false,
  }),
}));


vi.mock("@/hooks/use-quick-access", () => ({
  useQuickAccess: () => ({
    top3: ["dashboard", "employees"],
    recordAccess: mockRecordAccess,
  }),
}));

describe("DsmLauncherOverlay", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("does not render dialog content when isOpen = false", () => {
    render(<DsmLauncherOverlay isOpen={false} onClose={vi.fn()} />);
    expect(screen.queryByTestId("dsm-launcher-modal")).not.toBeInTheDocument();
  });

  it("renders DSM 7 modal and app grid when isOpen = true", () => {
    render(<DsmLauncherOverlay isOpen={true} onClose={vi.fn()} />);
    expect(screen.getByTestId("dsm-launcher-modal")).toBeInTheDocument();
    expect(screen.getByTestId("dsm-grid-view")).toBeInTheDocument();
    expect(screen.getByTestId("dsm-all-apps-section")).toBeInTheDocument();
  });

  it("renders quick access recent apps in grid view", () => {
    render(<DsmLauncherOverlay isOpen={true} onClose={vi.fn()} />);
    expect(screen.getByTestId("dsm-quick-access-section")).toBeInTheDocument();
  });

  it("calls onClose when close button is clicked", () => {
    const handleClose = vi.fn();
    render(<DsmLauncherOverlay isOpen={true} onClose={handleClose} />);

    const closeBtn = screen.getByTestId("dsm-launcher-close-btn");
    fireEvent.click(closeBtn);
    expect(handleClose).toHaveBeenCalled();
  });

  it("navigates and records access when an app squircle is clicked", () => {
    const handleClose = vi.fn();
    render(<DsmLauncherOverlay isOpen={true} onClose={handleClose} />);

    const dashboardIcon = screen.getByTestId("dsm-app-dashboard");
    fireEvent.click(dashboardIcon);

    expect(mockRecordAccess).toHaveBeenCalledWith("dashboard");
    expect(mockNavigate).toHaveBeenCalledWith({ to: "/" });
    expect(handleClose).toHaveBeenCalled();
  });
  it("keeps the Settings Hub entry in the launcher and does not expose child settings", () => {
    const handleClose = vi.fn();
    render(<DsmLauncherOverlay isOpen={true} onClose={handleClose} />);

    const settingsHubIcon = screen.getByTestId("dsm-app-settings");
    expect(settingsHubIcon).toBeInTheDocument();
    expect(screen.queryByTestId("dsm-setting-general")).not.toBeInTheDocument();

    fireEvent.click(settingsHubIcon);
    expect(mockNavigate).toHaveBeenCalledWith({ to: "/workspace/settings" });
    expect(handleClose).toHaveBeenCalled();
  });


  it("renders Audit Logs as a standalone app and navigates to its route", () => {
    const handleClose = vi.fn();
    render(<DsmLauncherOverlay isOpen={true} onClose={handleClose} />);

    const auditLogsIcon = screen.getByTestId("dsm-app-audit-logs");
    expect(auditLogsIcon).toBeInTheDocument();

    fireEvent.click(auditLogsIcon);

    expect(mockNavigate).toHaveBeenCalledWith({ to: "/audit-logs" });
    expect(handleClose).toHaveBeenCalled();
  });
  it("switches to search view when typing in search input", async () => {
    const user = userEvent.setup();
    render(<DsmLauncherOverlay isOpen={true} onClose={vi.fn()} />);

    const searchInput = screen.getByTestId("dsm-launcher-search-input");
    await user.type(searchInput, "ot");

    expect(screen.getByTestId("dsm-search-view")).toBeInTheDocument();
    expect(screen.queryByTestId("dsm-grid-view")).not.toBeInTheDocument();
  });

  it("finds Vietnamese feature with or without diacritics", async () => {
    const user = userEvent.setup();
    render(<DsmLauncherOverlay isOpen={true} onClose={vi.fn()} />);

    const searchInput = screen.getByTestId("dsm-launcher-search-input");

    // Search without diacritics "diem danh"
    await user.type(searchInput, "diem danh");
    expect(screen.getByTestId("dsm-action-clock-in")).toBeInTheDocument();

    // Clear search
    await user.clear(searchInput);

    // Search with diacritics "chấm công"
    await user.type(searchInput, "chấm công");
    expect(screen.getByTestId("dsm-action-clock-in")).toBeInTheDocument();
  });

  it("finds wifi setting action shortcut when searching 'wifi' and does not expose deep setting items", async () => {
    const user = userEvent.setup();
    render(<DsmLauncherOverlay isOpen={true} onClose={vi.fn()} />);

    const searchInput = screen.getByTestId("dsm-launcher-search-input");
    await user.type(searchInput, "wifi");

    // Does not expose raw child setting tab
    expect(screen.queryByTestId("dsm-setting-time")).not.toBeInTheDocument();
    // Exposes quick action shortcut
    expect(screen.getByTestId("dsm-action-setting-time-wifi")).toBeInTheDocument();
    // Desktop side preview displays preview
    expect(screen.getByTestId("dsm-side-preview")).toBeInTheDocument();
  });

  it("shows search empty state when no features match query", async () => {
    const user = userEvent.setup();
    render(<DsmLauncherOverlay isOpen={true} onClose={vi.fn()} />);

    const searchInput = screen.getByTestId("dsm-launcher-search-input");
    await user.type(searchInput, "nonexistent-keyword-xyz");

    expect(screen.getByTestId("dsm-search-empty")).toBeInTheDocument();
  });

  it("navigates grid via ArrowDown from search input", () => {
    render(<DsmLauncherOverlay isOpen={true} onClose={vi.fn()} />);

    const searchInput = screen.getByTestId("dsm-launcher-search-input");
    fireEvent.keyDown(searchInput, { key: "ArrowDown" });

    const firstApp = screen.getByTestId("dsm-app-dashboard");
    expect(firstApp).toHaveAttribute("data-focused", "true");
  });
  it("closes when clicking launcher empty space but keeps interactive controls open", () => {
    const handleClose = vi.fn();
    render(<DsmLauncherOverlay isOpen={true} onClose={handleClose} />);

    fireEvent.click(screen.getByTestId("dsm-grid-view"));
    expect(handleClose).toHaveBeenCalledTimes(1);

    fireEvent.click(screen.getByTestId("dsm-launcher-search-input"));
    expect(handleClose).toHaveBeenCalledTimes(1);
  });
});
