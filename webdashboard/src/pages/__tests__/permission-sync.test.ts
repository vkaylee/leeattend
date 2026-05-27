/**
 * Cross-Layer Permission Sync Test
 *
 * Ensures the frontend Permission Matrix UI stays perfectly synchronized
 * with the backend Rust Permission enum (Permission::all_permission_strings()).
 *
 * If this test fails, it means someone added/removed a permission on one side
 * without updating the other.
 */
import { describe, it, expect } from "vitest";

import { permissionGroups } from "../../lib/permission-groups";

/**
 * Canonical list of ALL backend permission strings.
 * Source of truth: apiserver/src/domain/permissions.rs → Permission::all_permission_strings()
 *
 * // IMPORTANT: This count must match exactly the number of permissions in 
 * // apiserver/src/domain/permissions.rs `test_total_permission_count_is_97`
 * const EXPECTED_BACKEND_PERMISSION_COUNT = 97;
 *
 * ⚠️  When adding a new Permission variant in Rust, you MUST also add it here
 *     AND in the permissionGroups of WorkspacePermissionsPage.tsx.
 */
const BACKEND_PERMISSION_STRINGS: ReadonlySet<string> = new Set([
    // Workspace
    "workspace:update", "workspace:delete", "workspace:settings:read", "workspace:settings:write",
    // Members
    "members:read", "members:invite", "members:roles:manage", "members:remove",
    // Employees
    "employees:read", "employees:create", "employees:update", "employees:delete", "employees:rehire", "employees:upload",
    // Departments
    "departments:read", "departments:create", "departments:update", "departments:delete", "departments:transfer",
    // Attendance
    "attendance:logs:read", "attendance:logs:import", "attendance:summaries:read",
    "attendance:adjustments:submit", "attendance:adjustments:approve", "attendance:reprocess", "attendance:staging:manage",
    // Overtime
    "overtime:claim", "overtime:approve", "overtime:policies:manage", "overtime:paper:upload",
    "overtime:request:create", "overtime:request:read", "overtime:approve:workflow", "overtime:quota:read",
    "approval_workflows:read", "approval_workflows:manage",
    // Leave
    "leave:requests:read", "leave:requests:submit", "leave:requests:approve", "leave:balances:read", "leave:balances:adjust",
    "leave:types:manage", "leave:policies:manage", "leave:accrual:run",
    // Shifts & Scheduling
    "shifts:read", "shifts:create", "shifts:update", "shifts:assign",
    "shift_swaps:manage", "shift_swaps:approve",
    "patterns:read", "patterns:create", "patterns:assign", "patterns:delete",
    "rosters:read", "rosters:manage", "assignment_rules:manage",
    // Payroll
    "payroll:read", "payroll:calculate", "payroll:settings:manage", "payroll:components:manage", "payroll:incentives:manage",
    // Holidays
    "holidays:read", "holidays:manage",
    // Reports & Dashboard
    "reports:read", "reports:export",
    "dashboard:read", "dashboard:preferences",
    // Agents & Devices
    "agents:read", "agents:manage", "devices:read", "devices:manage",
    // Recruitment
    "recruitment:read", "recruitment:manage",
    // Employment Phases
    "employment_phases:read", "employment_phases:manage",
    // Feature Flags
    "feature_flags:read", "feature_flags:manage",
    // Security & Audit (Premium)
    "audit_logs:read", "audit_logs:export", "security:settings:manage",
    // Bulk & Emergency (Premium)
    "employees:bulk_edit", "attendance:force_clock_out",
    // Documents (Premium)
    "documents:medical:read", "documents:contract:read",
    // Advanced Payroll & Approvals (Premium)
    "payroll:approve", "payroll:disburse", "payroll:sensitive:read",
    "leave:approve:l1", "leave:approve:l2",
    // Infrastructure
    "infra:monitor",
    // ESS
    "ess:profile", "ess:attendance", "ess:clock", "ess:leave", "ess:payslips", "ess:shift_swaps",
]);

