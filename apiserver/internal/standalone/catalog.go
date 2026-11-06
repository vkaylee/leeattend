package standalone

import (
	"context"
	"fmt"

	"github.com/jackc/pgx/v5/pgconn"
)

type dbExecutor interface {
	Exec(context.Context, string, ...any) (pgconn.CommandTag, error)
}

const seedSystemRolesSQL = `
INSERT INTO public.system_roles (id, name, description) VALUES
    ('c0000000-0000-0000-0000-000000000001', 'admin', 'System Administrator with full access to infrastructure and global configuration')
ON CONFLICT (name) DO NOTHING;
`

const seedWorkspaceRolesSQL = `
INSERT INTO public.workspace_roles (id, workspace_id, name, description, color, is_system, scope_type) VALUES
    ('a0000000-0000-0000-0000-000000000002', NULL, 'admin', 'Administrator with full access', 'rose', TRUE, 'global'),
    ('a0000000-0000-0000-0000-000000000003', NULL, 'hr_manager', 'HR Manager with employee and attendance control', 'purple', TRUE, 'global'),
    ('a0000000-0000-0000-0000-000000000004', NULL, 'shift_manager', 'Shift Manager for schedule planning', 'blue', TRUE, 'global'),
    ('a0000000-0000-0000-0000-000000000005', NULL, 'team_lead', 'Team Lead for daily attendance monitoring', 'green', TRUE, 'global'),
    ('a0000000-0000-0000-0000-000000000006', NULL, 'employee', 'Standard employee with self-service access', 'gray', TRUE, 'global')
ON CONFLICT (id) DO NOTHING;
`

