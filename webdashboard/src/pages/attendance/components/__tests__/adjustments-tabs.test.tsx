import { render, screen, fireEvent } from "@testing-library/react";

import { describe, expect, it, vi } from "vitest";

import { AdjustmentsTabs } from "../adjustments-tabs";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}));

describe("AdjustmentsTabs", () => {
  it("renders all tabs correctly", () => {
    const mockSetActiveTab = vi.fn();
    render(<AdjustmentsTabs activeTab="adjustments" setActiveTab={mockSetActiveTab} canManageBehalf={true} />);
    
    expect(screen.getByText("ATTENDANCE_CONFIRM.TABS.ADJUSTMENTS")).toBeInTheDocument();
    expect(screen.getByText("ATTENDANCE_CONFIRM.TABS.BEHALF_ASSIGNMENTS")).toBeInTheDocument();
  });

  it("calls setActiveTab when a tab is clicked", () => {
    const mockSetActiveTab = vi.fn();
    render(<AdjustmentsTabs activeTab="adjustments" setActiveTab={mockSetActiveTab} canManageBehalf={true} />);
    
    const behalfTab = screen.getByText("ATTENDANCE_CONFIRM.TABS.BEHALF_ASSIGNMENTS");
    fireEvent.click(behalfTab);
    expect(mockSetActiveTab).toHaveBeenCalledWith("behalf_assignments");

    const adjustmentsTab = screen.getByText("ATTENDANCE_CONFIRM.TABS.ADJUSTMENTS");
    fireEvent.click(adjustmentsTab);
    expect(mockSetActiveTab).toHaveBeenCalledWith("adjustments");
  });

  it("applies active styling to the currently active tab", () => {
    const mockSetActiveTab = vi.fn();
    const { rerender } = render(<AdjustmentsTabs activeTab="adjustments" setActiveTab={mockSetActiveTab} canManageBehalf={true} />);
    
    const adjustmentsTab = screen.getByText("ATTENDANCE_CONFIRM.TABS.ADJUSTMENTS").closest("button");
    expect(adjustmentsTab).toHaveClass("bg-white");

    // Rerender with behalf active
    rerender(<AdjustmentsTabs activeTab="behalf_assignments" setActiveTab={mockSetActiveTab} canManageBehalf={true} />);
    const behalfTab = screen.getByText("ATTENDANCE_CONFIRM.TABS.BEHALF_ASSIGNMENTS").closest("button");
    expect(behalfTab).toHaveClass("bg-white");
  });
});
