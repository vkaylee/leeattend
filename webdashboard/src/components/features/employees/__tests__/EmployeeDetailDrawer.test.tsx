import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import React from "react";
import EmployeeDetailDrawer from "../EmployeeDetailDrawer";
import type { Employee } from "@/lib/api";

// Mock scroll methods not available in jsdom
Element.prototype.scrollIntoView = vi.fn();

vi.mock("react-i18next", () => ({
    useTranslation: () => ({
        t: (key: string, fallback?: string) => fallback || key,
        i18n: { changeLanguage: () => Promise.resolve(), language: "en" },
    }),
}));

// Mock child components
vi.mock("@/components/payroll/EmployeeSalaryConfig", () => ({
    default: () => <div data-testid="salary-config">SalaryConfig</div>,
}));
vi.mock("@/components/employees/WorkingConditionConfig", () => ({
    default: () => <div data-testid="working-condition">WorkingCondition</div>,
}));

// ---------------------------------------------------------------------------
// Test data
// ---------------------------------------------------------------------------

const mockEmployee: Employee = {
    id: "emp-1",
    full_name: "Alice Nguyen",
    employee_id_number: "EMP-001",
    workspace_id: "ws-1",
    is_active: true,
    hire_date: "2024-06-15",
    created_at: "2024-06-15T00:00:00Z",
    updated_at: "2024-06-15T00:00:00Z",
};

const terminatedEmployee: Employee = {
    ...mockEmployee,
    id: "emp-2",
    full_name: "Bob Tran",
    is_active: false,
};

const mockDepartments = [
    { id: "dept-1", name: "Engineering", workspace_id: "ws-1", created_at: "2024-01-01T00:00:00Z", updated_at: "2024-01-01T00:00:00Z", active_employee_count: 5 },
];

const mockDeptHistory = [
    { department_id: "dept-1", effective_date: "2024-06-15" },
];

const mockEmploymentHistory = [
    { id: "h-1", hire_date: "2024-06-15", termination_date: null, is_rehire: false },
];

