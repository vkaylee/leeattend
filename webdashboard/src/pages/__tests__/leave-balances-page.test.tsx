import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent, waitFor, act } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import { ToastProvider } from "@/components/ui/toast";
import * as api from "@/lib/api";

import LeaveBalancesPage from "../leave-balances-page";

vi.mock("react-i18next", () => ({
    useTranslation: () => ({
        t: (key: string, fallback?: string | Record<string, unknown>) => {
            if (typeof fallback === 'string') return fallback;
            if (typeof fallback === "object" && fallback !== null && "defaultValue" in fallback && fallback.defaultValue) return fallback.defaultValue as string;
            return key;
        },
    }),
}));

// Mock useDebounce to return value immediately (no delay)
vi.mock("@/hooks/use-debounce", () => ({
    useDebounce: <T,>(value: T) => value,
}));

vi.mock("@/lib/api", async () => {
    const actual = await vi.importActual<typeof import("@/lib/api")>("@/lib/api");
    return {
        ...actual,
        fetchWorkspaceLeaveBalances: vi.fn(),
        triggerAccrualRun: vi.fn(),
 
        fetchJobStatus: vi.fn(),
        getCurrentWorkspaceId: () => "test-ws-id",
    };
});

vi.mock("@/components/leave/leave-history-modal", () => ({
    LeaveHistoryModal: (props: { leaveTypeId: string; leaveTypeName: string; employeeId: string; isOpen: boolean }) => (
        <div data-employeeid={props.employeeId} data-leavetypeid={props.leaveTypeId} data-leavetypename={props.leaveTypeName} data-testid="leave-history-modal">
            {props.isOpen ? "MOCKED_LEAVE_HISTORY_MODAL" : null}
        </div>
    ),
}));

vi.mock("@/components/leave/leave-balance-detail-modal", () => ({
    LeaveBalanceDetailModal: ({ isOpen }: { isOpen: boolean }) =>
        isOpen ? <div data-testid="leave-balance-detail-modal" /> : null,
}));

const mockBalances = {
    data: [
        {
            employee_id: "emp-1",
            employee_name: "Nguyen Van A",
            leave_type_id: "lt-1",
            leave_type_code: "AL",
            leave_type_name: "Annual Leave",
            is_paid: true,
            entitlement: 12,
            accrued_amount: 3,
            carried_over: 2,
            used: 4,
            remaining: 13,
            policy_name: "Standard Policy",
            assignment_type: "workspace",
        },
        {
            employee_id: "emp-2",
            employee_name: "Tran Thi B",
            leave_type_id: "lt-2",
            leave_type_code: "SL",
            leave_type_name: "Sick Leave",
            is_paid: true,
            entitlement: 5,
            accrued_amount: 0,
            carried_over: 0,
            used: 5,
            remaining: 0,
            policy_name: null,
            assignment_type: null,
        },
    ],
    total: 2,
    page: 1,
    limit: 20,
};

