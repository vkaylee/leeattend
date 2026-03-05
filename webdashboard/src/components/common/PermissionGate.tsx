import type { ReactNode } from "react";
import { useMyPermissions } from "@/hooks/use-permissions";

interface PermissionGateProps {
    /** Single permission to check */
    permission?: string;
    /** Multiple permissions — user needs at least ONE */
    anyOf?: string[];
    /** Multiple permissions — user needs ALL */
    allOf?: string[];
    /** Department context for scoped permissions */
    departmentId?: string;
    /** What to render when the user lacks permission (default: nothing) */
    fallback?: ReactNode;
    children: ReactNode;
}

/**
 * Conditionally renders children based on the current user's permissions.
 *
 * Usage:
 *   <PermissionGate permission="employees:create">
 *     <CreateEmployeeButton />
 *   </PermissionGate>
 *
 *   <PermissionGate anyOf={["employees:create", "employees:update"]}>
 *     <EmployeeForm />
 *   </PermissionGate>
 *
 *   <PermissionGate allOf={["payroll:read", "payroll:calculate"]}>
 *     <RunPayrollButton />
 *   </PermissionGate>
 */
export default function PermissionGate({
    permission,
    anyOf,
    allOf,
    departmentId,
    fallback = null,
    children,
}: PermissionGateProps) {
    const { hasPermission, hasAnyPermission, hasScopedPermission, isLoading } = useMyPermissions();

    // While loading, show nothing (prevents flash of forbidden content)
    if (isLoading) return null;

    let allowed = false;

    if (permission) {
        if (departmentId) {
            allowed = hasScopedPermission(permission, departmentId);
        } else {
            allowed = hasPermission(permission);
        }
    } else if (anyOf && anyOf.length > 0) {
        allowed = hasAnyPermission(...anyOf);
    } else if (allOf && allOf.length > 0) {
        allowed = allOf.every((p) => hasPermission(p));
    } else {
        // No permission specified — always show
        allowed = true;
    }

    return allowed ? <>{children}</> : <>{fallback}</>;
}
