import { render, screen, act } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import React from "react";
import WorkspacePermissionsPage from "../workspace/WorkspacePermissionsPage";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";
import { useWorkspaceRoles, useWorkspacePermissionsMatrix } from "../../hooks/use-workspace";
import { useMyPermissions } from "../../hooks/use-permissions";
import * as api from "../../lib/api";

vi.mock("react-i18next", () => ({
    useTranslation: () => ({
        t: (key: string, fallback?: string) => fallback || key,
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

describe("WorkspacePermissionsPage", () => {
    let queryClient: QueryClient;

    const mockRoles = [
        { id: "role-1", name: "Admin", is_system: true },
        { id: "role-2", name: "Manager", is_system: false },
    ];

    const mockMatrix = {
        "role-1": ["edit_workspace", "manage_roles", "view_employees"],
        "role-2": ["view_employees"],
    };

    beforeEach(() => {
        vi.clearAllMocks();
        queryClient = new QueryClient({
            defaultOptions: { queries: { retry: false } },
        });

        (useWorkspaceRoles as any).mockReturnValue({
            roles: mockRoles,
            isLoading: false,
        });

        (useWorkspacePermissionsMatrix as any).mockReturnValue({
            matrix: mockMatrix,
            isLoading: false,
            refetch: vi.fn(),
        });

        (useMyPermissions as any).mockReturnValue({
            hasPermission: () => true,
            hasAnyPermission: () => true,
            hasScopedPermission: () => true,
            isLoading: false,
        });
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

    it("renders matrix correctly", () => {
        renderWithProviders(<WorkspacePermissionsPage />);
        expect(screen.getByText("Admin")).toBeDefined();
        expect(screen.getByText("Manager")).toBeDefined();
        
        // Roles + Header + Checkboxes
        expect(screen.getAllByRole("checkbox").length).toBeGreaterThan(0);
    });

    it("hides save button and disables checkboxes if user lacks manage permission", () => {
        (useMyPermissions as any).mockReturnValue({
            hasPermission: () => false,
            isLoading: false,
        });

        renderWithProviders(<WorkspacePermissionsPage />);
        
        // Save Matrix button should be hidden
        expect(screen.queryByText("Save Matrix")).toBeNull();

        // Checkboxes should be disabled
        const checkboxes = screen.getAllByRole("checkbox") as HTMLInputElement[];
        checkboxes.forEach(cb => {
            expect(cb.disabled).toBe(true);
        });
    });

    it("allows editing custom roles if user has manage permission", async () => {
        const user = userEvent.setup();
        renderWithProviders(<WorkspacePermissionsPage />);
        
        const checkboxes = screen.getAllByRole("checkbox") as HTMLInputElement[];
        // Find a checkbox that is not disabled (custom role checkbox)
        const activeCheckbox = checkboxes.find(cb => !cb.disabled);
        expect(activeCheckbox).toBeDefined();

        if (activeCheckbox) {
            await user.click(activeCheckbox);
            expect(screen.getByText("Save Matrix")).toBeDefined();
        }
    });
});
