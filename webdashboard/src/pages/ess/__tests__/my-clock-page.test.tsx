import * as reactQuery from "@tanstack/react-query";
import { render, screen, fireEvent, act, waitFor } from "@testing-library/react";

import { describe, expect, it, vi, beforeEach } from "vitest";

import { createMockEmployee, createMockUseMyClockReturn, createMockUseMyProfile } from "@/__tests__/factories/test-factories";
import Button from "@/components/ui/button";
import * as toastHook from "@/components/ui/toast";
import * as essHooks from "@/hooks/use-ess";
import * as api from "@/lib/api";

import MyClockPage from "../my-clock-page";

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
    t: (key: string, defaultString?: string) => defaultString ?? key,
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
      <Button data-testid="punch-btn" onClick={onPunch} variant="ghost">Punch</Button>
      <Button data-testid="adjust-btn" onClick={onRequestAdjustment} variant="ghost">Adjust</Button>
    </div>
  )
}));

vi.mock("@/components/time-tracking/adjustment-request-modal", () => ({
  AdjustmentRequestModal: ({ isOpen, onClose, onSubmit }: any) => (
    isOpen ? (
      <div data-testid="adjustment-modal">
        <Button data-testid="close-modal" onClick={onClose} variant="ghost">Close</Button>
        <Button data-testid="submit-modal" onClick={() => onSubmit({ date: "2026-05-01", punchIn: "08:00", punchOut: "17:00", reason: "Forgot" })} variant="ghost">Submit</Button>
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
    vi.mocked(toastHook.useToast).mockReturnValue(mockToast as unknown as toastHook.ToastContextType);
    vi.mocked(api.getCurrentWorkspaceId).mockReturnValue("ws-123");
    vi.mocked(reactQuery.useQuery).mockReturnValue({ data: { timezone: "UTC" } } as unknown as ReturnType<typeof reactQuery.useQuery>);
  });

  it("renders loading state", () => {
    vi.mocked(essHooks.useMyProfile).mockReturnValue(createMockUseMyProfile({ isLoading: true }));
    vi.mocked(essHooks.useMyClock).mockReturnValue(createMockUseMyClockReturn({ isLoading: false }));

    render(<MyClockPage />);
    expect(screen.getByTestId("my-clock-skeleton")).toBeInTheDocument();
  });

  it("renders empty state without profile", () => {
    vi.mocked(essHooks.useMyProfile).mockReturnValue(createMockUseMyProfile({ isLoading: false, employee: undefined }));
    vi.mocked(essHooks.useMyClock).mockReturnValue(createMockUseMyClockReturn({ isLoading: false }));

    render(<MyClockPage />);
    expect(screen.getByText("common.no_profile")).toBeInTheDocument();
  });

  it("renders content and handles punch successfully", async () => {
    const mockPunch = vi.fn().mockResolvedValue(true);
    vi.mocked(essHooks.useMyProfile).mockReturnValue(createMockUseMyProfile({ 
      isLoading: false, 
      employee: createMockEmployee({ id: "emp-1", full_name: "John Doe" }) 
    }));
    vi.mocked(essHooks.useMyClock).mockReturnValue(createMockUseMyClockReturn({ 
      isLoading: false,
      punch: mockPunch,
      activeEntry: undefined,
      entries: [],
      isPunching: false,
    }));

    render(<MyClockPage />);
    
    expect(screen.getByTestId("punch-card")).toBeInTheDocument();
    
    act(() => {
      fireEvent.click(screen.getByTestId("punch-btn"));
    });

    await waitFor(() => { expect(mockPunch).toHaveBeenCalled(); });
    expect(mockToast.success).toHaveBeenCalledWith("Chấm công thành công");
  });

  it("handles punch error", async () => {
    const mockPunch = vi.fn().mockRejectedValue(new Error("Network Error"));
    vi.mocked(essHooks.useMyProfile).mockReturnValue(createMockUseMyProfile({ 
      isLoading: false, 
      employee: createMockEmployee({ id: "emp-1", full_name: "John Doe" }) 
    }));
    vi.mocked(essHooks.useMyClock).mockReturnValue(createMockUseMyClockReturn({ 
      isLoading: false,
      punch: mockPunch,
    }));

    render(<MyClockPage />);
    
    act(() => {
      fireEvent.click(screen.getByTestId("punch-btn"));
    });

    await waitFor(() => { expect(mockToast.error).toHaveBeenCalledWith("Network Error"); });
  });

  it("handles adjustment request successfully", async () => {
    const mockSubmit = vi.mocked(api.submitAttendanceAdjustment).mockResolvedValue({} as any);
    vi.mocked(essHooks.useMyProfile).mockReturnValue(createMockUseMyProfile({ 
      isLoading: false, 
      employee: createMockEmployee({ id: "emp-1", full_name: "John Doe" }) 
    }));
    vi.mocked(essHooks.useMyClock).mockReturnValue(createMockUseMyClockReturn({ 
      isLoading: false,
    }));

    render(<MyClockPage />);
    
    act(() => {
      fireEvent.click(screen.getByTestId("adjust-btn"));
    });

    expect(screen.getByTestId("adjustment-modal")).toBeInTheDocument();

    act(() => {
      fireEvent.click(screen.getByTestId("submit-modal"));
    });

    await waitFor(() => {
      expect(mockSubmit).toHaveBeenCalledWith("ws-123", {
        employee_id: "emp-1",
        reason: "Forgot",
        requested_punch_in_time: "2026-05-01T08:00Z",
        requested_punch_out_time: "2026-05-01T17:00Z",
        time_entry_id: undefined,
      });
    });
    expect(mockToast.success).toHaveBeenCalledWith("ess.clock.adjustment_requested");

    // Close modal
    act(() => {
      fireEvent.click(screen.getByTestId("close-modal"));
    });
    expect(screen.queryByTestId("adjustment-modal")).not.toBeInTheDocument();
  });

  it("handles adjustment request error", async () => {
    vi.mocked(api.submitAttendanceAdjustment).mockRejectedValue({ response: { data: { error: "Invalid time" } } });
    vi.mocked(essHooks.useMyProfile).mockReturnValue(createMockUseMyProfile({ 
      isLoading: false, 
      employee: createMockEmployee({ id: "emp-1" }) 
    }));
    vi.mocked(essHooks.useMyClock).mockReturnValue(createMockUseMyClockReturn({ isLoading: false }));

    render(<MyClockPage />);
    
    act(() => {
      fireEvent.click(screen.getByTestId("adjust-btn"));
    });

    act(() => {
      fireEvent.click(screen.getByTestId("submit-modal"));
    });

    await waitFor(() => { expect(mockToast.error).toHaveBeenCalledWith("Invalid time"); });
  });
});
