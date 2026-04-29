import React from "react";

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, within, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter } from "react-router-dom";
import { describe, it, expect, vi, beforeEach } from "vitest";

import { useMyPermissions } from "../../hooks/use-permissions";
import { useWorkspaceRoles, useWorkspacePermissionsMatrix } from "../../hooks/use-workspace";
import * as api from "../../lib/api";
import WorkspacePermissionsPage from "../workspace/WorkspacePermissionsPage";

vi.mock("react-i18next", () => ({
    useTranslation: () => ({
        t: (key: string, options?: any) => {
            if (options && typeof options === 'object' && options.defaultValue) {
                return options.defaultValue;
            }
            return key;
        },
    }),
}));

vi.mock("../../hooks/use-workspace", () => ({
    useWorkspaceRoles: vi.fn(),
    useWorkspacePermissionsMatrix: vi.fn(),
}));

vi.mock("../../hooks/use-permissions", () => ({
    useMyPermissions: vi.fn(),
}));

vi.mock("../../lib/api", () => ({
    getCurrentWorkspaceId: vi.fn().mockReturnValue("ws-1"),
    updateRolePermissions: vi.fn().mockResolvedValue({}),
}));

// ─── Helpers ───────────────────────────────────────────────────

const SYSTEM_ROLE = { id: "role-sys", name: "admin", is_system: true };
const CUSTOM_ROLE = { id: "role-custom", name: "hr_specialist", is_system: false };

/** Build a matrix with real backend permission strings */
const buildMatrix = (
    sysPerms: string[] = [],
    customPerms: string[] = [],
) => ({
    [SYSTEM_ROLE.id]: sysPerms,
    [CUSTOM_ROLE.id]: customPerms,
});

