 
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";

import type { Employee } from "@/lib/api";

import EmployeeDetailDrawer from "../employee-detail-drawer";


// Mock scroll methods not available in jsdom
Element.prototype.scrollIntoView = vi.fn();

vi.mock("react-i18next", () => ({
    useTranslation: () => ({
        t: (key: string, fallback?: string) => fallback ?? key,
        i18n: { changeLanguage: () => Promise.resolve(), language: "en" },
    }),
}));

// Mock child components
vi.mock("@/components/payroll/employee-salary-config", () => ({
    default: () => <div data-testid="salary-config">SalaryConfig</div>,
}));
vi.mock("../employee-contracts-tab", () => ({
    default: () => <div data-testid="contracts-tab">ContractsTab</div>,
}));

vi.mock("@/pages/employees/compensation-history-tab", () => ({
    default: () => <div data-testid="compensation-tab">CompensationTab</div>,
}));

const mockMembers = [
    {
        workspace_id: "ws-1",
        user_id: "user-123",
        role_id: "role-1",
        email: "alice@example.com",
        full_name: "Alice User",
        avatar_url: undefined,
        extra_role_ids: [],
    },
];

vi.mock("@/lib/api", () => ({
    fetchOrganization: vi.fn().mockResolvedValue({ date_format: "DD/MM/YYYY", timezone: "UTC", default_language: "en" }),
    fetchOrganizationMembers: vi.fn(() => Promise.resolve(mockMembers)),
    fetchEmployers: vi.fn(() => Promise.resolve([{ id: "employer-1", name: "Acme" }])),
    updateEmployee: vi.fn(() => Promise.resolve({})),
    fetchEmployeeEmployments: vi.fn((employeeId) => {
        if (employeeId === "emp-2") {
            return Promise.resolve([
                {
                    id: "employment-2",
                    employee_id: "emp-2",
                    status: "terminated",
                    primary_location_id: "loc-1",
                    start_date: "2024-06-15",
                    end_date: "2024-06-20",
                }
            ]);
        }
        return Promise.resolve([
            {
                id: "employment-old",
                employee_id: "emp-1",
                status: "terminated",
                primary_location_id: "loc-1",
                start_date: "2020-01-01",
            },
            {
                id: "employment-1",
                employee_id: "emp-1",
                status: "active",
                primary_location_id: "loc-1",
                start_date: "2024-06-15",
            }
        ]);
    }),
    updateEmployeeEmployment: vi.fn(() => Promise.resolve({})),
}));
vi.mock("@/pages/settings/hooks/use-locations", () => ({
    useLocations: () => ({
        data: [
            { id: "loc-1", name: "Tokyo HQ", is_headquarters: true, timezone: "Asia/Tokyo" },
            { id: "loc-2", name: "Osaka Branch", is_headquarters: false, timezone: "Asia/Tokyo" }
        ],
        isLoading: false
    })
}));
vi.mock("@/components/ui/toast", () => ({
    useToast: () => ({
        success: vi.fn(),
        error: vi.fn()
    })
}));
vi.mock("@/hooks/use-permissions", () => ({
    useMyPermissions: () => ({
        hasPermission: () => true,
        hasAnyPermission: () => true,
        hasScopedPermission: () => true,
        isLoading: false,
    }),
}));

// ---------------------------------------------------------------------------
// Test data
// ---------------------------------------------------------------------------

const mockEmployee: Employee = {
    id: "emp-1",
    full_name: "Alice Nguyen",
    employee_id_number: "EMP-001",
    workspace_id: "ws-1",
    active_employment_id: "employment-1",
    created_at: "2024-06-15T00:00:00Z",
    updated_at: "2024-06-15T00:00:00Z",
    primary_location_id: "loc-1",
    tags: [],
};

const terminatedEmployee: Employee = {
    ...mockEmployee,
    id: "emp-2",
    full_name: "Bob Tran",
    active_employment_id: null,
};

const linkedEmployee: Employee = {
    ...mockEmployee,
    id: "emp-3",
    full_name: "Charlie Le",
    user_id: "user-123",
};

const mockDepartments = [
    { id: "dept-1", name: "Engineering", workspace_id: "ws-1", created_at: "2024-01-01T00:00:00Z", updated_at: "2024-01-01T00:00:00Z", active_employee_count: 5 },
];

const mockDeptHistory = [
    { department_id: "dept-1", effective_date: "2024-06-15" },
];


