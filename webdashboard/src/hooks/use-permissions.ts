import { useQuery } from "@tanstack/react-query";
import { fetchMyPermissions, getCurrentWorkspaceId } from "@/lib/api";
import { useCallback } from "react";

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
                (r) => r.toLowerCase() === role.toLowerCase(),
            ),
        [query.data?.roles],
    );

    return {
        permissions: query.data?.permissions ?? [],
        roles: query.data?.roles ?? [],
        isLoading: query.isLoading,
        hasPermission,
        hasAnyPermission,
        hasRole,
    };
}
