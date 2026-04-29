import React from "react";

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";

import { useDeviceInfo, devicesKeys } from "../use-devices";

// Mock the API module
const mockFetchDeviceInfo = vi.fn();
vi.mock("@/lib/api", () => ({
    fetchDevices: vi.fn().mockResolvedValue([]),
    createDevice: vi.fn(),
    updateDevice: vi.fn(),
    deleteDevice: vi.fn(),
    fetchDeviceInfo: (...args: unknown[]) => mockFetchDeviceInfo(...args),
}));

function createWrapper() {
    const queryClient = new QueryClient({
        defaultOptions: {
            queries: { retry: false, gcTime: 0, staleTime: 0 },
        },
    });
    return function Wrapper({ children }: { children: React.ReactNode }) {
        return (
            <QueryClientProvider client={queryClient}>
                {children}
            </QueryClientProvider>
        );
    };
}

describe("useDeviceInfo", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it("fetches device info when enabled", async () => {
        const mockInfo = {
            users_count: 42,
            logs_count: 100,
            device_time: "2026-01-01 10:00:00",
            firmware: "FW 2.0",
            mac: "AA:BB:CC:DD:EE:FF",
            serial: "SN123456",
        };
        mockFetchDeviceInfo.mockResolvedValue(mockInfo);

        const { result } = renderHook(
            () => useDeviceInfo("dev-1", true),
            { wrapper: createWrapper() }
        );

        await waitFor(() => expect(result.current.data).toBeDefined());

        expect(mockFetchDeviceInfo).toHaveBeenCalledWith("dev-1");
        expect(result.current.data).toEqual(mockInfo);
    });

    it("does not fetch when enabled=false", async () => {
        const { result } = renderHook(
            () => useDeviceInfo("dev-1", false),
            { wrapper: createWrapper() }
        );

        // Should not fetch — query is disabled
        expect(mockFetchDeviceInfo).not.toHaveBeenCalled();
        expect(result.current.data).toBeUndefined();
    });

    it("does not fetch when id is empty string", async () => {
        const { result } = renderHook(
            () => useDeviceInfo("", true),
            { wrapper: createWrapper() }
        );

        expect(mockFetchDeviceInfo).not.toHaveBeenCalled();
        expect(result.current.data).toBeUndefined();
    });

    it("returns error when fetch fails", async () => {
        mockFetchDeviceInfo.mockRejectedValue(new Error("Agent offline"));

        const { result } = renderHook(
            () => useDeviceInfo("dev-1", true),
            { wrapper: createWrapper() }
        );

        await waitFor(() => expect(result.current.error).toBeTruthy());

        expect(result.current.error).toBeInstanceOf(Error);
        expect((result.current.error as Error).message).toBe("Agent offline");
    });

    it("generates correct query keys", () => {
        expect(devicesKeys.all).toEqual(["devices"]);
        expect(devicesKeys.info("dev-1")).toEqual(["devices", "dev-1", "info"]);
        expect(devicesKeys.info("dev-2")).toEqual(["devices", "dev-2", "info"]);
    });
});
