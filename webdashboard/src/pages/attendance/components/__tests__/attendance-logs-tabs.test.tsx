import { render, screen, fireEvent } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { AttendanceLogsTabs } from "../attendance-logs-tabs";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}));

describe("AttendanceLogsTabs", () => {
  it("renders all tabs correctly", () => {
    const mockSetActiveTab = vi.fn();
    render(<AttendanceLogsTabs activeTab="recorded" setActiveTab={mockSetActiveTab} />);
    
    expect(screen.getByText("attendance_logs.tabs.recorded")).toBeInTheDocument();
    expect(screen.getByText("attendance_logs.tabs.unknown")).toBeInTheDocument();
  });

  it("calls setActiveTab when a tab is clicked", () => {
    const mockSetActiveTab = vi.fn();
    render(<AttendanceLogsTabs activeTab="recorded" setActiveTab={mockSetActiveTab} />);
    
    const unknownTab = screen.getByText("attendance_logs.tabs.unknown");
    fireEvent.click(unknownTab);
    expect(mockSetActiveTab).toHaveBeenCalledWith("unknown");

    const recordedTab = screen.getByText("attendance_logs.tabs.recorded");
    fireEvent.click(recordedTab);
    expect(mockSetActiveTab).toHaveBeenCalledWith("recorded");
  });

  it("applies active styling to the currently active tab", () => {
    const mockSetActiveTab = vi.fn();
    const { rerender } = render(<AttendanceLogsTabs activeTab="recorded" setActiveTab={mockSetActiveTab} />);
    
    const recordedTab = screen.getByText("attendance_logs.tabs.recorded").closest("button");
    expect(recordedTab).toHaveClass("bg-primary-50");

    rerender(<AttendanceLogsTabs activeTab="unknown" setActiveTab={mockSetActiveTab} />);
    const unknownTab = screen.getByText("attendance_logs.tabs.unknown").closest("button");
    expect(unknownTab).toHaveClass("bg-amber-50");
  });

  it("renders staging count badge when provided", () => {
    const mockSetActiveTab = vi.fn();
    render(<AttendanceLogsTabs activeTab="recorded" setActiveTab={mockSetActiveTab} stagingCount={5} />);
    
    const badge = screen.getByText("5");
    expect(badge).toBeInTheDocument();
    expect(badge).toHaveClass("bg-amber-100");
  });

  it("does not render staging count badge when count is 0", () => {
    const mockSetActiveTab = vi.fn();
    render(<AttendanceLogsTabs activeTab="recorded" setActiveTab={mockSetActiveTab} stagingCount={0} />);
    
    expect(screen.queryByText("0")).not.toBeInTheDocument();
  });
});
