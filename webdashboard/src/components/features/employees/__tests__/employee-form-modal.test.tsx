import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";

import type { Employee } from "@/lib/api";

import { ToastProvider } from "@/components/ui/toast";
import * as api from "@/lib/api";

import EmployeeFormModal from "../employee-form-modal";

vi.mock("react-i18next", () => ({
    useTranslation: () => ({
        t: (key: string, options?: { defaultValue?: string }) => {
            if (options && typeof options === "object" && options.defaultValue) return options.defaultValue;
            return key;
        },
        i18n: { changeLanguage: () => Promise.resolve(), language: "en" },
    }),
}));

const mockMembers = [
    {
        workspace_id: "ws-1",
        user_id: "user-abc",
        role_id: "role-1",
        email: "user@example.com",
        full_name: "Test User",
        avatar_url: undefined,
        extra_role_ids: [],
    },
    {
        workspace_id: "ws-1",
        user_id: "user-xyz",
        role_id: "role-2",
        email: "other@example.com",
        full_name: "Other User",
        avatar_url: undefined,
        extra_role_ids: [],
    },
];

vi.mock("@/lib/api", () => ({
    createEmployee: vi.fn(),
    updateEmployee: vi.fn(),
    fetchWorkspaceMembers: vi.fn(() => Promise.resolve(mockMembers)),
    fetchEmployers: vi.fn(() => Promise.resolve([])),
    fetchEmployeeEmployments: vi.fn(() => Promise.resolve([
        {
            id: "employment-1",
            employee_id: "emp-1",
            employer_id: "employer-1",
            primary_location_id: "location-1",
            start_date: "2024-06-15",
            status: "active"
        }
    ])),
    updateEmployeeEmployment: vi.fn(() => Promise.resolve({})),
}));


const mockEmployee: Employee = {
    id: "emp-1",
    full_name: "Alice Nguyen",
    employee_id_number: "EMP-001",
    workspace_id: "ws-1",
    user_id: "user-abc",
    is_active: true,
    original_hire_date: "2020-01-01", hire_date: "2024-06-15",
    created_at: "2024-06-15T00:00:00Z",
    updated_at: "2024-06-15T00:00:00Z",
};

