import { useQuery } from "@tanstack/react-query";
import { useCallback, useMemo } from "react";

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

    const { data, isLoading } = useQuery({
        queryKey: myPermissionsKey(wsId),
        queryFn: fetchMyPermissions,
        enabled: !!wsId,
        staleTime: 5 * 60 * 1000,
        gcTime: 10 * 60 * 1000,
    });

    const permissions = useMemo(() => data?.permissions ?? [], [data?.permissions]);
    const roles = useMemo(() => data?.roles ?? [], [data?.roles]);
    const managedDeptIds = data?.managed_department_ids ?? null;

    const permissionSet = useMemo(() => new Set(permissions), [permissions]);

    const hasPermission = useCallback(
        (permission: string): boolean => permissionSet.has(permission),
        [permissionSet],
    );

    const hasAnyPermission = useCallback(
        (...perms: string[]): boolean => perms.some((p) => permissionSet.has(p)),
        [permissionSet],
    );

    const hasRole = useCallback(
        (role: string): boolean => checkHasRole(roles, role),
        [roles],
    );

    const hasScopedPermission = useCallback(
        (permission: string, targetDeptId?: string): boolean =>
            checkScopedPermission(permissionSet, managedDeptIds, permission, targetDeptId),
        [permissionSet, managedDeptIds],
    );

    return {
        permissions,
        roles,
        managedDepartmentIds: managedDeptIds,
        isLoading,
        hasPermission,
        hasAnyPermission,
        hasRole,
        hasScopedPermission,
    };
}