const defaultProps = {
    isOpen: true,
    onClose: vi.fn(),
    employee: mockEmployee,
    wsId: "ws-1",
    departments: mockDepartments,
    deptHistory: mockDeptHistory,
    employmentHistory: mockEmploymentHistory,
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
    beforeEach(() => { vi.clearAllMocks(); });

    // ── Rendering ──────────────────────────────────────────────

    it("renders nothing when employee is null", () => {
        const { container } = render(
            <EmployeeDetailDrawer {...defaultProps} employee={null} />
        );
        expect(container.innerHTML).toBe("");
    });

    it("renders employee name and ID when open", () => {
        render(<EmployeeDetailDrawer {...defaultProps} />);
        expect(screen.getByText("Alice Nguyen")).toBeDefined();
        expect(screen.getByText("EMP-001")).toBeDefined();
    });

    it("displays active status badge for active employees", () => {
        render(<EmployeeDetailDrawer {...defaultProps} />);
        expect(screen.getByText(/employees.table.active/i)).toBeDefined();
    });

    it("displays terminated status badge for terminated employees", () => {
        render(<EmployeeDetailDrawer {...defaultProps} employee={terminatedEmployee} />);
        expect(screen.getByText(/employees.table.terminated/i)).toBeDefined();
    });

    it("renders only one X close button (not two)", () => {
        render(<EmployeeDetailDrawer {...defaultProps} />);
        const closeButtons = document.querySelectorAll("svg.lucide-x");
        expect(closeButtons.length).toBe(1);
    });

    // ── Tabs ───────────────────────────────────────────────────

    it("starts on Profile tab by default", () => {
        render(<EmployeeDetailDrawer {...defaultProps} />);
        expect(screen.getByTestId("employees-btn-edit")).toBeDefined();
    });

    it("switches to Salary tab on click", async () => {
        const user = userEvent.setup();
        render(<EmployeeDetailDrawer {...defaultProps} />);

        await user.click(screen.getByTestId("employees-tab-salary"));
        expect(screen.getByTestId("salary-config")).toBeDefined();
    });

    it("switches to Policies tab on click", async () => {
        const user = userEvent.setup();
        render(<EmployeeDetailDrawer {...defaultProps} />);

        await user.click(screen.getByTestId("employees-tab-policies"));
        expect(screen.getByTestId("working-condition")).toBeDefined();
    });

    it("switches to History tab on click", async () => {
        const user = userEvent.setup();
        render(<EmployeeDetailDrawer {...defaultProps} />);

        await user.click(screen.getByTestId("employees-tab-history"));
        expect(screen.getByText(/employees.details.joined/i)).toBeDefined();
    });

    // ── Profile tab actions ────────────────────────────────────

    it("calls onEdit when edit button is clicked", async () => {
        const onEdit = vi.fn();
        const user = userEvent.setup();
        render(<EmployeeDetailDrawer {...defaultProps} onEdit={onEdit} />);

        await user.click(screen.getByTestId("employees-btn-edit"));
        expect(onEdit).toHaveBeenCalledWith(mockEmployee);
    });

    it("calls onTerminate when terminate button is clicked (active employee)", async () => {
        const onTerminate = vi.fn();
        const user = userEvent.setup();
        render(<EmployeeDetailDrawer {...defaultProps} onTerminate={onTerminate} />);

        await user.click(screen.getByTestId("employees-btn-terminate"));
        expect(onTerminate).toHaveBeenCalledWith(mockEmployee);
    });

    it("shows rehire button for terminated employee and calls onRehire", async () => {
        const onRehire = vi.fn();
        const user = userEvent.setup();
        render(
            <EmployeeDetailDrawer {...defaultProps} employee={terminatedEmployee} onRehire={onRehire} />
        );

        await user.click(screen.getByTestId("employees-btn-rehire"));
        expect(onRehire).toHaveBeenCalledWith(terminatedEmployee);
    });

    // ── Department section ─────────────────────────────────────

    it("displays current department name", () => {
        render(<EmployeeDetailDrawer {...defaultProps} />);
        expect(screen.getByText("Engineering")).toBeDefined();
    });

    it("shows 'not assigned' when no department history", () => {
        render(<EmployeeDetailDrawer {...defaultProps} deptHistory={[]} />);
        expect(screen.getByText(/employees.details.not_assigned_dept/i)).toBeDefined();
    });

    it("calls onTransfer when transfer button is clicked", async () => {
        const onTransfer = vi.fn();
        const user = userEvent.setup();
        render(<EmployeeDetailDrawer {...defaultProps} onTransfer={onTransfer} />);

        await user.click(screen.getByText(/employees.details.transfer/i));
        expect(onTransfer).toHaveBeenCalledWith("dept-1");
    });

    // ── Close ──────────────────────────────────────────────────

    it("calls onClose when X button is clicked", async () => {
        const onClose = vi.fn();
        const user = userEvent.setup();
        render(<EmployeeDetailDrawer {...defaultProps} onClose={onClose} />);

        const xButton = document.querySelector("svg.lucide-x")!.closest("button")!;
        await user.click(xButton);
        expect(onClose).toHaveBeenCalledTimes(1);
    });

    // ── Hire date ──────────────────────────────────────────────

    it("displays formatted hire date", () => {
        render(<EmployeeDetailDrawer {...defaultProps} />);
        expect(screen.getByText("2024-06-15")).toBeDefined();
    });

    // ── Keyboard tab navigation ────────────────────────────────

    it("ArrowRight moves to the next tab", async () => {
        const user = userEvent.setup();
        render(<EmployeeDetailDrawer {...defaultProps} />);

        // Start on Profile (has edit button)
        expect(screen.getByTestId("employees-btn-edit")).toBeDefined();

        // ArrowRight → Salary
        await user.keyboard("{ArrowRight}");
        expect(screen.getByTestId("salary-config")).toBeDefined();

        // ArrowRight → Policies
        await user.keyboard("{ArrowRight}");
        expect(screen.getByTestId("working-condition")).toBeDefined();
    });

    it("ArrowLeft moves to the previous tab", async () => {
        const user = userEvent.setup();
        render(<EmployeeDetailDrawer {...defaultProps} />);

        // ArrowRight to Salary first
        await user.keyboard("{ArrowRight}");
        expect(screen.getByTestId("salary-config")).toBeDefined();

        // ArrowLeft back to Profile
        await user.keyboard("{ArrowLeft}");
        expect(screen.getByTestId("employees-btn-edit")).toBeDefined();
    });

    it("ArrowLeft wraps from first tab to last tab", async () => {
        const user = userEvent.setup();
        render(<EmployeeDetailDrawer {...defaultProps} />);

        // On Profile → ArrowLeft → should wrap to History (last tab)
        await user.keyboard("{ArrowLeft}");
        expect(screen.getByText(/employees.details.joined/i)).toBeDefined();
    });

    it("ArrowRight wraps from last tab to first tab", async () => {
        const user = userEvent.setup();
        render(<EmployeeDetailDrawer {...defaultProps} />);

        // Navigate to History (last tab): Profile → Salary → Policies → History
        await user.keyboard("{ArrowRight}{ArrowRight}{ArrowRight}");
        expect(screen.getByText(/employees.details.joined/i)).toBeDefined();

        // ArrowRight → should wrap to Profile
        await user.keyboard("{ArrowRight}");
        expect(screen.getByTestId("employees-btn-edit")).toBeDefined();
    });

    // ── Keyboard scroll (ArrowUp/ArrowDown) ────────────────────

    it("ArrowDown scrolls drawer content down", async () => {
        const user = userEvent.setup();
        render(<EmployeeDetailDrawer {...defaultProps} />);

        const scrollContainer = document.querySelector(".overflow-y-auto")!;
        const scrollBySpy = vi.fn();
        scrollContainer.scrollBy = scrollBySpy;

        await user.keyboard("{ArrowDown}");
        expect(scrollBySpy).toHaveBeenCalledWith({ top: 80, behavior: "smooth" });
    });

    it("ArrowUp scrolls drawer content up", async () => {
        const user = userEvent.setup();
        render(<EmployeeDetailDrawer {...defaultProps} />);

        const scrollContainer = document.querySelector(".overflow-y-auto")!;
        const scrollBySpy = vi.fn();
        scrollContainer.scrollBy = scrollBySpy;

        await user.keyboard("{ArrowUp}");
        expect(scrollBySpy).toHaveBeenCalledWith({ top: -80, behavior: "smooth" });
    });
});
