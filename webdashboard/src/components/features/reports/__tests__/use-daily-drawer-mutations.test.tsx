import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook } from "@testing-library/react";
import React from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import type { DailyAttendanceReport } from "@/lib/api";

import * as api from "@/lib/api";

import { useDailyDrawerMutations } from "../use-daily-drawer-mutations";

vi.mock("@/lib/api", () => ({
  adminConfirmAttendance: vi.fn(),
  approveOt: vi.fn(),
  reprocessAttendance: vi.fn(),
  resolveUnscheduledSchedule: vi.fn(),
}));

vi.mock("@/components/ui/toast", () => ({
  useToast: () => ({ error: vi.fn(), success: vi.fn() }),
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

describe("useDailyDrawerMutations", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("calls atomic resolveUnscheduledSchedule with shift_id, reason and idempotency key", async () => {
    const queryClient = new QueryClient({ defaultOptions: { mutations: { retry: false } } });
    const wrapper = ({ children }: { children: React.ReactNode }) => (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );
    vi.mocked(api.resolveUnscheduledSchedule).mockResolvedValue({
      data: {
        batch_id: "batch-1",
        published_at: "2026-11-26T09:00:00Z",
        recalculated: 1,
        revision_id: "rev-1",
      },
    });
    const onResolveSuccess = vi.fn();

    const { result } = renderHook(
      () =>
        useDailyDrawerMutations({
          employeeId: "emp-1",
          onConfirmSuccess: vi.fn(),
          onReprocessSuccess: vi.fn(),
          onResolveSuccess,
        }),
      { wrapper },
    );

    await act(async () => {
      await result.current.resolveUnscheduledMutation.mutateAsync({
        reason: "Confirmed unscheduled overtime shift",
        row: {
          date: "2026-11-22",
          employee_id: "emp-1",
          employee_name: "Test",
          employment_id: "empl-1",
        } as DailyAttendanceReport,
        shiftId: "shift-1",
      });
    });

    expect(api.resolveUnscheduledSchedule).toHaveBeenCalledWith(
      "empl-1",
      "2026-11-22",
      { reason: "Confirmed unscheduled overtime shift", shift_id: "shift-1" },
      expect.any(String),
    );
    expect(onResolveSuccess).toHaveBeenCalledTimes(1);
  });
});
