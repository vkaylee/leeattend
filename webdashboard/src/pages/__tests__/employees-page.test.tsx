import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";

import { ToastProvider } from "../../components/ui/toast";
import * as api from "../../lib/api";
import { Employee, Department } from "../../lib/api/types";
import EmployeesPage from "../employees-page";


const mockNavigate = vi.fn();

// Mock TanStack Router to avoid needing full route tree in unit tests
vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => mockNavigate,
 
  useLocation: () => ({ pathname: "/", search: "", hash: "" }),
  Link: ({ to, children, ...props }: { to: string; children: React.ReactNode; [key: string]: unknown }) => <a href={to} {...props}>{children}</a>,
}));

// Mock IntersectionObserver for jsdom
class MockIntersectionObserver {
    observe() { /* empty */ }
    unobserve() { /* empty */ }
    disconnect() { /* empty */ }
 
}
window.IntersectionObserver = MockIntersectionObserver as unknown as typeof IntersectionObserver;

// Mock the API library
vi.mock("../../lib/api/custom-fields", async (importOriginal) => {
    const actual = await importOriginal<typeof import("../../lib/api/custom-fields")>();
    return {
        ...actual,
        fetchCustomFieldDefinitions: vi.fn().mockResolvedValue([]),
    };
});

vi.mock("../../lib/api", async (importOriginal) => {
    const actual = await importOriginal<typeof import("../../lib/api")>();
    return {
        ...actual,
        fetchOrganization: vi.fn().mockResolvedValue({ name: "Test Organization" }),
        fetchEmployees: vi.fn(),
        searchEmployees: vi.fn(),
        createEmployee: vi.fn(),
        updateEmployee: vi.fn(),
        deleteEmployee: vi.fn(),
        fetchDepartments: vi.fn(),
        fetchEmployeeDepartmentHistory: vi.fn(),
        transferEmployee: vi.fn(),
        fetchEmploymentHistory: vi.fn(),
        fetchEmployeeTenures: vi.fn().mockResolvedValue([]),
        createEmployeeTenure: vi.fn(),
        terminateEmployeeEmployment: vi.fn(() => Promise.resolve({ employment: {}, relationship_closed: true, tenure_closed: true, sessions_revoked: false })),
        fetchMyPermissions: vi.fn(),
        fetchOrganizationMembers: vi.fn().mockResolvedValue([]),
        fetchEmployeeEmployments: vi.fn((empId) => Promise.resolve([
            {
                id: `employment-id-${empId}`,
                employee_id: empId,
                employer_id: "employer-1",
                primary_location_id: "location-1",
                start_date: "2025-01-01",
                status: "active"
            }
        ])),
        updateEmployeeEmployment: vi.fn(() => Promise.resolve({})),
    };
});

vi.mock("@/hooks/use-permissions", () => ({
    useMyPermissions: () => ({
        hasPermission: () => true,
        hasAnyPermission: () => true,
        hasRole: () => true,
        hasScopedPermission: () => true,
        permissions: ["employees:read", "employees:create", "employees:update", "employees:delete"],
        roles: ["admin"],
        managedDepartmentIds: null,
        isLoading: false,
    }),
}));

// Override i18n
vi.mock("react-i18next", () => ({
 
    useTranslation: () => ({
        t: (key: string, options?: string | { defaultValue?: string }) => {
            if (typeof options === "string") return options;
            if (options && typeof options === "object" && options.defaultValue) return options.defaultValue;
            return key;
        },
        i18n: {
            changeLanguage: () => Promise.resolve(),
            language: "en",
        },
    }),
}));

// Mock child components that aren't relevant to these tests
vi.mock("@/components/payroll/employee-salary-config", () => ({
    default: () => <div data-testid="salary-config">SalaryConfig</div>,
}));
const mockEmployee = (id: string, name: string, active = true, department?: string): Employee => ({
    id,
    full_name: name,
    employee_id_number: `EMP-${id}`,
    workspace_id: "test-ws-id",
    active_employment_id: active ? `employment-id-${id}` : null,
    created_at: "2025-01-01T00:00:00Z",
    updated_at: "2025-01-01T00:00:00Z",
    tags: [],
    ...(department ? { department_name: department } : {}),
});

