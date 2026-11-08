import { renderHook, act } from "@testing-library/react";
import React from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useBatchApproval } from "../hooks/use-batch-approval";
import { UnifiedApprovalItem } from "../types/unified-approvals";
import * as api from "@/lib/api";
import * as swapApi from "@/lib/api/shift-swaps";
import * as otApi from "@/lib/api/attendance-overtime";

// ── Mocks ─────────────────────────────────────────────────────────────

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, fallback?: string) => fallback ?? key,
  }),
}));

const mockToastSuccess = vi.fn();
const mockToastError = vi.fn();

vi.mock("@/components/ui/toast", () => ({
  useToast: () => ({
    success: mockToastSuccess,
    error: mockToastError,
  }),
}));

vi.mock("@/lib/api", async (importOriginal) => {
  const actual = await importOriginal<typeof api>();
  return {
    ...actual,
    approveLeaveRequest: vi.fn().mockResolvedValue(undefined),
    approveAttendanceAdjustment: vi.fn().mockResolvedValue(undefined),
  };
});

vi.mock("@/lib/api/shift-swaps", () => ({
  approveShiftSwap: vi.fn().mockResolvedValue(undefined),
}));

vi.mock("@/lib/api/attendance-overtime", () => ({
  approveOt: vi.fn().mockResolvedValue(undefined),
}));

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return function Wrapper({ children }: { children: React.ReactNode }) {
    return (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );
  };
}

describe("useBatchApproval", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("approves multiple items across leave, adjustment, shift swap and overtime", async () => {
    const { result } = renderHook(() => useBatchApproval(), {
      wrapper: createWrapper(),
    });

    const items: UnifiedApprovalItem[] = [
      {
        id: "leave-1",
        type: "leave",
        status: "pending",
        requester: { id: "emp-1", name: "A" },
        submittedAt: "2026-11-01",
        targetDate: "2026-11-01",
        title: "Nghỉ phép",
        subtitle: "",
        details: { dateStr: "2026-11-01" },
      },
      {
        id: "adj-1",
        type: "adjustment",
        status: "pending",
        requester: { id: "emp-2", name: "B" },
        submittedAt: "2026-11-01",
        targetDate: "2026-11-01",
        title: "Giải trình",
        subtitle: "",
        details: { dateStr: "2026-11-01" },
      },
      {
        id: "swap-1",
        type: "shift_swap",
        status: "pending",
        requester: { id: "emp-3", name: "C" },
        submittedAt: "2026-11-01",
        targetDate: "2026-11-01",
        title: "Đổi ca",
        subtitle: "",
        details: { dateStr: "2026-11-01" },
      },
      {
        id: "ot-1",
        type: "overtime",
        status: "pending",
        requester: { id: "emp-4", name: "D" },
        submittedAt: "2026-11-01",
        targetDate: "2026-11-01",
        title: "Tăng ca",
        subtitle: "",
        details: {
          dateStr: "2026-11-01",
          rawPayload: { claimed_minutes: 90 },
        },
      },
    ];

    let batchRes;
    await act(async () => {
      batchRes = await result.current.batchApprove(items);
    });

    expect(batchRes?.succeeded).toHaveLength(4);
    expect(batchRes?.failed).toHaveLength(0);

    expect(api.approveLeaveRequest).toHaveBeenCalledWith("ws-test-123", "leave-1", {
      status: "approved",
    });
    expect(api.approveAttendanceAdjustment).toHaveBeenCalledWith(
      "ws-test-123",
      "adj-1",
      { status: "approved" }
    );
    expect(swapApi.approveShiftSwap).toHaveBeenCalledWith(
      "ws-test-123",
      "swap-1",
      { status: "approved" }
    );
    expect(otApi.approveOt).toHaveBeenCalledWith("ws-test-123", "emp-4", {
      date: "2026-11-01",
      approved_minutes: 90,
      status: "approved",
      manager_note: undefined,
    });
    expect(mockToastSuccess).toHaveBeenCalled();
  });

  it("rejects items with provided reason and calls appropriate APIs", async () => {
    const { result } = renderHook(() => useBatchApproval(), {
      wrapper: createWrapper(),
    });

    const items: UnifiedApprovalItem[] = [
      {
        id: "leave-1",
        type: "leave",
        status: "pending",
        requester: { id: "emp-1", name: "A" },
        submittedAt: "2026-11-01",
        targetDate: "2026-11-01",
        title: "Nghỉ phép",
        subtitle: "",
        details: { dateStr: "2026-11-01" },
      },
    ];

    let batchRes;
    await act(async () => {
      batchRes = await result.current.batchReject(items, "Lý do: Không đủ điều kiện");
    });

    expect(batchRes?.succeeded).toEqual(["leave-1"]);
    expect(api.approveLeaveRequest).toHaveBeenCalledWith("ws-test-123", "leave-1", {
      status: "rejected",
    });
    expect(mockToastSuccess).toHaveBeenCalled();
  });

  it("handles partial failures and reports failed count", async () => {
    vi.mocked(api.approveLeaveRequest).mockRejectedValueOnce(new Error("Network error"));

    const { result } = renderHook(() => useBatchApproval(), {
      wrapper: createWrapper(),
    });

    const items: UnifiedApprovalItem[] = [
      {
        id: "leave-fail",
        type: "leave",
        status: "pending",
        requester: { id: "emp-1", name: "A" },
        submittedAt: "2026-11-01",
        targetDate: "2026-11-01",
        title: "Nghỉ phép",
        subtitle: "",
        details: { dateStr: "2026-11-01" },
      },
      {
        id: "adj-ok",
        type: "adjustment",
        status: "pending",
        requester: { id: "emp-2", name: "B" },
        submittedAt: "2026-11-01",
        targetDate: "2026-11-01",
        title: "Giải trình",
        subtitle: "",
        details: { dateStr: "2026-11-01" },
      },
    ];

    let batchRes;
    await act(async () => {
      batchRes = await result.current.batchApprove(items);
    });

    expect(batchRes?.succeeded).toEqual(["adj-ok"]);
    expect(batchRes?.failed).toHaveLength(1);
    expect(batchRes?.failed[0].id).toBe("leave-fail");
    expect(mockToastError).toHaveBeenCalled();
  });
});