const seedWorkspaceRolePermissionsSQL = `
INSERT INTO workspace_role_permissions (role_id, permission_name) VALUES
    ('a0000000-0000-0000-0000-000000000002', 'approval_workflows:manage'),
    ('a0000000-0000-0000-0000-000000000002', 'approval_workflows:read'),
    ('a0000000-0000-0000-0000-000000000002', 'assignment_rules:manage'),
    ('a0000000-0000-0000-0000-000000000002', 'attendance:adjustments:approve'),
    ('a0000000-0000-0000-0000-000000000002', 'attendance:adjustments:manage_behalf'),
    ('a0000000-0000-0000-0000-000000000002', 'attendance:adjustments:submit'),
    ('a0000000-0000-0000-0000-000000000002', 'attendance:adjustments:submit_on_behalf'),
    ('a0000000-0000-0000-0000-000000000002', 'attendance:force_clock_out'),
    ('a0000000-0000-0000-0000-000000000002', 'attendance:logs:import'),
    ('a0000000-0000-0000-0000-000000000002', 'attendance:logs:read'),
    ('a0000000-0000-0000-0000-000000000002', 'attendance:reprocess'),
    ('a0000000-0000-0000-0000-000000000002', 'attendance:staging:manage'),
    ('a0000000-0000-0000-0000-000000000002', 'attendance:summaries:read'),
    ('a0000000-0000-0000-0000-000000000002', 'audit_logs:export'),
    ('a0000000-0000-0000-0000-000000000002', 'audit_logs:read'),
    ('a0000000-0000-0000-0000-000000000002', 'dashboard:employee_absence:read'),
    ('a0000000-0000-0000-0000-000000000002', 'dashboard:preferences'),
    ('a0000000-0000-0000-0000-000000000002', 'dashboard:read'),
    ('a0000000-0000-0000-0000-000000000002', 'departments:create'),
    ('a0000000-0000-0000-0000-000000000002', 'departments:delete'),
    ('a0000000-0000-0000-0000-000000000002', 'departments:read'),
    ('a0000000-0000-0000-0000-000000000002', 'departments:transfer'),
    ('a0000000-0000-0000-0000-000000000002', 'departments:update'),
    ('a0000000-0000-0000-0000-000000000002', 'documents:contract:read'),
    ('a0000000-0000-0000-0000-000000000002', 'documents:medical:read'),
    ('a0000000-0000-0000-0000-000000000002', 'employees:bulk_edit'),
    ('a0000000-0000-0000-0000-000000000002', 'employees:create'),
    ('a0000000-0000-0000-0000-000000000002', 'employees:delete'),
    ('a0000000-0000-0000-0000-000000000002', 'employees:read'),
    ('a0000000-0000-0000-0000-000000000002', 'employees:rehire'),
    ('a0000000-0000-0000-0000-000000000002', 'employees:update'),
    ('a0000000-0000-0000-0000-000000000002', 'employees:upload'),
    ('a0000000-0000-0000-0000-000000000002', 'employment_phases:manage'),
    ('a0000000-0000-0000-0000-000000000002', 'employment_phases:read'),
    ('a0000000-0000-0000-0000-000000000002', 'ess:attendance'),
    ('a0000000-0000-0000-0000-000000000002', 'ess:clock'),
    ('a0000000-0000-0000-0000-000000000002', 'ess:leave'),
    ('a0000000-0000-0000-0000-000000000002', 'ess:payslips'),
    ('a0000000-0000-0000-0000-000000000002', 'ess:profile'),
    ('a0000000-0000-0000-0000-000000000002', 'ess:shift_swaps'),
    ('a0000000-0000-0000-0000-000000000002', 'feature_flags:manage'),
    ('a0000000-0000-0000-0000-000000000002', 'feature_flags:read'),
    ('a0000000-0000-0000-0000-000000000002', 'holidays:manage'),
    ('a0000000-0000-0000-0000-000000000002', 'holidays:read'),
    ('a0000000-0000-0000-0000-000000000002', 'infra:monitor'),
    ('a0000000-0000-0000-0000-000000000002', 'leave:accrual:run'),
    ('a0000000-0000-0000-0000-000000000002', 'leave:approve:l1'),
    ('a0000000-0000-0000-0000-000000000002', 'leave:approve:l2'),
    ('a0000000-0000-0000-0000-000000000002', 'leave:balances:adjust'),
    ('a0000000-0000-0000-0000-000000000002', 'leave:balances:read'),
    ('a0000000-0000-0000-0000-000000000002', 'leave:policies:manage'),
    ('a0000000-0000-0000-0000-000000000002', 'leave:requests:approve'),
    ('a0000000-0000-0000-0000-000000000002', 'leave:requests:read'),
    ('a0000000-0000-0000-0000-000000000002', 'leave:requests:submit'),
    ('a0000000-0000-0000-0000-000000000002', 'leave:requests:submit_on_behalf'),
    ('a0000000-0000-0000-0000-000000000002', 'leave:types:manage'),
    ('a0000000-0000-0000-0000-000000000002', 'members:invite'),
    ('a0000000-0000-0000-0000-000000000002', 'members:read'),
    ('a0000000-0000-0000-0000-000000000002', 'members:remove'),
    ('a0000000-0000-0000-0000-000000000002', 'members:roles:manage'),
    ('a0000000-0000-0000-0000-000000000002', 'overtime:approve'),
    ('a0000000-0000-0000-0000-000000000002', 'overtime:approve:workflow'),
    ('a0000000-0000-0000-0000-000000000002', 'overtime:claim'),
    ('a0000000-0000-0000-0000-000000000002', 'overtime:claim_on_behalf'),
    ('a0000000-0000-0000-0000-000000000002', 'overtime:paper:upload'),
    ('a0000000-0000-0000-0000-000000000002', 'overtime:policies:manage'),
    ('a0000000-0000-0000-0000-000000000002', 'overtime:quota:read'),
    ('a0000000-0000-0000-0000-000000000002', 'overtime:request:create'),
    ('a0000000-0000-0000-0000-000000000002', 'overtime:request:create_on_behalf'),
    ('a0000000-0000-0000-0000-000000000002', 'overtime:request:read'),
    ('a0000000-0000-0000-0000-000000000002', 'patterns:assign'),
    ('a0000000-0000-0000-0000-000000000002', 'patterns:create'),
    ('a0000000-0000-0000-0000-000000000002', 'patterns:delete'),
    ('a0000000-0000-0000-0000-000000000002', 'patterns:read'),
    ('a0000000-0000-0000-0000-000000000002', 'payroll:approve'),
    ('a0000000-0000-0000-0000-000000000002', 'payroll:calculate'),
    ('a0000000-0000-0000-0000-000000000002', 'payroll:components:manage'),
    ('a0000000-0000-0000-0000-000000000002', 'payroll:incentives:manage'),
    ('a0000000-0000-0000-0000-000000000002', 'payroll:read'),
    ('a0000000-0000-0000-0000-000000000002', 'payroll:sensitive:read'),
    ('a0000000-0000-0000-0000-000000000002', 'payroll:settings:manage'),
    ('a0000000-0000-0000-0000-000000000002', 'recruitment:manage'),
    ('a0000000-0000-0000-0000-000000000002', 'recruitment:read'),
    ('a0000000-0000-0000-0000-000000000002', 'reports:export'),
    ('a0000000-0000-0000-0000-000000000002', 'reports:read'),
    ('a0000000-0000-0000-0000-000000000002', 'rosters:manage'),
    ('a0000000-0000-0000-0000-000000000002', 'rosters:read'),
    ('a0000000-0000-0000-0000-000000000002', 'security:settings:manage'),
    ('a0000000-0000-0000-0000-000000000002', 'shift_swaps:approve'),
    ('a0000000-0000-0000-0000-000000000002', 'shift_swaps:manage'),
    ('a0000000-0000-0000-0000-000000000002', 'shifts:assign'),
    ('a0000000-0000-0000-0000-000000000002', 'shifts:create'),
    ('a0000000-0000-0000-0000-000000000002', 'shifts:read'),
    ('a0000000-0000-0000-0000-000000000002', 'shifts:update'),
    ('a0000000-0000-0000-0000-000000000002', 'workspace:settings:read'),
    ('a0000000-0000-0000-0000-000000000002', 'workspace:settings:write'),
    ('a0000000-0000-0000-0000-000000000002', 'payroll:disburse'),
    ('a0000000-0000-0000-0000-000000000002', 'workspace:update'),
    ('a0000000-0000-0000-0000-000000000003', 'approval_workflows:read'),
    ('a0000000-0000-0000-0000-000000000003', 'attendance:adjustments:approve'),
    ('a0000000-0000-0000-0000-000000000003', 'attendance:adjustments:manage_behalf'),
    ('a0000000-0000-0000-0000-000000000003', 'attendance:adjustments:submit_on_behalf'),
    ('a0000000-0000-0000-0000-000000000003', 'attendance:logs:read'),
    ('a0000000-0000-0000-0000-000000000003', 'attendance:summaries:read'),
    ('a0000000-0000-0000-0000-000000000003', 'dashboard:employee_absence:read'),
    ('a0000000-0000-0000-0000-000000000003', 'dashboard:read'),
    ('a0000000-0000-0000-0000-000000000003', 'departments:read'),
    ('a0000000-0000-0000-0000-000000000003', 'documents:contract:read'),
    ('a0000000-0000-0000-0000-000000000003', 'employees:bulk_edit'),
    ('a0000000-0000-0000-0000-000000000003', 'employees:create'),
    ('a0000000-0000-0000-0000-000000000003', 'employees:read'),
    ('a0000000-0000-0000-0000-000000000003', 'employees:update'),
    ('a0000000-0000-0000-0000-000000000003', 'employment_phases:manage'),
    ('a0000000-0000-0000-0000-000000000003', 'employment_phases:read'),
    ('a0000000-0000-0000-0000-000000000003', 'holidays:read'),
    ('a0000000-0000-0000-0000-000000000003', 'leave:approve:l2'),
    ('a0000000-0000-0000-0000-000000000003', 'leave:requests:approve'),
    ('a0000000-0000-0000-0000-000000000003', 'leave:requests:read'),
    ('a0000000-0000-0000-0000-000000000003', 'leave:requests:submit'),
    ('a0000000-0000-0000-0000-000000000003', 'leave:requests:submit_on_behalf'),
    ('a0000000-0000-0000-0000-000000000003', 'overtime:approve'),
    ('a0000000-0000-0000-0000-000000000003', 'overtime:approve:workflow'),
    ('a0000000-0000-0000-0000-000000000003', 'overtime:claim'),
    ('a0000000-0000-0000-0000-000000000003', 'overtime:claim_on_behalf'),
    ('a0000000-0000-0000-0000-000000000003', 'overtime:paper:upload'),
    ('a0000000-0000-0000-0000-000000000003', 'overtime:policies:manage'),
    ('a0000000-0000-0000-0000-000000000003', 'overtime:quota:read'),
    ('a0000000-0000-0000-0000-000000000003', 'overtime:request:create'),
    ('a0000000-0000-0000-0000-000000000003', 'overtime:request:create_on_behalf'),
    ('a0000000-0000-0000-0000-000000000003', 'overtime:request:read'),
    ('a0000000-0000-0000-0000-000000000003', 'payroll:approve'),
    ('a0000000-0000-0000-0000-000000000003', 'payroll:read'),
    ('a0000000-0000-0000-0000-000000000003', 'payroll:sensitive:read'),
    ('a0000000-0000-0000-0000-000000000004', 'attendance:logs:read'),
    ('a0000000-0000-0000-0000-000000000004', 'employees:read'),
    ('a0000000-0000-0000-0000-000000000004', 'employment_phases:read'),
    ('a0000000-0000-0000-0000-000000000004', 'overtime:approve'),
    ('a0000000-0000-0000-0000-000000000004', 'overtime:claim'),
    ('a0000000-0000-0000-0000-000000000004', 'overtime:request:create'),
    ('a0000000-0000-0000-0000-000000000004', 'overtime:request:create_on_behalf'),
    ('a0000000-0000-0000-0000-000000000004', 'overtime:request:read'),
    ('a0000000-0000-0000-0000-000000000004', 'rosters:manage'),
    ('a0000000-0000-0000-0000-000000000004', 'rosters:read'),
    ('a0000000-0000-0000-0000-000000000004', 'shifts:assign'),
    ('a0000000-0000-0000-0000-000000000004', 'shifts:create'),
    ('a0000000-0000-0000-0000-000000000004', 'shifts:read'),
    ('a0000000-0000-0000-0000-000000000004', 'shifts:update'),
    ('a0000000-0000-0000-0000-000000000005', 'attendance:summaries:read'),
    ('a0000000-0000-0000-0000-000000000005', 'employees:read'),
    ('a0000000-0000-0000-0000-000000000005', 'leave:approve:l1'),
    ('a0000000-0000-0000-0000-000000000005', 'leave:requests:approve'),
    ('a0000000-0000-0000-0000-000000000005', 'leave:requests:read'),
    ('a0000000-0000-0000-0000-000000000005', 'leave:requests:submit'),
    ('a0000000-0000-0000-0000-000000000005', 'overtime:approve'),
    ('a0000000-0000-0000-0000-000000000005', 'overtime:claim'),
    ('a0000000-0000-0000-0000-000000000005', 'overtime:request:create'),
    ('a0000000-0000-0000-0000-000000000005', 'overtime:request:create_on_behalf'),
    ('a0000000-0000-0000-0000-000000000005', 'overtime:request:read'),
    ('a0000000-0000-0000-0000-000000000006', 'ess:attendance'),
    ('a0000000-0000-0000-0000-000000000006', 'ess:clock'),
    ('a0000000-0000-0000-0000-000000000006', 'ess:leave'),
    ('a0000000-0000-0000-0000-000000000006', 'ess:payslips'),
    ('a0000000-0000-0000-0000-000000000006', 'ess:profile'),
    ('a0000000-0000-0000-0000-000000000006', 'ess:shift_swaps'),
    ('a0000000-0000-0000-0000-000000000006', 'overtime:claim'),
    ('a0000000-0000-0000-0000-000000000006', 'overtime:request:create'),
    ('a0000000-0000-0000-0000-000000000006', 'overtime:request:read')
ON CONFLICT DO NOTHING;
`

