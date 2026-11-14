import { renderHook, act } from "@testing-library/react";

import { describe, it, expect, beforeEach, vi } from "vitest";

import { useDesktopLayout } from "../use-desktop-layout";

describe("useDesktopLayout", () => {
  beforeEach(() => {
    localStorage.clear();
    vi.clearAllMocks();
  });

  it("initializes with default enabled widgets and order", () => {
    const { result } = renderHook(() => useDesktopLayout());

    expect(result.current.enabledIds.length).toBeGreaterThanOrEqual(4);
    expect(result.current.isWidgetEnabled("quick-punch")).toBe(true);
    expect(result.current.isWidgetEnabled("system-health")).toBe(true);
    expect(result.current.isCustomizing).toBe(false);
  });

  it("toggles widget off and on", () => {
    const { result } = renderHook(() => useDesktopLayout());

    expect(result.current.isWidgetEnabled("quick-punch")).toBe(true);

    act(() => {
      result.current.toggleWidget("quick-punch");
    });
    expect(result.current.isWidgetEnabled("quick-punch")).toBe(false);
    expect(result.current.enabledIds).not.toContain("quick-punch");

    act(() => {
      result.current.toggleWidget("quick-punch");
    });
    expect(result.current.isWidgetEnabled("quick-punch")).toBe(true);
    expect(result.current.enabledIds).toContain("quick-punch");
  });

  it("opens and closes customizer modal", () => {
    const { result } = renderHook(() => useDesktopLayout());

    expect(result.current.isCustomizing).toBe(false);

    act(() => {
      result.current.openCustomizer();
    });
    expect(result.current.isCustomizing).toBe(true);

    act(() => {
      result.current.closeCustomizer();
    });
    expect(result.current.isCustomizing).toBe(false);
  });

  it("resets to default layout", () => {
    const { result } = renderHook(() => useDesktopLayout());

    act(() => {
      result.current.toggleWidget("quick-punch");
      result.current.toggleWidget("system-health");
    });
    expect(result.current.isWidgetEnabled("quick-punch")).toBe(false);

    act(() => {
      result.current.resetToDefault();
    });
    expect(result.current.isWidgetEnabled("quick-punch")).toBe(true);
    expect(result.current.isWidgetEnabled("system-health")).toBe(true);
  });
});
