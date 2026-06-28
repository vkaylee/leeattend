import { renderHook, act } from "@testing-library/react";

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

import { useNavSidebar } from "@/hooks/use-nav-sidebar";

describe("useNavSidebar", () => {
  beforeEach(() => {
    localStorage.clear();
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  // ── Pin state ──────────────────────────────────────────────────

  it("defaults isPinned to false when no localStorage entry", () => {
    const { result } = renderHook(() => useNavSidebar("/"));
    expect(result.current.isPinned).toBe(false);
  });

  it("reads isPinned from localStorage", () => {
    localStorage.setItem("nav-rail-pinned", "true");
    const { result } = renderHook(() => useNavSidebar("/"));
    expect(result.current.isPinned).toBe(true);
  });

  it("togglePin flips and persists", () => {
    const { result } = renderHook(() => useNavSidebar("/"));

    act(() => { result.current.togglePin(); });
    expect(result.current.isPinned).toBe(true);
    expect(localStorage.getItem("nav-rail-pinned")).toBe("true");

    act(() => { result.current.togglePin(); });
    expect(result.current.isPinned).toBe(false);
    expect(localStorage.getItem("nav-rail-pinned")).toBe("false");
  });

  // ── Active group ───────────────────────────────────────────────

  it("starts with null activeGroup", () => {
    const { result } = renderHook(() => useNavSidebar("/"));
    expect(result.current.activeGroup).toBeNull();
  });

  it("setActiveGroup updates immediately", () => {
    const { result } = renderHook(() => useNavSidebar("/"));

    act(() => { result.current.setActiveGroup("org"); });
    expect(result.current.activeGroup).toBe("org");

    act(() => { result.current.setActiveGroup(null); });
    expect(result.current.activeGroup).toBeNull();
  });

  // ── Hover debounce ─────────────────────────────────────────────

  it("handleGroupHover opens panel after 150ms", () => {
    const { result } = renderHook(() => useNavSidebar("/"));

    act(() => { result.current.handleGroupHover("time"); });
    // Should not be active immediately
    expect(result.current.activeGroup).toBeNull();

    // Advance past debounce
    act(() => { vi.advanceTimersByTime(150); });
    expect(result.current.activeGroup).toBe("time");
  });

  it("handleGroupHover cancels previous pending open", () => {
    const { result } = renderHook(() => useNavSidebar("/"));

    act(() => { result.current.handleGroupHover("time"); });
    act(() => { vi.advanceTimersByTime(100); });
    // Switch to a different group before 150ms
    act(() => { result.current.handleGroupHover("org"); });

    // First group's timer should be cancelled
    act(() => { vi.advanceTimersByTime(100); }); // 100+100=200ms from first call, 100ms from second
    expect(result.current.activeGroup).toBeNull();

    // Full 150ms from second call
    act(() => { vi.advanceTimersByTime(50); });
    expect(result.current.activeGroup).toBe("org");
  });

  it("handleGroupHover(null) closes after 300ms", () => {
    const { result } = renderHook(() => useNavSidebar("/"));

    // First open
    act(() => { result.current.handleGroupHover("org"); });
    act(() => { vi.advanceTimersByTime(150); });
    expect(result.current.activeGroup).toBe("org");

    // Then close
    act(() => { result.current.handleGroupHover(null); });
    // Still active during close delay
    act(() => { vi.advanceTimersByTime(200); });
    expect(result.current.activeGroup).toBe("org");

    // Fully closed after 300ms
    act(() => { vi.advanceTimersByTime(100); });
    expect(result.current.activeGroup).toBeNull();
  });

  it("handleGroupHover does nothing when pinned", () => {
    const { result } = renderHook(() => useNavSidebar("/"));

    act(() => { result.current.togglePin(); }); // Pin
    act(() => { result.current.handleGroupHover("time"); });
    act(() => { vi.advanceTimersByTime(200); });

    // Should NOT change when pinned
    expect(result.current.activeGroup).toBeNull();
  });

  it("setActiveGroup bypasses debounce", () => {
    const { result } = renderHook(() => useNavSidebar("/"));

    act(() => { result.current.setActiveGroup("ess"); });
    expect(result.current.activeGroup).toBe("ess");
  });

  // ── Click-open blocks hover-close ────────────────────────────

  it("clicked-open panel stays open on hover-leave", () => {
    const { result } = renderHook(() => useNavSidebar("/"));

    // Click opens
    act(() => { result.current.setActiveGroup("org"); });
    expect(result.current.activeGroup).toBe("org");

    // Mouse leaves rail — should NOT close because it was opened by click
    act(() => { result.current.handleGroupHover(null); });
    act(() => { vi.advanceTimersByTime(500); });
    expect(result.current.activeGroup).toBe("org");
  });

  it("clicked-open lock resets when mouse re-enters a group", () => {
    const { result } = renderHook(() => useNavSidebar("/"));

    // Click opens
    act(() => { result.current.setActiveGroup("org"); });
    expect(result.current.activeGroup).toBe("org");

    // Mouse enters a different group — lock clears, normal hover takes over
    act(() => { result.current.handleGroupHover("time"); });
    act(() => { vi.advanceTimersByTime(150); });
    expect(result.current.activeGroup).toBe("time");
  });

  it("handlePanelEnter cancels pending close but preserves click lock", () => {
    const { result } = renderHook(() => useNavSidebar("/"));

    act(() => { result.current.setActiveGroup("org"); });
    // Panel mouse enter
    act(() => { result.current.handlePanelEnter(); });

    // Panel mouse leave — should NOT close because clicked lock is still active
    act(() => { result.current.handlePanelLeave(); });
    act(() => { vi.advanceTimersByTime(500); });
    expect(result.current.activeGroup).toBe("org");
  });

  it("handlePanelLeave never closes the panel (no-op)", () => {
    const { result } = renderHook(() => useNavSidebar("/"));

    // Open by hover
    act(() => { result.current.handleGroupHover("org"); });
    act(() => { vi.advanceTimersByTime(150); });
    expect(result.current.activeGroup).toBe("org");

    // Leave panel — should stay open with new behavior
    act(() => { result.current.handlePanelLeave(); });
    act(() => { vi.advanceTimersByTime(500); });
    expect(result.current.activeGroup).toBe("org");
  });

  it("panel closes only when mouse leaves the rail", () => {
    const { result } = renderHook(() => useNavSidebar("/"));

    // Hover opens
    act(() => { result.current.handleGroupHover("org"); });
    act(() => { vi.advanceTimersByTime(150); });
    expect(result.current.activeGroup).toBe("org");

    // Leave panel — stays open
    act(() => { result.current.handlePanelLeave(); });
    expect(result.current.activeGroup).toBe("org");

    // Leave rail entirely — closes after 300ms
    act(() => { result.current.handleGroupHover(null); });
    act(() => { vi.advanceTimersByTime(300); });
    expect(result.current.activeGroup).toBeNull();
  });

  it("full flow: hover open → panel enter → panel leave → rail leave closes", () => {
    const { result } = renderHook(() => useNavSidebar("/"));

    // 1. Hover opens
    act(() => { result.current.handleGroupHover("org"); });
    act(() => { vi.advanceTimersByTime(150); });
    expect(result.current.activeGroup).toBe("org");

    // 2. Mouse enters panel
    act(() => { result.current.handlePanelEnter(); });
    expect(result.current.activeGroup).toBe("org");

    // 3. Mouse leaves panel — panel stays open
    act(() => { result.current.handlePanelLeave(); });
    act(() => { vi.advanceTimersByTime(200); });
    expect(result.current.activeGroup).toBe("org");

    // 4. Mouse leaves rail — panel closes after 300ms
    act(() => { result.current.handleGroupHover(null); });
    act(() => { vi.advanceTimersByTime(300); });
    expect(result.current.activeGroup).toBeNull();
  });

  it("stale timer from before navigation is ignored", () => {
    const { result, rerender } = renderHook(
      ({ path }) => useNavSidebar(path),
      { initialProps: { path: "/employees" } },
    );

    // Start hover timer while on /employees
    act(() => { result.current.handleGroupHover("org"); });

    // Navigate to / (simulating clicking Dashboard icon)
    rerender({ path: "/" });

    // Advance past the hover timer — should NOT open panel
    // because the pathname changed since the timer was set
    act(() => { vi.advanceTimersByTime(200); });
    expect(result.current.activeGroup).toBeNull();
  });

  it("new hover after navigation still works", () => {
    const { result, rerender } = renderHook(
      ({ path }) => useNavSidebar(path),
      { initialProps: { path: "/employees" } },
    );

    // Navigate to / (simulating clicking Dashboard)
    rerender({ path: "/" });

    // Start a NEW hover after navigation
    act(() => { result.current.handleGroupHover("org"); });
    act(() => { vi.advanceTimersByTime(150); });
    // Should open because timer was set AFTER navigation
    expect(result.current.activeGroup).toBe("org");
  });

  // ── Path-based active group ────────────────────────────────────

  it("detects dashboard group from / path", () => {
    const { result } = renderHook(() => useNavSidebar("/"));
    expect(result.current.pathActiveGroup).toBe("dashboard");
  });

  it("detects org group from /employees path", () => {
    const { result } = renderHook(() => useNavSidebar("/employees"));
    expect(result.current.pathActiveGroup).toBe("org");
  });

  it("detects time group from /attendance-logs path", () => {
    const { result } = renderHook(() => useNavSidebar("/attendance-logs"));
    expect(result.current.pathActiveGroup).toBe("time");
  });

  // ── Mobile state ───────────────────────────────────────────────

  it("mobile drawer defaults to closed", () => {
    const { result } = renderHook(() => useNavSidebar("/"));
    expect(result.current.isMobileOpen).toBe(false);
  });

  it("setMobileOpen toggles the mobile drawer", () => {
    const { result } = renderHook(() => useNavSidebar("/"));

    act(() => { result.current.setMobileOpen(true); });
    expect(result.current.isMobileOpen).toBe(true);

    act(() => { result.current.setMobileOpen(false); });
    expect(result.current.isMobileOpen).toBe(false);
  });
});
