import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import EmployeeDailyDrawer from "../EmployeeDailyDrawer";
import { BrowserRouter } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ToastProvider } from "@/components/ui/Toast";
import * as api from "@/lib/api";
import React from "react";

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
        fetchWorkspaceAttendanceAdjustments: vi.fn(),
        adminConfirmAttendance: vi.fn(),
        approveOt: vi.fn(),
    };
});

// Mock DebugContext
vi.mock("@/context/DebugContext", () => ({
    useDebug: () => ({ isEnabled: false }),
}));

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
        status: "present",
        ot_claim_status: "pending_manager",
        generation_metadata: null,
    };

    const defaultProps = {
        isOpen: true,
        onClose: vi.fn(),
        workspaceId: "ws-1",
        employeeId: "emp-1",
        employeeName: "John Doe",
        dateRange: { start: "2026-01-01", end: "2026-01-31" },
        workUnitMinutes: 60,
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

        (api.fetchAttendanceReport as any).mockResolvedValue({
            data: [mockDailyRow],
            total: 1,
        });
        (api.fetchWorkspaceAttendanceAdjustments as any).mockResolvedValue([]);
        (api.reprocessAttendance as any).mockResolvedValue(true);
        (api.adminConfirmAttendance as any).mockResolvedValue(true);
        (api.approveOt as any).mockResolvedValue(true);
    });

    const renderWithProviders = (ui: React.ReactElement) => {
        return render(
            <QueryClientProvider client={queryClient}>
                <ToastProvider>
                    <BrowserRouter>{ui}</BrowserRouter>
                </ToastProvider>
            </QueryClientProvider>
        );
    };

    it("does not render when not open", () => {
        renderWithProviders(<EmployeeDailyDrawer {...defaultProps} isOpen={false} />);
        const drawerHeader = screen.queryByText("John Doe");
        expect(drawerHeader).toBeNull();
    });

    it("fetches data and renders the drawer content when opened", async () => {
        renderWithProviders(<EmployeeDailyDrawer {...defaultProps} />);

        // Header should contain the name
        expect(await screen.findByText("John Doe")).toBeDefined();

        // Check if API was called
        await waitFor(() => {
            expect(api.fetchAttendanceReport).toHaveBeenCalledWith(
                "ws-1",
                "2026-01-01",
                "2026-01-31",
                "daily",
                1,
                100,
                undefined,
                ["emp-1"]
            );
        });

        // Wait for the table row to render
        const shiftText = await screen.findByText(/08:00 — 17:00/i);
        expect(shiftText).toBeDefined();
    });

    it("opens reprocess modal and calls reprocess mutation", async () => {
        const user = userEvent.setup();
        renderWithProviders(<EmployeeDailyDrawer {...defaultProps} />);

        // Wait for data to load
        await screen.findByText(/08:00 — 17:00/i);

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
            expect(api.reprocessAttendance).toHaveBeenCalledWith(
                "ws-1",
                "emp-1",
                "2026-01-15",
                "2026-01-15"
            );
        });
    });

    it("opens confirm attendance modal and calls adminConfirmAttendance mutation", async () => {
        const user = userEvent.setup();
        renderWithProviders(<EmployeeDailyDrawer {...defaultProps} />);

        // Wait for data to load
        await screen.findByText(/08:00 — 17:00/i);

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
        await screen.findByText(/08:00 — 17:00/i);

        // Find the OT approve button
        const approveBtn = await screen.findByTitle("reports.ot_approve.approve");
        await user.click(approveBtn);

        await waitFor(() => {
            expect(api.approveOt).toHaveBeenCalledWith(
                "ws-1",
                "emp-1",
                {
                    date: "2026-01-15",
                    approved_minutes: 60,
                    status: "approved"
                }
            );
        });

        // Also check reject
        const rejectBtn = await screen.findByTitle("reports.ot_approve.reject");
        await user.click(rejectBtn);

        await waitFor(() => {
            expect(api.approveOt).toHaveBeenCalledWith(
                "ws-1",
                "emp-1",
                {
                    date: "2026-01-15",
                    approved_minutes: 60,
                    status: "rejected"
                }
            );
        });
    });

});
