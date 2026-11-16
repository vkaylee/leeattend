import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach, type Mock } from "vitest";

import { ToastProvider } from "@/components/ui/toast";
import * as api from "@/lib/api";

import EmployeeDailyDrawer from "../employee-daily-drawer";


// Mock TanStack Router
vi.mock("@tanstack/react-router", () => ({
    useNavigate: () => vi.fn(),
    useLocation: () => ({ pathname: "/", search: "", hash: "" }),
 
    Link: ({ to, children, ...props }: { to: string; children: React.ReactNode } & Record<string, unknown>) => <a href={to} {...props}>{children}</a>,
}));

// Mock i18next
vi.mock("react-i18next", () => ({
    useTranslation: () => ({ t: (key: string) => key }),
}));

// Mock the API library
vi.mock("@/lib/api", async (importOriginal) => {
    const actual = await importOriginal<typeof api>();
    return {
        ...actual,
        fetchAttendanceReport: vi.fn(),
        reprocessAttendance: vi.fn(),
        fetchAttendanceAdjustments: vi.fn(),
        adminConfirmAttendance: vi.fn(),
        approveOt: vi.fn(),
        fetchRawAttendanceLogs: vi.fn(),
        fetchShifts: vi.fn(),
        upsertRoster: vi.fn(),
    };
});


