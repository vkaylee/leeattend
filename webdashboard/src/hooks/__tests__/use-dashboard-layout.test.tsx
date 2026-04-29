import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";

import * as api from "@/lib/api";

import { useDashboardLayout } from "../use-dashboard-layout";

vi.mock("@/lib/api", async () => {
    const actual = await vi.importActual("@/lib/api");
    return {
        ...actual,
        apiFetch: vi.fn(),
    };
});

vi.mock("../use-workspace", () => ({
    useWorkspace: () => ({
        workspace: { id: "ws-test-123", name: "Test Workspace" },
    }),
}));

const createWrapper = () => {
    const queryClient = new QueryClient({
        defaultOptions: {
            queries: { retry: false },
        },
    });
    return ({ children }: { children: React.ReactNode }) => (
        <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );
};

const mockPreferences = {
    id: "pref-123",
    user_id: "user-123",
    workspace_id: "ws-test-123",
    active_view: "attendance",
    visible_widgets: { kpi_cards: true, peak_times: false },
    widget_order: ["attendance_trend", "donut_chart"],
    collapsed_sections: { summary: true, charts: false, actions: true }
};

describe("useDashboardLayout hook", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it("fetches dashboard preferences and applies active view", async () => {
        (api.apiFetch as any).mockResolvedValueOnce({
            ok: true,
            json: () => Promise.resolve(mockPreferences),
        });

        const { result } = renderHook(() => useDashboardLayout(), {
            wrapper: createWrapper(),
        });

        await waitFor(() => expect(result.current.isLoading).toBe(false));

        expect(api.apiFetch).toHaveBeenCalledWith("/dashboard/preferences");
        expect(result.current.activeView).toBe("attendance");
    });

    it("sets default overview preset configuration correctly", async () => {
        (api.apiFetch as any).mockResolvedValueOnce({
            ok: true,
            json: () => Promise.resolve({
                id: "pref-123", active_view: "overview"
            }),
        });

        const { result } = renderHook(() => useDashboardLayout(), {
            wrapper: createWrapper(),
        });

        await waitFor(() => expect(result.current.isLoading).toBe(false));

        expect(result.current.activeView).toBe("overview");
        expect(result.current.visibleWidgets.kpi_cards).toBe(true);
        expect(result.current.visibleWidgets.attendance_trend).toBe(true);
        expect(result.current.visibleWidgets.overtime_trend).toBe(false);
    });

    it("triggers API PUT when setting active view", async () => {
        (api.apiFetch as any).mockResolvedValue({
            ok: true,
            json: () => Promise.resolve({ active_view: "overview" }),
        });

        const { result } = renderHook(() => useDashboardLayout(), {
            wrapper: createWrapper(),
        });

        await waitFor(() => expect(result.current.isLoading).toBe(false));

        result.current.setActiveView("trends");

        await waitFor(() => expect(api.apiFetch).toHaveBeenCalledTimes(2));

        const putCallArgs = (api.apiFetch as any).mock.calls[1];
        expect(putCallArgs[0]).toBe("/dashboard/preferences");
        expect(putCallArgs[1].method).toBe("PUT");

        const body = JSON.parse(putCallArgs[1].body);
        expect(body.active_view).toBe("trends");
        // Ensure trends preset config is passed
        expect(body.visible_widgets.overtime_trend).toBe(true);
        expect(body.visible_widgets.kpi_cards).toBe(false);
    });

    it("triggers API PUT when toggling section", async () => {
        (api.apiFetch as any).mockResolvedValue({
            ok: true,
            json: () => Promise.resolve(mockPreferences),
        });

        const { result } = renderHook(() => useDashboardLayout(), {
            wrapper: createWrapper(),
        });

        await waitFor(() => expect(result.current.isLoading).toBe(false));

        // In mockPreferences, summary is true
        expect(result.current.collapsedSections.summary).toBe(true);

        result.current.toggleSection("summary");

        await waitFor(() => expect(api.apiFetch).toHaveBeenCalledTimes(2));

        const putCallArgs = (api.apiFetch as any).mock.calls[1];
        const body = JSON.parse(putCallArgs[1].body);
        expect(body.collapsed_sections.summary).toBe(false);
    });
});
