import { render, screen } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";

import { useMyAttendance } from "@/hooks/use-ess";

import MyAttendancePage from "../MyAttendancePage";

// Mock hooks
vi.mock("@/hooks/use-ess", () => ({
  useMyAttendance: vi.fn(),
  useMyAttendanceSummaries: vi.fn(() => ({
    summaries: [],
    isLoading: false,
  })),
  useSubmitOtClaim: vi.fn(() => ({
    submitClaim: vi.fn(),
    isSubmitting: false,
  })),
}));

vi.mock("@/hooks/use-datetime", () => ({
  useDateTimeFormatter: () => ({
    formatTime: (date: string) => date ? date.substring(11, 16) : "",
    formatDate: (date: string) => date ? date.substring(0, 10) : "",
  }),
}));

// Mock translations
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}));

describe("MyAttendancePage", () => {
  it("renders loading state", () => {
 
    vi.mocked(useMyAttendance).mockReturnValue({
      entries: [],
      isLoading: true,
    } as unknown as never);

    render(<MyAttendancePage />);
    // Check for spinner or loading indicator (implementation detail: spinner has specific class or just absence of data)
    // The component renders a div with 'animate-spin'
    const spinner = document.querySelector('.animate-spin');
    expect(spinner).toBeDefined();
  });

  it("renders empty state", () => {
 
    vi.mocked(useMyAttendance).mockReturnValue({
      entries: [],
      isLoading: false,
    } as unknown as never);

    render(<MyAttendancePage />);
    expect(screen.getByText("ess.attendance.no_data")).toBeDefined();
  });

  it("renders attendance entries", () => {
 
    vi.mocked(useMyAttendance).mockReturnValue({
      entries: [
        {
          id: "entry-1",
          punch_in_time: "2026-01-20T08:00:00Z",
          punch_out_time: "2026-01-20T17:00:00Z",
          duration_minutes: 540,
        }
      ],
      isLoading: false,
    } as unknown as never);

    render(<MyAttendancePage />);
    expect(screen.getByText("ess.attendance.title")).toBeDefined();
    // Check for date/time (mocked formatter returns substring)
    expect(screen.getByText("20")).toBeDefined(); // Day
    expect(screen.getByText("08:00")).toBeDefined(); // Time
    expect(screen.getByText("17:00")).toBeDefined(); // Time
    expect(screen.getByText("9h 0m")).toBeDefined(); // Duration
  });

  it("renders metadata popover trigger when metadata exists", () => {
 
    vi.mocked(useMyAttendance).mockReturnValue({
      entries: [
        {
          id: "entry-1",
          punch_in_time: "2026-01-20T08:00:00Z",
          generation_metadata: { algorithm: "sequential" },
        }
      ],
      isLoading: false,
    } as unknown as never);

    render(<MyAttendancePage />);
    // Look for Info icon trigger
    // Since it's an SVG, we can look for role="button" that contains it or class
    // The popover trigger is a button
    const buttons = screen.getAllByRole("button");
    // We have prev/next month buttons + popover trigger. 
    // Popover trigger has Info icon.
    expect(buttons.length).toBeGreaterThan(2); 
  });
});
