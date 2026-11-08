package standalone

import (
	"context"
	"fmt"

	"github.com/jackc/pgx/v5/pgconn"
)

type dbExecutor interface {
	Exec(context.Context, string, ...any) (pgconn.CommandTag, error)
}

const seedOrganizationRolesSQL = `
INSERT INTO public.roles (id, name, description, color, is_system, scope_type) VALUES
    ('a0000000-0000-0000-0000-000000000002', 'admin', 'Administrator with full access', 'rose', TRUE, 'global'),
    ('a0000000-0000-0000-0000-000000000003', 'hr_manager', 'HR Manager with employee and attendance control', 'purple', TRUE, 'global'),
    ('a0000000-0000-0000-0000-000000000004', 'shift_manager', 'Shift Manager for schedule planning', 'blue', TRUE, 'global'),
    ('a0000000-0000-0000-0000-000000000005', 'team_lead', 'Team Lead for daily attendance monitoring', 'green', TRUE, 'global'),
    ('a0000000-0000-0000-0000-000000000006', 'employee', 'Standard employee with self-service access', 'gray', TRUE, 'global')
ON CONFLICT (name) DO NOTHING;
`
const seedOrganizationRolePermissionsSQL = `
INSERT INTO public.role_permissions (role_id, permission_name)
SELECT r.id, p.permission_name
FROM (VALUES
    ('admin', 'approval_workflows:manage'),
    ('admin', 'approval_workflows:read'),
    ('admin', 'assignment_rules:manage'),
    ('admin', 'attendance:adjustments:approve'),
    ('admin', 'attendance:adjustments:manage_behalf'),
    ('admin', 'attendance:adjustments:submit'),
    ('admin', 'attendance:adjustments:submit_on_behalf'),
    ('admin', 'attendance:force_clock_out'),
    ('admin', 'attendance:logs:import'),
    ('admin', 'attendance:logs:read'),
    ('admin', 'attendance:reprocess'),
    ('admin', 'attendance:staging:manage'),
    ('admin', 'attendance:summaries:read'),
    ('admin', 'audit_logs:export'),
    ('admin', 'audit_logs:read'),
    ('admin', 'dashboard:employee_absence:read'),
    ('admin', 'dashboard:preferences'),
    ('admin', 'dashboard:read'),
    ('admin', 'departments:create'),
    ('admin', 'departments:delete'),
    ('admin', 'departments:read'),
    ('admin', 'departments:transfer'),
    ('admin', 'departments:update'),
    ('admin', 'documents:contract:read'),
    ('admin', 'documents:medical:read'),
    ('admin', 'employees:bulk_edit'),
    ('admin', 'employees:create'),
    ('admin', 'employees:delete'),
    ('admin', 'employees:read'),
    ('admin', 'employees:rehire'),
    ('admin', 'employees:update'),
    ('admin', 'employees:upload'),
    ('admin', 'employment_phases:manage'),
    ('admin', 'employment_phases:read'),
    ('admin', 'ess:attendance'),
    ('admin', 'ess:clock'),
    ('admin', 'ess:leave'),
    ('admin', 'ess:payslips'),
    ('admin', 'ess:profile'),
    ('admin', 'files:manage'),
    ('admin', 'files:read'),
    ('admin', 'holidays:create'),
    ('admin', 'holidays:delete'),
    ('admin', 'holidays:read'),
    ('admin', 'holidays:update'),
    ('admin', 'leave_types:manage'),
    ('admin', 'leave_types:read'),
    ('admin', 'leaves:approve'),
    ('admin', 'leaves:assign'),
    ('admin', 'leaves:balance:read'),
    ('admin', 'leaves:cancel'),
    ('admin', 'leaves:import'),
    ('admin', 'leaves:read'),
    ('admin', 'leaves:submit'),
    ('admin', 'lifecycle:manage'),
    ('admin', 'lifecycle:read'),
    ('admin', 'members:invite'),
    ('admin', 'members:read'),
    ('admin', 'members:remove'),
    ('admin', 'members:roles:manage'),
    ('admin', 'organization:settings:read'),
    ('admin', 'organization:settings:write'),
    ('admin', 'overtime:approve'),
    ('admin', 'overtime:assign'),
    ('admin', 'overtime:read'),
    ('admin', 'overtime:request:create'),
    ('admin', 'overtime:request:read'),
    ('admin', 'payroll:calculate'),
    ('admin', 'payroll:export'),
    ('admin', 'payroll:manage'),
    ('admin', 'payroll:read'),
    ('admin', 'reports:create'),
    ('admin', 'reports:read'),
    ('admin', 'roles:create'),
    ('admin', 'roles:delete'),
    ('admin', 'roles:read'),
    ('admin', 'roles:update'),
    ('admin', 'rosters:assign'),
    ('admin', 'rosters:delete'),
    ('admin', 'rosters:read'),
    ('admin', 'schedules:read'),
    ('admin', 'shift_swaps:approve'),
    ('admin', 'shift_swaps:create'),
    ('admin', 'shift_swaps:read'),
    ('admin', 'shifts:create'),
    ('admin', 'shifts:delete'),
    ('admin', 'shifts:read'),
    ('admin', 'shifts:update'),
    ('admin', 'working_patterns:create'),
    ('admin', 'working_patterns:delete'),
    ('admin', 'working_patterns:read'),
    ('admin', 'working_patterns:update'),
    ('hr_manager', 'approval_workflows:read'),
    ('hr_manager', 'attendance:adjustments:approve'),
    ('hr_manager', 'attendance:adjustments:manage_behalf'),
    ('hr_manager', 'attendance:adjustments:submit'),
    ('hr_manager', 'attendance:logs:import'),
    ('hr_manager', 'attendance:logs:read'),
    ('hr_manager', 'attendance:summaries:read'),
    ('hr_manager', 'dashboard:employee_absence:read'),
    ('hr_manager', 'dashboard:preferences'),
    ('hr_manager', 'dashboard:read'),
    ('hr_manager', 'departments:read'),
    ('hr_manager', 'documents:contract:read'),
    ('hr_manager', 'documents:medical:read'),
    ('hr_manager', 'employees:bulk_edit'),
    ('hr_manager', 'employees:create'),
    ('hr_manager', 'employees:read'),
    ('hr_manager', 'employees:update'),
    ('hr_manager', 'employees:upload'),
    ('hr_manager', 'employment_phases:read'),
    ('hr_manager', 'ess:attendance'),
    ('hr_manager', 'ess:clock'),
    ('hr_manager', 'ess:leave'),
    ('hr_manager', 'ess:payslips'),
    ('hr_manager', 'ess:profile'),
    ('hr_manager', 'files:read'),
    ('hr_manager', 'holidays:read'),
    ('hr_manager', 'leave_types:read'),
    ('hr_manager', 'leaves:approve'),
    ('hr_manager', 'leaves:assign'),
    ('hr_manager', 'leaves:balance:read'),
    ('hr_manager', 'leaves:cancel'),
    ('hr_manager', 'leaves:import'),
    ('hr_manager', 'leaves:read'),
    ('hr_manager', 'leaves:submit'),
    ('hr_manager', 'lifecycle:manage'),
    ('hr_manager', 'lifecycle:read'),
    ('hr_manager', 'members:read'),
    ('hr_manager', 'overtime:approve'),
    ('hr_manager', 'overtime:assign'),
    ('hr_manager', 'overtime:read'),
    ('hr_manager', 'payroll:calculate'),
    ('hr_manager', 'payroll:export'),
    ('hr_manager', 'payroll:manage'),
    ('hr_manager', 'payroll:read'),
    ('hr_manager', 'reports:create'),
    ('hr_manager', 'reports:read'),
    ('hr_manager', 'rosters:read'),
    ('hr_manager', 'schedules:read'),
    ('hr_manager', 'shift_swaps:read'),
    ('hr_manager', 'shifts:read'),
    ('hr_manager', 'working_patterns:read'),
    ('shift_manager', 'attendance:adjustments:submit'),
    ('shift_manager', 'attendance:logs:read'),
    ('shift_manager', 'attendance:summaries:read'),
    ('shift_manager', 'dashboard:preferences'),
    ('shift_manager', 'dashboard:read'),
    ('shift_manager', 'departments:read'),
    ('shift_manager', 'employees:read'),
    ('shift_manager', 'ess:attendance'),
    ('shift_manager', 'ess:clock'),
    ('shift_manager', 'ess:leave'),
    ('shift_manager', 'ess:payslips'),
    ('shift_manager', 'ess:profile'),
    ('shift_manager', 'holidays:read'),
    ('shift_manager', 'leaves:read'),
    ('shift_manager', 'rosters:assign'),
    ('shift_manager', 'rosters:delete'),
    ('shift_manager', 'rosters:read'),
    ('shift_manager', 'schedules:read'),
    ('shift_manager', 'shift_swaps:approve'),
    ('shift_manager', 'shift_swaps:create'),
    ('shift_manager', 'shift_swaps:read'),
    ('shift_manager', 'shifts:create'),
    ('shift_manager', 'shifts:delete'),
    ('shift_manager', 'shifts:read'),
    ('shift_manager', 'shifts:update'),
    ('shift_manager', 'working_patterns:create'),
    ('shift_manager', 'working_patterns:delete'),
    ('shift_manager', 'working_patterns:read'),
    ('shift_manager', 'working_patterns:update'),
    ('team_lead', 'attendance:adjustments:approve'),
    ('team_lead', 'attendance:adjustments:submit'),
    ('team_lead', 'attendance:logs:read'),
    ('team_lead', 'attendance:summaries:read'),
    ('team_lead', 'dashboard:preferences'),
    ('team_lead', 'dashboard:read'),
    ('team_lead', 'departments:read'),
    ('team_lead', 'employees:read'),
    ('team_lead', 'ess:attendance'),
    ('team_lead', 'ess:clock'),
    ('team_lead', 'ess:leave'),
    ('team_lead', 'ess:payslips'),
    ('team_lead', 'ess:profile'),
    ('team_lead', 'holidays:read'),
    ('team_lead', 'leaves:read'),
    ('team_lead', 'overtime:approve'),
    ('team_lead', 'overtime:read'),
    ('team_lead', 'rosters:read'),
    ('team_lead', 'schedules:read'),
    ('team_lead', 'shift_swaps:read'),
    ('team_lead', 'shifts:read'),
    ('employee', 'attendance:adjustments:submit'),
    ('employee', 'ess:attendance'),
    ('employee', 'ess:clock'),
    ('employee', 'ess:leave'),
    ('employee', 'ess:payslips'),
    ('employee', 'ess:profile'),
    ('employee', 'holidays:read'),
    ('employee', 'leaves:submit'),
    ('employee', 'overtime:request:create'),
    ('employee', 'overtime:request:read')
) AS p(role_name, permission_name)
JOIN public.roles r ON r.name = p.role_name
ON CONFLICT (role_id, permission_name) DO NOTHING;
`

