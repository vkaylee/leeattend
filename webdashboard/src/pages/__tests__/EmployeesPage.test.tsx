import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import EmployeesPage from "../EmployeesPage";
import { MemoryRouter } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ToastProvider } from "../../components/ui/Toast";
import * as api from "../../lib/api";
import React from "react";

// Mock IntersectionObserver for jsdom
class MockIntersectionObserver {
    observe() { }
    unobserve() { }
    disconnect() { }
}
window.IntersectionObserver = MockIntersectionObserver as any;

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
}));

// Override i18n
vi.mock("react-i18next", () => ({
    useTranslation: () => ({
        t: (key: string, fallback?: string) => fallback || key,
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

const mockEmployee = (id: string, name: string, active = true): api.Employee => ({
    id,
    full_name: name,
    employee_id_number: `EMP-${id}`,
    workspace_id: "test-ws-id",
    is_active: active,
    hire_date: "2025-01-01",
    created_at: "2025-01-01T00:00:00Z",
    updated_at: "2025-01-01T00:00:00Z",
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
        (api.getCurrentWorkspaceId as any).mockReturnValue("test-ws-id");
        (api.searchEmployees as any).mockResolvedValue({
            data: [mockEmployee("1", "Alice Nguyen"), mockEmployee("2", "Bob Tran")],
            total: 2,
            page: 1,
            limit: 20,
        });
        (api.fetchDepartments as any).mockResolvedValue([]);
        (api.fetchEmployeeDepartmentHistory as any).mockResolvedValue([]);
        (api.fetchEmploymentHistory as any).mockResolvedValue([]);
    });

    const renderPage = (initialEntries = ["/"]) =>
        render(
            <QueryClientProvider client={queryClient}>
                <ToastProvider>
                    <MemoryRouter initialEntries={initialEntries}>
                        <EmployeesPage />
                    </MemoryRouter>
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
        (api.searchEmployees as any).mockResolvedValue({
            data: [],
            total: 0,
            page: 1,
            limit: 20,
        });
        renderPage();
        expect(await screen.findByText(/employees.table.no_data/i)).toBeDefined();
    });

    // ────────────────────────────────────────────────────────────
    // Lazy load / infinite scroll
    // ────────────────────────────────────────────────────────────

    it("uses PAGE_SIZE of 20 for infinite scroll", async () => {
        renderPage();
        await waitFor(() => {
            const calls = (api.searchEmployees as any).mock.calls;
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
        (api.searchEmployees as any).mockResolvedValue({
            data: [mockEmployee("1", "Alice Nguyen")],
            total: 10,
            page: 1,
            limit: 20,
        });

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
                const calls = (api.searchEmployees as any).mock.calls;
                const searchCall = calls.find((c: any[]) => c[1] === "Bob");
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

        const addBtn = screen.getByTestId("add-employee-btn");
        await user.click(addBtn);

        // Form inputs should be visible
        expect(screen.getByTestId("employees-form-name-input")).toBeDefined();
        expect(screen.getByTestId("employees-form-id-input")).toBeDefined();
    });
});
