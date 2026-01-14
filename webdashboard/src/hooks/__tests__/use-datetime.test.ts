import { renderHook } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { useDateTimeFormatter } from "../use-datetime";
import { useWorkspace } from "../use-workspace";

// Mock useWorkspace
vi.mock("../use-workspace", () => ({
  useWorkspace: vi.fn(),
}));

describe("useDateTimeFormatter hook", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("formats date and time correctly with default workspace settings (VN)", () => {
    (useWorkspace as any).mockReturnValue({
      workspace: {
        date_format: "DD/MM/YYYY",
        time_format: "24h",
      }
    });

    const { result } = renderHook(() => useDateTimeFormatter());
    
    // 2026-01-14 15:30:00
    const testDate = new Date(2026, 0, 14, 15, 30, 0);

    expect(result.current.formatDate(testDate)).toBe("14/01/2026");
    expect(result.current.formatTime(testDate)).toBe("15:30");
    expect(result.current.formatDateTime(testDate)).toBe("14/01/2026 15:30");
  });

  it("formats date and time correctly with US settings", () => {
    (useWorkspace as any).mockReturnValue({
      workspace: {
        date_format: "MM/DD/YYYY",
        time_format: "12h",
      }
    });

    const { result } = renderHook(() => useDateTimeFormatter());
    
    const testDate = new Date(2026, 0, 14, 15, 30, 0);

    expect(result.current.formatDate(testDate)).toBe("01/14/2026");
    expect(result.current.formatTime(testDate)).toBe("03:30 PM");
    expect(result.current.formatDateTime(testDate)).toBe("01/14/2026 03:30 PM");
  });

  it("formats date correctly with ISO setting", () => {
    (useWorkspace as any).mockReturnValue({
      workspace: {
        date_format: "YYYY-MM-DD",
        time_format: "24h",
      }
    });

    const { result } = renderHook(() => useDateTimeFormatter());
    
    const testDate = new Date(2026, 0, 14, 15, 30, 0);

    expect(result.current.formatDate(testDate)).toBe("2026-01-14");
  });

  it("handles null and undefined dates gracefully", () => {
    (useWorkspace as any).mockReturnValue({ workspace: null });

    const { result } = renderHook(() => useDateTimeFormatter());

    expect(result.current.formatDate(null)).toBe("---");
    expect(result.current.formatDate(undefined)).toBe("---");
    expect(result.current.formatTime(null)).toBe("---");
    expect(result.current.formatDateTime(null)).toBe("---");
  });

  it("handles raw time strings (HH:mm:ss)", () => {
    (useWorkspace as any).mockReturnValue({
      workspace: {
        date_format: "DD/MM/YYYY",
        time_format: "12h",
      }
    });

    const { result } = renderHook(() => useDateTimeFormatter());

    expect(result.current.formatTime("08:00:00")).toBe("08:00 AM");
    expect(result.current.formatTime("17:30:00")).toBe("05:30 PM");
  });

  it("falls back to default settings when workspace is loading", () => {
    (useWorkspace as any).mockReturnValue({ workspace: undefined });

    const { result } = renderHook(() => useDateTimeFormatter());
    
    const testDate = new Date(2026, 0, 14, 8, 30, 0);

    // Should use DD/MM/YYYY and 24h
    expect(result.current.formatDate(testDate)).toBe("14/01/2026");
    expect(result.current.formatTime(testDate)).toBe("08:30");
  });
});
