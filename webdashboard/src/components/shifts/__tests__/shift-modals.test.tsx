import { render, screen } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { ShiftTimelinePreview } from "../shift-timeline-preview";

// Mock i18next
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, defaultVal?: string) => defaultVal || key,
    i18n: { language: "en" },
  }),
}));

describe("ShiftTimelinePreview Tests", () => {
  it("renders basic shift info and duration correctly", () => {
    render(
      <ShiftTimelinePreview
        allowEarlyOt={false}
        breakEnd=""
        breakStart=""
        isAutoDeduct={true}
        isBreakOvertime={false}
        isFixedBreak={false}
        maxEarlyOtMinutes={0}
        shiftBreak={60}
        shiftEnd="17:00"
        shiftGrace={15}
        shiftName="Morning Shift"
        shiftStart="08:00"
      />
    );

    // Verify shift name
    expect(screen.getByText("Morning Shift")).toBeDefined();
    // Verify shift hours (08:00 - 17:00 is 9.0 hours)
    expect(screen.getByText("9.0")).toBeDefined();
    // Working hours (9 - 1 hour break = 8.00)
    expect(screen.getByText("8.00")).toBeDefined();
  });

  it("handles cross-midnight shifts correctly", () => {
    render(
      <ShiftTimelinePreview
        allowEarlyOt={false}
        breakEnd=""
        breakStart=""
        isAutoDeduct={true}
        isBreakOvertime={false}
        isFixedBreak={false}
        maxEarlyOtMinutes={0}
        shiftBreak={60}
        shiftEnd="06:00"
        shiftGrace={0}
        shiftName="Night Shift"
        shiftStart="22:00"
      />
    );

    // 22:00 to 06:00 is 8 hours duration
    expect(screen.getByText("8.0")).toBeDefined();
    // 8 - 1 hour break = 7.00 working hours
    expect(screen.getByText("7.00")).toBeDefined();
  });

  it("renders fixed break details when isFixedBreak is true", () => {
    render(
      <ShiftTimelinePreview
        allowEarlyOt={false}
        breakEnd="13:00"
        breakStart="12:00"
        isAutoDeduct={true}
        isBreakOvertime={false}
        isFixedBreak={true}
        maxEarlyOtMinutes={0}
        shiftBreak={0}
        shiftEnd="17:00"
        shiftGrace={0}
        shiftName="Day Shift"
        shiftStart="08:00"
      />
    );

    // Check that the break time interval is shown
    expect(screen.getByText("12:00 - 13:00")).toBeDefined();
  });

  it("renders early OT details when allowed", () => {
    render(
      <ShiftTimelinePreview
        allowEarlyOt={true}
        breakEnd=""
        breakStart=""
        isAutoDeduct={false}
        isBreakOvertime={false}
        isFixedBreak={false}
        maxEarlyOtMinutes={60}
        shiftBreak={0}
        shiftEnd="17:00"
        shiftGrace={0}
        shiftName="OT Shift"
        shiftStart="08:00"
      />
    );

    // Check for allow early OT label/text
    expect(screen.getByText("shifts.modal.allow_early_ot")).toBeDefined();
    expect(screen.getByText(/60/)).toBeDefined();
  });
});