describe("EmployeesPage", () => {
    let queryClient: QueryClient;

    beforeEach(() => {
        vi.clearAllMocks();
        queryClient = new QueryClient({
            defaultOptions: {
                queries: { retry: false, gcTime: 0, staleTime: 0 },
            },
        });
 
 
        vi.mocked(api.searchEmployees).mockResolvedValue({
            data: [mockEmployee("1", "Alice Nguyen"), mockEmployee("2", "Bob Tran")],
            total: 2,
            page: 1,
            limit: 20,
        });
 
        vi.mocked(api.fetchDepartments).mockResolvedValue([]);
 
        vi.mocked(api.fetchEmployeeDepartmentHistory).mockResolvedValue([]);
 
        vi.mocked(api.fetchEmploymentHistory).mockResolvedValue([]);
 
        vi.mocked(api.fetchMyPermissions).mockResolvedValue({ permissions: ["employees:create", "employees:update", "employees:delete"], roles: ["admin"], managed_department_ids: [] });
    });

    const renderPage = (_initialEntries = ["/"]) =>
        render(
            <QueryClientProvider client={queryClient}>
                <ToastProvider>
                    <EmployeesPage />
                </ToastProvider>
            </QueryClientProvider>
        );

    // ────────────────────────────────────────────────────────────
    // Basic rendering
    // ────────────────────────────────────────────────────────────

    it("renders employee list", async () => {
        renderPage();
        // ResponsiveTable renders both desktop and mobile views
        expect((await screen.findAllByText("Alice Nguyen")).length).toBeGreaterThan(0);
        expect(screen.getAllByText("Bob Tran").length).toBeGreaterThan(0);
    });

    it("keeps mobile employee cards in a constrained column layout", async () => {
        renderPage();
        const cards = await screen.findAllByRole("button", { name: /Alice Nguyen/i });

        expect(cards.some((card) => card.className.includes("flex-col"))).toBe(true);
        expect(cards.some((card) => card.className.includes("min-w-0"))).toBe(true);
    });

    it("shows empty state when no employees exist", async () => {

        vi.mocked(api.searchEmployees).mockResolvedValue({
            data: [],
            total: 0,
            page: 1,
            limit: 20,
        });
        renderPage();
        // ResponsiveTable renders both desktop and mobile empty states
        expect((await screen.findAllByText(/employees.table.no_data/i)).length).toBeGreaterThan(0);
    });

    // ────────────────────────────────────────────────────────────
    // Department column
    // ────────────────────────────────────────────────────────────

    it("renders department column header", async () => {
        renderPage();
        void (await screen.findAllByText("Alice Nguyen"))[0];
        // ResponsiveTable renders both desktop and mobile views
        expect(screen.getAllByText("common.department").length).toBeGreaterThan(0);
    });

    it("displays department name when employee has a department", async () => {

        vi.mocked(api.searchEmployees).mockResolvedValue({
            data: [mockEmployee("1", "Alice Nguyen", true, "Engineering"), mockEmployee("2", "Bob Tran", true, "Sales")],
            total: 2,
            page: 1,
            limit: 20,
        });
        renderPage();
        // ResponsiveTable renders both desktop and mobile views
        expect((await screen.findAllByText("Engineering")).length).toBeGreaterThan(0);
        expect(screen.getAllByText("Sales").length).toBeGreaterThan(0);
    });

    it("displays dash when employee has no department", async () => {
 
        vi.mocked(api.searchEmployees).mockResolvedValue({
            data: [mockEmployee("1", "Alice Nguyen")],
            total: 1,
            page: 1,
            limit: 20,
        });
        renderPage();
        void (await screen.findAllByText("Alice Nguyen"))[0];
        // ResponsiveTable renders both desktop and mobile views
        expect(screen.getAllByText("—").length).toBeGreaterThan(0);
    });

    it("handles mixed employees with and without departments", async () => {
 
        vi.mocked(api.searchEmployees).mockResolvedValue({
            data: [mockEmployee("1", "Alice Nguyen", true, "HR"), mockEmployee("2", "Bob Tran")],
            total: 2,
            page: 1,
            limit: 20,
        });
        renderPage();
        // ResponsiveTable renders both desktop and mobile views
        expect((await screen.findAllByText("HR")).length).toBeGreaterThan(0);
        expect(screen.getAllByText("—").length).toBeGreaterThan(0);
    });

    // ────────────────────────────────────────────────────────────
    // Lazy load / infinite scroll
    // ────────────────────────────────────────────────────────────

    it("uses PAGE_SIZE of 20 for infinite scroll", async () => {
        renderPage();
        await waitFor(() => {
            expect(api.searchEmployees).toHaveBeenCalledWith(
                expect.objectContaining({ page: 1, limit: 20 })
            );
        });
    });

    it("does not render pagination controls", async () => {
        renderPage();
        void (await screen.findAllByText("Alice Nguyen"))[0];

        // PaginationControls should be gone
        expect(screen.queryByText(/common.pagination.prev/i)).toBeNull();
        expect(screen.queryByText(/common.pagination.next/i)).toBeNull();
        expect(screen.queryByText(/Page \d+ \//i)).toBeNull();
    });

    it("displays floating record count badge with loaded/total", async () => {
 
        vi.mocked(api.searchEmployees).mockResolvedValue({
            data: [mockEmployee("1", "Alice Nguyen")],
            total: 10,
            page: 1,
            limit: 20,
        });

        renderPage();
        void (await screen.findAllByText("Alice Nguyen"))[0];

        const badge = document.querySelector(".fixed.z-50");
        expect(badge).toBeDefined();
        expect(badge?.textContent).toContain("1");
        expect(badge?.textContent).toContain("10");
    });

    // ────────────────────────────────────────────────────────────
    // Search
    // ────────────────────────────────────────────────────────────

    it("triggers searchEmployees API on search input with debounce", async () => {
        const user = userEvent.setup();
        renderPage();
        void (await screen.findAllByText("Alice Nguyen"))[0];

        const searchInput = screen.getByTestId("employee-search");
        await user.type(searchInput, "Bob");

        await waitFor(
            () => {
                expect(api.searchEmployees).toHaveBeenCalledWith(
                    expect.objectContaining({ query: "Bob" })
                );
            },
            { timeout: 3000 }
        );
    });

    // ────────────────────────────────────────────────────────────
    // Detail panel
    // ────────────────────────────────────────────────────────────

    it("navigates to employee detail page when clicking an employee row", async () => {
        const user = userEvent.setup();
        renderPage();
        const row = (await screen.findAllByText("Alice Nguyen"))[0];
        await user.click(row);

        expect(mockNavigate).toHaveBeenCalledWith({
            to: "/employees/$employeeId",
            params: { employeeId: "1" },
        });
    });

    // ────────────────────────────────────────────────────────────
    // Employee CRUD
    // ────────────────────────────────────────────────────────────

    it("navigates to the employee create page when clicking the add button", async () => {
        const user = userEvent.setup();
        renderPage();
        void (await screen.findAllByText("Alice Nguyen"))[0];

        const addButton = await screen.findByTestId("add-employee-btn");
        await user.click(addButton);

        expect(mockNavigate).toHaveBeenCalledWith({ to: "/employees/new" });
    });

    it("handles terminating an employee with useConfirm", async () => {
        const user = userEvent.setup();
        renderPage();

        // click row actions button to open action dropdown
        const actionsButton = await screen.findByTestId("employee-actions-1");
        await user.click(actionsButton);

        // find terminate button in actions menu
        const terminateOption = await screen.findByText(/Thôi việc|employees\.details\.terminate/i);
        await user.click(terminateOption);

        // terminate modal appears — must fill required fields before confirming
        const reasonSelect = await screen.findByRole("combobox");
        fireEvent.change(reasonSelect, { target: { value: "voluntary_resignation" } });

        const confirmModalButton = await screen.findByTestId("confirm-terminate-btn");
        fireEvent.click(confirmModalButton);

        await waitFor(() => {
            expect(api.terminateEmployeeEmployment).toHaveBeenCalledWith(
                "1",
                "employment-id-1",
                expect.objectContaining({ termination_reason: "voluntary_resignation", rehire_eligible: true })
            );
        });
    });

    // ────────────────────────────────────────────────────────────
    // Column sorting
    // ────────────────────────────────────────────────────────────

    const findHeaderByText = (text: string) => {
        const ths = document.querySelectorAll("th");
        for (const th of ths) {
            if (th.textContent.includes(text)) return th;
        }
        throw new Error(`Header containing "${text}" not found`);
    };

    it("sends sort_by=name and sort_dir=asc when clicking Name header", async () => {
        const user = userEvent.setup();
        renderPage();
        void (await screen.findAllByText("Alice Nguyen"))[0];

 
        vi.mocked(api.searchEmployees).mockClear();
        const nameHeader = findHeaderByText("employees.table.name");
        await user.click(nameHeader);

        await waitFor(() => {
            expect(api.searchEmployees).toHaveBeenCalledWith(
                expect.objectContaining({ sortBy: "name", sortDir: "asc" })
            );
        });
    });

    it("toggles sort_dir to desc when clicking the same header twice", async () => {
        const user = userEvent.setup();
        renderPage();
        void (await screen.findAllByText("Alice Nguyen"))[0];

        await user.click(findHeaderByText("employees.table.name"));

        await waitFor(() => {
            expect(api.searchEmployees).toHaveBeenCalledWith(
                expect.objectContaining({ sortBy: "name", sortDir: "asc" })
            );
        }, { timeout: 3000 });

 
        vi.mocked(api.searchEmployees).mockClear();
        // Re-query header since DOM is re-rendered after first click
        await user.click(findHeaderByText("employees.table.name"));

        await waitFor(() => {
            expect(api.searchEmployees).toHaveBeenCalledWith(
                expect.objectContaining({ sortBy: "name", sortDir: "desc" })
            );
        }, { timeout: 3000 });
    });

    it("renders status filter button in Status column header", async () => {
        renderPage();
        void (await screen.findAllByText("Alice Nguyen"))[0];

        const filterButton = screen.getByTestId("status-filter-btn");
        expect(filterButton).toBeDefined();
    });

    it("opens status filter popover and shows status options", async () => {
        const user = userEvent.setup();
        renderPage();
        void (await screen.findAllByText("Alice Nguyen"))[0];

        const filterButton = screen.getByTestId("status-filter-btn");
        await user.click(filterButton);

        await waitFor(() => {
            expect(screen.getByTestId("status-filter-option-active")).toBeDefined();
            expect(screen.getByTestId("status-filter-option-terminated")).toBeDefined();
            expect(screen.getByTestId("status-filter-option-all")).toBeDefined();
        });
    });

    it("sends status=terminated when selecting the terminated option", async () => {
        const user = userEvent.setup();
        renderPage();
        void (await screen.findAllByText("Alice Nguyen"))[0];

        // Open status filter popover
        await user.click(screen.getByTestId("status-filter-btn"));

        // Click "Terminated" option
        vi.mocked(api.searchEmployees).mockClear();
        const terminatedOption = await screen.findByTestId("status-filter-option-terminated");
        await user.click(terminatedOption);

        await waitFor(() => {
            expect(api.searchEmployees).toHaveBeenCalledWith(
                expect.objectContaining({ status: "terminated" })
            );
        });
    });

    it("removes status filter and resets to active when clicking the status chip X", async () => {
        const user = userEvent.setup();
        renderPage();
        void (await screen.findAllByText("Alice Nguyen"))[0];

        // Select terminated via popover
        await user.click(screen.getByTestId("status-filter-btn"));
        await user.click(await screen.findByTestId("status-filter-option-terminated"));

        // Wait for status chip to appear
        await waitFor(() => {
            expect(screen.getByTestId("active-filter-chips-chip-status-terminated")).toBeDefined();
        });

        // Click the chip to remove it
        vi.mocked(api.searchEmployees).mockClear();
        await user.click(screen.getByTestId("active-filter-chips-chip-status-terminated"));

        await waitFor(() => {
            expect(api.searchEmployees).toHaveBeenCalledWith(
                expect.objectContaining({ status: "active" })
            );
        });
    });


    it("sends sort_by=department when clicking Department header", async () => {
        const user = userEvent.setup();
        renderPage();
        void (await screen.findAllByText("Alice Nguyen"))[0];

 
        vi.mocked(api.searchEmployees).mockClear();
        const deptHeader = findHeaderByText("common.department");
        await user.click(deptHeader);

        await waitFor(() => {
            expect(api.searchEmployees).toHaveBeenCalledWith(
                expect.objectContaining({ sortBy: "department", sortDir: "asc" })
            );
        });
    });

    // ────────────────────────────────────────────────────────────
    // Department filter (checkbox popover)
    // ────────────────────────────────────────────────────────────

    const mockDepartments: Department[] = [
        { id: "dept-1", name: "Engineering", workspace_id: "test-ws-id", created_at: "2025-01-01T00:00:00Z", updated_at: "2025-01-01T00:00:00Z", active_employee_count: 0 },
        { id: "dept-2", name: "Sales", workspace_id: "test-ws-id", created_at: "2025-01-01T00:00:00Z", updated_at: "2025-01-01T00:00:00Z", active_employee_count: 0 },
        { id: "dept-3", name: "HR", workspace_id: "test-ws-id", created_at: "2025-01-01T00:00:00Z", updated_at: "2025-01-01T00:00:00Z", active_employee_count: 0 },
    ];

    it("renders department filter button in header", async () => {
 
        vi.mocked(api.fetchDepartments).mockResolvedValue(mockDepartments);
        renderPage();
        void (await screen.findAllByText("Alice Nguyen"))[0];

        const filterButton = screen.getByTestId("dept-filter-btn");
        expect(filterButton).toBeDefined();
    });

    it("opens filter popover and shows department checkboxes", async () => {
        const user = userEvent.setup();
 
        vi.mocked(api.fetchDepartments).mockResolvedValue(mockDepartments);
        renderPage();
        void (await screen.findAllByText("Alice Nguyen"))[0];

        const filterButton = screen.getByTestId("dept-filter-btn");
        await user.click(filterButton);

        // All departments should be visible as options (multiple instances due to ResponsiveTable)
        await waitFor(() => {
            expect(screen.getAllByText("Engineering").length).toBeGreaterThan(0);
            expect(screen.getAllByText("Sales").length).toBeGreaterThan(0);
            expect(screen.getAllByText("HR").length).toBeGreaterThan(0);
        });
    });

    it("sends department_id filter when selecting a department", async () => {
        const user = userEvent.setup();
 
        vi.mocked(api.fetchDepartments).mockResolvedValue(mockDepartments);
        renderPage();
        void (await screen.findAllByText("Alice Nguyen"))[0];

        // Open filter popover
        const filterButton = screen.getByTestId("dept-filter-btn");
        await user.click(filterButton);

        // Click "Engineering" checkbox
 
        vi.mocked(api.searchEmployees).mockClear();
        const engOption = await screen.findByTestId("dept-filter-option-dept-1");
        await user.click(engOption);

        await waitFor(() => {
            expect(api.searchEmployees).toHaveBeenCalledWith(
                expect.objectContaining({ departmentIds: ["dept-1"] })
            );
        });
    });

    it("sends multiple department_ids when selecting multiple departments", async () => {
        const user = userEvent.setup();
 
        vi.mocked(api.fetchDepartments).mockResolvedValue(mockDepartments);
        renderPage();
        void (await screen.findAllByText("Alice Nguyen"))[0];

        // Open filter popover
        await user.click(screen.getByTestId("dept-filter-btn"));

        // Select Engineering and Sales in one popover session
 
        vi.mocked(api.searchEmployees).mockClear();
        await user.click(await screen.findByTestId("dept-filter-option-dept-1"));
        await user.click(await screen.findByTestId("dept-filter-option-dept-2"));

        await waitFor(() => {
            expect(api.searchEmployees).toHaveBeenCalledWith(
                expect.objectContaining({ departmentIds: ["dept-1", "dept-2"] })
            );
        });
    });

    it("clears filter when clicking the chip X button", async () => {
        const user = userEvent.setup();
 
        vi.mocked(api.fetchDepartments).mockResolvedValue(mockDepartments);
        renderPage();
        void (await screen.findAllByText("Alice Nguyen"))[0];

        // Select one department via popover
        await user.click(screen.getByTestId("dept-filter-btn"));
        await user.click(await screen.findByTestId("dept-filter-option-dept-1"));

        // Wait for filter chip to appear
        await waitFor(() => {
            expect(screen.getByTestId("active-filter-chips")).toBeDefined();
        });

        // Click the filter chip to remove it
 
        vi.mocked(api.searchEmployees).mockClear();
        await user.click(screen.getByTestId("active-filter-chips-chip-dept-dept-1"));

        await waitFor(() => {
            expect(api.searchEmployees).toHaveBeenCalledWith(
                expect.objectContaining({ departmentIds: undefined })
            );
        });
    });

    // ────────────────────────────────────────────────────────────
    // UX improvements
    // ────────────────────────────────────────────────────────────

    it("shows filter chips with department names when filter is active", async () => {
        const user = userEvent.setup();
 
        vi.mocked(api.fetchDepartments).mockResolvedValue(mockDepartments);
        renderPage();
        void (await screen.findAllByText("Alice Nguyen"))[0];

        // Select two departments
        await user.click(screen.getByTestId("dept-filter-btn"));
        await user.click(await screen.findByTestId("dept-filter-option-dept-1"));
        await user.click(await screen.findByTestId("dept-filter-option-dept-3"));

        // Chips area should appear with both department names
        await waitFor(() => {
            const chipsArea = screen.getByTestId("active-filter-chips");
            expect(chipsArea).toBeDefined();
            expect(screen.getByTestId("active-filter-chips-chip-dept-dept-1")).toBeDefined();
            expect(screen.getByTestId("active-filter-chips-chip-dept-dept-3")).toBeDefined();
        });
    });

    it("does not show filter chips when no filter is active", async () => {
 
        vi.mocked(api.fetchDepartments).mockResolvedValue(mockDepartments);
        renderPage();
        void (await screen.findAllByText("Alice Nguyen"))[0];

        expect(screen.queryByTestId("active-filter-chips")).toBeNull();
    });

    it("shows badge count on filter icon matching selected count", async () => {
        const user = userEvent.setup();
 
        vi.mocked(api.fetchDepartments).mockResolvedValue(mockDepartments);
        renderPage();
        void (await screen.findAllByText("Alice Nguyen"))[0];

        // Select two departments
        await user.click(screen.getByTestId("dept-filter-btn"));
        await user.click(await screen.findByTestId("dept-filter-option-dept-1"));
        await user.click(await screen.findByTestId("dept-filter-option-dept-2"));

        // Badge should show "2"
        await waitFor(() => {
            const filterButton = screen.getByTestId("dept-filter-btn");
            const badge = filterButton.querySelector("span");
            expect(badge?.textContent).toBe("2");
        });
    });

    it("clears all filters when clicking 'Xóa tất cả' in chips area", async () => {
        const user = userEvent.setup();
 
        vi.mocked(api.fetchDepartments).mockResolvedValue(mockDepartments);
        renderPage();
        void (await screen.findAllByText("Alice Nguyen"))[0];

        // Select a department
        await user.click(screen.getByTestId("dept-filter-btn"));
        await user.click(await screen.findByTestId("dept-filter-option-dept-1"));

        // Wait for chips to appear
        await waitFor(() => {
            expect(screen.getByTestId("active-filter-chips")).toBeDefined();
        });

        // Click "Xóa tất cả" button
 
        vi.mocked(api.searchEmployees).mockClear();
        const clearAllButton = screen.getByText("Xóa tất cả");
        await user.click(clearAllButton);

        // Filter should be cleared
        await waitFor(() => {
            expect(api.searchEmployees).toHaveBeenCalledWith(
                expect.objectContaining({ departmentIds: undefined })
            );
        });
        // Chips should disappear
        expect(screen.queryByTestId("active-filter-chips")).toBeNull();
    });

    it("does not show 'Xóa bộ lọc' button in popover when no filter is active", async () => {
        const user = userEvent.setup();
 
        vi.mocked(api.fetchDepartments).mockResolvedValue(mockDepartments);
        renderPage();
        void (await screen.findAllByText("Alice Nguyen"))[0];

        // Open popover without any active filter
        await user.click(screen.getByTestId("dept-filter-btn"));
        await waitFor(() => {
            expect(screen.getAllByText("Engineering").length).toBeGreaterThan(0);
        });
        // "Xóa bộ lọc" should NOT be visible when no filter active
        expect(screen.queryByText("Xóa bộ lọc")).toBeNull();
    });

    it("shows empty state in popover when no departments exist", async () => {
        const user = userEvent.setup();
 
        vi.mocked(api.fetchDepartments).mockResolvedValue([]);
        renderPage();
        void (await screen.findAllByText("Alice Nguyen"))[0];

        await user.click(screen.getByTestId("dept-filter-btn"));

        await waitFor(() => {
            expect(screen.getByText("Chưa có phòng ban")).toBeDefined();
        });
    });

});