/** Extract all permission IDs from the UI permissionGroups */
function extractUiPermissionIds(): string[] {
    return permissionGroups.flatMap((group) =>
        group.permissions.map((p) => p.id),
    );
}

describe("Permission Sync: Backend ↔ Frontend", () => {

    // ─── Count Verification ────────────────────────────────────

    it("backend defines exactly 97 permissions", () => {
        expect(BACKEND_PERMISSION_STRINGS.size).toBe(97);
    });

    it("UI permissionGroups defines exactly 97 permissions", () => {
        const uiIds = extractUiPermissionIds();
        expect(uiIds.length).toBe(97);
    });

    it("backend count matches UI count", () => {
        const uiIds = extractUiPermissionIds();
        expect(uiIds.length).toBe(BACKEND_PERMISSION_STRINGS.size);
    });

    // ─── 1:1 Coverage ──────────────────────────────────────────

    it("every UI permission ID exists in the backend", () => {
        const uiIds = extractUiPermissionIds();
        const missing = uiIds.filter((id) => !BACKEND_PERMISSION_STRINGS.has(id));
        expect(missing).toEqual([]);
    });

    it("every backend permission ID exists in the UI", () => {
        const uiIdSet = new Set(extractUiPermissionIds());
        const missing = [...BACKEND_PERMISSION_STRINGS].filter((id) => !uiIdSet.has(id));
        expect(missing).toEqual([]);
    });

    // ─── Duplicates ────────────────────────────────────────────

    it("UI has no duplicate permission IDs", () => {
        const uiIds = extractUiPermissionIds();
        const seen = new Set<string>();
        const duplicates: string[] = [];
        for (const id of uiIds) {
            if (seen.has(id)) duplicates.push(id);
            seen.add(id);
        }
        expect(duplicates).toEqual([]);
    });

    // ─── Format Consistency ────────────────────────────────────

    it("all permission IDs use colon-separated lowercase format", () => {
        const uiIds = extractUiPermissionIds();
        const invalid = uiIds.filter((id) => !/^[a-z][a-z0-9_]*(?::[a-z][a-z0-9_]*)*$/.test(id));
        expect(invalid).toEqual([]);
    });

    // ─── Group Integrity ───────────────────────────────────────

    it("every permission group has a unique ID", () => {
        const groupIds = permissionGroups.map((g) => g.id);
        expect(new Set(groupIds).size).toBe(groupIds.length);
    });

    it("every permission group has at least one permission", () => {
        for (const group of permissionGroups) {
            expect(group.permissions.length).toBeGreaterThan(0);
        }
    });

    it("every permission has a non-empty name and description", () => {
        const uiPerms = permissionGroups.flatMap((g) => g.permissions);
        for (const perm of uiPerms) {
            expect(perm.name.length).toBeGreaterThan(0);
            expect(perm.desc.length).toBeGreaterThan(0);
        }
    });

    // ─── Premium Permission Groups Present ─────────────────────

    it("includes all premium enterprise permission groups", () => {
        const groupIds = new Set(permissionGroups.map((g) => g.id));
        expect(groupIds.has("security")).toBe(true);
        expect(groupIds.has("documents")).toBe(true);
        expect(groupIds.has("infra")).toBe(true);
        expect(groupIds.has("ess")).toBe(true);
        expect(groupIds.has("recruitment")).toBe(true);
    });

    it("leave group contains multi-level approval permissions", () => {
        const leaveGroup = permissionGroups.find((g) => g.id === "leave");
        const leaveIds = leaveGroup!.permissions.map((p) => p.id);
        expect(leaveIds).toContain("leave:approve:l1");
        expect(leaveIds).toContain("leave:approve:l2");
        expect(leaveIds).toContain("leave:requests:approve");
    });

    it("payroll group contains advanced approval permissions", () => {
        const payrollGroup = permissionGroups.find((g) => g.id === "payroll");
        const payrollIds = payrollGroup!.permissions.map((p) => p.id);
        expect(payrollIds).toContain("payroll:approve");
        expect(payrollIds).toContain("payroll:disburse");
        expect(payrollIds).toContain("payroll:sensitive:read");
    });
});
