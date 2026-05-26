import { render, screen, fireEvent, act } from "@testing-library/react";
import { describe, expect, it, vi, beforeEach } from "vitest";

import * as hooks from "@/hooks/use-ess";
import MyAttendancePage from "../my-attendance-page";

// Mock the hooks
vi.mock("@/hooks/use-ess", () => ({
  useMyAttendance: vi.fn(),
  useMyAttendanceSummaries: vi.fn(),
}));

// Mock child components
vi.mock("../components/attendance-header", () => ({
  AttendanceHeader: ({ handleNextMonth, handlePreviousMonth, handleMonthYearChange }: any) => (
    <div data-testid="attendance-header">
      <button data-testid="prev-month" onClick={handlePreviousMonth}>Prev</button>
      <button data-testid="next-month" onClick={handleNextMonth}>Next</button>
      <button data-testid="change-month" onClick={() => handleMonthYearChange(5, 2026)}>Change Month</button>
    </div>
  ),
}));

vi.mock("../components/attendance-table", () => ({
  AttendanceTable: ({ entries, setSelectedSummary, setClaimOpen }: any) => (
    <div data-testid="attendance-table">
      {entries.map((e: any) => (
        <div key={e.id} data-testid={`entry-${e.id}`}>
          {e.status}
          <button
            data-testid={`claim-btn-${e.id}`}
            onClick={() => {
              setSelectedSummary({ date: e.date, total_overtime_minutes: 60 });
              setClaimOpen(true);
            }}
          >
            Claim
          </button>
        </div>
      ))}
    </div>
  ),
}));

vi.mock("@/components/ess/claim-ot-dialog", () => ({
  ClaimOtDialog: ({ open, onOpenChange, summary }: any) => (
    open ? (
      <div data-testid="claim-dialog">
        Claiming OT for {summary?.date}
        <button data-testid="close-claim" onClick={() => onOpenChange(false)}>Close</button>
      </div>
    ) : null
  ),
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}));

describe("MyAttendancePage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders loading state", () => {
    vi.mocked(hooks.useMyAttendance).mockReturnValue({
      entries: [],
      isLoading: true,
      error: null,
      mutate: vi.fn(),
    } as any);

    vi.mocked(hooks.useMyAttendanceSummaries).mockReturnValue({
      summaries: [],
      isLoading: false,
      error: null,
      mutate: vi.fn(),
    } as any);

    render(<MyAttendancePage />);
    expect(screen.getByText("common.loading")).toBeInTheDocument();
  });

  it("renders empty state", () => {
    vi.mocked(hooks.useMyAttendance).mockReturnValue({
      entries: [],
      isLoading: false,
      error: null,
      mutate: vi.fn(),
    } as any);

    vi.mocked(hooks.useMyAttendanceSummaries).mockReturnValue({
      summaries: [],
      isLoading: false,
      error: null,
      mutate: vi.fn(),
    } as any);

    render(<MyAttendancePage />);
    expect(screen.getByText("ess.attendance.no_data")).toBeInTheDocument();
  });

  it("renders entries and handles claim dialog", () => {
    const mockEntries = [{ id: "1", date: "2026-05-01", status: "present" }];
    
    vi.mocked(hooks.useMyAttendance).mockReturnValue({
      entries: mockEntries,
      isLoading: false,
      error: null,
      mutate: vi.fn(),
    } as any);

    vi.mocked(hooks.useMyAttendanceSummaries).mockReturnValue({
      summaries: [{ date: "2026-05-01", total_overtime_minutes: 60 }],
      isLoading: false,
      error: null,
      mutate: vi.fn(),
    } as any);

    render(<MyAttendancePage />);
    
    // Check table rendered
    expect(screen.getByTestId("attendance-table")).toBeInTheDocument();
    expect(screen.getByTestId("entry-1")).toBeInTheDocument();

    // Open claim dialog
    act(() => {
      fireEvent.click(screen.getByTestId("claim-btn-1"));
    });

    expect(screen.getByTestId("claim-dialog")).toBeInTheDocument();
    expect(screen.getByText("Claiming OT for 2026-05-01")).toBeInTheDocument();

    // Close claim dialog
    act(() => {
      fireEvent.click(screen.getByTestId("close-claim"));
    });

    expect(screen.queryByTestId("claim-dialog")).not.toBeInTheDocument();
  });

  it("handles month navigation", () => {
    vi.mocked(hooks.useMyAttendance).mockReturnValue({
      entries: [],
      isLoading: false,
      error: null,
      mutate: vi.fn(),
    } as any);

    vi.mocked(hooks.useMyAttendanceSummaries).mockReturnValue({
      summaries: [],
      isLoading: false,
      error: null,
      mutate: vi.fn(),
    } as any);

    render(<MyAttendancePage />);

    act(() => {
      fireEvent.click(screen.getByTestId("next-month"));
    });
    expect(screen.getByTestId("attendance-header")).toBeInTheDocument();

    act(() => {
      fireEvent.click(screen.getByTestId("prev-month"));
    });
    expect(screen.getByTestId("attendance-header")).toBeInTheDocument();

    act(() => {
      fireEvent.click(screen.getByTestId("change-month"));
    });
    expect(screen.getByTestId("attendance-header")).toBeInTheDocument();
  });
});
