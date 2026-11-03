import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach, type Mock } from "vitest";

import attendance from "@/i18n/locales/vi/attendance.json";

import {
  useWorkingPatternForm,
  isCycleLengthInvalid,
  convertGridToDetails,
} from "../use-working-pattern-form";

// The setup's react-i18next mock resolves keys against the merged vi locale,
// so patterns.* keys translate to real Vietnamese strings.
const PATTERN_MSG = attendance.patterns.messages;

const routerMock = vi.hoisted(() => ({ useNavigate: vi.fn() }));
vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => routerMock.useNavigate(),
}));

const api = vi.hoisted(() => ({
  createWorkingPattern: vi.fn(),
  updateWorkingPattern: vi.fn(),
  fetchShifts: vi.fn(),
  fetchWorkingPatterns: vi.fn(),
  fetchPatternDetails: vi.fn(),
  getStandaloneOrganizationId: vi.fn(() => "ws-test-123"),
}));

vi.mock("@/lib/api", () => api);

const toastMock = vi.hoisted(() => ({
  success: vi.fn(),
  error: vi.fn(),
  info: vi.fn(),
  warning: vi.fn(),
  show: vi.fn(),
}));

vi.mock("@/components/ui/toast", () => ({
  useToast: () => toastMock,
}));

const WS_ID = "ws-test-123";

let queryClient: QueryClient;
let navigateFn: ReturnType<typeof vi.fn>;

function createWrapper() {
  queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  const Wrapper = ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
  Wrapper.displayName = "QueryClientWrapper";
  return Wrapper;
}

function setupQueries() {
  (api.fetchShifts as Mock).mockResolvedValue([]);
  (api.fetchWorkingPatterns as Mock).mockResolvedValue([]);
  (api.fetchPatternDetails as Mock).mockResolvedValue([]);
}

describe("pure validation helpers", () => {
  it("isCycleLengthInvalid only rejects out-of-range rotation cycles", () => {
    expect(isCycleLengthInvalid("rotation", 0)).toBe(true);
    expect(isCycleLengthInvalid("rotation", -5)).toBe(true);
    expect(isCycleLengthInvalid("rotation", 367)).toBe(true);
    expect(isCycleLengthInvalid("rotation", 1)).toBe(false);
    expect(isCycleLengthInvalid("rotation", 366)).toBe(false);
    expect(isCycleLengthInvalid("weekly", 0)).toBe(false);
    expect(isCycleLengthInvalid("weekly", 1000)).toBe(false);
  });

  it("convertGridToDetails flattens shift ids and emits null for empty days", () => {
    const grid: string[][] = [["s1", "s2"], [], ["s3"]];
    expect(convertGridToDetails(grid)).toEqual([
      { schedule_day: 0, shift_id: "s1" },
      { schedule_day: 0, shift_id: "s2" },
      { schedule_day: 1, shift_id: null },
      { schedule_day: 2, shift_id: "s3" },
    ]);
  });
});

