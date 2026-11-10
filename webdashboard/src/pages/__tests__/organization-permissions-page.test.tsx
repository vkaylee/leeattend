import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor, within } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";

import { useMyPermissions } from "../../hooks/use-permissions";
import { usePermissionCatalog, useOrganizationRoles, useOrganizationPermissionsMatrix } from "@/hooks/use-organization";
import * as api from "../../lib/api";
import { OrganizationRole } from "../../lib/api/types";
import OrganizationPermissionsPage from "../organization/organization-permissions-page";
vi.mock("react-i18next", () => ({
 
    useTranslation: () => ({
        t: (key: string, options?: { defaultValue?: string }) => {
            if (options && typeof options === 'object' && options.defaultValue) {
                return options.defaultValue;
            }
            return key;
        },
    }),
}));

vi.mock("@/hooks/use-organization", () => ({
    usePermissionCatalog: vi.fn(),
    useOrganizationRoles: vi.fn(),
    useOrganizationPermissionsMatrix: vi.fn(),
}));

vi.mock("../../hooks/use-permissions", () => ({
    useMyPermissions: vi.fn(),
}));

vi.mock("../../lib/api", () => ({
    updateRolePermissions: vi.fn().mockResolvedValue({}),
}));

// ─── Helpers ───────────────────────────────────────────────────

const SYSTEM_ROLE = { id: "role-sys", name: "admin", is_system: true };
const CUSTOM_ROLE = { id: "role-custom", name: "hr_specialist", is_system: false };

const CATALOG_GROUPS = [
    "attendance", "departments", "documents", "employees", "employment_phases", "ess",
    "holidays", "infra", "leave", "members", "overtime", "payroll", "recruitment",
    "reports", "security", "shifts", "organization",
];

const REQUIRED_CATALOG_IDS = [
    "overtime:request:create_on_behalf",
    "organization:settings:read",
    "members:roles:manage",
    "employees:read",
    "payroll:read",
    "audit_logs:read",
    "leave:approve:l1",
];

const PERMISSION_CATALOG = [
    ...REQUIRED_CATALOG_IDS.map((id, index) => ({
        id,
        group: id.startsWith("overtime") ? "overtime" : CATALOG_GROUPS[index],
        label_key: `permissions.items.${id}.name`,
    })),
    ...Array.from({ length: 96 }, (_, index) => ({
        id: `fixture:permission:${index}`,
        group: CATALOG_GROUPS[index % CATALOG_GROUPS.length],
        label_key: `permissions.items.fixture:permission:${index}.name`,
    })),
];

/** Build a matrix with real backend permission strings */
const buildMatrix = (
    sysPerms: string[] = [],
    customPerms: string[] = [],
) => ({
    [SYSTEM_ROLE.id]: sysPerms,
    [CUSTOM_ROLE.id]: customPerms,
});

