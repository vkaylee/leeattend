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
    render(<AdjustmentsTabs activeTab="pending" setActiveTab={mockSetActiveTab} />);
    
    expect(screen.getByText("common.status_pending")).toBeInTheDocument();
    expect(screen.getByText("common.status_approved")).toBeInTheDocument();
    expect(screen.getByText("common.status_rejected")).toBeInTheDocument();
  });

  it("calls setActiveTab when a tab is clicked", () => {
    const mockSetActiveTab = vi.fn();
    render(<AdjustmentsTabs activeTab="pending" setActiveTab={mockSetActiveTab} />);
    
    const approvedTab = screen.getByText("common.status_approved");
    fireEvent.click(approvedTab);
    expect(mockSetActiveTab).toHaveBeenCalledWith("approved");

    const rejectedTab = screen.getByText("common.status_rejected");
    fireEvent.click(rejectedTab);
    expect(mockSetActiveTab).toHaveBeenCalledWith("rejected");

    const pendingTab = screen.getByText("common.status_pending");
    fireEvent.click(pendingTab);
    expect(mockSetActiveTab).toHaveBeenCalledWith("pending");
  });

  it("applies active styling to the currently active tab", () => {
    const mockSetActiveTab = vi.fn();
    const { rerender } = render(<AdjustmentsTabs activeTab="pending" setActiveTab={mockSetActiveTab} />);
    
    // Check if the pending tab has the active background class
    const pendingTab = screen.getByText("common.status_pending").closest("button");
    expect(pendingTab).toHaveClass("bg-white");

    // Rerender with approved active
    rerender(<AdjustmentsTabs activeTab="approved" setActiveTab={mockSetActiveTab} />);
    const approvedTab = screen.getByText("common.status_approved").closest("button");
    expect(approvedTab).toHaveClass("bg-white");

    // Rerender with rejected active
    rerender(<AdjustmentsTabs activeTab="rejected" setActiveTab={mockSetActiveTab} />);
    const rejectedTab = screen.getByText("common.status_rejected").closest("button");
    expect(rejectedTab).toHaveClass("bg-white");
  });
});