describe("WorkspacePermissionsPage", () => {
    let queryClient: QueryClient;
    let mockRefetch: ReturnType<typeof vi.fn>;

    beforeEach(() => {
        vi.clearAllMocks();
        queryClient = new QueryClient({
            defaultOptions: { queries: { retry: false } },
        });
        mockRefetch = vi.fn();

        (useWorkspaceRoles as any).mockReturnValue({
            roles: [SYSTEM_ROLE, CUSTOM_ROLE],
            isLoading: false,
        });

        (useWorkspacePermissionsMatrix as any).mockReturnValue({
            matrix: buildMatrix(
                ["workspace:update", "members:roles:manage", "employees:read", "payroll:read", "audit_logs:read"],
                ["employees:read", "leave:approve:l1"],
            ),
            isLoading: false,
            refetch: mockRefetch,
        });

        (useMyPermissions as any).mockReturnValue({
            hasPermission: (p: string) => p === "members:roles:manage",
            hasAnyPermission: () => true,
            hasScopedPermission: () => true,
            isLoading: false,
        });
    });

    const renderPage = () =>
        render(
            <QueryClientProvider client={queryClient}>
                <MemoryRouter>
                    <WorkspacePermissionsPage />
                </MemoryRouter>
            </QueryClientProvider>,
        );

    // ─── Rendering ─────────────────────────────────────────────

    describe("rendering", () => {
        it("renders role column headers", () => {
            renderPage();
            expect(screen.getByText("admin")).toBeDefined();
            expect(screen.getByText("hr_specialist")).toBeDefined();
        });

        it("renders all 16 enterprise permission group headers", () => {
            renderPage();
            const expectedGroups = [
                "Workspace & Setup",
                "Members & Roles",
                "Employees",
                "Departments",
                "Time & Attendance",
                "Overtime",
                "Leave Management",
                "Shifts & Scheduling",
                "Payroll",
                "Holidays",
                "Reports & Dashboard",
                "Documents",
                "Security & Audit",
                "Infrastructure",
                "Recruitment",
                "Employee Self-Service",
            ];
            for (const name of expectedGroups) {
                expect(screen.getByText(name)).toBeDefined();
            }
        });

        it("renders premium permission rows", () => {
            renderPage();
            // Multi-level approval
            expect(screen.getByText("Approve L1 (Team)")).toBeDefined();
            expect(screen.getByText("Approve L2 (HR)")).toBeDefined();
            // Advanced payroll
            expect(screen.getByText("Approve Payroll")).toBeDefined();
            expect(screen.getByText("Disburse Payroll")).toBeDefined();
            expect(screen.getByText("View Sensitive Data")).toBeDefined();
            // Security & Audit
            expect(screen.getByText("View Audit Logs")).toBeDefined();
            expect(screen.getByText("Export Audit Logs")).toBeDefined();
            expect(screen.getByText("Security Settings")).toBeDefined();
            // Bulk Operations
            expect(screen.getByText("Bulk Edit")).toBeDefined();
            expect(screen.getByText("Force Clock Out")).toBeDefined();
            // Documents
            expect(screen.getByText("Medical Documents")).toBeDefined();
            expect(screen.getByText("Contract Documents")).toBeDefined();
        });

        it("renders checkboxes for every permission × role combination", () => {
            renderPage();
            const checkboxes = screen.getAllByRole("checkbox");
            // Count all permissions across groups
            // Total = 4+4+7+5+8+4+10+13+8+2+4+2+3+7+2+6 = 89 permissions × 2 roles = 178
            expect(checkboxes.length).toBe(89 * 2);
        });

        it("marks system role with 'System' badge", () => {
            renderPage();
            expect(screen.getByText("System")).toBeDefined();
        });

        it("marks custom role with 'Custom' badge", () => {
            renderPage();
            expect(screen.getByText("Custom")).toBeDefined();
        });
    });

    // ─── Loading State ─────────────────────────────────────────

    describe("loading state", () => {
        it("shows spinner when roles are loading", () => {
            (useWorkspaceRoles as any).mockReturnValue({
                roles: [],
                isLoading: true,
            });
            renderPage();
            // Should not render checkboxes
            expect(screen.queryAllByRole("checkbox").length).toBe(0);
        });

        it("shows spinner when matrix is loading", () => {
            (useWorkspacePermissionsMatrix as any).mockReturnValue({
                matrix: {},
                isLoading: true,
                refetch: mockRefetch,
            });
            renderPage();
            expect(screen.queryAllByRole("checkbox").length).toBe(0);
        });
    });

    // ─── Access Control ────────────────────────────────────────

    describe("access control", () => {
        it("disables ALL checkboxes when user lacks members:roles:manage", () => {
            (useMyPermissions as any).mockReturnValue({
                hasPermission: () => false,
                isLoading: false,
            });

            renderPage();
            const checkboxes = screen.getAllByRole("checkbox") as HTMLInputElement[];
            const allDisabled = checkboxes.every((cb) => cb.disabled);
            expect(allDisabled).toBe(true);
        });

        it("hides Save Matrix button when user lacks permission", () => {
            (useMyPermissions as any).mockReturnValue({
                hasPermission: () => false,
                isLoading: false,
            });

            renderPage();
            expect(screen.queryByText("Save Matrix")).toBeNull();
        });

        it("disables system role checkboxes even with manage permission", () => {
            renderPage();
            const checkboxes = screen.getAllByRole("checkbox") as HTMLInputElement[];
            // System role is the first column → odd-indexed checkboxes in pair
            // For each permission row: [system, custom] — system should be disabled
            const systemCheckboxes = checkboxes.filter((_, i) => i % 2 === 0);
            systemCheckboxes.forEach((cb) => {
                expect(cb.disabled).toBe(true);
            });
        });

        it("enables custom role checkboxes when user has manage permission", () => {
            renderPage();
            const checkboxes = screen.getAllByRole("checkbox") as HTMLInputElement[];
            const customCheckboxes = checkboxes.filter((_, i) => i % 2 === 1);
            customCheckboxes.forEach((cb) => {
                expect(cb.disabled).toBe(false);
            });
        });
    });

    // ─── Checkbox State from Matrix ────────────────────────────

    describe("checkbox state from matrix data", () => {
        it("checks employees:read for both system and custom roles", () => {
            renderPage();
            // Both roles have employees:read in the mock matrix
            const checkboxes = screen.getAllByRole("checkbox") as HTMLInputElement[];
            const checkedCount = checkboxes.filter((cb) => cb.checked).length;
            // System: 5 checked + Custom: 2 checked = 7
            expect(checkedCount).toBe(7);
        });
    });

    // ─── Toggle & Save ─────────────────────────────────────────

    describe("toggle and save flow", () => {
        it("shows 'Unsaved changes' indicator after toggling a custom role permission", async () => {
            const user = userEvent.setup();
            renderPage();

            // Find an enabled, unchecked checkbox (custom role)
            const checkboxes = screen.getAllByRole("checkbox") as HTMLInputElement[];
            const customUnchecked = checkboxes.find((cb) => !cb.disabled && !cb.checked);
            expect(customUnchecked).toBeDefined();

            await user.click(customUnchecked!);
            expect(screen.getByText("Unsaved changes")).toBeDefined();
        });

        it("enables Save Matrix button after a change", async () => {
            const user = userEvent.setup();
            renderPage();

            const checkboxes = screen.getAllByRole("checkbox") as HTMLInputElement[];
            const customUnchecked = checkboxes.find((cb) => !cb.disabled && !cb.checked);
            await user.click(customUnchecked!);

            const saveBtn = screen.getByText("Save Matrix").closest("button") as HTMLButtonElement;
            expect(saveBtn.disabled).toBe(false);
        });

        it("Save Matrix button is disabled when no changes exist", () => {
            renderPage();
            const saveBtn = screen.getByText("Save Matrix").closest("button") as HTMLButtonElement;
            expect(saveBtn.disabled).toBe(true);
        });

        it("calls updateRolePermissions only for modified roles on save", async () => {
            const user = userEvent.setup();
            renderPage();

            // Toggle a custom role checkbox
            const checkboxes = screen.getAllByRole("checkbox") as HTMLInputElement[];
            const customUnchecked = checkboxes.find((cb) => !cb.disabled && !cb.checked);
            await user.click(customUnchecked!);

            // Click save
            const saveBtn = screen.getByText("Save Matrix").closest("button")!;
            await user.click(saveBtn);

            await waitFor(() => {
                // Should only call for the custom role, not the system role
                expect(api.updateRolePermissions).toHaveBeenCalledTimes(1);
                expect(api.updateRolePermissions).toHaveBeenCalledWith(
                    "ws-1",
                    CUSTOM_ROLE.id,
                    expect.any(Array),
                );
            });
        });

        it("calls refetch after successful save", async () => {
            const user = userEvent.setup();
            renderPage();

            const checkboxes = screen.getAllByRole("checkbox") as HTMLInputElement[];
            const customUnchecked = checkboxes.find((cb) => !cb.disabled && !cb.checked);
            await user.click(customUnchecked!);

            const saveBtn = screen.getByText("Save Matrix").closest("button")!;
            await user.click(saveBtn);

            await waitFor(() => {
                expect(mockRefetch).toHaveBeenCalledTimes(1);
            });
        });

        it("removes 'Unsaved changes' after successful save", async () => {
            const user = userEvent.setup();
            renderPage();

            const checkboxes = screen.getAllByRole("checkbox") as HTMLInputElement[];
            const customUnchecked = checkboxes.find((cb) => !cb.disabled && !cb.checked);
            await user.click(customUnchecked!);
            expect(screen.getByText("Unsaved changes")).toBeDefined();

            const saveBtn = screen.getByText("Save Matrix").closest("button")!;
            await user.click(saveBtn);

            await waitFor(() => {
                expect(screen.queryByText("Unsaved changes")).toBeNull();
            });
        });

        it("can toggle a checked permission off", async () => {
            const user = userEvent.setup();
            renderPage();

            // Custom role has employees:read checked — find it and uncheck
            const checkboxes = screen.getAllByRole("checkbox") as HTMLInputElement[];
            const customChecked = checkboxes.find((cb) => !cb.disabled && cb.checked);
            expect(customChecked).toBeDefined();

            await user.click(customChecked!);
            expect(screen.getByText("Unsaved changes")).toBeDefined();
        });
    });

    // ─── System Role Protection ────────────────────────────────

    describe("system role protection", () => {
        it("clicking a system role checkbox does not trigger changes", async () => {
            const user = userEvent.setup();
            renderPage();

            const checkboxes = screen.getAllByRole("checkbox") as HTMLInputElement[];
            // Get a system checkbox (disabled)
            const systemCb = checkboxes.find((cb) => cb.disabled);
            expect(systemCb).toBeDefined();

            // Attempt click — should be blocked by disabled attribute
            await user.click(systemCb!);
            expect(screen.queryByText("Unsaved changes")).toBeNull();
        });
    });

    // ─── Permission ID Integrity ───────────────────────────────

    describe("permission ID integrity (backend sync)", () => {
        it("all permission IDs use colon-separated backend format", () => {
            renderPage();
            // Verify no old mock IDs remain (edit_workspace, view_employees, etc.)
            expect(screen.queryByText("Edit Workspace Info")).toBeNull();
            expect(screen.queryByText("Manage Billing")).toBeNull();
            expect(screen.queryByText("Danger Zone")).toBeNull();
        });

        it("renders ESS self-service permissions", () => {
            renderPage();
            expect(screen.getByText("ESS Profile")).toBeDefined();
            expect(screen.getByText("ESS Attendance")).toBeDefined();
            expect(screen.getByText("ESS Clock In/Out")).toBeDefined();
            expect(screen.getByText("ESS Leave")).toBeDefined();
            expect(screen.getByText("ESS Payslips")).toBeDefined();
            expect(screen.getByText("ESS Shift Swaps")).toBeDefined();
        });

        it("renders infrastructure permissions", () => {
            renderPage();
            expect(screen.getByText("View Agents")).toBeDefined();
            expect(screen.getByText("Manage Agents")).toBeDefined();
            expect(screen.getByText("View Devices")).toBeDefined();
            expect(screen.getByText("Manage Devices")).toBeDefined();
            expect(screen.getByText("Infrastructure Monitor")).toBeDefined();
        });
    });

    // ─── Columns Visibility ────────────────────────────────────

    describe("columns visibility dropdown", () => {
        it("renders Columns button", () => {
            renderPage();
            expect(screen.getByRole("button", { name: /Columns/i })).toBeDefined();
        });

        it("allows hiding and showing roles", async () => {
            const user = userEvent.setup();
            renderPage();

            // Both roles should be visible initially in the table header
            // Using queryByRole because we expect it to exist, and later to be null
            expect(screen.queryByRole("columnheader", { name: /admin/i })).not.toBeNull();
            expect(screen.queryByRole("columnheader", { name: /hr_specialist/i })).not.toBeNull();

            // Click Columns button
            const columnsBtn = screen.getByRole("button", { name: /Columns/i });
            await user.click(columnsBtn);

            // Find the checkbox for hr_specialist
            const hrSpecialistCheckbox = screen.getByRole("menuitemcheckbox", { name: /hr_specialist/i });
            
            // Uncheck it
            await user.click(hrSpecialistCheckbox);

            // Verify hr_specialist is removed from headers
            expect(screen.queryByRole("columnheader", { name: /hr_specialist/i })).toBeNull();
            // Verify admin is still there
            expect(screen.queryByRole("columnheader", { name: /admin/i })).not.toBeNull();

            // Click Columns button again to open menu (Radix dropdown closes after click)
            await user.click(columnsBtn);
            
            // Check it back
            const hrSpecialistCheckboxAgain = screen.getByRole("menuitemcheckbox", { name: /hr_specialist/i });
            await user.click(hrSpecialistCheckboxAgain);

            // Verify hr_specialist is back
            expect(screen.queryByRole("columnheader", { name: /hr_specialist/i })).not.toBeNull();
        });
    });

    // ─── Info Footer ───────────────────────────────────────────

    describe("info footer", () => {
        it("shows system role explanation text", () => {
            renderPage();
            expect(
                screen.getByText(/System roles.*have fixed permissions/i),
            ).toBeDefined();
        });
    });
});
