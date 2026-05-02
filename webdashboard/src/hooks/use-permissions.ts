import { useQuery } from "@tanstack/react-query";
import { useCallback } from "react";

import { fetchMyPermissions, getCurrentWorkspaceId } from "@/lib/api";

export const myPermissionsKey = (wsId: string) =>
    ["my-permissions", wsId] as const;

/**
 * Hook that fetches and caches the current user's effective permissions
 * for the active workspace.
 *
 * Usage:
 *   const { hasPermission, permissions, roles, isLoading } = useMyPermissions();
 *   if (hasPermission("employees:create")) { ... }
 */
export function useMyPermissions() {
    const wsId = getCurrentWorkspaceId() || "";

    const query = useQuery({
        queryKey: myPermissionsKey(wsId),
        queryFn: fetchMyPermissions,
        enabled: !!wsId,
        staleTime: 5 * 60 * 1000, // cache for 5 minutes — permissions don't change often
        gcTime: 10 * 60 * 1000,
    });

    const permissionSet = new Set(query.data?.permissions ?? []);

    const hasPermission = useCallback(
        (permission: string): boolean => permissionSet.has(permission),
        // eslint-disable-next-line react-hooks/exhaustive-deps
        [query.data?.permissions],
    );

    const hasAnyPermission = useCallback(
        (...permissions: string[]): boolean =>
            permissions.some((p) => permissionSet.has(p)),
        // eslint-disable-next-line react-hooks/exhaustive-deps
        [query.data?.permissions],
    );

    const hasRole = useCallback(
        (role: string): boolean =>
            (query.data?.roles ?? []).some(
                (r: string) => r.toLowerCase() === role.toLowerCase(),
            ),
        [query.data?.roles],
    );

    const hasScopedPermission = useCallback(
        (permission: string, targetDepartmentId?: string): boolean => {
            if (!permissionSet.has(permission)) return false;

            const allowedDepts = query.data?.managed_department_ids;

            // null or undefined means global scope
            if (allowedDepts === null || allowedDepts === undefined) {
                return true;
            }

            // Must match one of the allowed departments
            if (targetDepartmentId && Array.isArray(allowedDepts)) {
                return allowedDepts.includes(targetDepartmentId);
            }

            return false;
        },
        // eslint-disable-next-line react-hooks/exhaustive-deps
        [query.data?.permissions, query.data?.managed_department_ids],
    );

    return {
        permissions: query.data?.permissions ?? [],
        roles: query.data?.roles ?? [],
        managedDepartmentIds: query.data?.managed_department_ids ?? null,
        isLoading: query.isLoading,
        hasPermission,
        hasAnyPermission,
        hasRole,
        hasScopedPermission,
    };
}
