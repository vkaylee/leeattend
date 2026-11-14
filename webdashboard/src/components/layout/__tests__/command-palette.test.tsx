import { render, screen, fireEvent } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import { CommandPalette } from "../command-palette";

const mockNavigate = vi.fn();
vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => mockNavigate,
}));

vi.mock("@/hooks/use-permissions", () => ({
  useMyPermissions: () => ({
    hasAnyPermission: () => true,
    hasPermission: () => true,
  }),
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, defaultVal?: unknown) => {
      if (defaultVal && typeof defaultVal === "object" && "defaultValue" in defaultVal) {
        const val = defaultVal.defaultValue;
        if (typeof val === "string") return val;
      }
      if (typeof defaultVal === "string") return defaultVal;
      return key;
    },
    i18n: { language: "vi" },
  }),
}));

describe("CommandPalette Component", () => {
  const onClose = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("does not render dialog content when isOpen is false", () => {
    render(<CommandPalette isOpen={false} onClose={onClose} />);
    expect(screen.queryByTestId("command-palette-modal")).not.toBeInTheDocument();
  });

  it("renders command palette dialog when isOpen is true", () => {
    render(<CommandPalette isOpen={true} onClose={onClose} />);
    expect(screen.getByTestId("command-palette-modal")).toBeInTheDocument();
    expect(screen.getByTestId("command-palette-input")).toBeInTheDocument();
    expect(screen.getByTestId("command-palette-list")).toBeInTheDocument();
  });

  it("navigates and closes when an item is selected", () => {
    render(<CommandPalette isOpen={true} onClose={onClose} />);

    const dashboardItem = screen.getByTestId("command-item-app-dashboard");
    expect(dashboardItem).toBeInTheDocument();

    fireEvent.click(dashboardItem);
    expect(onClose).toHaveBeenCalledTimes(1);
    expect(mockNavigate).toHaveBeenCalledWith({ to: "/" });
  });

  it("closes when Escape key is pressed", () => {
    render(<CommandPalette isOpen={true} onClose={onClose} />);

    fireEvent.keyDown(window, { key: "Escape" });
    expect(onClose).toHaveBeenCalledTimes(1);
  });
});
