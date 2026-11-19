import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";
import React from "react";

import { beforeEach, describe, expect, it, vi } from "vitest";

import { useUnifiedInbox } from "../use-unified-inbox";

const {
  permissions,
  fetchLeaveRequests,
  fetchAttendanceAdjustments,
  fetchShiftSwaps,
  fetchAttendanceSummaries,
  useAllEmployees,
} = vi.hoisted(() => ({
  permissions: new Set<string>(),
  fetchLeaveRequests: vi.fn().mockResolvedValue({ data: [] }),
  fetchAttendanceAdjustments: vi.fn().mockResolvedValue({ data: [] }),
  fetchShiftSwaps: vi.fn().mockResolvedValue([]),
  fetchAttendanceSummaries: vi.fn().mockResolvedValue([]),
  useAllEmployees: vi.fn(() => ({ employees: [] })),
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (_key: string, fallback: string) => fallback }),
}));

vi.mock("@/hooks/use-permissions", () => ({
  useMyPermissions: () => ({
    hasAnyPermission: (...required: string[]) =>
      required.some((permission) => permissions.has(permission)),
    hasPermission: (permission: string) => permissions.has(permission),
  }),
}));

vi.mock("@/hooks/use-departments", () => ({
  useDepartments: () => ({ data: [] }),
}));

vi.mock("@/hooks/use-employees", () => ({
  useAllEmployees: (...args: Parameters<typeof useAllEmployees>) => useAllEmployees(...args),
}));

vi.mock("@/lib/api", () => ({
  fetchAttendanceAdjustments,
  fetchLeaveRequests,
}));

vi.mock("@/lib/api/attendance-overtime", () => ({
  fetchAttendanceSummaries,
}));

vi.mock("@/lib/api/shift-swaps", () => ({
  fetchShiftSwaps,
}));

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });

  return function Wrapper({ children }: { children: React.ReactNode }) {
    return (
      <QueryClientProvider client={queryClient}>
        {children}
      </QueryClientProvider>
    );
  };
}

describe("useUnifiedInbox", () => {
  beforeEach(() => {
    permissions.clear();
    vi.clearAllMocks();
  });

  it("does not fetch the employee roster without employee read permission", async () => {
    renderHook(() => useUnifiedInbox(), { wrapper: createWrapper() });

    await waitFor(() => {
      expect(fetchLeaveRequests).not.toHaveBeenCalled();
    });

    expect(useAllEmployees).toHaveBeenCalledWith({ enabled: false });
  });

  it("does not fetch approval sources outside current permissions", async () => {
    permissions.add("attendance:adjustments:approve");
    permissions.add("overtime:approve");

    renderHook(() => useUnifiedInbox(), { wrapper: createWrapper() });

    await waitFor(() => {
      expect(fetchAttendanceAdjustments).toHaveBeenCalledWith({ limit: 100 });
      expect(fetchAttendanceSummaries).toHaveBeenCalled();
    });

    expect(fetchLeaveRequests).not.toHaveBeenCalled();
    expect(fetchShiftSwaps).not.toHaveBeenCalled();
  });
});
