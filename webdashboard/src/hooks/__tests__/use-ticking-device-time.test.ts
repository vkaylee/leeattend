import { renderHook, act } from "@testing-library/react";

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

import { useTickingDeviceTime } from "../use-ticking-device-time";

describe("useTickingDeviceTime", () => {
    beforeEach(() => {
        vi.useFakeTimers();
    });

    afterEach(() => {
        vi.useRealTimers();
    });

    it("returns null when deviceTime is undefined", () => {
        const { result } = renderHook(() =>
            useTickingDeviceTime(undefined, "2026-03-08 10:00:00"),
        );
        expect(result.current).toBeNull();
    });

    it("returns null when lastSync is undefined", () => {
        const { result } = renderHook(() =>
            useTickingDeviceTime("2026-03-08 10:00:00", undefined),
        );
        expect(result.current).toBeNull();
    });

    it("returns null when deviceTime is invalid", () => {
        const { result } = renderHook(() =>
            useTickingDeviceTime("not-a-date", "2026-03-08 10:00:00"),
        );
        expect(result.current).toBeNull();
    });

    it("returns null when lastSync is invalid", () => {
        const { result } = renderHook(() =>
            useTickingDeviceTime("2026-03-08 10:00:00", "not-a-date"),
        );
        expect(result.current).toBeNull();
    });

    it("ticks forward after 1 second", () => {
        const now = new Date("2026-03-08T10:00:00").getTime();
        vi.setSystemTime(now);

        // device_time and last_sync are the same → offset = 0
        const { result } = renderHook(() =>
            useTickingDeviceTime("2026-03-08 10:00:00", "2026-03-08 10:00:00"),
        );

        expect(result.current).toBe("2026-03-08 10:00:00");

        act(() => {
            vi.advanceTimersByTime(1000);
        });

        expect(result.current).toBe("2026-03-08 10:00:01");
    });

    it("ticks forward after 5 seconds", () => {
        const now = new Date("2026-03-08T10:00:00").getTime();
        vi.setSystemTime(now);

        const { result } = renderHook(() =>
            useTickingDeviceTime("2026-03-08 10:00:00", "2026-03-08 10:00:00"),
        );

        act(() => {
            vi.advanceTimersByTime(5000);
        });

        expect(result.current).toBe("2026-03-08 10:00:05");
    });

    it("preserves offset when device clock is behind last_sync", () => {
        // device_time = 09:59:58, last_sync = 10:00:00 → offset = -2s
        // device is 2 seconds behind the sync timestamp
        const now = new Date("2026-03-08T10:05:00").getTime();
        vi.setSystemTime(now);

        const { result } = renderHook(() =>
            useTickingDeviceTime("2026-03-08 09:59:58", "2026-03-08 10:00:00"),
        );

        // display = Date.now() + offset = 10:05:00 + (-2s) = 10:04:58
        expect(result.current).toBe("2026-03-08 10:04:58");

        act(() => {
            vi.advanceTimersByTime(3000);
        });

        // 10:05:03 + (-2s) = 10:05:01
        expect(result.current).toBe("2026-03-08 10:05:01");
    });

    it("works correctly with stale data after page reload", () => {
        // Data was synced 2 hours ago at 08:00:01, device reported 08:00:00
        // Now it's 10:00:00 → should show ~09:59:59 (current - 1s offset)
        const now = new Date("2026-03-08T10:00:00").getTime();
        vi.setSystemTime(now);

        const { result } = renderHook(() =>
            useTickingDeviceTime("2026-03-08 08:00:00", "2026-03-08 08:00:01"),
        );

        // offset = 08:00:00 - 08:00:01 = -1s
        // display = 10:00:00 + (-1s) = 09:59:59
        expect(result.current).toBe("2026-03-08 09:59:59");

        act(() => {
            vi.advanceTimersByTime(2000);
        });

        expect(result.current).toBe("2026-03-08 10:00:01");
    });

    it("cleans up interval on unmount", () => {
        const clearIntervalSpy = vi.spyOn(globalThis, "clearInterval");
        const now = new Date("2026-03-08T10:00:00").getTime();
        vi.setSystemTime(now);

        const { unmount } = renderHook(() =>
            useTickingDeviceTime("2026-03-08 10:00:00", "2026-03-08 10:00:00"),
        );

        unmount();

        expect(clearIntervalSpy).toHaveBeenCalled();
        clearIntervalSpy.mockRestore();
    });
});