describe("EmployeeDailyDrawer", () => {
    let queryClient: QueryClient;

    const mockDailyRow = {
        employee_id: "emp-1",
        employee_name: "John Doe",
        date: "2026-01-15",
        actual_punch_in: "2026-01-15T01:00:00Z", // 08:00 Local
        actual_punch_out: "2026-01-15T10:00:00Z", // 17:00 Local
        scheduled_start: "08:00:00",
        scheduled_end: "17:00:00",
        break_duration_minutes: 60,
        break_start: null,
        break_end: null,
        is_break_auto_deduct: false,
        std_work_units: 480,
        ot_work_units: 1, // 1 hour OT
        computed_std_hours: 8.0,
        computed_ot_hours: 1.0,
        unit_type: "DAY",
        unit_coefficient_hours: 8.0,
        status: "present",
        ot_claim_status: "pending_manager",
        generation_metadata: null,
    };

    const defaultProps = {
        isOpen: true,
        onClose: vi.fn(),employeeId: "emp-1",
        employeeName: "John Doe",
        dateRange: { start: "2026-01-01", end: "2026-01-31" },
    };

    beforeEach(() => {
        vi.clearAllMocks();
        queryClient = new QueryClient({
            defaultOptions: {
                queries: {
                    retry: false,
                    gcTime: 0,
                    staleTime: 0,
                },
            },
        });

 
        (api.fetchAttendanceReport as Mock).mockResolvedValue({
            data: [mockDailyRow],
            total: 1,
        });
 
        (api.fetchAttendanceAdjustments as Mock).mockResolvedValue({ data: [], total: 0, page: 1, limit: 20 });
 
        (api.reprocessAttendance as Mock).mockResolvedValue(true);
 
        (api.adminConfirmAttendance as Mock).mockResolvedValue(true);
 
        (api.approveOt as Mock).mockResolvedValue(true);
 
        (api.fetchRawAttendanceLogs as Mock).mockResolvedValue({
            data: [],
            total: 0,
            page: 1,
            limit: 100
        });
 
        (api.fetchShifts as Mock).mockResolvedValue([
            {
                id: "shift-morning",
                workspace_id: "ws-1",
                name: "Ca sáng",
                start_time: "08:00:00",
                end_time: "17:00:00",
                break_duration_minutes: 60,
                created_at: "2026-01-01T00:00:00Z",
                updated_at: "2026-01-01T00:00:00Z",
            },
        ]);
 
        (api.upsertRoster as Mock).mockResolvedValue({});
    });

    const renderWithProviders = (ui: React.ReactElement, _initialEntries = ["/"]) => {
        return render(
            <QueryClientProvider client={queryClient}>
                <ToastProvider>
                    {ui}
                </ToastProvider>
            </QueryClientProvider>
        );
    };

    it("does not render when not open", () => {
        renderWithProviders(<EmployeeDailyDrawer {...defaultProps} isOpen={false} />);
        const drawerHeader = screen.queryByText("John Doe");
        expect(drawerHeader).toBeNull();
    });

    it("shows total regular and overtime hours summary", async () => {
        renderWithProviders(<EmployeeDailyDrawer {...defaultProps} />);

        // Wait for data to load (desktop + mobile both render)
        const shiftTexts = await screen.findAllByText(/08:00 — 17:00/i);
        expect(shiftTexts.length).toBeGreaterThan(0);

        // computed_std_hours=8.0 -> 8h
        const stdElements = await screen.findAllByText("8h");
        expect(stdElements.length).toBeGreaterThanOrEqual(1);

        // computed_ot_hours=1.0 -> 1h
        const otElements = await screen.findAllByText("1h");
        expect(otElements.length).toBeGreaterThanOrEqual(1);

        // Check i18n keys for labels
        expect(screen.getByText("reports.quick_stats.std_hours")).toBeInTheDocument();
        expect(screen.getByText("reports.quick_stats.ot_hours")).toBeInTheDocument();
    });

    it("fetches data and renders the drawer content when opened", async () => {
        renderWithProviders(<EmployeeDailyDrawer {...defaultProps} />);

        // Header should contain the name (desktop + mobile both render)
        const names = await screen.findAllByText("John Doe");
        expect(names.length).toBeGreaterThan(0);

        // Check if API was called
        await waitFor(() => {
            expect(api.fetchAttendanceReport).toHaveBeenCalledWith({start: "2026-01-01",
                end: "2026-01-31",
                type: "daily",
                page: 1,
                limit: 100,
                employeeIds: ["emp-1"]
            });
        });

        // Wait for the table row to render
        const shiftTexts = await screen.findAllByText(/08:00 — 17:00/i);
        expect(shiftTexts.length).toBeGreaterThan(0);

        // Verify that the sheet panel stays full-width on mobile and wide on desktop.
        const sheetPanel = screen.getByTestId("sheet-panel");
        expect(screen.getByRole("dialog", { name: "John Doe" })).toBe(sheetPanel);
        expect(sheetPanel).toHaveClass("!w-full", "!max-w-full", "sm:!w-[95vw]", "sm:!max-w-[95vw]", "xl:!w-[1200px]");
    });

    it("exposes tab semantics and keeps the active panel associated", async () => {
        const user = userEvent.setup();
        renderWithProviders(<EmployeeDailyDrawer {...defaultProps} />);

        const tablist = await screen.findByRole("tablist", { name: "reports.tabs.label" });
        const tabs = screen.getAllByRole("tab");
        expect(tablist).toHaveAttribute("aria-label", "reports.tabs.label");
        expect(tabs).toHaveLength(2);

        const dailyTab = screen.getByRole("tab", { name: /reports.tabs.daily/i });
        const rawTab = screen.getByRole("tab", { name: /reports.view_raw_logs/i });
        const dailyPanel = screen.getByRole("tabpanel");

        expect(dailyTab).toHaveAttribute("aria-selected", "true");
        expect(rawTab).toHaveAttribute("aria-selected", "false");
        expect(dailyTab).toHaveAttribute("aria-controls", dailyPanel.id);
        expect(dailyPanel).toHaveAttribute("aria-labelledby", dailyTab.id);

        const allPanels = screen.getAllByRole("tabpanel", { hidden: true });
        expect(allPanels).toHaveLength(2);
        const rawPanelReference = allPanels.find((panel) => panel.id === rawTab.getAttribute("aria-controls"));
        expect(rawPanelReference).not.toBeUndefined();
        expect(rawPanelReference).toHaveAttribute("aria-labelledby", rawTab.id);

        await user.click(rawTab);

        const rawPanel = await screen.findByRole("tabpanel");
        expect(rawTab).toHaveAttribute("aria-selected", "true");
        expect(dailyTab).toHaveAttribute("aria-selected", "false");
        expect(rawTab).toHaveAttribute("aria-controls", rawPanel.id);
        expect(rawPanel).toHaveAttribute("aria-labelledby", rawTab.id);
    });

    it("moves focus with arrow and Home/End tab navigation", async () => {
        renderWithProviders(<EmployeeDailyDrawer {...defaultProps} />);

        const dailyTab = await screen.findByRole("tab", { name: /reports.tabs.daily/i });
        const rawTab = screen.getByRole("tab", { name: /reports.view_raw_logs/i });

        dailyTab.focus();
        fireEvent.keyDown(dailyTab, { key: "ArrowRight" });
        expect(rawTab).toHaveFocus();
        expect(rawTab).toHaveAttribute("tabindex", "0");
        expect(dailyTab).toHaveAttribute("tabindex", "-1");

        fireEvent.keyDown(rawTab, { key: "Home" });
        expect(dailyTab).toHaveFocus();

        fireEvent.keyDown(dailyTab, { key: "End" });
        expect(rawTab).toHaveFocus();
    });

    it("keeps the responsive header controls touch-friendly", async () => {
        renderWithProviders(
            <EmployeeDailyDrawer
                {...defaultProps}
                employerId="employer-1"
                employers={[
                    { employerId: "employer-1", employerName: "Acme Corporation" },
                    { employerId: "employer-2", employerName: "Globex Corporation" },
                ]}
            />,
        );

        const employerTrigger = await screen.findByRole("button", { name: /Acme Corporation/i });
        const tablist = screen.getByRole("tablist");
        const tabs = screen.getAllByRole("tab");
        const metrics = Array.from(document.querySelectorAll("[aria-busy]"));

        expect(employerTrigger).toHaveClass("min-h-11", "w-full", "sm:w-auto");
        expect(tablist).toHaveClass("min-h-11", "w-full", "sm:w-auto");
        expect(tablist).toHaveAttribute("aria-orientation", "horizontal");
        expect(tabs).toHaveLength(2);
        tabs.forEach((tab) => expect(tab).toHaveClass("min-h-9"));
        expect(metrics).toHaveLength(2);
        metrics.forEach((metric) => expect(metric).toHaveClass("min-h-11"));
    });

    it("keeps KPI slots mounted while report data is loading", async () => {
        (api.fetchAttendanceReport as Mock).mockReturnValueOnce(new Promise(() => undefined));

        renderWithProviders(<EmployeeDailyDrawer {...defaultProps} />);

        const metricLabels = [
            "reports.quick_stats.std_hours",
            "reports.quick_stats.ot_hours",
        ];

        metricLabels.forEach((label) => {
            const metric = screen.getByText(label).closest("[aria-busy]");
            expect(metric).not.toBeNull();
            expect(metric).toHaveAttribute("aria-busy", "true");
            expect(metric).toHaveClass("min-h-11");
        });
    });

    it("keeps both KPI slots visible after switching to raw logs", async () => {
        const user = userEvent.setup();
        renderWithProviders(<EmployeeDailyDrawer {...defaultProps} />);

        await screen.findAllByText(/08:00 — 17:00/i);
        await user.click(screen.getByRole("tab", { name: /reports.view_raw_logs/i }));

        const metricLabels = [
            "reports.quick_stats.std_hours",
            "reports.quick_stats.ot_hours",
        ];

        metricLabels.forEach((label) => {
            const metric = screen.getByText(label).closest("[aria-busy]");
            expect(metric).not.toBeNull();
            expect(metric).toHaveClass("min-h-11");
        });
    });

    it("opens reprocess modal and calls reprocess mutation", async () => {
        const user = userEvent.setup();
        renderWithProviders(<EmployeeDailyDrawer {...defaultProps} />);

        // Wait for data to load
        await screen.findAllByText(/08:00 — 17:00/i);

        // Click reprocess button (has title reports.reprocess.tooltip)
        const reprocessBtns = await screen.findAllByTitle("reports.reprocess.tooltip");
        await user.click(reprocessBtns[0]);

        // Check if modal opened
        const confirmModalTitle = await screen.findByText("reports.reprocess.confirm_title");
        expect(confirmModalTitle).toBeDefined();

        // Click the actual confirm button inside the modal
        const confirmButton = await screen.findByRole("button", { name: /reports.reprocess.confirm/i });
        await user.click(confirmButton);

        await waitFor(() => {
            expect(api.reprocessAttendance).toHaveBeenCalledWith("emp-1",
                "2026-01-15",
                "2026-01-15"
            );
        });
    });

    it("debounces double-click on reprocess confirm button: only one mutation", async () => {
        const user = userEvent.setup();
        renderWithProviders(<EmployeeDailyDrawer {...defaultProps} />);

        // Wait for data to load
        await screen.findAllByText(/08:00 — 17:00/i);

        // Open the reprocess modal
        const reprocessBtns = await screen.findAllByTitle("reports.reprocess.tooltip");
        await user.click(reprocessBtns[0]);
        await screen.findByText("reports.reprocess.confirm_title");

        // Click confirm twice quickly: the second click must be ignored by the
        // lead-edge in-flight guard, so reprocessAttendance is invoked exactly
        // once (no concurrent reprocess / no duplicated sessions).
        const confirmButton = await screen.findByRole("button", {
            name: /reports.reprocess.confirm/i,
        });
        await user.click(confirmButton);
        await user.click(confirmButton);

        await waitFor(() => {
            expect(api.reprocessAttendance).toHaveBeenCalledTimes(1);
        });
    });

    it("opens confirm attendance modal and calls adminConfirmAttendance mutation", async () => {
        const user = userEvent.setup();
        renderWithProviders(<EmployeeDailyDrawer {...defaultProps} />);

        // Wait for data to load
        await screen.findAllByText(/08:00 — 17:00/i);

        // Assuming existing logic, the confirm button has title "common.attendance_confirm"
        const confirmBtns = await screen.findAllByTitle("common.attendance_confirm");
        await user.click(confirmBtns[0]);

        // Check if modal opened
        const modalTitle = await screen.findByText("reports.confirm.modal_title");
        expect(modalTitle).toBeDefined();

        // Type a reason
        const reasonInput = screen.getByPlaceholderText("reports.confirm.reason_placeholder");
        await user.type(reasonInput, "Test reason");

        // Click confirm inside modal
        const confirmButtons = await screen.findAllByRole("button", { name: /reports.confirm.submit/i });
        await user.click(confirmButtons[0]);

        await waitFor(() => {
            expect(api.adminConfirmAttendance).toHaveBeenCalled();
        });
    });

    it("calls approveOt mutation when OT approve/reject buttons are clicked", async () => {
        const user = userEvent.setup();
        renderWithProviders(<EmployeeDailyDrawer {...defaultProps} />);

        // Wait for data to load
        await screen.findAllByText(/08:00 — 17:00/i);

        // Find the OT approve button
        const approveButton = await screen.findByTitle("reports.ot_approve.approve");
        await user.click(approveButton);

        await waitFor(() => {
            expect(api.approveOt).toHaveBeenCalledWith("emp-1",
                {
                    date: "2026-01-15",
                    approved_minutes: 60,
                    status: "approved"
                }
            );
        });

        // Also check reject
        const rejectButton = await screen.findByTitle("reports.ot_approve.reject");
        await user.click(rejectButton);

        await waitFor(() => {
            expect(api.approveOt).toHaveBeenCalledWith("emp-1",
                {
                    date: "2026-01-15",
                    approved_minutes: 60,
                    status: "rejected"
                }
            );
        });
    });

    it("toggles to raw logs view and fetches raw logs", async () => {
        const user = userEvent.setup();
        renderWithProviders(<EmployeeDailyDrawer {...defaultProps} />);

        // Wait for daily report data to load
        await screen.findAllByText(/08:00 — 17:00/i);

        // Find the "View Raw Logs" toggle button
        const rawLogsToggle = await screen.findByRole("tab", { name: /View Raw Logs|reports.view_raw_logs/i });
        await user.click(rawLogsToggle);

        // Verify the API was called
        await waitFor(() => {
            expect(api.fetchRawAttendanceLogs).toHaveBeenCalledWith({
                employee_id: "emp-1",
                from: new Date("2026-01-01T00:00:00").toISOString(),
                to: new Date("2026-01-31T23:59:59").toISOString(),
                page: 1,
                limit: 100
            });
        });

        // Verify the raw logs table renders header
        expect(await screen.findByText(/common.source/i)).toBeInTheDocument();
    });

    it("content wrapper uses flex-col layout to fill drawer height", async () => {
        renderWithProviders(<EmployeeDailyDrawer {...defaultProps} />);

        // Wait for the sheet to render
        await screen.findByText("John Doe");

        // The content wrapper inside the Sheet should have flex flex-col for height filling
        const sheetPanel = screen.getByTestId("sheet-panel");
        // Find the content wrapper with -mx-10 class (negative margin wrapper)
        const contentWrapper = sheetPanel.querySelector(".-mx-4");
        expect(contentWrapper).not.toBeNull();
        expect(contentWrapper!.className).toContain("flex");
        expect(contentWrapper!.className).toContain("flex-col");
    });

    it("sheet content area has overflow-hidden to let inner tables scroll", async () => {
        renderWithProviders(<EmployeeDailyDrawer {...defaultProps} />);

        await screen.findByText("John Doe");

        // The Sheet content area should have !overflow-hidden
        const sheetPanel = screen.getByTestId("sheet-panel");
        const contentArea = sheetPanel.querySelector(".overflow-y-auto");
        expect(contentArea).not.toBeNull();
        expect(contentArea!.className).toContain("!overflow-hidden");
    });

    it("raw logs view uses flex-1 min-h-0 for dynamic height", async () => {
        const user = userEvent.setup();
        renderWithProviders(<EmployeeDailyDrawer {...defaultProps} />);

        await screen.findAllByText(/08:00 — 17:00/i);

        // Switch to raw logs view
        const rawLogsToggle = await screen.findByRole("tab", { name: /View Raw Logs|reports.view_raw_logs/i });
        await user.click(rawLogsToggle);

        await waitFor(() => {
            expect(api.fetchRawAttendanceLogs).toHaveBeenCalled();
        });

        // The raw logs panel should fill the available height and expose an inner scroller.
        const rawPanel = screen.getByRole("tabpanel");
        expect(rawPanel).toHaveClass("flex", "flex-1", "min-h-0");
        expect(rawPanel.querySelector(".overflow-auto")).not.toBeNull();
    });

    it("switches tabs using ArrowLeft and ArrowRight keys", async () => {
        renderWithProviders(<EmployeeDailyDrawer {...defaultProps} />);

        // By default it is in daily mode
        const dailyToggle = await screen.findByRole("tab", { name: /reports.tabs.daily/i });
        const rawLogsToggle = await screen.findByRole("tab", { name: /reports.view_raw_logs/i });

        // The daily toggle should have the active class initially
        expect(dailyToggle.className).toContain("text-primary-600");

        // Press ArrowRight to switch to raw mode
        fireEvent.keyDown(dailyToggle, { key: "ArrowRight" });

        // Now raw log toggle should be active
        await waitFor(() => {
            expect(rawLogsToggle.className).toContain("text-primary-600");
        });

        expect(api.fetchRawAttendanceLogs).toHaveBeenCalled();

        // Press ArrowLeft to switch back to daily mode
        fireEvent.keyDown(rawLogsToggle, { key: "ArrowLeft" });

        // Now daily toggle should be active again
        await waitFor(() => {
            expect(dailyToggle.className).toContain("text-primary-600");
        });
    });

    it("does not switch tabs when typing in an input field", async () => {
        renderWithProviders(<EmployeeDailyDrawer {...defaultProps} />);

        const dailyToggle = await screen.findByRole("tab", { name: /reports.tabs.daily/i });
        const rawLogsToggle = await screen.findByRole("tab", { name: /reports.view_raw_logs/i });

        // Create a dummy input to simulate typing
        const input = document.createElement("input");
        document.body.appendChild(input);

        // Simulate pressing ArrowRight while focused on input
        fireEvent.keyDown(input, { key: "ArrowRight" });

        // Should STILL be daily mode
        expect(dailyToggle.className).toContain("text-primary-600");
        expect(rawLogsToggle.className).not.toContain("text-primary-600");

        document.body.removeChild(input);
    });

    it("does not switch tabs from a tabpanel", async () => {
        renderWithProviders(<EmployeeDailyDrawer {...defaultProps} />);

        const dailyTab = await screen.findByRole("tab", { name: /reports.tabs.daily/i });
        const rawTab = screen.getByRole("tab", { name: /reports.view_raw_logs/i });
        const dailyPanel = screen.getByRole("tabpanel");

        dailyPanel.focus();
        fireEvent.keyDown(dailyPanel, { key: "ArrowRight" });
        fireEvent.keyDown(dailyPanel, { key: "Home" });

        expect(dailyTab).toHaveAttribute("aria-selected", "true");
        expect(rawTab).toHaveAttribute("aria-selected", "false");
        expect(dailyPanel).toHaveFocus();
    });

    it("resets to daily tab when re-opened", async () => {
        const { rerender } = renderWithProviders(<EmployeeDailyDrawer {...defaultProps} />);

        // Find toggles
        const rawLogsToggle = await screen.findByRole("tab", { name: /reports.view_raw_logs/i });

        // Switch to raw
        fireEvent.click(rawLogsToggle);

        // Verify it switched
        await waitFor(() => {
            expect(rawLogsToggle.className).toContain("text-primary-600");
        });

        // Close drawer
        rerender(
            <QueryClientProvider client={queryClient}>
                <ToastProvider>
                    <EmployeeDailyDrawer {...defaultProps} isOpen={false} />
                </ToastProvider>
            </QueryClientProvider>
        );

        // Open drawer again
        rerender(
            <QueryClientProvider client={queryClient}>
                <ToastProvider>
                    <EmployeeDailyDrawer {...defaultProps} isOpen={true} />
                </ToastProvider>
            </QueryClientProvider>
        );

        // Find standard toggle again
        const dailyToggle2 = await screen.findByRole("tab", { name: /reports.tabs.daily/i });

        // It should be reset back to daily view
        await waitFor(() => {
            expect(dailyToggle2.className).toContain("text-primary-600");
        });
    });

    it("scrolls content down/up with ArrowDown/ArrowUp keys", async () => {
        renderWithProviders(<EmployeeDailyDrawer {...defaultProps} />);

        // Wait for content to render
        await screen.findAllByText(/08:00 — 17:00/i);

        // Find the content wrapper that has the ref
        const sheetPanel = screen.getByTestId("sheet-panel");
        const contentWrapper = sheetPanel.querySelector(".-mx-4");
        expect(contentWrapper).not.toBeNull();

        // The scroll container is the overflow-auto child or the wrapper itself
        const scrollContainer = contentWrapper!.querySelector(".overflow-auto") ?? contentWrapper!;
        const scrollBySpy = vi.fn();
        scrollContainer.scrollBy = scrollBySpy;

        // Press ArrowDown
        fireEvent.keyDown(window, { key: "ArrowDown" });

        expect(scrollBySpy).toHaveBeenCalledWith({ top: 100, behavior: "smooth" });

        scrollBySpy.mockClear();

        // Press ArrowUp
        fireEvent.keyDown(window, { key: "ArrowUp" });

        expect(scrollBySpy).toHaveBeenCalledWith({ top: -100, behavior: "smooth" });
    });

    it("opens resolve modal when unscheduled badge is clicked", async () => {
        const user = userEvent.setup();
        const unscheduledRow = {
            ...mockDailyRow,
            status: "unscheduled",
            scheduled_start: null,
            scheduled_end: null,
        };
 
        (api.fetchAttendanceReport as Mock).mockResolvedValue({
            data: [unscheduledRow],
            total: 1,
        });

        renderWithProviders(<EmployeeDailyDrawer {...defaultProps} />);

        // Wait for data to load
        await screen.findByText("John Doe");

        // Find and click the unscheduled badge (desktop + mobile both render)
        const badges = await screen.findAllByTitle("reports.status.unscheduled_tooltip");
        await user.click(badges[0]);

        // The resolve modal should open
        expect(await screen.findByText("reports.unscheduled.resolve_title")).toBeInTheDocument();

        // Shift selector should be present
        expect(screen.getByText("reports.unscheduled.select_shift")).toBeInTheDocument();

        // The mock shift should appear in the dropdown
        const select = screen.getByRole("combobox");
        expect(select).toBeInTheDocument();
    });

    it("submits resolve modal: calls upsertRoster then reprocessAttendance", async () => {
        const user = userEvent.setup();
        const unscheduledRow = {
            ...mockDailyRow,
            status: "unscheduled",
            scheduled_start: null,
            scheduled_end: null,
        };
 
        (api.fetchAttendanceReport as Mock).mockResolvedValue({
            data: [unscheduledRow],
            total: 1,
        });

        renderWithProviders(<EmployeeDailyDrawer {...defaultProps} />);

        // Wait for data + click badge
        const names = await screen.findAllByText("John Doe");
        expect(names.length).toBeGreaterThan(0);
        const badge = (await screen.findAllByTitle("reports.status.unscheduled_tooltip"))[0];
        await user.click(badge);

        // Wait for modal
        await screen.findByText("reports.unscheduled.resolve_title");

        // Select a shift from the dropdown
        const select = screen.getByRole("combobox");
        await user.selectOptions(select, "shift-morning");

        // Click submit button
        const submitButton = screen.getByRole("button", { name: /reports.unscheduled.submit/i });
        await user.click(submitButton);

        // Verify upsertRoster was called first
        await waitFor(() => {
            expect(api.upsertRoster).toHaveBeenCalledWith("emp-1",
                { date: "2026-01-15", shift_id: "shift-morning" }
            );
        });

        // Then reprocessAttendance
        await waitFor(() => {
            expect(api.reprocessAttendance).toHaveBeenCalledWith("emp-1",
                "2026-01-15",
                "2026-01-15"
            );
        });
    });
});
