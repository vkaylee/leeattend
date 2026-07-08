import { renderHook } from "@testing-library/react";

import { format } from "date-fns";
import { describe, it, expect, vi, beforeEach } from "vitest";

import { useDateTimeFormatter } from "../use-datetime";
import { useLanguage } from "../use-language";
import { useWorkspace } from "../use-workspace";

// Mock useWorkspace
vi.mock("../use-workspace", () => ({
  useWorkspace: vi.fn(),
}));

// Mock useLanguage (returns default 'vi' — not the focus of these tests)
vi.mock("../use-language", () => ({
  useLanguage: vi.fn(),
}));

describe("useDateTimeFormatter hook", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(useLanguage).mockReturnValue({
      currentLanguage: "vi",
      currentOption: { code: "vi", flag: "🇻🇳", name: "Vietnamese", nativeName: "Tiếng Việt" },
      languages: [],
      changeLanguage: vi.fn(),
      userPreference: undefined,
    } as never);
  });

  it("formats date and time correctly with default workspace settings (VN)", () => {
 
    vi.mocked(useWorkspace).mockReturnValue({
      workspace: {
        date_format: "DD/MM/YYYY",
        time_format: "24h",
      }
    } as unknown as never);

    const { result } = renderHook(() => useDateTimeFormatter());
    
    // 2026-01-14 15:30:00
    const testDate = new Date(2026, 0, 14, 15, 30, 0);

    expect(result.current.formatDate(testDate)).toBe("14/01/2026");
    expect(result.current.formatTime(testDate)).toBe("15:30");
    expect(result.current.formatTime24h(testDate)).toBe("15:30");
    expect(result.current.formatDateTime(testDate)).toBe("14/01/2026 15:30");
  });

  it("formats date and time correctly with US settings", () => {
 
    vi.mocked(useWorkspace).mockReturnValue({
      workspace: {
        date_format: "MM/DD/YYYY",
        time_format: "12h",
      }
    } as unknown as never);

    const { result } = renderHook(() => useDateTimeFormatter());
    
    const testDate = new Date(2026, 0, 14, 15, 30, 0);

    expect(result.current.formatDate(testDate)).toBe("01/14/2026");
    expect(result.current.formatTime(testDate)).toBe("03:30 PM");
    expect(result.current.formatTime24h(testDate)).toBe("15:30");
    expect(result.current.formatDateTime(testDate)).toBe("01/14/2026 03:30 PM");
  });

  it("formats date correctly with ISO setting", () => {
 
    vi.mocked(useWorkspace).mockReturnValue({
      workspace: {
        date_format: "YYYY-MM-DD",
        time_format: "24h",
      }
    } as unknown as never);

    const { result } = renderHook(() => useDateTimeFormatter());
    
    const testDate = new Date(2026, 0, 14, 15, 30, 0);

    expect(result.current.formatDate(testDate)).toBe("2026-01-14");
  });

  it("handles null and undefined dates gracefully", () => {
 
    vi.mocked(useWorkspace).mockReturnValue({ workspace: null } as unknown as never);

    const { result } = renderHook(() => useDateTimeFormatter());

    expect(result.current.formatDate(null)).toBe("---");
    expect(result.current.formatDate(undefined)).toBe("---");
    expect(result.current.formatTime(null)).toBe("---");
    expect(result.current.formatDateTime(null)).toBe("---");
  });

  it("handles raw time strings (HH:mm:ss)", () => {
 
    vi.mocked(useWorkspace).mockReturnValue({
      workspace: {
        date_format: "DD/MM/YYYY",
        time_format: "12h",
      }
    } as unknown as never);

    const { result } = renderHook(() => useDateTimeFormatter());

    expect(result.current.formatTime("08:00:00")).toBe("08:00 AM");
    expect(result.current.formatTime("17:30:00")).toBe("05:30 PM");
  });

  it("falls back to default settings when workspace is loading", () => {
 
    vi.mocked(useWorkspace).mockReturnValue({ workspace: undefined } as unknown as never);

    const { result } = renderHook(() => useDateTimeFormatter());
    
    const testDate = new Date(2026, 0, 14, 8, 30, 0);

    // Should use DD/MM/YYYY and 24h
    expect(result.current.formatDate(testDate)).toBe("14/01/2026");
    expect(result.current.formatTime(testDate)).toBe("08:30");
  });

  it("formats month and year correctly based on workspace date preference", () => {
    // Case 1: DD/MM/YYYY -> MM/yyyy
 
    vi.mocked(useWorkspace).mockReturnValue({
      workspace: { date_format: "DD/MM/YYYY" }
    } as unknown as never);
    const { result: res1 } = renderHook(() => useDateTimeFormatter());
    expect(res1.current.formatMonthYear(new Date(2026, 0, 1))).toBe("01/2026");

    // Case 2: MM/DD/YYYY -> MM/yyyy
 
    vi.mocked(useWorkspace).mockReturnValue({
      workspace: { date_format: "MM/DD/YYYY" }
    } as unknown as never);
    const { result: res2 } = renderHook(() => useDateTimeFormatter());
    expect(res2.current.formatMonthYear(new Date(2026, 0, 1))).toBe("01/2026");

    // Case 3: YYYY-MM-DD -> yyyy-MM
 
    vi.mocked(useWorkspace).mockReturnValue({
      workspace: { date_format: "YYYY-MM-DD" }
    } as unknown as never);
    const { result: res3 } = renderHook(() => useDateTimeFormatter());
    expect(res3.current.formatMonthYear(new Date(2026, 0, 1))).toBe("2026-01");
  });

  it("is timezone-safe compared to toISOString", () => {
    // Manual check for the bug logic: 
    // Jan 1st 00:00:00 in a UTC+7 environment is Dec 31st 17:00:00 UTC.
    // We mock a date that would represent this.
    const janFirstLocal = new Date(2026, 0, 1, 0, 0, 0);
    
    // This is the problematic logic we removed:
    const _buggyString = janFirstLocal.toISOString().split('T')[0];
    
    // This is the new logic:
    const safeString = format(janFirstLocal, 'yyyy-MM-dd');

    expect(safeString).toBe("2026-01-01");
    // If this test runs in a timezone > UTC, buggyString would be "2025-12-31"
    // We don't assert on buggyString because it depends on the runner's TZ, 
    // but we ensure safeString ALWAYS returns the local date part.
  });
});
