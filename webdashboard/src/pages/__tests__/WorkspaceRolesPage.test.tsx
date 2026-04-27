import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import React from "react";
import WorkspaceRolesPage from "../workspace/WorkspaceRolesPage";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";
import { useWorkspaceRoles } from "../../hooks/use-workspace";
import { useMyPermissions } from "../../hooks/use-permissions";

vi.mock("react-i18next", () => ({
    useTranslation: () => ({
        t: (key: string, fallback?: string) => fallback || key,
    }),
}));

vi.mock("../../hooks/use-workspace", () => ({
    useWorkspaceRoles: vi.fn(),
}));

vi.mock("../../hooks/use-permissions", () => ({
    useMyPermissions: vi.fn(),
}));

vi.mock("@/components/features/settings/RoleFormModal", () => ({
    default: ({ isOpen, editingRole }: any) => {
        if (!isOpen) return null;
        return <div data-testid="role-form-modal">Modal: {editingRole ? editingRole.name : 'Create'}</div>;
    }
}));

describe("WorkspaceRolesPage", () => {
    let queryClient: QueryClient;

    const mockRoles = [
        { id: "role-1", name: "Owner", is_system: true, description: "Workspace Owner", color: "rose" },
        { id: "role-2", name: "Manager", is_system: false, description: "Custom Manager", color: "indigo" },
    ];

    beforeEach(() => {
        vi.clearAllMocks();
        queryClient = new QueryClient({
            defaultOptions: { queries: { retry: false } },
        });

        (useWorkspaceRoles as any).mockReturnValue({
            roles: mockRoles,
            isLoading: false,
            deleteRole: vi.fn(),
        });

        (useMyPermissions as any).mockReturnValue({
            hasPermission: () => true,
            hasAnyPermission: () => true,
            hasScopedPermission: () => true,
            isLoading: false,
        });
        
        window.confirm = vi.fn().mockReturnValue(true);
    });

    const renderWithProviders = (ui: React.ReactElement) => {
        return render(
            <QueryClientProvider client={queryClient}>
                <MemoryRouter>
                    {ui}
                </MemoryRouter>
            </QueryClientProvider>
        );
    };

    it("renders roles list correctly", () => {
        renderWithProviders(<WorkspaceRolesPage />);
        expect(screen.getByText("Owner")).toBeDefined();
        expect(screen.getByText("Manager")).toBeDefined();
        expect(screen.getByText("Workspace Owner")).toBeDefined();
    });

    it("hides action buttons if user lacks manage permission", () => {
        (useMyPermissions as any).mockReturnValue({
            hasPermission: () => false,
            hasAnyPermission: () => false,
            hasScopedPermission: () => false,
            isLoading: false,
        });

        renderWithProviders(<WorkspaceRolesPage />);
        
        expect(screen.queryByText("Create Role")).toBeNull();
        expect(screen.queryByText("Manage Access")).toBeNull();
    });

    it("shows action buttons if user has manage permission", () => {
        (useMyPermissions as any).mockReturnValue({
            hasPermission: () => true,
            hasAnyPermission: () => true,
            hasScopedPermission: () => true,
            isLoading: false,
        });

        renderWithProviders(<WorkspaceRolesPage />);
        
        expect(screen.getByText("Create Role")).toBeDefined();
        // Menu toggle buttons should exist
        const moreButtons = screen.queryAllByRole("button");
        expect(moreButtons.length).toBeGreaterThan(0);
    });
});