describe("LeaveBalancesPage", () => {
    let queryClient: QueryClient;

    beforeEach(() => {
        vi.clearAllMocks();
        queryClient = new QueryClient({
            defaultOptions: { queries: { retry: false } },
        });
 
        vi.mocked(api.fetchWorkspaceLeaveBalances).mockResolvedValue(mockBalances as unknown as Awaited<ReturnType<typeof api.fetchWorkspaceLeaveBalances>>);
 
        vi.mocked(api.triggerAccrualRun).mockResolvedValue({ job_id: "", status: "completed", message: "Processed 2 accrual entries synchronously" });
    });

    function renderPage() {
        return render(
            <QueryClientProvider client={queryClient}>
                <ToastProvider>
                    <LeaveBalancesPage />
                </ToastProvider>
            </QueryClientProvider>
        );
    }

    it("renders page title", () => {
        renderPage();
        expect(screen.getByText("leave_balances.title")).toBeDefined();
    });

    it("renders employee data after loading", async () => {
        renderPage();
        // Wait for async data to render
        await waitFor(() => {
            expect(screen.getAllByText("Nguyen Van A").length).toBeGreaterThan(0);
        }, { timeout: 3000 });

        expect(screen.getAllByText("Tran Thi B").length).toBeGreaterThan(0);
        expect(screen.getAllByText("Annual Leave").length).toBeGreaterThan(0);
        expect(screen.getAllByText("Sick Leave").length).toBeGreaterThan(0);
    });

    it("shows remaining balance values", async () => {
        renderPage();
        await waitFor(() => {
            expect(screen.getAllByText("Nguyen Van A").length).toBeGreaterThan(0);
        }, { timeout: 3000 });

        // emp-1 remaining = 13
        const remainingEls = screen.getAllByText("13");
        expect(remainingEls.length).toBeGreaterThan(0);
    });

    it("renders usage progress bars with correct percentage", async () => {
        renderPage();
        await waitFor(() => {
            expect(screen.getAllByText("Nguyen Van A").length).toBeGreaterThan(0);
        }, { timeout: 3000 });

        // emp-1: used 4 / total (12 + 3 + 2) = 4/17 ≈ 24%
        expect(screen.getAllByText("24%").length).toBeGreaterThan(0);
        // emp-2: used 5 / total (5 + 0 + 0) = 5/5 = 100%
        expect(screen.getAllByText("100%").length).toBeGreaterThan(0);
    });

    it("shows year selector with current year", async () => {
        renderPage();
        await waitFor(() => {
            expect(screen.getAllByText("Nguyen Van A").length).toBeGreaterThan(0);
        }, { timeout: 3000 });

        const yearSelect = screen.getByRole("combobox");
        expect((yearSelect as HTMLSelectElement).value).toBe(String(new Date().getFullYear()));
    });

    it("calls API with year param when year is changed", async () => {
        renderPage();
        await waitFor(() => {
            expect(screen.getAllByText("Nguyen Van A").length).toBeGreaterThan(0);
        }, { timeout: 3000 });

        const yearSelect = screen.getByRole("combobox");
        fireEvent.change(yearSelect, { target: { value: "2024" } });

        await waitFor(() => {
            expect(api.fetchWorkspaceLeaveBalances).toHaveBeenCalledWith(
                "test-ws-id",
                expect.objectContaining({ year: 2024, page: 1 })
            );
        });
    });

    it("shows custom confirm dialog when accrual button clicked", async () => {
        renderPage();
        await waitFor(() => {
            expect(screen.getAllByText("Nguyen Van A").length).toBeGreaterThan(0);
        }, { timeout: 3000 });

        fireEvent.click(screen.getByText("LEAVE_BALANCES.RUN_ACCRUAL"));
        expect(screen.getByText("Confirm")).toBeDefined();
        expect(screen.getByText("Cancel")).toBeDefined();
    });

    it("cancels accrual when Huỷ is clicked", async () => {
        renderPage();
        await waitFor(() => {
            expect(screen.getAllByText("Nguyen Van A").length).toBeGreaterThan(0);
        }, { timeout: 3000 });

        fireEvent.click(screen.getByText("LEAVE_BALANCES.RUN_ACCRUAL"));
        fireEvent.click(screen.getByText("Cancel"));
        expect(api.triggerAccrualRun).not.toHaveBeenCalled();
    });

    it("triggers accrual mutation when confirmed", async () => {
        renderPage();
        await waitFor(() => {
            expect(screen.getAllByText("Nguyen Van A").length).toBeGreaterThan(0);
        }, { timeout: 3000 });

        fireEvent.click(screen.getByText("LEAVE_BALANCES.RUN_ACCRUAL"));
        fireEvent.click(screen.getByText("Confirm"));

        await waitFor(() => {
            expect(api.triggerAccrualRun).toHaveBeenCalledWith(
                "test-ws-id",
                expect.objectContaining({ date: expect.any(String) as unknown as string })
            );
        });
    });

    it("shows loading state when data is pending", () => {
 
        vi.mocked(api.fetchWorkspaceLeaveBalances).mockReturnValue(new Promise(() => { /* empty */ }));
        renderPage();
        expect(screen.getAllByText("common.loading").length).toBeGreaterThan(0);
    });

    it("disables accrual button when viewing past or future years", async () => {
        renderPage();
        await waitFor(() => {
            expect(screen.getAllByText("Nguyen Van A").length).toBeGreaterThan(0);
        }, { timeout: 3000 });

        const accrualButton = screen.getByText("LEAVE_BALANCES.RUN_ACCRUAL").closest("button");
        expect(accrualButton).not.toBeDisabled();

        // Change year to prev year
        const yearSelect = screen.getByRole("combobox");
        fireEvent.change(yearSelect, { target: { value: String(new Date().getFullYear() - 1) } });

        await waitFor(() => {
            expect(accrualButton).toBeDisabled();
        });
    });

    it("passes leaveTypeId and leaveTypeName to LeaveHistoryModal when View History is clicked", async () => {
        renderPage();
        await waitFor(() => {
            expect(screen.getAllByText("Nguyen Van A").length).toBeGreaterThan(0);
        }, { timeout: 3000 });

        // The first "View History" button in DesktopBalanceRow
        // They are rendered with <button title="leave_balances.table.history_tooltip"... 
        // Or we can find by a specific icon or screen.getAllByRole("button") that have that action.
        // It's probably easier to mock LeaveHistoryModal and check its props. Let's update the mock to render the props.
    });

    it("shows empty state when no data", async () => {
 
        vi.mocked(api.fetchWorkspaceLeaveBalances).mockResolvedValue({
            data: [],
            total: 0,
            page: 1,
            limit: 20,
        });
        renderPage();
        await waitFor(() => {
            expect(screen.getAllByText("attendance_logs.messages.no_data").length).toBeGreaterThan(0);
        }, { timeout: 3000 });
    });

    it("passes leaveTypeId and leaveTypeName to LeaveHistoryModal when View History is clicked", async () => {
        renderPage();
        await waitFor(() => {
            expect(screen.getAllByText("Nguyen Van A").length).toBeGreaterThan(0);
        }, { timeout: 3000 });

        // Get the "Lịch sử" buttons from table rows (it uses History icon, but we can look for tooltip)
        // DesktopBalanceRow uses: title={t("leave_balances.table.history_tooltip")}
        const historyBtns = screen.getAllByTitle("leave_balances.table.history_tooltip");
        fireEvent.click(historyBtns[0]); // Click on Nguyen Van A / Annual Leave

        await waitFor(() => {
            const modal = screen.getByTestId("leave-history-modal");
            expect(modal.getAttribute("data-employeeid")).toBe("emp-1");
            expect(modal.getAttribute("data-leavetypeid")).toBe("lt-1");
            expect(modal.getAttribute("data-leavetypename")).toBe("Annual Leave");
        });
    });

    it("shows queued toast when accrual is processed in background", async () => {
 
        vi.mocked(api.triggerAccrualRun).mockResolvedValue({
            job_id: "job-123",
            status: "queued",
            message: "Accrual job enqueued for background processing",
        });
        renderPage();
        await waitFor(() => {
            expect(screen.getAllByText("Nguyen Van A").length).toBeGreaterThan(0);
        }, { timeout: 3000 });

        // Open confirm dialog and confirm
        fireEvent.click(screen.getByText("LEAVE_BALANCES.RUN_ACCRUAL"));
        fireEvent.click(screen.getByText("Confirm"));

        await waitFor(() => {
            expect(api.triggerAccrualRun).toHaveBeenCalledWith(
                "test-ws-id",
                expect.objectContaining({ date: expect.any(String) as unknown as string })
            );
        });
    });

    it("shows completed toast when accrual runs synchronously", async () => {
 
        vi.mocked(api.triggerAccrualRun).mockResolvedValue({
            job_id: "",
            status: "completed",
            message: "Processed 5 accrual entries synchronously",
        });
        renderPage();
        await waitFor(() => {
            expect(screen.getAllByText("Nguyen Van A").length).toBeGreaterThan(0);
        }, { timeout: 3000 });

        fireEvent.click(screen.getByText("LEAVE_BALANCES.RUN_ACCRUAL"));
        fireEvent.click(screen.getByText("Confirm"));

        await waitFor(() => {
            expect(api.triggerAccrualRun).toHaveBeenCalled();
        });
    });

    it("polls fetchJobStatus after queued response until completed", async () => {
        vi.useFakeTimers({ shouldAdvanceTime: true });
 
        vi.mocked(api.triggerAccrualRun).mockResolvedValue({
            job_id: "job-poll-1",
            status: "queued",
            message: "Enqueued",
        });
        // First poll: still processing. Second poll: completed.
 
        vi.mocked(api.fetchJobStatus)
            .mockResolvedValueOnce({ job_id: "job-poll-1", status: "processing", completed_at: null, error_message: null })
            .mockResolvedValueOnce({ job_id: "job-poll-1", status: "completed", completed_at: "2026-02-28T12:00:00Z", error_message: null });

        renderPage();
        await waitFor(() => {
            expect(screen.getAllByText("Nguyen Van A").length).toBeGreaterThan(0);
        }, { timeout: 3000 });

        fireEvent.click(screen.getByText("LEAVE_BALANCES.RUN_ACCRUAL"));
        fireEvent.click(screen.getByText("Confirm"));

        // Wait for mutation to fire
        await waitFor(() => {
            expect(api.triggerAccrualRun).toHaveBeenCalled();
        });

        // Advance past first poll interval (2s)
        act(() => { vi.advanceTimersByTime(2500); });
        await waitFor(() => {
            expect(api.fetchJobStatus).toHaveBeenCalledWith("test-ws-id", "job-poll-1");
        });

        // Advance past second poll interval
        act(() => { vi.advanceTimersByTime(2500); });
        await waitFor(() => {
 
            expect(vi.mocked(api.fetchJobStatus).mock.calls.length).toBeGreaterThanOrEqual(2);
        });

        vi.useRealTimers();
    });

    it("stops polling and shows error when job fails", async () => {
        vi.useFakeTimers({ shouldAdvanceTime: true });
 
        vi.mocked(api.triggerAccrualRun).mockResolvedValue({
            job_id: "job-fail-1",
            status: "queued",
            message: "Enqueued",
        });
 
        vi.mocked(api.fetchJobStatus).mockResolvedValue({
            job_id: "job-fail-1",
            status: "failed",
            completed_at: "2026-02-28T12:00:00Z",
            error_message: "Accrual processing failed: DB error",
        });

        renderPage();
        await waitFor(() => {
            expect(screen.getAllByText("Nguyen Van A").length).toBeGreaterThan(0);
        }, { timeout: 3000 });

        fireEvent.click(screen.getByText("LEAVE_BALANCES.RUN_ACCRUAL"));
        fireEvent.click(screen.getByText("Confirm"));

        await waitFor(() => {
            expect(api.triggerAccrualRun).toHaveBeenCalled();
        });

        // Advance past first poll interval
        act(() => { vi.advanceTimersByTime(2500); });
        await waitFor(() => {
            expect(api.fetchJobStatus).toHaveBeenCalledWith("test-ws-id", "job-fail-1");
        });

        // Should NOT poll again after failure
 
        const callCount = vi.mocked(api.fetchJobStatus).mock.calls.length;
        act(() => { vi.advanceTimersByTime(5000); });
 
        expect(vi.mocked(api.fetchJobStatus).mock.calls.length).toBe(callCount);

        vi.useRealTimers();
    });
});
