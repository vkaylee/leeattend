import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi } from "vitest";
import { PriorityLegend } from "../PriorityLegend";

// Mock translation
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}));

describe("PriorityLegend", () => {
  it("renders collapsed initially", () => {
    render(<PriorityLegend />);
    expect(screen.getByText("shifts.traceability.priority_hierarchy")).toBeDefined();
    // Content should not be visible
    expect(screen.queryByText("shifts.traceability.p100_manual")).toBeNull();
  });

  it("expands to show legend items when clicked", async () => {
    const user = userEvent.setup();
    render(<PriorityLegend />);

    const button = screen.getByRole("button");
    await user.click(button);

    // Should show items
    expect(screen.getByText("shifts.traceability.p100_manual")).toBeDefined();
    expect(screen.getByText("shifts.traceability.p10_dept_fixed")).toBeDefined();
    
    // Check for new labels
    expect(screen.getByText("shifts.traceability.highest_priority")).toBeDefined();
    expect(screen.getByText("shifts.traceability.lowest_priority")).toBeDefined();
  });
});
