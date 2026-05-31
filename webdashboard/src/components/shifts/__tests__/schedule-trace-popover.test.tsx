import { render, screen } from "@testing-library/react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi } from "vitest";

import { ScheduleTrace } from "../../../lib/api/types";

import { ScheduleTracePopover } from "../schedule-trace-popover";

// Mock translation locally to ensure we test against keys, not translated values
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}));

describe("ScheduleTracePopover", () => {
  const mockTraces: ScheduleTrace[] = [
    {
      rule_name: "Manual Shift",
      priority: 100,
      status: "applied",
      reason: "Manual assignment"
    },
    {
      rule_name: "Dept Shift",
      priority: 40,
      status: "overridden",
      reason: "Overridden by roster"
    }
  ];

  it("renders the trigger icon", () => {
    render(<ScheduleTracePopover source="roster" traces={mockTraces} />);
    // The component renders an Info icon, we can look for it or the container
    // Since it's an SVG, we might look for a role or class if available, 
    // but the component wrapper has a specific class or we can just check if it renders without crashing.
    // The code shows it renders a button (Trigger).
    const trigger = screen.getByRole("button");
    expect(trigger).toBeDefined();
  });

  it("shows trace details when clicked (Manual Roster case)", async () => {
    const user = userEvent.setup();
    render(<ScheduleTracePopover source="roster" traces={mockTraces} />);

    const trigger = screen.getByRole("button");
    await user.click(trigger);

    // Check for Winner
    expect(screen.getByText("shifts.traceability.winner")).toBeDefined();
    expect(screen.getByText("Manual Shift")).toBeDefined();
    
    // Check for Overridden
    expect(screen.getByText((content) => content.includes("shifts.traceability.show_overridden"))).toBeDefined();
    
    // We need to click "Show overridden" to see the overridden rule?
    // The component has: onClick={() => setShowOverridden(!showOverridden)}
    // And renders: {t("shifts.traceability.show_overridden")} ({overriddenRules.length})
    
    // Check for Highest Priority Hint (P100)
    expect(screen.getByText("shifts.traceability.highest_priority_hint")).toBeDefined();
  });

  it("shows correct hint for lower priority rules", async () => {
    const lowPriorityTraces: ScheduleTrace[] = [
      {
        rule_name: "Dept Shift",
        priority: 40,
        status: "applied",
        reason: "Applied"
      }
    ];

    const user = userEvent.setup();
    render(<ScheduleTracePopover source="department_pattern" traces={lowPriorityTraces} />);

    const trigger = screen.getByRole("button");
    await user.click(trigger);

    // Check for Override Hint (P < 100)
    expect(screen.getByText("shifts.traceability.override_hint")).toBeDefined();
  });

  it("shows resolved shift details when shiftName is provided", async () => {
    const user = userEvent.setup();
    render(
      <ScheduleTracePopover 
        source="roster" 
        traces={mockTraces} 
        shiftName="Day Shift" 
        startTime="08:00:00" 
        endTime="17:00:00" 
        shiftSymbol="DS" 
      />
    );

    const trigger = screen.getByRole("button");
    await user.click(trigger);

    expect(screen.getByText("shifts.traceability.applied_shift")).toBeDefined();
    expect(screen.getByText("Day Shift")).toBeDefined();
    expect(screen.getByText("08:00 - 17:00")).toBeDefined();
    expect(screen.getByText("DS")).toBeDefined();
  });

  it("shows Off Day when shiftName is not provided", async () => {
    const user = userEvent.setup();
    render(
      <ScheduleTracePopover 
        source="roster" 
        traces={mockTraces} 
      />
    );

    const trigger = screen.getByRole("button");
    await user.click(trigger);

    expect(screen.getByText("shifts.traceability.applied_shift")).toBeDefined();
    expect(screen.getByText("shifts.off_day")).toBeDefined();
    expect(screen.getByText("OFF")).toBeDefined();
  });

  it("stops propagation when the trigger is clicked", async () => {
    const parentClick = vi.fn();
    const user = userEvent.setup();

    render(
      <div onClick={parentClick}>
        <ScheduleTracePopover source="roster" traces={mockTraces} />
      </div>
    );

    const trigger = screen.getByRole("button");
    await user.click(trigger);

    expect(parentClick).not.toHaveBeenCalled();
  });
});
