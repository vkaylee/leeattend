import { render, screen, fireEvent, act } from "@testing-library/react";
import { describe, expect, it, vi, beforeEach } from "vitest";

import MyClockPage from "../my-clock-page";
import * as essHooks from "@/hooks/use-ess";
import * as api from "@/lib/api";
import * as toastHook from "@/components/ui/toast";
import * as reactQuery from "@tanstack/react-query";

// Mock hooks and libs
vi.mock("@/hooks/use-ess", () => ({
  useMyClock: vi.fn(),
  useMyProfile: vi.fn(),
}));

vi.mock("@/lib/api", () => ({
  fetchWorkspace: vi.fn(),
  getCurrentWorkspaceId: vi.fn(),
  submitAttendanceAdjustment: vi.fn(),
}));

vi.mock("@/lib/datetime", () => ({
  toUTCISO: vi.fn((d, t) => `${d}T${t}Z`),
}));

vi.mock("@/components/ui/toast", () => ({
  useToast: vi.fn(),
}));

vi.mock("@tanstack/react-query", () => ({
  useQuery: vi.fn(),
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, defaultString?: string) => defaultString || key,
  }),
}));

vi.mock("../components/history-table", () => ({
  HistoryTable: () => <div data-testid="history-table" />
}));

vi.mock("../components/my-clock-skeleton", () => ({
  MyClockSkeleton: () => <div data-testid="my-clock-skeleton" />
}));

vi.mock("@/components/time-tracking/punch-card", () => ({
  PunchCard: ({ onPunch, onRequestAdjustment }: any) => (
    <div data-testid="punch-card">
      <button data-testid="punch-btn" onClick={onPunch}>Punch</button>
      <button data-testid="adjust-btn" onClick={onRequestAdjustment}>Adjust</button>
    </div>
  )
}));

vi.mock("@/components/time-tracking/adjustment-request-modal", () => ({
  AdjustmentRequestModal: ({ isOpen, onClose, onSubmit }: any) => (
    isOpen ? (
      <div data-testid="adjustment-modal">
        <button data-testid="close-modal" onClick={onClose}>Close</button>
        <button data-testid="submit-modal" onClick={() => onSubmit({ date: "2026-05-01", punchIn: "08:00", punchOut: "17:00", reason: "Forgot" })}>Submit</button>
      </div>
    ) : null
  )
}));

describe("MyClockPage", () => {
  const mockToast = {
    success: vi.fn(),
    error: vi.fn(),
  };

  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(toastHook.useToast).mockReturnValue(mockToast as any);
    vi.mocked(api.getCurrentWorkspaceId).mockReturnValue("ws-123");
    vi.mocked(reactQuery.useQuery).mockReturnValue({ data: { timezone: "UTC" } } as any);
  });

  it("renders loading state", () => {
    vi.mocked(essHooks.useMyProfile).mockReturnValue({ isLoading: true } as any);
    vi.mocked(essHooks.useMyClock).mockReturnValue({ isLoading: false } as any);

    render(<MyClockPage />);
    expect(screen.getByTestId("my-clock-skeleton")).toBeInTheDocument();
  });

  it("renders empty state without profile", () => {
    vi.mocked(essHooks.useMyProfile).mockReturnValue({ isLoading: false, employee: null } as any);
    vi.mocked(essHooks.useMyClock).mockReturnValue({ isLoading: false } as any);

    render(<MyClockPage />);
    expect(screen.getByText("common.no_profile")).toBeInTheDocument();
  });

  it("renders content and handles punch successfully", async () => {
    const mockPunch = vi.fn().mockResolvedValue(true);
    vi.mocked(essHooks.useMyProfile).mockReturnValue({ 
      isLoading: false, 
      employee: { id: "emp-1", full_name: "John Doe" } 
    } as any);
    vi.mocked(essHooks.useMyClock).mockReturnValue({ 
      isLoading: false,
      punch: mockPunch,
      activeEntry: null,
      entries: [],
      isPunching: false,
    } as any);

    render(<MyClockPage />);
    
    expect(screen.getByTestId("punch-card")).toBeInTheDocument();
    
    await act(async () => {
      fireEvent.click(screen.getByTestId("punch-btn"));
    });

    expect(mockPunch).toHaveBeenCalled();
    expect(mockToast.success).toHaveBeenCalledWith("Chấm công thành công");
  });

  it("handles punch error", async () => {
    const mockPunch = vi.fn().mockRejectedValue(new Error("Network Error"));
    vi.mocked(essHooks.useMyProfile).mockReturnValue({ 
      isLoading: false, 
      employee: { id: "emp-1", full_name: "John Doe" } 
    } as any);
    vi.mocked(essHooks.useMyClock).mockReturnValue({ 
      isLoading: false,
      punch: mockPunch,
    } as any);

    render(<MyClockPage />);
    
    await act(async () => {
      fireEvent.click(screen.getByTestId("punch-btn"));
    });

    expect(mockToast.error).toHaveBeenCalledWith("Network Error");
  });

  it("handles adjustment request successfully", async () => {
    const mockSubmit = vi.mocked(api.submitAttendanceAdjustment).mockResolvedValue({} as any);
    vi.mocked(essHooks.useMyProfile).mockReturnValue({ 
      isLoading: false, 
      employee: { id: "emp-1", full_name: "John Doe" } 
    } as any);
    vi.mocked(essHooks.useMyClock).mockReturnValue({ 
      isLoading: false,
    } as any);

    render(<MyClockPage />);
    
    act(() => {
      fireEvent.click(screen.getByTestId("adjust-btn"));
    });

    expect(screen.getByTestId("adjustment-modal")).toBeInTheDocument();

    await act(async () => {
      fireEvent.click(screen.getByTestId("submit-modal"));
    });

    expect(mockSubmit).toHaveBeenCalledWith("ws-123", {
      employee_id: "emp-1",
      reason: "Forgot",
      requested_punch_in_time: "2026-05-01T08:00Z",
      requested_punch_out_time: "2026-05-01T17:00Z",
      time_entry_id: undefined,
    });
    expect(mockToast.success).toHaveBeenCalledWith("ess.clock.adjustment_requested");

    // Close modal
    act(() => {
      fireEvent.click(screen.getByTestId("close-modal"));
    });
    expect(screen.queryByTestId("adjustment-modal")).not.toBeInTheDocument();
  });

  it("handles adjustment request error", async () => {
    vi.mocked(api.submitAttendanceAdjustment).mockRejectedValue({ response: { data: { error: "Invalid time" } } } as any);
    vi.mocked(essHooks.useMyProfile).mockReturnValue({ 
      isLoading: false, 
      employee: { id: "emp-1" } 
    } as any);
    vi.mocked(essHooks.useMyClock).mockReturnValue({ isLoading: false } as any);

    render(<MyClockPage />);
    
    act(() => {
      fireEvent.click(screen.getByTestId("adjust-btn"));
    });

    await act(async () => {
      fireEvent.click(screen.getByTestId("submit-modal"));
    });

    expect(mockToast.error).toHaveBeenCalledWith("Invalid time");
  });
});
