import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor, act } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";

import * as api from "@/lib/api/helpers";

import { DashboardPreferencesError, useDashboardLayout } from "../use-dashboard-layout";

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
    collapsed_sections: { summary: true, charts: false, actions: true },
};

const responseFor = (overrides: Record<string, unknown> = {}) => ({
    ...mockPreferences,
    ...overrides,
});

const asResponse = (payload: unknown, ok = true) => ({
    ok,
    json: () => Promise.resolve(payload),
} as Response);

describe("useDashboardLayout hook", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it("fetches preferences and normalizes persisted legacy and current widget IDs", async () => {
        vi.mocked(api.apiFetch).mockResolvedValueOnce(asResponse(responseFor({
            visible_widgets: {
                kpi_cards: false,
                department_chart: true,
                legacy_widget: true,
            },
            widget_order: ["legacy_widget", "department_chart", "department_chart"],
        })));

        const { result } = renderHook(() => useDashboardLayout(), { wrapper: createWrapper() });

        await waitFor(() => expect(result.current.isLoading).toBe(false));

        expect(result.current.activeView).toBe("attendance");
        expect(result.current.visibleWidgets.kpi_cards).toBe(false);
        expect(result.current.visibleWidgets.department_chart).toBe(true);
        expect(result.current.visibleWidgets.overtime_trend).toBe(false);
        expect(result.current.widgetOrder[0]).toBe("department_chart");
        expect(result.current.widgetOrder).not.toContain("legacy_widget");
        expect(new Set(result.current.widgetOrder).size).toBe(result.current.allWidgets.length);
    });

    it("rejects a response that does not match the current preferences contract", async () => {
        vi.mocked(api.apiFetch).mockResolvedValueOnce(asResponse({ active_view: "overview" }));

        const { result } = renderHook(() => useDashboardLayout(), { wrapper: createWrapper() });

        await waitFor(() => expect(result.current.isLoading).toBe(false));

        expect(result.current.error).toBeInstanceOf(DashboardPreferencesError);
        expect(result.current.error?.message).toContain("invalid response");
    });

    it("uses backend-matching defaults for an overview response with omitted widget values", async () => {
        vi.mocked(api.apiFetch).mockResolvedValueOnce(asResponse(responseFor({
            active_view: "overview",
            visible_widgets: {},
            widget_order: [],
            collapsed_sections: {},
        })));

        const { result } = renderHook(() => useDashboardLayout(), { wrapper: createWrapper() });

        await waitFor(() => expect(result.current.isLoading).toBe(false));

        expect(result.current.activeView).toBe("overview");
        expect(result.current.visibleWidgets.kpi_cards).toBe(true);
        expect(result.current.visibleWidgets.department_chart).toBe(true);
        expect(result.current.visibleWidgets.recruitment_sources).toBe(true);
        expect(result.current.visibleWidgets.overtime_trend).toBe(false);
        expect(result.current.collapsedSections.actions).toBe(true);
        expect(result.current.widgetOrder).toHaveLength(result.current.allWidgets.length);
    });

    it("sends the selected view and preset values in a PUT request", async () => {
        vi.mocked(api.apiFetch).mockImplementation((_url, init) => Promise.resolve(
            asResponse(responseFor({ active_view: init?.method === "PUT" ? "trends" : "attendance" }))
        ));

        const { result } = renderHook(() => useDashboardLayout(), { wrapper: createWrapper() });
        await waitFor(() => expect(result.current.isLoading).toBe(false));

        act(() => result.current.setActiveView("trends"));
        await waitFor(() => expect(api.apiFetch).toHaveBeenCalledTimes(2));

        const [, init] = vi.mocked(api.apiFetch).mock.calls[1];
        expect(init?.method).toBe("PUT");
        const body = JSON.parse(init?.body as string) as {
            active_view: string;
            visible_widgets: Record<string, boolean>;
        };
        expect(body.active_view).toBe("trends");
        expect(body.visible_widgets.overtime_trend).toBe(true);
        expect(body.visible_widgets.kpi_cards).toBe(false);
    });

    it("rolls back the latest optimistic mutation when saving fails", async () => {
        vi.mocked(api.apiFetch).mockImplementation((url, init) => {
            if (init?.method === "PUT") return Promise.resolve(asResponse({}, false));
            return Promise.resolve(asResponse(mockPreferences));
        });

        const { result } = renderHook(() => useDashboardLayout(), { wrapper: createWrapper() });
        await waitFor(() => expect(result.current.isLoading).toBe(false));
        expect(result.current.activeView).toBe("attendance");

        act(() => result.current.setActiveView("trends"));
        expect(result.current.activeView).toBe("trends");

        await waitFor(() => expect(result.current.activeView).toBe("attendance"));
        await waitFor(() => expect(result.current.saveError).toBeInstanceOf(DashboardPreferencesError));
        expect(result.current.saveError?.message).toContain("could not be saved");
        expect(result.current.error).toBeNull();
    });

    it("serializes rapid mutations so the server receives the newest state last", async () => {
        let resolveFirst: ((value: Response) => void) | undefined;
        let resolveSecond: ((value: Response) => void) | undefined;
        const firstPut = new Promise<Response>((resolve) => { resolveFirst = resolve; });
        const secondPut = new Promise<Response>((resolve) => { resolveSecond = resolve; });
        let putCount = 0;

        vi.mocked(api.apiFetch).mockImplementation((_url, init) => {
            if (init?.method === "PUT") {
                putCount += 1;
                return putCount === 1 ? firstPut : secondPut;
            }
            return Promise.resolve(asResponse(mockPreferences));
        });

        const { result } = renderHook(() => useDashboardLayout(), { wrapper: createWrapper() });
        await waitFor(() => expect(result.current.isLoading).toBe(false));

        act(() => {
            result.current.setActiveView("trends");
            result.current.setActiveView("attendance");
        });
        expect(result.current.activeView).toBe("attendance");
        await waitFor(() => expect(putCount).toBe(1));

        resolveFirst?.(asResponse(responseFor({ active_view: "trends" })));
        await waitFor(() => expect(putCount).toBe(2));
        const [, secondInit] = vi.mocked(api.apiFetch).mock.calls[2];
        const secondBody = JSON.parse(secondInit?.body as string) as { active_view: string };
        expect(secondBody.active_view).toBe("attendance");

        resolveSecond?.(asResponse(responseFor({ active_view: "attendance" })));
        await waitFor(() => expect(result.current.isSaving).toBe(false));
        expect(result.current.activeView).toBe("attendance");
    });

    it("builds rapid widget toggles from the latest optimistic state", async () => {
        let resolvePut: ((value: Response) => void) | undefined;
        const pendingPut = new Promise<Response>((resolve) => { resolvePut = resolve; });
        vi.mocked(api.apiFetch).mockImplementation((_url, init) => {
            if (init?.method === "PUT") return pendingPut;
            return Promise.resolve(asResponse(mockPreferences));
        });

        const { result } = renderHook(() => useDashboardLayout(), { wrapper: createWrapper() });
        await waitFor(() => expect(result.current.isLoading).toBe(false));

        act(() => {
            result.current.toggleWidget("kpi_cards");
            result.current.toggleWidget("department_chart");
        });

        expect(result.current.visibleWidgets.kpi_cards).toBe(false);
        expect(result.current.visibleWidgets.department_chart).toBe(false);
        resolvePut?.(asResponse(responseFor({ active_view: "custom" })));
        await waitFor(() => expect(result.current.isSaving).toBe(false));
    });
});
