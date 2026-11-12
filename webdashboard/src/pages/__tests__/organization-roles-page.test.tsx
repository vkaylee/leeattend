import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import { useMyPermissions } from "../../hooks/use-permissions";
import { useOrganizationRoles } from "@/hooks/use-organization";
import { OrganizationRole } from "../../lib/api/types";
import OrganizationRolesPage from "../organization/organization-roles-page";
vi.mock("react-i18next", () => ({
    useTranslation: () => ({
        t: (key: string, options?: { defaultValue?: string }) => {
            if (options && typeof options === 'object' && options.defaultValue) return options.defaultValue;
            return key;
 
        },
    }),
}));

vi.mock("@/hooks/use-organization", () => ({
    useOrganizationRoles: vi.fn(),
}));

vi.mock("../../hooks/use-permissions", () => ({
    useMyPermissions: vi.fn(),
}));

vi.mock("@tanstack/react-router", () => ({
    Link: ({ children, to, className }: { children: React.ReactNode; to: string; className?: string }) => <a className={className} href={to}>{children}</a>,
}));

vi.mock("@/components/features/settings/role-form-modal", () => ({
 
    default: ({ isOpen, editingRole }: { isOpen: boolean; editingRole?: { name: string } }) => {
        if (!isOpen) return null;
        return <div data-testid="role-form-modal">Modal: {editingRole ? editingRole.name : 'Create'}</div>;
    }
 
}));

describe("OrganizationRolesPage", () => {
    let queryClient: QueryClient;

    const mockRoles = [
        { id: "role-1", name: "Owner", is_system: true, description: "Workspace Owner", color: "rose" },
        { id: "role-2", name: "Manager", is_system: false, description: "Custom Manager", color: "primary" },
    ];

    beforeEach(() => {
        vi.clearAllMocks();
        queryClient = new QueryClient({
            defaultOptions: { queries: { retry: false } },
        });

        vi.mocked(useOrganizationRoles).mockReturnValue({
            roles: mockRoles as unknown as OrganizationRole[],
            isLoading: false,
            createRole: vi.fn() as unknown as (_arguments: unknown) => Promise<unknown>,
 
            isCreating: false,
            updateRole: vi.fn() as unknown as (_arguments: unknown) => Promise<unknown>,
            isUpdating: false,
            deleteRole: vi.fn() as unknown as (_arguments: unknown) => Promise<unknown>,
            isDeleting: false,
        } as unknown as ReturnType<typeof useOrganizationRoles>);
 

        vi.mocked(useMyPermissions).mockReturnValue({
            hasPermission: () => true,
            hasAnyPermission: () => true,
            hasScopedPermission: () => true,
            hasRole: () => true,
            isLoading: false,
            permissions: [],
            roles: [],
            managedDepartmentIds: null
        });
        
        window.confirm = vi.fn().mockReturnValue(true);
    });

    const renderWithProviders = (ui: React.ReactElement) => {
        return render(
            <QueryClientProvider client={queryClient}>
                {ui}
            </QueryClientProvider>
        );
    };

    it("renders roles list correctly", () => {
        renderWithProviders(<OrganizationRolesPage />);
        expect(screen.getByRole("list", { name: "Organization roles" })).toBeDefined();
        expect(screen.getByText("Owner")).toBeDefined();
        expect(screen.getByText("Manager")).toBeDefined();
        expect(screen.getAllByText("Workspace Owner")).toHaveLength(2);
        expect(screen.queryByText(/Role ID:/)).toBeNull();
    });

    it("shows an empty state when no roles exist", () => {
        vi.mocked(useOrganizationRoles).mockReturnValue({
            roles: [],
            isLoading: false,
        } as unknown as ReturnType<typeof useOrganizationRoles>);

        renderWithProviders(<OrganizationRolesPage />);

        expect(screen.getByText("No roles yet")).toBeDefined();
        expect(screen.getByText("Create a role to define reusable access levels for your organization.")).toBeDefined();
    });

    it("shows a skeleton while roles are loading", () => {
        vi.mocked(useOrganizationRoles).mockReturnValue({
            roles: [],
            isLoading: true,
        } as unknown as ReturnType<typeof useOrganizationRoles>);

        renderWithProviders(<OrganizationRolesPage />);

        expect(screen.getByTestId("roles-list-skeleton")).toBeDefined();
    });

    it("hides action buttons if user lacks manage permission", () => {
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

        renderWithProviders(<OrganizationRolesPage />);
        
        expect(screen.queryByText("Create Role")).toBeNull();
        expect(screen.queryByText("Manage Access")).toBeNull();
    });

    it("shows action buttons if user has manage permission", () => {
        vi.mocked(useMyPermissions).mockReturnValue({
            hasPermission: () => true,
            hasAnyPermission: () => true,
            hasScopedPermission: () => true,
            hasRole: () => true,
            isLoading: false,
            permissions: [],
            roles: [],
            managedDepartmentIds: null
        });

        renderWithProviders(<OrganizationRolesPage />);
        
        expect(screen.getByText("Create Role")).toBeDefined();
        // Menu toggle buttons should exist
        const moreButtons = screen.queryAllByRole("button");
        expect(moreButtons.length).toBeGreaterThan(0);
    });
});