describe("EmployeeFormModal", () => {
    let queryClient: QueryClient;

    const defaultProps = {
        isOpen: true,
        onClose: vi.fn(),
        editingEmployee: null,
        wsId: "ws-1",
        onSuccess: vi.fn(),
    };

    beforeEach(() => {
        vi.clearAllMocks();
        queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    });

    const renderWithClient = (ui: React.ReactElement) => {
        return render(
            <QueryClientProvider client={queryClient}>
                <ToastProvider>{ui}</ToastProvider>
            </QueryClientProvider>
        );
    };

    it("renders nothing when isOpen is false", () => {
        renderWithClient(
            <EmployeeFormModal {...defaultProps} isOpen={false} />
        );
        expect(screen.queryByTestId("employees-form-name-input")).toBeNull();
    });

    it("renders form fields in add mode", () => {
        renderWithClient(<EmployeeFormModal {...defaultProps} />);
        expect(screen.getByTestId("employees-form-name-input")).toBeDefined();
        expect(screen.getByTestId("employees-form-id-input")).toBeDefined();
        expect(screen.getByTestId("employees-form-hire-date-input")).toBeDefined();
        expect(screen.getByTestId("employees-form-active-checkbox")).toBeDefined();
    });

    it("renders form fields in edit mode with pre-filled values", async () => {
        renderWithClient(
            <EmployeeFormModal {...defaultProps} editingEmployee={mockEmployee} />
        );
        const nameInput = screen.getByTestId("employees-form-name-input");
        expect((nameInput as HTMLInputElement).value).toBe("Alice Nguyen");

        const idInput = screen.getByTestId("employees-form-id-input");
        expect((idInput as HTMLInputElement).value).toBe("EMP-001");

        const hireInput = screen.getByTestId("employees-form-hire-date-input");
        await waitFor(() => {
            expect((hireInput as HTMLInputElement).value).toBe("2024-06-15");
        });
    });

    it("shows user combobox with linked user in edit mode", async () => {
        renderWithClient(
            <EmployeeFormModal {...defaultProps} editingEmployee={mockEmployee} />
        );
        await waitFor(() => {
            expect(screen.getByText("Test User")).toBeDefined();
        });
    });

    it("calls createEmployee on submit in add mode", async () => {
        const user = userEvent.setup();
        vi.mocked(api.createEmployee).mockResolvedValue(mockEmployee);
        renderWithClient(<EmployeeFormModal {...defaultProps} />);

        await user.type(screen.getByTestId("employees-form-name-input"), "New Emp");
        await user.type(screen.getByTestId("employees-form-id-input"), "EMP-100");
        await user.type(screen.getByTestId("employees-form-hire-date-input"), "2025-01-01");
        await user.click(screen.getByTestId("save-employee-btn"));

        await waitFor(() => {
            expect(api.createEmployee).toHaveBeenCalledWith(
                "ws-1",
                expect.objectContaining({
                    full_name: "New Emp",
                    employee_id_number: "EMP-100",
                    hire_date: "2025-01-01",
                })
            );
        });
    });

    it("passes user_id to createEmployee when user is selected from combobox", async () => {
        const user = userEvent.setup();
        vi.mocked(api.createEmployee).mockResolvedValue(mockEmployee);
        renderWithClient(<EmployeeFormModal {...defaultProps} />);

        await user.type(screen.getByTestId("employees-form-name-input"), "New Emp");
        await user.type(screen.getByTestId("employees-form-id-input"), "EMP-100");
        await user.type(screen.getByTestId("employees-form-hire-date-input"), "2025-01-01");

        const select = document.querySelector('select[data-testid="employees-form-user-combobox"]');
        fireEvent.change(select!, { target: { value: "user-xyz" } });

        await user.click(screen.getByTestId("save-employee-btn"));

        await waitFor(() => {
            expect(api.createEmployee).toHaveBeenCalledWith(
                "ws-1",
                expect.objectContaining({
                    full_name: "New Emp",
                    user_id: "user-xyz",
                })
            );
        });
    });

    it("calls updateEmployee on submit in edit mode", async () => {
        const user = userEvent.setup();
        vi.mocked(api.updateEmployee).mockResolvedValue(mockEmployee);
        renderWithClient(
            <EmployeeFormModal {...defaultProps} editingEmployee={mockEmployee} />
        );

        await user.click(screen.getByTestId("save-employee-btn"));

        await waitFor(() => {
            expect(api.updateEmployee).toHaveBeenCalledWith(
                "ws-1",
                "emp-1",
                expect.objectContaining({
                    full_name: "Alice Nguyen",
                })
            );
        });
    });

    it("passes user_id to updateEmployee when editing and user is selected", async () => {
        const user = userEvent.setup();
        vi.mocked(api.updateEmployee).mockResolvedValue(mockEmployee);
        renderWithClient(
            <EmployeeFormModal {...defaultProps} editingEmployee={mockEmployee} />
        );

        await user.click(screen.getByTestId("save-employee-btn"));

        await waitFor(() => {
            expect(api.updateEmployee).toHaveBeenCalledWith(
                "ws-1",
                "emp-1",
                expect.objectContaining({
                    full_name: "Alice Nguyen",
                    user_id: "user-abc",
                })
            );
        });
    });

    it("calls onClose when cancel is clicked", async () => {
        const onClose = vi.fn();
        const user = userEvent.setup();
        renderWithClient(
            <EmployeeFormModal {...defaultProps} onClose={onClose} />
        );

        const cancelButton = screen.getByText(/employees.form.cancel/i);
        await user.click(cancelButton);
        expect(onClose).toHaveBeenCalledTimes(1);
    });

    it("calls onSuccess after successful mutation", async () => {
        const user = userEvent.setup();
        const onSuccess = vi.fn();
        vi.mocked(api.createEmployee).mockResolvedValue(mockEmployee);
        renderWithClient(
            <EmployeeFormModal {...defaultProps} onSuccess={onSuccess} />
        );

        await user.type(screen.getByTestId("employees-form-name-input"), "New Emp");
        await user.type(screen.getByTestId("employees-form-id-input"), "EMP-100");
        await user.type(screen.getByTestId("employees-form-hire-date-input"), "2025-01-01");
        await user.click(screen.getByTestId("save-employee-btn"));

        await waitFor(() => {
            expect(onSuccess).toHaveBeenCalledTimes(1);
        });
    });

    it("shows loading state on submit button while mutation is pending", async () => {
        const user = userEvent.setup();
        vi.mocked(api.createEmployee).mockImplementation(() => new Promise(() => { /* empty */ }));
        renderWithClient(<EmployeeFormModal {...defaultProps} />);

        await user.type(screen.getByTestId("employees-form-name-input"), "New Emp");
        await user.type(screen.getByTestId("employees-form-id-input"), "EMP-100");
        await user.type(screen.getByTestId("employees-form-hire-date-input"), "2025-01-01");
        await user.click(screen.getByTestId("save-employee-btn"));

        const saveButton = screen.getByTestId("save-employee-btn");
        expect(saveButton).toBeDisabled();
    });
});
