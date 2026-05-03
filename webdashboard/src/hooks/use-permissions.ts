import { useQuery } from "@tanstack/react-query";
import { useCallback } from "react";

import { fetchMyPermissions, getCurrentWorkspaceId } from "@/lib/api";

export const myPermissionsKey = (wsId: string) =>
    ["my-permissions", wsId] as const;

function checkHasRole(roles: string[], role: string): boolean {
  return roles.some((r) => r.toLowerCase() === role.toLowerCase());
}

function checkScopedPermission(permissionSet: Set<string>, allowedDepts: string[] | null | undefined, permission: string, targetDepartmentId?: string): boolean {
  if (!permissionSet.has(permission)) return false;
  if (allowedDepts === null || allowedDepts === undefined) return true;
  if (targetDepartmentId && Array.isArray(allowedDepts)) return allowedDepts.includes(targetDepartmentId);
  return false;
}

/**
 * Hook that fetches and caches the current user's effective permissions
 * for the active workspace.
 *
 * Usage:
 *   const { hasPermission, permissions, roles, isLoading } = useMyPermissions();
 *   if (hasPermission("employees:create")) { ... }
 */
export function useMyPermissions() {
    const wsId = getCurrentWorkspaceId() ?? "";

    const query = useQuery({
        queryKey: myPermissionsKey(wsId),
        queryFn: fetchMyPermissions,
        enabled: !!wsId,
        staleTime: 5 * 60 * 1000,
        gcTime: 10 * 60 * 1000,
    });

    const permissionSet = new Set(query.data?.permissions ?? []);
    const roles = query.data?.roles ?? [];

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
        (role: string): boolean => checkHasRole(roles, role),
        // eslint-disable-next-line react-hooks/exhaustive-deps
        [query.data?.roles],
    );

    const hasScopedPermission = useCallback(
        (permission: string, targetDepartmentId?: string): boolean =>
            checkScopedPermission(permissionSet, query.data?.managed_department_ids, permission, targetDepartmentId),
        // eslint-disable-next-line react-hooks/exhaustive-deps
        [query.data?.permissions, query.data?.managed_department_ids],
    );

    return {
        permissions: query.data?.permissions ?? [],
        roles,
        managedDepartmentIds: query.data?.managed_department_ids ?? null,
        isLoading: query.isLoading,
        hasPermission,
        hasAnyPermission,
        hasRole,
        hasScopedPermission,
    };
}

