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
        intervals={[{
          intervalName: "Morning Shift",
          startTime: "08:00",
          endTime: "17:00",
          breakDuration: 60,
          breakStart: "",
          breakEnd: "",
          breakPunchMode: "none",
          isFixedBreak: false,
        }]}
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
        intervals={[{
          intervalName: "Night Shift",
          startTime: "22:00",
          endTime: "06:00",
          breakDuration: 60,
          breakStart: "",
          breakEnd: "",
          breakPunchMode: "none",
          isFixedBreak: false,
        }]}
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
        intervals={[{
          intervalName: "Day Shift",
          startTime: "08:00",
          endTime: "17:00",
          breakDuration: 0,
          breakStart: "12:00",
          breakEnd: "13:00",
          breakPunchMode: "none",
          isFixedBreak: true,
        }]}
      />
    );

    // Check that the break time interval is shown
    expect(screen.getByText("12:00 - 13:00")).toBeDefined();
  });

});
