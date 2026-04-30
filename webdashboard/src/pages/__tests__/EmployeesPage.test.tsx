import React from "react";

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";

import { ToastProvider } from "../../components/ui/Toast";
import * as api from "../../lib/api";
import { Employee, PaginatedResponse, Department, EmploymentPeriod, EmployeeDepartmentHistory } from "../../lib/api/types";
import EmployeesPage from "../EmployeesPage";


// Mock TanStack Router to avoid needing full route tree in unit tests
vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => vi.fn(),
 
  useLocation: () => ({ pathname: "/", search: "", hash: "" }),
  Link: ({ to, children, ...props }: { to: string; children: React.ReactNode; [key: string]: unknown }) => <a href={to} {...props}>{children}</a>,
}));

// Mock IntersectionObserver for jsdom
class MockIntersectionObserver {
    observe() { }
    unobserve() { }
    disconnect() { }
 
}
window.IntersectionObserver = MockIntersectionObserver as unknown as typeof IntersectionObserver;

// Mock the API library
vi.mock("../../lib/api", () => ({
    fetchEmployees: vi.fn(),
    searchEmployees: vi.fn(),
    createEmployee: vi.fn(),
    updateEmployee: vi.fn(),
    deleteEmployee: vi.fn(),
    getCurrentWorkspaceId: vi.fn(),
    fetchDepartments: vi.fn(),
    fetchEmployeeDepartmentHistory: vi.fn(),
    transferEmployee: vi.fn(),
    fetchEmploymentHistory: vi.fn(),
    rehireEmployee: vi.fn(),
    fetchMyPermissions: vi.fn(),
    fetchWorkspaceMembers: vi.fn().mockResolvedValue([]),
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
vi.mock("@/components/payroll/EmployeeSalaryConfig", () => ({
    default: () => <div data-testid="salary-config">SalaryConfig</div>,
}));
vi.mock("@/components/employees/WorkingConditionConfig", () => ({
    default: () => <div data-testid="working-condition">WorkingCondition</div>,
}));

const mockEmployee = (id: string, name: string, active = true, department?: string): Employee => ({
    id,
    full_name: name,
    employee_id_number: `EMP-${id}`,
    workspace_id: "test-ws-id",
    is_active: active,
    hire_date: "2025-01-01",
    created_at: "2025-01-01T00:00:00Z",
    updated_at: "2025-01-01T00:00:00Z",
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
 
        vi.mocked(api.getCurrentWorkspaceId).mockReturnValue("test-ws-id");
 
        vi.mocked(api.searchEmployees).mockResolvedValue({
            data: [mockEmployee("1", "Alice Nguyen"), mockEmployee("2", "Bob Tran")],
            total: 2,
            page: 1,
            limit: 20,
        } as unknown as PaginatedResponse<Employee>);
 
        vi.mocked(api.fetchDepartments).mockResolvedValue([] as unknown as Department[]);
 
        vi.mocked(api.fetchEmployeeDepartmentHistory).mockResolvedValue([] as unknown as EmployeeDepartmentHistory[]);
 
        vi.mocked(api.fetchEmploymentHistory).mockResolvedValue([] as unknown as EmploymentPeriod[]);
 
        vi.mocked(api.fetchMyPermissions).mockResolvedValue({ permissions: ["employees:create", "employees:update", "employees:delete"], roles: ["admin"], managed_department_ids: [] } as unknown as Awaited<ReturnType<typeof api.fetchMyPermissions>>);
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
        expect(await screen.findByText("Alice Nguyen")).toBeDefined();
        expect(screen.getByText("Bob Tran")).toBeDefined();
    });

    it("shows empty state when no employees exist", async () => {
 
        vi.mocked(api.searchEmployees).mockResolvedValue({
            data: [],
            total: 0,
            page: 1,
            limit: 20,
        } as unknown as PaginatedResponse<Employee>);
        renderPage();
        expect(await screen.findByText(/employees.table.no_data/i)).toBeDefined();
    });

    // ────────────────────────────────────────────────────────────
    // Department column
    // ────────────────────────────────────────────────────────────

    it("renders department column header", async () => {
        renderPage();
        await screen.findByText("Alice Nguyen");
        expect(screen.getByText("common.department")).toBeDefined();
    });

    it("displays department name when employee has a department", async () => {
 
        vi.mocked(api.searchEmployees).mockResolvedValue({
            data: [mockEmployee("1", "Alice Nguyen", true, "Engineering"), mockEmployee("2", "Bob Tran", true, "Sales")],
            total: 2,
            page: 1,
            limit: 20,
        } as unknown as PaginatedResponse<Employee>);
        renderPage();
        expect(await screen.findByText("Engineering")).toBeDefined();
        expect(screen.getByText("Sales")).toBeDefined();
    });

    it("displays dash when employee has no department", async () => {
 
        vi.mocked(api.searchEmployees).mockResolvedValue({
            data: [mockEmployee("1", "Alice Nguyen")],
            total: 1,
            page: 1,
            limit: 20,
        } as unknown as PaginatedResponse<Employee>);
        renderPage();
        await screen.findByText("Alice Nguyen");
        expect(screen.getByText("—")).toBeDefined();
    });

    it("handles mixed employees with and without departments", async () => {
 
        vi.mocked(api.searchEmployees).mockResolvedValue({
            data: [mockEmployee("1", "Alice Nguyen", true, "HR"), mockEmployee("2", "Bob Tran")],
            total: 2,
            page: 1,
            limit: 20,
        } as unknown as PaginatedResponse<Employee>);
        renderPage();
        expect(await screen.findByText("HR")).toBeDefined();
        expect(screen.getByText("—")).toBeDefined();
    });

    // ────────────────────────────────────────────────────────────
    // Lazy load / infinite scroll
    // ────────────────────────────────────────────────────────────

    it("uses PAGE_SIZE of 20 for infinite scroll", async () => {
        renderPage();
        await waitFor(() => {
 
            const calls = vi.mocked(api.searchEmployees).mock.calls;
            expect(calls.length).toBeGreaterThan(0);
            // args: wsId, query, page, limit
            expect(calls[0][2]).toBe(1);  // page = 1
            expect(calls[0][3]).toBe(20); // limit = 20
        });
    });

    it("does not render pagination controls", async () => {
        renderPage();
        await screen.findByText("Alice Nguyen");

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
        } as unknown as PaginatedResponse<Employee>);

        renderPage();
        await screen.findByText("Alice Nguyen");

        const badge = document.querySelector(".fixed.bottom-6");
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
        await screen.findByText("Alice Nguyen");

        const searchInput = screen.getByTestId("employee-search");
        await user.type(searchInput, "Bob");

        await waitFor(
            () => {
 
                const calls = vi.mocked(api.searchEmployees).mock.calls;
 
                const searchCall = calls.find((c) => c[1] === "Bob");
                expect(searchCall).toBeDefined();
            },
            { timeout: 3000 }
        );
    });

    // ────────────────────────────────────────────────────────────
    // Detail panel
    // ────────────────────────────────────────────────────────────

    it("opens detail panel when clicking an employee row", async () => {
        const user = userEvent.setup();
        renderPage();
        const row = await screen.findByText("Alice Nguyen");
        await user.click(row);

        // Detail header should appear with employee name (large heading)
        const headings = await screen.findAllByText("Alice Nguyen");
        // At least 2: one in the list, one in the detail panel
        expect(headings.length).toBeGreaterThanOrEqual(2);
    });

    // ────────────────────────────────────────────────────────────
    // Employee CRUD
    // ────────────────────────────────────────────────────────────

    it("opens add-employee modal when clicking the add button", async () => {
        const user = userEvent.setup();
        renderPage();
        await screen.findByText("Alice Nguyen");

        const addBtn = await screen.findByTestId("add-employee-btn");
        await user.click(addBtn);

        // Form inputs should be visible
        expect(screen.getByTestId("employees-form-name-input")).toBeDefined();
        expect(screen.getByTestId("employees-form-id-input")).toBeDefined();
    });

    it("handles terminating an employee with useConfirm", async () => {
        const user = userEvent.setup();
        renderPage();
        
        // click row to open drawer
        const row = await screen.findByText("Alice Nguyen");
        await user.click(row);

        // find terminate button in drawer
        const terminateBtn = await screen.findByTestId("employees-btn-terminate");
        await user.click(terminateBtn);

        // confirm modal appears, we can use fireEvent to bypass pointer-events: none
        const confirmModalBtn = await screen.findByRole("button", { name: "Confirm" });
        fireEvent.click(confirmModalBtn);

        await waitFor(() => {
            expect(api.updateEmployee).toHaveBeenCalledWith(
                "test-ws-id", 
                "1", 
                expect.objectContaining({ is_active: false })
            );
        });
    });

    // ────────────────────────────────────────────────────────────
    // Column sorting
    // ────────────────────────────────────────────────────────────

    const findHeaderByText = (text: string) => {
        const ths = document.querySelectorAll("th");
        for (const th of ths) {
            if (th.textContent?.includes(text)) return th;
        }
        throw new Error(`Header containing "${text}" not found`);
    };

    it("sends sort_by=name and sort_dir=asc when clicking Name header", async () => {
        const user = userEvent.setup();
        renderPage();
        await screen.findByText("Alice Nguyen");

 
        vi.mocked(api.searchEmployees).mockClear();
        const nameHeader = findHeaderByText("employees.table.name");
        await user.click(nameHeader);

        await waitFor(() => {
 
            const calls = vi.mocked(api.searchEmployees).mock.calls;
 
            const sortCall = calls.find((c) => c[4] === "name");
            expect(sortCall).toBeDefined();
            expect(sortCall![5]).toBe("asc");
        });
    });

    it("toggles sort_dir to desc when clicking the same header twice", async () => {
        const user = userEvent.setup();
        renderPage();
        await screen.findByText("Alice Nguyen");

        await user.click(findHeaderByText("employees.table.name"));

        await waitFor(() => {
 
            const calls = vi.mocked(api.searchEmployees).mock.calls;
 
            expect(calls.some((c) => c[4] === "name" && c[5] === "asc")).toBe(true);
        }, { timeout: 3000 });

 
        vi.mocked(api.searchEmployees).mockClear();
        // Re-query header since DOM is re-rendered after first click
        await user.click(findHeaderByText("employees.table.name"));

        await waitFor(() => {
 
            const calls = vi.mocked(api.searchEmployees).mock.calls;
 
            const descCall = calls.find((c) => c[4] === "name" && c[5] === "desc");
            expect(descCall).toBeDefined();
        }, { timeout: 3000 });
    });

    it("sends sort_by=status when clicking Status header", async () => {
        const user = userEvent.setup();
        renderPage();
        await screen.findByText("Alice Nguyen");

 
        vi.mocked(api.searchEmployees).mockClear();
        const statusHeader = findHeaderByText("employees.table.status");
        await user.click(statusHeader);

        await waitFor(() => {
 
            const calls = vi.mocked(api.searchEmployees).mock.calls;
 
            const sortCall = calls.find((c) => c[4] === "status");
            expect(sortCall).toBeDefined();
            expect(sortCall![5]).toBe("asc");
        });
    });


    it("sends sort_by=department when clicking Department header", async () => {
        const user = userEvent.setup();
        renderPage();
        await screen.findByText("Alice Nguyen");

 
        vi.mocked(api.searchEmployees).mockClear();
        const deptHeader = findHeaderByText("common.department");
        await user.click(deptHeader);

        await waitFor(() => {
 
            const calls = vi.mocked(api.searchEmployees).mock.calls;
 
            const sortCall = calls.find((c) => c[4] === "department");
            expect(sortCall).toBeDefined();
            expect(sortCall![5]).toBe("asc");
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
 
        vi.mocked(api.fetchDepartments).mockResolvedValue(mockDepartments as unknown as Department[]);
        renderPage();
        await screen.findByText("Alice Nguyen");

        const filterBtn = screen.getByTestId("dept-filter-btn");
        expect(filterBtn).toBeDefined();
    });

    it("opens filter popover and shows department checkboxes", async () => {
        const user = userEvent.setup();
 
        vi.mocked(api.fetchDepartments).mockResolvedValue(mockDepartments as unknown as Department[]);
        renderPage();
        await screen.findByText("Alice Nguyen");

        const filterBtn = screen.getByTestId("dept-filter-btn");
        await user.click(filterBtn);

        // All departments should be visible as options
        await waitFor(() => {
            expect(screen.getByText("Engineering")).toBeDefined();
            expect(screen.getByText("Sales")).toBeDefined();
            expect(screen.getByText("HR")).toBeDefined();
        });
    });

    it("sends department_id filter when selecting a department", async () => {
        const user = userEvent.setup();
 
        vi.mocked(api.fetchDepartments).mockResolvedValue(mockDepartments as unknown as Department[]);
        renderPage();
        await screen.findByText("Alice Nguyen");

        // Open filter popover
        const filterBtn = screen.getByTestId("dept-filter-btn");
        await user.click(filterBtn);

        // Click "Engineering" checkbox
 
        vi.mocked(api.searchEmployees).mockClear();
        const engOption = await screen.findByTestId("dept-filter-option-dept-1");
        await user.click(engOption);

        await waitFor(() => {
 
            const calls = vi.mocked(api.searchEmployees).mock.calls;
 
            const filterCall = calls.find((c) => Array.isArray(c[6]) && c[6].includes("dept-1"));
            expect(filterCall).toBeDefined();
        });
    });

    it("sends multiple department_ids when selecting multiple departments", async () => {
        const user = userEvent.setup();
 
        vi.mocked(api.fetchDepartments).mockResolvedValue(mockDepartments as unknown as Department[]);
        renderPage();
        await screen.findByText("Alice Nguyen");

        // Open filter popover
        await user.click(screen.getByTestId("dept-filter-btn"));

        // Select Engineering and Sales in one popover session
 
        vi.mocked(api.searchEmployees).mockClear();
        await user.click(await screen.findByTestId("dept-filter-option-dept-1"));
        await user.click(await screen.findByTestId("dept-filter-option-dept-2"));

        await waitFor(() => {
 
            const calls = vi.mocked(api.searchEmployees).mock.calls;
 
            const multiCall = calls.find((c) =>
                Array.isArray(c[6]) && c[6].includes("dept-1") && c[6].includes("dept-2")
            );
            expect(multiCall).toBeDefined();
        });
    });

    it("clears filter when clicking the chip X button", async () => {
        const user = userEvent.setup();
 
        vi.mocked(api.fetchDepartments).mockResolvedValue(mockDepartments as unknown as Department[]);
        renderPage();
        await screen.findByText("Alice Nguyen");

        // Select one department via popover
        await user.click(screen.getByTestId("dept-filter-btn"));
        await user.click(await screen.findByTestId("dept-filter-option-dept-1"));

        // Wait for filter chip to appear
        await waitFor(() => {
            expect(screen.getByTestId("active-filter-chips")).toBeDefined();
        });

        // Click the filter chip to remove it
 
        vi.mocked(api.searchEmployees).mockClear();
        await user.click(screen.getByTestId("dept-chip-dept-1"));

        await waitFor(() => {
 
            const calls = vi.mocked(api.searchEmployees).mock.calls;
            // Should send undefined (no filter) when empty
 
            const clearCall = calls.find((c) => c[6] === undefined);
            expect(clearCall).toBeDefined();
        });
    });

    // ────────────────────────────────────────────────────────────
    // UX improvements
    // ────────────────────────────────────────────────────────────

    it("shows filter chips with department names when filter is active", async () => {
        const user = userEvent.setup();
 
        vi.mocked(api.fetchDepartments).mockResolvedValue(mockDepartments as unknown as Department[]);
        renderPage();
        await screen.findByText("Alice Nguyen");

        // Select two departments
        await user.click(screen.getByTestId("dept-filter-btn"));
        await user.click(await screen.findByTestId("dept-filter-option-dept-1"));
        await user.click(await screen.findByTestId("dept-filter-option-dept-3"));

        // Chips area should appear with both department names
        await waitFor(() => {
            const chipsArea = screen.getByTestId("active-filter-chips");
            expect(chipsArea).toBeDefined();
            expect(screen.getByTestId("dept-chip-dept-1")).toBeDefined();
            expect(screen.getByTestId("dept-chip-dept-3")).toBeDefined();
        });
    });

    it("does not show filter chips when no filter is active", async () => {
 
        vi.mocked(api.fetchDepartments).mockResolvedValue(mockDepartments as unknown as Department[]);
        renderPage();
        await screen.findByText("Alice Nguyen");

        expect(screen.queryByTestId("active-filter-chips")).toBeNull();
    });

    it("shows badge count on filter icon matching selected count", async () => {
        const user = userEvent.setup();
 
        vi.mocked(api.fetchDepartments).mockResolvedValue(mockDepartments as unknown as Department[]);
        renderPage();
        await screen.findByText("Alice Nguyen");

        // Select two departments
        await user.click(screen.getByTestId("dept-filter-btn"));
        await user.click(await screen.findByTestId("dept-filter-option-dept-1"));
        await user.click(await screen.findByTestId("dept-filter-option-dept-2"));

        // Badge should show "2"
        await waitFor(() => {
            const filterBtn = screen.getByTestId("dept-filter-btn");
            const badge = filterBtn.querySelector("span");
            expect(badge?.textContent).toBe("2");
        });
    });

    it("clears all filters when clicking 'Xóa tất cả' in chips area", async () => {
        const user = userEvent.setup();
 
        vi.mocked(api.fetchDepartments).mockResolvedValue(mockDepartments as unknown as Department[]);
        renderPage();
        await screen.findByText("Alice Nguyen");

        // Select a department
        await user.click(screen.getByTestId("dept-filter-btn"));
        await user.click(await screen.findByTestId("dept-filter-option-dept-1"));

        // Wait for chips to appear
        await waitFor(() => {
            expect(screen.getByTestId("active-filter-chips")).toBeDefined();
        });

        // Click "Xóa tất cả" button
 
        vi.mocked(api.searchEmployees).mockClear();
        const clearAllBtn = screen.getByText("Xóa tất cả");
        await user.click(clearAllBtn);

        // Filter should be cleared
        await waitFor(() => {
 
            const calls = vi.mocked(api.searchEmployees).mock.calls;
 
            expect(calls.some((c) => c[6] === undefined)).toBe(true);
        });
        // Chips should disappear
        expect(screen.queryByTestId("active-filter-chips")).toBeNull();
    });

    it("does not show 'Xóa bộ lọc' button in popover when no filter is active", async () => {
        const user = userEvent.setup();
 
        vi.mocked(api.fetchDepartments).mockResolvedValue(mockDepartments as unknown as Department[]);
        renderPage();
        await screen.findByText("Alice Nguyen");

        // Open popover without any active filter
        await user.click(screen.getByTestId("dept-filter-btn"));
        await waitFor(() => {
            expect(screen.getByText("Engineering")).toBeDefined();
        });
        // "Xóa bộ lọc" should NOT be visible when no filter active
        expect(screen.queryByText("Xóa bộ lọc")).toBeNull();
    });

    it("shows empty state in popover when no departments exist", async () => {
        const user = userEvent.setup();
 
        vi.mocked(api.fetchDepartments).mockResolvedValue([] as unknown as Department[]);
        renderPage();
        await screen.findByText("Alice Nguyen");

        await user.click(screen.getByTestId("dept-filter-btn"));

        await waitFor(() => {
            expect(screen.getByText("Chưa có phòng ban")).toBeDefined();
        });
    });

});