describe("OrganizationPermissionsPage", () => {
    let queryClient: QueryClient;
    let mockRefetch: ReturnType<typeof vi.fn>;

    beforeEach(() => {
        vi.clearAllMocks();
        queryClient = new QueryClient({
            defaultOptions: { queries: { retry: false } },
        });
        mockRefetch = vi.fn();

        vi.mocked(usePermissionCatalog).mockReturnValue({
            catalog: PERMISSION_CATALOG,
            isError: false,
            isLoading: false,
        });

        vi.mocked(useOrganizationRoles).mockReturnValue({
            roles: [SYSTEM_ROLE, CUSTOM_ROLE] as unknown as WorkspaceRole[],
            isLoading: false,
            createRole: vi.fn() as unknown as (_arguments: unknown) => Promise<unknown>,
 
            isCreating: false,
            updateRole: vi.fn() as unknown as (_arguments: unknown) => Promise<unknown>,
            isUpdating: false,
            deleteRole: vi.fn() as unknown as (_arguments: unknown) => Promise<unknown>,
            isDeleting: false,
        } as unknown as ReturnType<typeof useOrganizationRoles>);

        vi.mocked(useOrganizationPermissionsMatrix).mockReturnValue({
            matrix: buildMatrix(
 
                ["organization:settings:read", "members:roles:manage", "employees:read", "payroll:read", "audit_logs:read"],
                ["employees:read", "leave:approve:l1"],
            ),
            isLoading: false,
            refetch: mockRefetch as unknown as () => Promise<unknown>,
        } as unknown as ReturnType<typeof useOrganizationPermissionsMatrix>);

        vi.mocked(useMyPermissions).mockReturnValue({
            hasPermission: (p: string) => p === "members:roles:manage",
            hasAnyPermission: () => true,
            hasScopedPermission: () => true,
            hasRole: () => true,
            isLoading: false,
            permissions: ["members:roles:manage"],
            roles: ["admin"],
            managedDepartmentIds: null
        });
    });

    const renderPage = () =>
        render(
            <QueryClientProvider client={queryClient}>
                <OrganizationPermissionsPage />
            </QueryClientProvider>,
        );

    // ─── Rendering ─────────────────────────────────────────────

    describe("rendering", () => {
        it("renders role column headers", () => {
            renderPage();
            const desktop = within(screen.getByTestId("permissions-desktop"));
            expect(desktop.getByText("admin")).toBeDefined();
            expect(desktop.getByText("hr_specialist")).toBeDefined();
        });

        it("renders all catalog-provided permission group headers", () => {
            renderPage();
            const desktop = within(screen.getByTestId("permissions-desktop"));
            for (const group of CATALOG_GROUPS) {
                expect(desktop.getByText(group)).toBeDefined();
            }
        });

        it("renders the production-blocking pre-OT-on-behalf permission", () => {
            renderPage();
            expect(within(screen.getByTestId("permissions-desktop")).getAllByText("overtime:request:create_on_behalf")).toHaveLength(2);
        });

        it("renders checkboxes for every permission × role combination", () => {
            renderPage();
            const checkboxes = within(screen.getByTestId("permissions-desktop")).getAllByRole("checkbox").filter(c => c.tagName === "INPUT");
            // Total = 103 catalog permissions × 2 roles = 206
            expect(checkboxes.length).toBe(103 * 2);
        });

        it("marks system role with 'System' badge", () => {
            renderPage();
            expect(within(screen.getByTestId("permissions-desktop")).getByText("System")).toBeDefined();
        });


        it("marks custom role with 'Custom' badge", () => {
            renderPage();
            expect(within(screen.getByTestId("permissions-desktop")).getByText("Custom")).toBeDefined();
        });
    });

    // ─── Loading State ─────────────────────────────────────────

    describe("loading state", () => {
 
        it("shows spinner when roles are loading", () => {
        vi.mocked(useOrganizationRoles).mockReturnValue({
                roles: [],
                isLoading: true,
                createRole: vi.fn() as unknown as (_arguments: unknown) => Promise<unknown>,
                isCreating: false,
                updateRole: vi.fn() as unknown as (_arguments: unknown) => Promise<unknown>,
                isUpdating: false,
                deleteRole: vi.fn() as unknown as (_arguments: unknown) => Promise<unknown>,
                isDeleting: false,
            } as unknown as ReturnType<typeof useOrganizationRoles>);
            renderPage();
            // Should not render checkboxes
            expect(screen.queryAllByRole("checkbox").length).toBe(0);
 
        });

        it("shows spinner when matrix is loading", () => {
            vi.mocked(useOrganizationPermissionsMatrix).mockReturnValue({
                matrix: {},
                isLoading: true,
                refetch: mockRefetch as unknown as () => Promise<unknown>,
            } as unknown as ReturnType<typeof useOrganizationPermissionsMatrix>);
            renderPage();
            expect(screen.queryAllByRole("checkbox").length).toBe(0);
        });
    });
 

    // ─── Access Control ────────────────────────────────────────

    describe("access control", () => {
        it("disables ALL checkboxes when user lacks members:roles:manage", () => {
            vi.mocked(useMyPermissions).mockReturnValue({
                hasPermission: () => false,
                hasAnyPermission: () => false,
                hasScopedPermission: () => false,
                hasRole: () => false,
                isLoading: false,
                permissions: [],
                roles: [],
                managedDepartmentIds: null
            });

            renderPage();
            const checkboxes = within(screen.getByTestId("permissions-desktop")).getAllByRole("checkbox").filter(c => c.tagName === "INPUT");
            const allDisabled = checkboxes.every((callback) => (callback as HTMLInputElement).disabled);
            expect(allDisabled).toBe(true);
        });

        it("hides Save Matrix button when user lacks permission", () => {
            vi.mocked(useMyPermissions).mockReturnValue({
                hasPermission: () => false,
                hasAnyPermission: () => false,
                hasScopedPermission: () => false,
                hasRole: () => false,
                isLoading: false,
                permissions: [],
                roles: [],
                managedDepartmentIds: null
            });

            renderPage();
            expect(screen.queryByText("Save Matrix")).toBeNull();
        });

        it("disables system role checkboxes even with manage permission", () => {
            renderPage();
            const checkboxes = within(screen.getByTestId("permissions-desktop")).getAllByRole("checkbox").filter(c => c.tagName === "INPUT");
            // System role is the first column → odd-indexed checkboxes in pair
            // For each permission row: [system, custom] — system should be disabled
            const systemCheckboxes = checkboxes.filter((_, index) => index % 2 === 0);
            systemCheckboxes.forEach((callback) => {
                expect((callback as HTMLInputElement).disabled).toBe(true);
            });
        });

        it("enables custom role checkboxes when user has manage permission", () => {
            renderPage();
            const checkboxes = within(screen.getByTestId("permissions-desktop")).getAllByRole("checkbox").filter(c => c.tagName === "INPUT");
            const customCheckboxes = checkboxes.filter((_, index) => index % 2 === 1);
            customCheckboxes.forEach((callback) => {
                expect((callback as HTMLInputElement).disabled).toBe(false);
            });
        });
    });

    // ─── Checkbox State from Matrix ────────────────────────────

    describe("checkbox state from matrix data", () => {
        it("checks employees:read for both system and custom roles", () => {
            renderPage();
            // Both roles have employees:read in the mock matrix
            const checkboxes = within(screen.getByTestId("permissions-desktop")).getAllByRole("checkbox").filter(c => c.tagName === "INPUT");
            const checkedCount = checkboxes.filter((callback) => (callback as HTMLInputElement).checked).length;
            // System: 5 checked + Custom: 2 checked = 7
            expect(checkedCount).toBe(7);
        });

        it("updates checkboxes when matrix data changes after initial render", async () => {
            const { rerender } = renderPage();
            
            // First check initial state (7 checked)
            const checkboxes = within(screen.getByTestId("permissions-desktop")).getAllByRole("checkbox").filter(c => c.tagName === "INPUT");
            expect(checkboxes.filter(callback => (callback as HTMLInputElement).checked).length).toBe(7);

            // Change mock return value to a different matrix (e.g., only 1 permission)
            vi.mocked(useOrganizationPermissionsMatrix).mockReturnValue({
                matrix: buildMatrix(
                    ["organization:settings:read"],
                    [],
                ),
                isLoading: false,
                refetch: vi.fn(),
            });


            // Trigger rerender of the same component
            rerender(
                <QueryClientProvider client={queryClient}>
                    <OrganizationPermissionsPage />
                </QueryClientProvider>
            );

            // Now check if UI updated to only 1 checked checkbox (wait for async update)
            await waitFor(() => {
                const checkedCheckboxes = (within(screen.getByTestId("permissions-desktop")).getAllByRole("checkbox").filter(c => c.tagName === "INPUT"))
                    .filter(callback => (callback as HTMLInputElement).checked);
                expect(checkedCheckboxes.length).toBe(1);
            });
        });


    });

    // ─── Toggle & Save ─────────────────────────────────────────

    describe("toggle and save flow", () => {
        it("shows 'Unsaved changes' indicator after toggling a custom role permission", async () => {
            const user = userEvent.setup();
            renderPage();

            // Find an enabled, unchecked checkbox (custom role)
            const checkboxes = within(screen.getByTestId("permissions-desktop")).getAllByRole("checkbox").filter(c => c.tagName === "INPUT");
            const customUnchecked = checkboxes.find((callback) => !(callback as HTMLInputElement).disabled && !(callback as HTMLInputElement).checked);
            expect(customUnchecked).toBeDefined();

            await user.click(customUnchecked!);
            expect(screen.getByText("Unsaved changes")).toBeDefined();
        });

        it("enables Save Matrix button after a change", async () => {
            const user = userEvent.setup();
            renderPage();

            const checkboxes = within(screen.getByTestId("permissions-desktop")).getAllByRole("checkbox").filter(c => c.tagName === "INPUT");
            const customUnchecked = checkboxes.find((callback) => !(callback as HTMLInputElement).disabled && !(callback as HTMLInputElement).checked);
            await user.click(customUnchecked!);

            const saveButton = screen.getByText("Save Matrix").closest("button")!;
            expect(saveButton.disabled).toBe(false);
        });

        it("Save Matrix button is disabled when no changes exist", () => {
            renderPage();
            const saveButton = screen.getByText("Save Matrix").closest("button")!;
            expect(saveButton.disabled).toBe(true);
        });

        it("calls updateRolePermissions only for modified roles on save", async () => {
            const user = userEvent.setup();
            renderPage();

            // Toggle a custom role checkbox
            const checkboxes = within(screen.getByTestId("permissions-desktop")).getAllByRole("checkbox").filter(c => c.tagName === "INPUT");
            const customUnchecked = checkboxes.find((callback) => !(callback as HTMLInputElement).disabled && !(callback as HTMLInputElement).checked);
            await user.click(customUnchecked!);

            // Click save
            const saveButton = screen.getByText("Save Matrix").closest("button")!;
            await user.click(saveButton);

            await waitFor(() => {
                // Should only call for the custom role, not the system role
                expect(api.updateRolePermissions).toHaveBeenCalledTimes(1);
                expect(api.updateRolePermissions).toHaveBeenCalledWith(
                    CUSTOM_ROLE.id,
                    expect.any(Array),
                );
            });
        });

        it("calls refetch after successful save", async () => {
            const user = userEvent.setup();
            renderPage();

            const checkboxes = within(screen.getByTestId("permissions-desktop")).getAllByRole("checkbox").filter(c => c.tagName === "INPUT");
            const customUnchecked = checkboxes.find((callback) => !(callback as HTMLInputElement).disabled && !(callback as HTMLInputElement).checked);
            await user.click(customUnchecked!);

            const saveButton = screen.getByText("Save Matrix").closest("button")!;
            await user.click(saveButton);

            await waitFor(() => {
                expect(mockRefetch).toHaveBeenCalledTimes(1);
            });
        });

        it("removes 'Unsaved changes' after successful save", async () => {
            const user = userEvent.setup();
            renderPage();

            const checkboxes = within(screen.getByTestId("permissions-desktop")).getAllByRole("checkbox").filter(c => c.tagName === "INPUT");
            const customUnchecked = checkboxes.find((callback) => !(callback as HTMLInputElement).disabled && !(callback as HTMLInputElement).checked);
            await user.click(customUnchecked!);
            expect(screen.getByText("Unsaved changes")).toBeDefined();

            const saveButton = screen.getByText("Save Matrix").closest("button")!;
            await user.click(saveButton);

            await waitFor(() => {
                expect(screen.queryByText("Unsaved changes")).toBeNull();
            });
        });

        it("can toggle a checked permission off", async () => {
            const user = userEvent.setup();
            renderPage();

            // Custom role has employees:read checked — find it and uncheck
            const checkboxes = within(screen.getByTestId("permissions-desktop")).getAllByRole("checkbox").filter(c => c.tagName === "INPUT");
            const customChecked = checkboxes.find((callback) => !(callback as HTMLInputElement).disabled && (callback as HTMLInputElement).checked);
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

            const checkboxes = within(screen.getByTestId("permissions-desktop")).getAllByRole("checkbox").filter(c => c.tagName === "INPUT");
            // Get a system checkbox (disabled)
            const systemCallback = checkboxes.find((callback) => (callback as HTMLInputElement).disabled);
            expect(systemCallback).toBeDefined();

            // Attempt click — should be blocked by disabled attribute
            await user.click(systemCallback!);
            expect(screen.queryByText("Unsaved changes")).toBeNull();
        });
    });

    // ─── Permission ID Integrity ───────────────────────────────

    describe("permission ID integrity (backend sync)", () => {
        it("all permission IDs use colon-separated backend format", () => {
            renderPage();
            // Verify no old mock IDs remain (edit_organization, view_employees, etc.)
            expect(screen.queryByText("Edit Workspace Info")).toBeNull();
            expect(screen.queryByText("Manage Billing")).toBeNull();
            expect(screen.queryByText("Danger Zone")).toBeNull();
        });

        it("renders catalog groups that previously relied on the static matrix", () => {
            renderPage();
            const desktop = within(screen.getByTestId("permissions-desktop"));
            expect(desktop.getByText("ess")).toBeDefined();
            expect(desktop.getByText("infra")).toBeDefined();
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
            const columnsButton = screen.getByRole("button", { name: /Columns/i });
            await user.click(columnsButton);

            // Find the checkbox for hr_specialist
            const hrSpecialistCheckbox = screen.getByRole("menuitemcheckbox", { name: /hr_specialist/i });
            
            // Uncheck it
            await user.click(hrSpecialistCheckbox);

            // Verify hr_specialist is removed from headers
            expect(screen.queryByRole("columnheader", { name: /hr_specialist/i })).toBeNull();
            // Verify admin is still there
            expect(screen.queryByRole("columnheader", { name: /admin/i })).not.toBeNull();

            // Click Columns button again to open menu (Radix dropdown closes after click)
            await user.click(columnsButton);
            
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
