import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, act } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import { useShiftEditor } from "../use-shift-editor";

// ── Mocks ──────────────────────────────────────────────────────────────────

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (k: string) => k }),
}));

vi.mock("@/components/ui/toast", () => ({
  useToast: () => ({
    success: vi.fn(),
    error: vi.fn(),
  }),
}));

vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => vi.fn(),
  useBlocker: () => ({ state: "inactive" }),
}));

vi.mock("@/lib/api", () => ({
  fetchShifts: vi.fn(),
  createShift: vi.fn(),
  updateShift: vi.fn(),
  fetchDefaultAttendanceRules: vi.fn().mockResolvedValue({}),
}));

// ── Helpers ────────────────────────────────────────────────────────────────

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });
  return function Wrapper({ children }: { children: React.ReactNode }) {
    return (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );
  };
}

// ── Tests ──────────────────────────────────────────────────────────────────

describe("useShiftEditor", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("create mode (no shiftId)", () => {
    it("returns default initial state", () => {
      const { result } = renderHook(() => useShiftEditor(), {
        wrapper: createWrapper(),
      });

      expect(result.current.isEditMode).toBe(false);
      expect(result.current.shiftName).toBe("");
      expect(result.current.shiftSymbol).toBe("");
      expect(result.current.locationId).toBeNull();
      expect(result.current.employerId).toBeNull();
      expect(result.current.isDirty).toBe(false);
      expect(result.current.isSaveDisabled).toBe(true); // empty name
      expect(result.current.isLoadingShifts).toBe(false);
    });

    it("starts with one default interval", () => {
      const { result } = renderHook(() => useShiftEditor(), {
        wrapper: createWrapper(),
      });

      expect(result.current.intervalDrafts).toHaveLength(1);
      const iv = result.current.intervalDrafts[0];
      expect(iv.startTime).toBe("08:00");
      expect(iv.endTime).toBe("17:00");
      expect(iv.breakDuration).toBe(60);
    });

    it("isSaveDisabled is true when shiftName is empty", () => {
      const { result } = renderHook(() => useShiftEditor(), {
        wrapper: createWrapper(),
      });

      expect(result.current.isSaveDisabled).toBe(true);
    });

    it("isSaveDisabled becomes false when shiftName is set", async () => {
      const { result } = renderHook(() => useShiftEditor(), {
        wrapper: createWrapper(),
      });

      await act(async () => {
        result.current.setShiftName("Morning Shift");
      });

      expect(result.current.shiftName).toBe("Morning Shift");
      expect(result.current.isDirty).toBe(true);
      expect(result.current.isSaveDisabled).toBe(false);
    });
  });

  describe("interval management", () => {
    it("addInterval appends a new default interval", async () => {
      const { result } = renderHook(() => useShiftEditor(), {
        wrapper: createWrapper(),
      });

      expect(result.current.intervalDrafts).toHaveLength(1);

      await act(async () => {
        result.current.addInterval();
      });

      expect(result.current.intervalDrafts).toHaveLength(2);
      expect(result.current.isDirty).toBe(true);
    });

    it("removeInterval does nothing when only one interval remains", () => {
      const { result } = renderHook(() => useShiftEditor(), {
        wrapper: createWrapper(),
      });

      const key = result.current.intervalDrafts[0].key;
      result.current.removeInterval(key);

      expect(result.current.intervalDrafts).toHaveLength(1); // unchanged
    });

    it("removeInterval removes the specified interval", async () => {
      const { result } = renderHook(() => useShiftEditor(), {
        wrapper: createWrapper(),
      });

      await act(async () => {
        result.current.addInterval();
      });
      const secondKey = result.current.intervalDrafts[1].key;

      await act(async () => {
        result.current.removeInterval(secondKey);
      });

      expect(result.current.intervalDrafts).toHaveLength(1);
    });

    it("updateIntervalDraft patches a specific interval", async () => {
      const { result } = renderHook(() => useShiftEditor(), {
        wrapper: createWrapper(),
      });

      const key = result.current.intervalDrafts[0].key;
      await act(async () => {
        result.current.updateIntervalDraft(key, { startTime: "09:00", intervalName: "Core" });
      });

      const updated = result.current.intervalDrafts[0];
      expect(updated.startTime).toBe("09:00");
      expect(updated.intervalName).toBe("Core");
      expect(updated.endTime).toBe("17:00"); // unchanged
      expect(result.current.isDirty).toBe(true);
    });
  });

  describe("correction mode", () => {
    it("has isCorrection false by default", () => {
      const { result } = renderHook(() => useShiftEditor(), {
        wrapper: createWrapper(),
      });

      expect(result.current.isCorrection).toBe(false);
      expect(result.current.correctionEffectiveDate).toBeTruthy(); // today's date
    });

    it("setIsCorrection toggles correction flag and marks dirty", async () => {
      const { result } = renderHook(() => useShiftEditor(), {
        wrapper: createWrapper(),
      });

      await act(async () => {
        result.current.setIsCorrection(true);
      });

      expect(result.current.isCorrection).toBe(true);
      expect(result.current.isDirty).toBe(true);
    });
  });

  describe("location and employer", () => {
    it("setLocationId updates and marks dirty", async () => {
      const { result } = renderHook(() => useShiftEditor(), {
        wrapper: createWrapper(),
      });

      await act(async () => {
        result.current.setLocationId("loc-1");
      });

      expect(result.current.locationId).toBe("loc-1");
      expect(result.current.isDirty).toBe(true);
    });

    it("setEmployerId updates and marks dirty", async () => {
      const { result } = renderHook(() => useShiftEditor(), {
        wrapper: createWrapper(),
      });

      await act(async () => {
        result.current.setEmployerId("emp-1");
      });

      expect(result.current.employerId).toBe("emp-1");
      expect(result.current.isDirty).toBe(true);
    });
  });

});