const seedSalaryBaseGroupsSQL = `
DO $$
BEGIN
    IF EXISTS (SELECT FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'salary_base_groups') THEN
        INSERT INTO public.salary_base_groups (id, name, slug, description, is_system) VALUES
            ('b0000000-0000-0000-0000-000000000001', 'Gross Salary', 'gross', 'Base for all basic, taxable, and insurable income components', TRUE),
            ('b0000000-0000-0000-0000-000000000002', 'Net Salary', 'net', 'Final take-home pay after all deductions', TRUE)
        ON CONFLICT (id) DO NOTHING;
    END IF;
END $$;
`

// EnsureDefaultRolesAndPermissions seeds baseline organization roles, granular permissions,
// and salary groups idempotently.
func EnsureDefaultRolesAndPermissions(ctx context.Context, tx dbExecutor) error {
	if _, err := tx.Exec(ctx, seedOrganizationRolesSQL); err != nil {
		return fmt.Errorf("standalone: seed roles: %w", err)
	}
	if _, err := tx.Exec(ctx, seedOrganizationRolePermissionsSQL); err != nil {
		return fmt.Errorf("standalone: seed role permissions: %w", err)
	}
	if _, err := tx.Exec(ctx, seedSalaryBaseGroupsSQL); err != nil {
		return fmt.Errorf("standalone: seed salary base groups: %w", err)
	}
	return nil
}