describe("useWorkingPatternForm", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    api.getStandaloneOrganizationId.mockReturnValue(WS_ID);
    navigateFn = vi.fn();
    routerMock.useNavigate.mockReturnValue(navigateFn);
    setupQueries();
  });

  describe("initial state", () => {
    it("starts as an empty weekly pattern with a 7-day grid", () => {
      const { result } = renderHook(() => useWorkingPatternForm(), { wrapper: createWrapper() });

      expect(result.current.isEditMode).toBe(false);
      expect(result.current.wsId).toBe(WS_ID);
      expect(result.current.name).toBe("");
      expect(result.current.description).toBe("");
      expect(result.current.patternType).toBe("weekly");
      expect(result.current.cycleLengthDays).toBe(7);
      expect(result.current.grid).toHaveLength(7);
      expect(result.current.grid.every((day: string[]) => day.length === 0)).toBe(true);
      expect(result.current.isDirty).toBe(false);
      expect(result.current.isSaving).toBe(false);
    });

    it("fetches available shifts for the active workspace", async () => {
      (api.fetchShifts as Mock).mockResolvedValue([{ id: "s1", name: "Morning" }]);

      const { result } = renderHook(() => useWorkingPatternForm(), { wrapper: createWrapper() });

      await waitFor(() => expect(result.current.shifts).toHaveLength(1));
      expect(api.fetchShifts).toHaveBeenCalledWith(WS_ID);
    });
  });

  describe("grid actions", () => {
    it("adds a shift to a day and marks the form dirty", () => {
      const { result } = renderHook(() => useWorkingPatternForm(), { wrapper: createWrapper() });

      act(() => {
        result.current.addShiftToDay(0, "s1");
      });

      expect(result.current.grid[0]).toEqual(["s1"]);
      expect(result.current.isDirty).toBe(true);
    });

    it("does not duplicate a shift already assigned to the same day", () => {
      const { result } = renderHook(() => useWorkingPatternForm(), { wrapper: createWrapper() });

      act(() => {
        result.current.addShiftToDay(0, "s1");
        result.current.addShiftToDay(0, "s1");
      });

      expect(result.current.grid[0]).toEqual(["s1"]);
    });

    it("removes a shift from a day and marks the form dirty", () => {
      const { result } = renderHook(() => useWorkingPatternForm(), { wrapper: createWrapper() });

      act(() => {
        result.current.addShiftToDay(0, "s1");
      });
      act(() => {
        result.current.removeShiftFromDay(0, "s1");
      });

      expect(result.current.grid[0]).toEqual([]);
      expect(result.current.isDirty).toBe(true);
    });

    it("leaves the grid untouched when removing a shift that is not present", () => {
      const { result } = renderHook(() => useWorkingPatternForm(), { wrapper: createWrapper() });

      act(() => {
        result.current.removeShiftFromDay(0, "s1");
      });

      expect(result.current.grid[0]).toEqual([]);
    });

    it("clears every day of the grid and marks the form dirty", () => {
      const { result } = renderHook(() => useWorkingPatternForm(), { wrapper: createWrapper() });

      act(() => {
        result.current.addShiftToDay(0, "s1");
        result.current.addShiftToDay(2, "s2");
      });
      act(() => {
        result.current.clearAllDays();
      });

      expect(result.current.grid.every((day: string[]) => day.length === 0)).toBe(true);
      expect(result.current.isDirty).toBe(true);
    });
  });

  describe("prefillStandard", () => {
    it("fills Monday-Friday with the first shift when shifts exist", async () => {
      (api.fetchShifts as Mock).mockResolvedValue([{ id: "s1", name: "Morning" }]);
      const { result } = renderHook(() => useWorkingPatternForm(), { wrapper: createWrapper() });

      await waitFor(() => expect(result.current.shifts).toHaveLength(1));

      act(() => {
        result.current.prefillStandard();
      });

      expect(result.current.grid[1]).toEqual(["s1"]);
      expect(result.current.grid[5]).toEqual(["s1"]);
      expect(result.current.grid[0]).toEqual([]);
      expect(result.current.grid[6]).toEqual([]);
      expect(result.current.isDirty).toBe(true);
    });

    it("shows an error toast and does nothing when no shifts are available", async () => {
      const { result } = renderHook(() => useWorkingPatternForm(), { wrapper: createWrapper() });

      await waitFor(() => expect(result.current.shifts).toHaveLength(0));

      act(() => {
        result.current.prefillStandard();
      });

      expect(toastMock.error).toHaveBeenCalledWith(PATTERN_MSG.no_shifts_available);
      expect(result.current.grid.every((day: string[]) => day.length === 0)).toBe(true);
      expect(result.current.isDirty).toBe(false);
    });
  });

  describe("pattern type transitions", () => {
    it("switches to rotation without resizing a grid that already matches the cycle", () => {
      const { result } = renderHook(() => useWorkingPatternForm(), { wrapper: createWrapper() });

      act(() => {
        result.current.handlePatternTypeChange("rotation");
      });

      expect(result.current.patternType).toBe("rotation");
      expect(result.current.grid).toHaveLength(7);
      expect(result.current.isDirty).toBe(true);
    });

    it("truncates the grid when a rotation cycle is shorter than the weekly grid", () => {
      const { result } = renderHook(() => useWorkingPatternForm(), { wrapper: createWrapper() });

      act(() => {
        result.current.handleCycleLengthChange(3);
      });
      act(() => {
        result.current.handlePatternTypeChange("rotation");
      });

      expect(result.current.cycleLengthDays).toBe(3);
      expect(result.current.grid).toHaveLength(3);
    });

    it("pads the grid back to 7 days when switching from rotation back to weekly", () => {
      const { result } = renderHook(() => useWorkingPatternForm(), { wrapper: createWrapper() });

      act(() => {
        result.current.handleCycleLengthChange(3);
        result.current.handlePatternTypeChange("rotation");
      });
      act(() => {
        result.current.handlePatternTypeChange("weekly");
      });

      expect(result.current.patternType).toBe("weekly");
      expect(result.current.grid).toHaveLength(7);
    });
  });

  describe("cycle length changes", () => {
    it("pads the grid when the cycle grows", () => {
      const { result } = renderHook(() => useWorkingPatternForm(), { wrapper: createWrapper() });

      act(() => {
        result.current.handleCycleLengthChange(10);
      });

      expect(result.current.cycleLengthDays).toBe(10);
      expect(result.current.grid).toHaveLength(10);
      expect(result.current.isDirty).toBe(true);
    });

    it("truncates the grid when the cycle shrinks", () => {
      const { result } = renderHook(() => useWorkingPatternForm(), { wrapper: createWrapper() });

      act(() => {
        result.current.handleCycleLengthChange(4);
      });

      expect(result.current.cycleLengthDays).toBe(4);
      expect(result.current.grid).toHaveLength(4);
    });

    it("clamps lengths to the supported 1..366 range", () => {
      const { result } = renderHook(() => useWorkingPatternForm(), { wrapper: createWrapper() });

      act(() => {
        result.current.handleCycleLengthChange(0);
      });
      expect(result.current.cycleLengthDays).toBe(1);
      expect(result.current.grid).toHaveLength(1);

      act(() => {
        result.current.handleCycleLengthChange(400);
      });
      expect(result.current.cycleLengthDays).toBe(366);
      expect(result.current.grid).toHaveLength(366);
    });
  });

  describe("edit mode initialization", () => {
    it("hydrates name, description, type, cycle length and grid from pattern details", async () => {
      (api.fetchWorkingPatterns as Mock).mockResolvedValue([
        {
          id: "tpl-1",
          name: "Weekend Rotation",
          description: "Sat/Sun",
          pattern_type: "rotation",
          cycle_length_days: 3,
        },
      ]);
      (api.fetchPatternDetails as Mock).mockResolvedValue([
        { schedule_day: 0, shift_id: "s1" },
        { schedule_day: 1, shift_id: null },
        { schedule_day: 2, shift_id: "s2" },
      ]);

      const { result } = renderHook(() => useWorkingPatternForm("tpl-1"), {
        wrapper: createWrapper(),
      });

      await waitFor(() => expect(result.current.name).toBe("Weekend Rotation"));

      expect(result.current.isEditMode).toBe(true);
      expect(result.current.description).toBe("Sat/Sun");
      expect(result.current.patternType).toBe("rotation");
      expect(result.current.cycleLengthDays).toBe(3);
      expect(result.current.grid).toHaveLength(3);
      expect(result.current.grid[0]).toEqual(["s1"]);
      expect(result.current.grid[1]).toEqual([]);
      expect(result.current.grid[2]).toEqual(["s2"]);
    });

    it("does not hydrate when the current pattern is missing", async () => {
      (api.fetchWorkingPatterns as Mock).mockResolvedValue([]);

      const { result } = renderHook(() => useWorkingPatternForm("missing-id"), {
        wrapper: createWrapper(),
      });

      await waitFor(() => expect(result.current.isLoadingPatterns).toBe(false));

      expect(result.current.name).toBe("");
      expect(result.current.patternType).toBe("weekly");
    });
  });

  describe("createPattern mutation", () => {
    it("creates the pattern, invalidates the cache, navigates and resets dirty on success", async () => {
      (api.createWorkingPattern as Mock).mockResolvedValue({ id: "tpl-new", name: "New" });
      const { result } = renderHook(() => useWorkingPatternForm(), { wrapper: createWrapper() });
      const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

      const payload = { name: "New", grid: [] };
      act(() => {
        result.current.createPattern(payload as never);
      });

      await waitFor(() => {
        expect(api.createWorkingPattern).toHaveBeenCalledWith(WS_ID, payload);
      });
      await waitFor(() => {
        expect(toastMock.success).toHaveBeenCalledWith(PATTERN_MSG.create_success);
      });
      await waitFor(() => {
        expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["working-patterns"] });
      });
      await waitFor(() => expect(navigateFn).toHaveBeenCalledWith({
        to: "/shifts/rules",
        search: { tab: "patterns" },
      }));
      await waitFor(() => expect(result.current.isDirty).toBe(false));
    });

    it("shows an error toast when creation fails", async () => {
      (api.createWorkingPattern as Mock).mockRejectedValue(new Error("boom"));
      const { result } = renderHook(() => useWorkingPatternForm(), { wrapper: createWrapper() });

      act(() => {
        result.current.createPattern({ name: "New" } as never);
      });

      await waitFor(() => {
        expect(toastMock.error).toHaveBeenCalledWith(PATTERN_MSG.create_error);
      });
      expect(navigateFn).not.toHaveBeenCalled();
    });

    it("reports isSaving while the creation mutation is pending", async () => {
      let resolveCreate: ((value: unknown) => void) | undefined;
      const deferred = new Promise((resolve) => { resolveCreate = resolve; });
      (api.createWorkingPattern as Mock).mockReturnValue(deferred);
      const { result } = renderHook(() => useWorkingPatternForm(), { wrapper: createWrapper() });

      act(() => {
        result.current.createPattern({ name: "New" } as never);
      });
      await waitFor(() => expect(result.current.isSaving).toBe(true));

      await act(async () => {
        resolveCreate?.({});
      });
      await waitFor(() => expect(result.current.isSaving).toBe(false));
    });
  });

  describe("updatePattern mutation", () => {
    it("updates the pattern, invalidates both caches and navigates on success", async () => {
      (api.updateWorkingPattern as Mock).mockResolvedValue({ id: "tpl-1", name: "Updated" });
      const { result } = renderHook(() => useWorkingPatternForm("tpl-1"), {
        wrapper: createWrapper(),
      });
      const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

      const payload = { name: "Updated" };
      act(() => {
        result.current.updatePattern(payload as never);
      });

      await waitFor(() => {
        expect(api.updateWorkingPattern).toHaveBeenCalledWith(WS_ID, "tpl-1", payload);
      });
      await waitFor(() => {
        expect(toastMock.success).toHaveBeenCalledWith(PATTERN_MSG.update_success);
      });
      await waitFor(() => {
        expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["working-patterns"] });
        expect(invalidateSpy).toHaveBeenCalledWith({
          queryKey: ["pattern-details", WS_ID, "tpl-1"],
        });
      });
      await waitFor(() => expect(navigateFn).toHaveBeenCalledWith({
        to: "/shifts/rules",
        search: { tab: "patterns" },
      }));
      await waitFor(() => expect(result.current.isDirty).toBe(false));
    });

    it("shows an error toast when the update fails", async () => {
      (api.updateWorkingPattern as Mock).mockRejectedValue(new Error("boom"));
      const { result } = renderHook(() => useWorkingPatternForm("tpl-1"), {
        wrapper: createWrapper(),
      });

      act(() => {
        result.current.updatePattern({ name: "Updated" } as never);
      });

      await waitFor(() => {
        expect(toastMock.error).toHaveBeenCalledWith(PATTERN_MSG.update_error);
      });
      expect(navigateFn).not.toHaveBeenCalled();
    });
  });
});
