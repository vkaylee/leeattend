import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor, act } from "@testing-library/react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import * as api from "@/lib/api/helpers";

import { useDashboardLayout } from "../use-dashboard-layout";

vi.mock("@/lib/api/helpers", async () => {
    const actual = await vi.importActual("@/lib/api/helpers");
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
    const Wrapper = ({ children }: { children: React.ReactNode }) => (
        <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );
    Wrapper.displayName = "QueryClientWrapper";
    return Wrapper;
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
 
        vi.mocked(api.apiFetch).mockResolvedValueOnce({
            ok: true,
            json: () => Promise.resolve(mockPreferences),
        } as Response);

        const { result } = renderHook(() => useDashboardLayout(), {
            wrapper: createWrapper(),
        });

        await waitFor(() => { expect(result.current.isLoading).toBe(false); });

        expect(api.apiFetch).toHaveBeenCalledWith("/v1/dashboard/preferences");
        expect(result.current.activeView).toBe("attendance");
    });

    it("sets default overview preset configuration correctly", async () => {
 
        vi.mocked(api.apiFetch).mockResolvedValueOnce({
            ok: true,
            json: () => Promise.resolve({
                id: "pref-123", active_view: "overview"
            }),
        } as Response);

        const { result } = renderHook(() => useDashboardLayout(), {
            wrapper: createWrapper(),
        });

        await waitFor(() => { expect(result.current.isLoading).toBe(false); });

        expect(result.current.activeView).toBe("overview");
        expect(result.current.visibleWidgets.kpi_cards).toBe(true);
        expect(result.current.visibleWidgets.attendance_trend).toBe(true);
        expect(result.current.visibleWidgets.overtime_trend).toBe(false);
    });

    it("triggers API PUT when setting active view", async () => {
 
        vi.mocked(api.apiFetch).mockResolvedValue({
            ok: true,
            json: () => Promise.resolve({ active_view: "overview" }),
        } as Response);

        const { result } = renderHook(() => useDashboardLayout(), {
            wrapper: createWrapper(),
        });

        await waitFor(() => { expect(result.current.isLoading).toBe(false); });

        result.current.setActiveView("trends");

        await waitFor(() => { expect(api.apiFetch).toHaveBeenCalledTimes(2); });

 
        const putCallArguments = vi.mocked(api.apiFetch).mock.calls[1];
        expect(putCallArguments[0]).toBe("/v1/dashboard/preferences");
        expect(putCallArguments[1]!.method).toBe("PUT");

        const body = JSON.parse(putCallArguments[1]!.body as string) as { active_view: string, visible_widgets: { overtime_trend: boolean, kpi_cards: boolean } };
        expect(body.active_view).toBe("trends");
        // Ensure trends preset config is passed
        expect(body.visible_widgets.overtime_trend).toBe(true);
        expect(body.visible_widgets.kpi_cards).toBe(false);
    });

    it("triggers API PUT when toggling section", async () => {
 
        vi.mocked(api.apiFetch).mockResolvedValue({
            ok: true,
            json: () => Promise.resolve(mockPreferences),
        } as Response);

        const { result } = renderHook(() => useDashboardLayout(), {
            wrapper: createWrapper(),
        });

        await waitFor(() => { expect(result.current.isLoading).toBe(false); });

        // In mockPreferences, summary is true
        expect(result.current.collapsedSections.summary).toBe(true);

        result.current.toggleSection("summary");

        await waitFor(() => { expect(api.apiFetch).toHaveBeenCalledTimes(2); });

 
        const putCallArguments = vi.mocked(api.apiFetch).mock.calls[1];
        const body = JSON.parse(putCallArguments[1]!.body as string) as { collapsed_sections: { summary: boolean } };
        expect(body.collapsed_sections.summary).toBe(false);
    });

    it("updates layout state instantly before the API mutation resolves (non-blocking)", async () => {
        let resolveMutationPromise: (value: Response) => void = (_value: Response) => { /* noop */ };
        const slowMutationPromise = new Promise<Response>((resolve) => {
            resolveMutationPromise = resolve;
        });

        vi.mocked(api.apiFetch).mockResolvedValueOnce({
            ok: true,
            json: () => Promise.resolve(mockPreferences),
        } as Response);

        vi.mocked(api.apiFetch).mockImplementation((url: any, init?: any): Promise<Response> => {
            if (url === "/dashboard/preferences" && init?.method === "PUT") {
                return slowMutationPromise;
            }
            return Promise.resolve({
                ok: true,
                json: () => Promise.resolve({}),
            } as Response);
        });

        const { result } = renderHook(() => useDashboardLayout(), {
            wrapper: createWrapper(),
        });

        await waitFor(() => { expect(result.current.isLoading).toBe(false); });
        expect(result.current.activeView).toBe("attendance");

        act(() => {
            result.current.setActiveView("trends");
        });

        expect(result.current.activeView).toBe("trends");
        expect(result.current.isSaving).toBe(true);

        resolveMutationPromise({
            ok: true,
            json: () => Promise.resolve({
                ...mockPreferences,
                active_view: "trends",
            }),
        } as Response);

        await waitFor(() => { expect(result.current.isSaving).toBe(false); });
        expect(result.current.activeView).toBe("trends");
    });

    it("handles empty custom preferences by returning default widgets and order", async () => {
        vi.mocked(api.apiFetch).mockResolvedValueOnce({
            ok: true,
            json: () => Promise.resolve({
                id: "pref-123",
                user_id: "user-123",
                workspace_id: "ws-test-123",
                active_view: "custom",
                visible_widgets: {},
                widget_order: [],
                collapsed_sections: {},
            }),
        } as Response);

        const { result } = renderHook(() => useDashboardLayout(), {
            wrapper: createWrapper(),
        });

        await waitFor(() => { expect(result.current.isLoading).toBe(false); });
        
        expect(result.current.activeView).toBe("custom");
        expect(result.current.visibleWidgets.kpi_cards).toBe(true);
        expect(result.current.visibleWidgets.donut_chart).toBe(true);
        expect(result.current.widgetOrder).toContain("kpi_cards");
        expect(result.current.widgetOrder.length).toBe(result.current.allWidgets.length);
    });
});