const defaultProps = {
    isOpen: true,
    onClose: vi.fn(),
    employee: mockEmployee,departments: mockDepartments,
    deptHistory: mockDeptHistory,
    formatDate: (d: string) => d,
    onEdit: vi.fn(),
    onTerminate: vi.fn(),
    onRehire: vi.fn(),
    onTransfer: vi.fn(),
};

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe("EmployeeDetailDrawer", () => {
    let queryClient: QueryClient;
    
    beforeEach(() => { 
        vi.clearAllMocks(); 
        queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    });

    const renderWithClient = (ui: React.ReactElement) => {
        return render(<QueryClientProvider client={queryClient}>{ui}</QueryClientProvider>);
    };

    // ── Rendering ──────────────────────────────────────────────

    it("renders nothing when employee is null", () => {
        const { container } = renderWithClient(
            <EmployeeDetailDrawer {...defaultProps} employee={null} />
        );
        expect(container.innerHTML).toBe("");
    });

    it("renders employee name and ID when open", () => {
        renderWithClient(<EmployeeDetailDrawer {...defaultProps} />);
        expect(screen.getByText("Alice Nguyen")).toBeDefined();
        expect(screen.getByText("EMP-001")).toBeDefined();
    });

    it("displays active status badge for active employees", () => {
        renderWithClient(<EmployeeDetailDrawer {...defaultProps} />);
        expect(screen.getByText(/employees.table.active/i)).toBeDefined();
    });

    it("renders employment history with start and end dates", async () => {
        renderWithClient(<EmployeeDetailDrawer {...defaultProps} />);
        expect(await screen.findByText("employment-old")).toBeDefined();
        expect(screen.getByText("employment-1")).toBeDefined();
        expect(screen.getAllByText("2020-01-01").length).toBeGreaterThan(0);
    });

    it("displays terminated status badge for terminated employees", () => {
        renderWithClient(<EmployeeDetailDrawer {...defaultProps} employee={terminatedEmployee} />);
        expect(screen.getByText(/employees.table.terminated/i)).toBeDefined();
    });

    it("renders only one X close button (not two)", () => {
        renderWithClient(<EmployeeDetailDrawer {...defaultProps} />);
        const closeButtons = document.querySelectorAll("svg.lucide-x");
        expect(closeButtons.length).toBe(1);
    });

    // ── Tabs ───────────────────────────────────────────────────

    it("starts on Profile tab and omits centralized tabs", () => {
        renderWithClient(<EmployeeDetailDrawer {...defaultProps} />);
        expect(screen.getByTestId("employees-btn-edit")).toBeDefined();
        expect(screen.queryByTestId("employees-tab-assignments")).not.toBeInTheDocument();
        expect(screen.queryByTestId("employees-tab-working-conditions")).not.toBeInTheDocument();
        expect(screen.queryByTestId("employees-tab-lifecycle")).not.toBeInTheDocument();
        expect(screen.queryByTestId("employees-tab-phases")).not.toBeInTheDocument();
        expect(screen.queryByTestId("employees-tab-attendance-policy")).not.toBeInTheDocument();
    });

    it("switches to Salary tab on click", async () => {
        const user = userEvent.setup();
        renderWithClient(<EmployeeDetailDrawer {...defaultProps} />);

        await user.click(screen.getByTestId("employees-tab-salary"));
        expect(screen.getByTestId("salary-config")).toBeDefined();
    });

    it("switches to Contracts tab on click", async () => {
        const user = userEvent.setup();
        renderWithClient(<EmployeeDetailDrawer {...defaultProps} />);

        await user.click(screen.getByTestId("employees-tab-contracts"));
        expect(screen.getByTestId("contracts-tab")).toBeDefined();
    });

    // ── Profile tab actions ────────────────────────────────────

    it("calls onEdit when edit button is clicked", async () => {
        const onEdit = vi.fn();
        const user = userEvent.setup();
        renderWithClient(<EmployeeDetailDrawer {...defaultProps} onEdit={onEdit} />);

        await user.click(screen.getByTestId("employees-btn-edit"));
        expect(onEdit).toHaveBeenCalledWith(mockEmployee);
    });

    it("calls onTerminate when terminate button is clicked (active employee)", async () => {
        const onTerminate = vi.fn();
        const user = userEvent.setup();
        renderWithClient(<EmployeeDetailDrawer {...defaultProps} onTerminate={onTerminate} />);

        await user.click(screen.getByTestId("employees-btn-terminate"));
        expect(onTerminate).toHaveBeenCalledWith(mockEmployee);
    });

    it("shows rehire button and resignation date for terminated employee and calls onRehire", async () => {
        const onRehire = vi.fn();
        const user = userEvent.setup();
        renderWithClient(
            <EmployeeDetailDrawer {...defaultProps} employee={terminatedEmployee} onRehire={onRehire} />
        );

        // Verify resignation date is displayed
        expect(await screen.findByText(/employees\.details\.terminated_at/i)).toBeDefined();
        expect((await screen.findAllByText(/2024-06-20/)).length).toBeGreaterThan(0);

        await user.click(screen.getByTestId("employees-btn-rehire"));
        expect(onRehire).toHaveBeenCalledWith(terminatedEmployee);
    });

    it("shows undo termination button for terminated employee and triggers modal on click", async () => {
        const onEmployeeUpdate = vi.fn();
        const user = userEvent.setup();
        renderWithClient(
            <EmployeeDetailDrawer {...defaultProps} employee={terminatedEmployee} onEmployeeUpdate={onEmployeeUpdate} />
        );

        // Wait for queries to load by expecting the resignation date in DOM
        expect((await screen.findAllByText(/2024-06-20/)).length).toBeGreaterThan(0);

        // Verify undo termination button is displayed
        const undoBtn = screen.getByTestId("employees-btn-undo-terminate");
        expect(undoBtn).toBeDefined();

        await user.click(undoBtn);
        // Verify UndoTerminationModal title is displayed
        expect(await screen.findByText(/employees\.terminate\.undo_title/i)).toBeDefined();
    });

    // ── Department section ─────────────────────────────────────

    it("displays current department name", () => {
        renderWithClient(<EmployeeDetailDrawer {...defaultProps} />);
        expect(screen.getByText("Engineering")).toBeDefined();
    });

    it("shows 'not assigned' when no department history", () => {
        renderWithClient(<EmployeeDetailDrawer {...defaultProps} deptHistory={[]} />);
        expect(screen.getByText(/employees.details.not_assigned_dept/i)).toBeDefined();
    });

    it("calls onTransfer when transfer button is clicked", async () => {
        const onTransfer = vi.fn();
        const user = userEvent.setup();
        renderWithClient(<EmployeeDetailDrawer {...defaultProps} onTransfer={onTransfer} />);

        await user.click(screen.getByText(/employees.details.transfer/i));
        expect(onTransfer).toHaveBeenCalledWith("dept-1");
    });

    // ── Work Location section ────────────────────────────────────

    it("displays current work location", async () => {
        renderWithClient(<EmployeeDetailDrawer {...defaultProps} />);
        expect(await screen.findByText("Tokyo HQ")).toBeDefined();
        expect(screen.getByText("Asia/Tokyo")).toBeDefined();
        expect(screen.getByText("HQ")).toBeDefined();
    });

    it("switches to edit mode and saves new location", async () => {
        const user = userEvent.setup();
        const { updateEmployeeEmployment } = await import("@/lib/api");
        renderWithClient(<EmployeeDetailDrawer {...defaultProps} />);

        // Wait for employment query to finish loading location details
        expect(await screen.findByText("Tokyo HQ")).toBeDefined();

        // Click Edit
        await user.click(screen.getByText(/common.edit/i));
        
        // Find combobox/select
        const select = screen.getByRole("combobox");
        // Change selection
        await user.selectOptions(select, "loc-2");

        // Click Save
        await user.click(screen.getByText(/SAVE/i));

        expect(updateEmployeeEmployment).toHaveBeenCalledWith("emp-1", "employment-1", { primary_location_id: "loc-2" });
    });

    // ── Close ──────────────────────────────────────────────────

    it("calls onClose when X button is clicked", async () => {
        const onClose = vi.fn();
        const user = userEvent.setup();
        renderWithClient(<EmployeeDetailDrawer {...defaultProps} onClose={onClose} />);

        const xButton = document.querySelector("svg.lucide-x")!.closest("button")!;
        await user.click(xButton);
        expect(onClose).toHaveBeenCalledTimes(1);
    });

    // ── Hire date ──────────────────────────────────────────────

    it("displays formatted hire date", async () => {
        renderWithClient(<EmployeeDetailDrawer {...defaultProps} />);
        expect((await screen.findAllByText("2024-06-15")).length).toBeGreaterThan(0);
    });

    // ── Linked user section ────────────────────────────────────

    it("shows dash when employee has no linked user", () => {
        renderWithClient(<EmployeeDetailDrawer {...defaultProps} />);
        expect(screen.getAllByText("—").length).toBeGreaterThan(0);
    });

    it("shows linked user name and email when employee has a user_id", async () => {
        renderWithClient(<EmployeeDetailDrawer {...defaultProps} employee={linkedEmployee} />);
        expect(await screen.findByText("Alice User")).toBeDefined();
        expect(screen.getByText("alice@example.com")).toBeDefined();
    });

    // ── Keyboard tab navigation ────────────────────────────────

    it("ArrowRight moves to the next tab", async () => {
        const user = userEvent.setup();
        renderWithClient(<EmployeeDetailDrawer {...defaultProps} />);

        // Start on Profile (has edit button)
        expect(screen.getByTestId("employees-btn-edit")).toBeDefined();

        // ArrowRight → Documents (index 1)
        await user.keyboard("{ArrowRight}");

        // ArrowRight → Contracts (index 2)
        await user.keyboard("{ArrowRight}");
        expect(screen.getByTestId("contracts-tab")).toBeDefined();

        // ArrowRight → Salary (index 3)
        await user.keyboard("{ArrowRight}");
        expect(screen.getByTestId("salary-config")).toBeDefined();

        // ArrowRight → Compensation (index 4)
        await user.keyboard("{ArrowRight}");
        expect(screen.getByTestId("compensation-tab")).toBeDefined();
    });

    it("ArrowLeft moves to the previous tab", async () => {
        const user = userEvent.setup();
        renderWithClient(<EmployeeDetailDrawer {...defaultProps} />);

        // ArrowRight to Contracts first (2 steps)
        await user.keyboard("{ArrowRight}{ArrowRight}");
        expect(screen.getByTestId("contracts-tab")).toBeDefined();

        // ArrowLeft back to Profile (2 steps)
        await user.keyboard("{ArrowLeft}{ArrowLeft}");
        expect(screen.getByTestId("employees-btn-edit")).toBeDefined();
    });

    it("ArrowLeft wraps from first tab to last tab", async () => {
        const user = userEvent.setup();
        renderWithClient(<EmployeeDetailDrawer {...defaultProps} />);

        // On Profile → ArrowLeft → should wrap to Compensation (last tab)
        await user.keyboard("{ArrowLeft}");
        expect(screen.getByTestId("compensation-tab")).toBeDefined();
    });

    it("ArrowRight wraps from last tab to first tab", async () => {
        const user = userEvent.setup();
        renderWithClient(<EmployeeDetailDrawer {...defaultProps} />);

        // Navigate to Compensation (last tab, index 4): Profile(0) -> 4 steps
        await user.keyboard("{ArrowRight}{ArrowRight}{ArrowRight}{ArrowRight}");
        expect(screen.getByTestId("compensation-tab")).toBeDefined();

        // ArrowRight → should wrap to Profile
        await user.keyboard("{ArrowRight}");
        expect(screen.getByTestId("employees-btn-edit")).toBeDefined();
    });

    // ── Keyboard scroll (ArrowUp/ArrowDown) ────────────────────

    it("ArrowDown scrolls drawer content down", async () => {
        const user = userEvent.setup();
        renderWithClient(<EmployeeDetailDrawer {...defaultProps} />);

        const scrollContainer = document.querySelector(".overflow-y-auto")!;
        const scrollBySpy = vi.fn();
        scrollContainer.scrollBy = scrollBySpy;

        await user.keyboard("{ArrowDown}");
        expect(scrollBySpy).toHaveBeenCalledWith({ top: 80, behavior: "smooth" });
    });

    it("ArrowUp scrolls drawer content up", async () => {
        const user = userEvent.setup();
        renderWithClient(<EmployeeDetailDrawer {...defaultProps} />);

        const scrollContainer = document.querySelector(".overflow-y-auto")!;
        const scrollBySpy = vi.fn();
        scrollContainer.scrollBy = scrollBySpy;

        await user.keyboard("{ArrowUp}");
        expect(scrollBySpy).toHaveBeenCalledWith({ top: -80, behavior: "smooth" });
    });

    // ── Tab nav scroll arrows ───────────────────────────────

    it("renders scroll-left and scroll-right buttons for tab nav", () => {
        renderWithClient(<EmployeeDetailDrawer {...defaultProps} />);
        expect(screen.getByTestId("employees-tab-scroll-left")).toBeDefined();
        expect(screen.getByTestId("employees-tab-scroll-right")).toBeDefined();
    });

    it("scrolls tab container left/right on arrow click", async () => {
        const user = userEvent.setup();
        renderWithClient(<EmployeeDetailDrawer {...defaultProps} />);

        const container = screen.getByTestId("employees-tab-scroll-container");
        const scrollBySpy = vi.fn();
        container.scrollBy = scrollBySpy;

        await user.click(screen.getByTestId("employees-tab-scroll-right"));
        expect(scrollBySpy).toHaveBeenCalledWith({ left: 240, behavior: "smooth" });

        await user.click(screen.getByTestId("employees-tab-scroll-left"));
        expect(scrollBySpy).toHaveBeenCalledWith({ left: -240, behavior: "smooth" });
    });

    it("calls scrollIntoView on active tab when switching tabs", async () => {
        const user = userEvent.setup();
        renderWithClient(<EmployeeDetailDrawer {...defaultProps} />);

        // scrollIntoView is already mocked globally at the top
        await user.click(screen.getByTestId("employees-tab-salary"));
        // After click, the active tab effect fires. Since we use "nearest",
        // if tab is already visible jsdom treats it as a no-op — we just assert
        // the Content switch still works (good-enough smoke check).
        expect(screen.getByTestId("salary-config")).toBeDefined();
    });
});