const seedSalaryBaseGroupsSQL = `
DO $$
BEGIN
    IF EXISTS (SELECT FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'salary_base_groups') THEN
        INSERT INTO public.salary_base_groups (id, workspace_id, name, slug, description, is_system) VALUES
            ('b0000000-0000-0000-0000-000000000001', NULL, 'Gross Salary', 'gross', 'Base for all basic, taxable, and insurable income components', TRUE),
            ('b0000000-0000-0000-0000-000000000002', NULL, 'Net Salary', 'net', 'Final take-home pay after all deductions', TRUE)
        ON CONFLICT (id) DO NOTHING;
    END IF;
END $$;
`

// EnsureDefaultRolesAndPermissions seeds the baseline platform roles, workspace template roles,
// granular permissions, and salary groups idempotently.
func EnsureDefaultRolesAndPermissions(ctx context.Context, tx dbExecutor) error {
	if _, err := tx.Exec(ctx, seedSystemRolesSQL); err != nil {
		return fmt.Errorf("standalone: seed system roles: %w", err)
	}
	if _, err := tx.Exec(ctx, seedWorkspaceRolesSQL); err != nil {
		return fmt.Errorf("standalone: seed workspace roles: %w", err)
	}
	if _, err := tx.Exec(ctx, seedWorkspaceRolePermissionsSQL); err != nil {
		return fmt.Errorf("standalone: seed role permissions: %w", err)
	}
	if _, err := tx.Exec(ctx, seedSalaryBaseGroupsSQL); err != nil {
		return fmt.Errorf("standalone: seed salary base groups: %w", err)
	}
	return nil
}
