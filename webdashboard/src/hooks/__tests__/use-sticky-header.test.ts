import { renderHook, act } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { useStickyHeader } from "../use-sticky-header";

// Mock ResizeObserver
class MockResizeObserver {
    callback: ResizeObserverCallback;
    constructor(callback: ResizeObserverCallback) {
        this.callback = callback;
    }
    observe() { }
    unobserve() { }
    disconnect() { }
}
window.ResizeObserver = MockResizeObserver as any;

describe("useStickyHeader", () => {
    beforeEach(() => {
        vi.clearAllMocks();
        // Reset scroll position
        Object.defineProperty(window, "scrollY", { value: 0, writable: true, configurable: true });
    });

    it("returns correct initial state", () => {
        const { result } = renderHook(() => useStickyHeader());

        expect(result.current.isSticky).toBe(false);
        expect(result.current.isVisible).toBe(true);
        expect(result.current.headerHeight).toBe(0);
        expect(result.current.headerRef).toBeDefined();
        expect(result.current.headerRef.current).toBeNull();
    });

    it("returns visible and not sticky when disabled", () => {
        const { result } = renderHook(() => useStickyHeader({ enabled: false }));

        expect(result.current.isSticky).toBe(false);
        expect(result.current.isVisible).toBe(true);
    });

    it("headerRef is a valid ref object", () => {
        const { result } = renderHook(() => useStickyHeader());
        expect(result.current.headerRef).toHaveProperty("current");
    });

    it("accepts custom threshold parameter", () => {
        const { result } = renderHook(() => useStickyHeader({ threshold: 50 }));
        // Should not throw and should return valid state
        expect(result.current.isVisible).toBe(true);
        expect(result.current.isSticky).toBe(false);
    });

    it("adds and removes scroll listener", () => {
        const addSpy = vi.spyOn(window, "addEventListener");
        const removeSpy = vi.spyOn(window, "removeEventListener");

        const { unmount } = renderHook(() => useStickyHeader());

        expect(addSpy).toHaveBeenCalledWith("scroll", expect.any(Function), { passive: true });

        unmount();

        expect(removeSpy).toHaveBeenCalledWith("scroll", expect.any(Function));

        addSpy.mockRestore();
        removeSpy.mockRestore();
    });

    it("does not add scroll listener when disabled", () => {
        const addSpy = vi.spyOn(window, "addEventListener");

        renderHook(() => useStickyHeader({ enabled: false }));

        const scrollCalls = addSpy.mock.calls.filter(
            (call) => call[0] === "scroll"
        );
        expect(scrollCalls.length).toBe(0);

        addSpy.mockRestore();
    });
});
