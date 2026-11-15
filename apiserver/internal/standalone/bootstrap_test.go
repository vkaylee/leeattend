package standalone

import (
	"context"
	"errors"
	"fmt"
	"regexp"
	"strings"
	"testing"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
	"github.com/vkaylee/leeattend/apiserver/internal/auth"
)

type bootstrapRow struct {
	value any
	err   error
}

func (r bootstrapRow) Scan(dest ...any) error {
	if r.err != nil {
		return r.err
	}
	switch target := dest[0].(type) {
	case *int:
		*target = r.value.(int)
	case *uuid.UUID:
		*target = r.value.(uuid.UUID)
	default:
		return pgx.ErrNoRows
	}
	return nil
}

type bootstrapTx struct {
	settingsCount int
	userID        uuid.UUID
	userExists    bool
	roleID        uuid.UUID
	commits       int
	executed      []string
}

func (tx *bootstrapTx) Begin(context.Context) (pgx.Tx, error) { return tx, nil }
func (tx *bootstrapTx) Commit(context.Context) error          { tx.commits++; return nil }
func (tx *bootstrapTx) Rollback(context.Context) error        { return nil }
func (tx *bootstrapTx) Exec(_ context.Context, sql string, _ ...any) (pgconn.CommandTag, error) {
	tx.executed = append(tx.executed, sql)
	if strings.Contains(sql, "INSERT INTO organization_settings") {
		tx.settingsCount = 1
	}
	if strings.Contains(sql, "INSERT INTO users") {
		tx.userExists = true
		tx.userID = uuid.New()
	}
	return pgconn.NewCommandTag("INSERT 0 1"), nil
}
func (tx *bootstrapTx) Query(context.Context, string, ...any) (pgx.Rows, error) { return nil, nil }
func (tx *bootstrapTx) QueryRow(_ context.Context, sql string, _ ...any) pgx.Row {
	switch {
	case strings.Contains(sql, "count(*) FROM organization_settings"):
		return bootstrapRow{value: tx.settingsCount}
	case strings.Contains(sql, "SELECT id FROM users"):
		if !tx.userExists {
			return bootstrapRow{err: pgx.ErrNoRows}
		}
		return bootstrapRow{value: tx.userID}
	case strings.Contains(sql, "SELECT id FROM roles"):
		return bootstrapRow{value: tx.roleID}
	default:
		return bootstrapRow{err: pgx.ErrNoRows}
	}
}
func (tx *bootstrapTx) Conn() *pgx.Conn { return nil }
func (tx *bootstrapTx) CopyFrom(context.Context, pgx.Identifier, []string, pgx.CopyFromSource) (int64, error) {
	return 0, nil
}
func (tx *bootstrapTx) SendBatch(context.Context, *pgx.Batch) pgx.BatchResults { return nil }
func (tx *bootstrapTx) LargeObjects() pgx.LargeObjects                         { return pgx.LargeObjects{} }
func (tx *bootstrapTx) Prepare(context.Context, string, string) (*pgconn.StatementDescription, error) {
	return nil, nil
}

func TestBootstrapInitializesSharedSchemaTransactionIdempotently(t *testing.T) {
	tx := &bootstrapTx{roleID: uuid.New()}
	if err := Bootstrap(context.Background(), tx, "Example Company", "admin@example.invalid", "password"); err != nil {
		t.Fatalf("first bootstrap failed: %v", err)
	}
	if err := Bootstrap(context.Background(), tx, "Example Company", "admin@example.invalid", "password"); err != nil {
		t.Fatalf("second bootstrap failed: %v", err)
	}
	if tx.commits != 2 {
		t.Fatalf("expected two committed bootstrap transactions, got %d", tx.commits)
	}
	settingsInserts, userInserts, formulaRules, formulaVersions := 0, 0, 0, 0
	for _, sql := range tx.executed {
		if strings.Contains(sql, "INSERT INTO organization_settings") {
			settingsInserts++
		}
		if strings.Contains(sql, "INSERT INTO users") {
			userInserts++
		}
		if strings.Contains(sql, "INSERT INTO payroll_formula_rules") {
			formulaRules++
		}
		if strings.Contains(sql, "INSERT INTO payroll_formula_rule_versions") {
			formulaVersions++
		}
	}
	if settingsInserts != 1 || userInserts != 1 || formulaRules != 2 || formulaVersions != 2 {
		t.Fatalf("bootstrap was not idempotent: settings=%d users=%d formula_rules=%d formula_versions=%d", settingsInserts, userInserts, formulaRules, formulaVersions)
	}
}

func TestBootstrapRejectsMultipleOrganizationSettings(t *testing.T) {
	tx := &bootstrapTx{settingsCount: 2, roleID: uuid.New()}
	if err := Bootstrap(context.Background(), tx, "Example Company", "admin@example.invalid", "password"); err == nil {
		t.Fatal("expected bootstrap to reject multiple organization settings rows")
	} else if !errors.Is(err, ErrInvalidBootstrap) {
		t.Fatalf("expected invalid bootstrap error, got %v", err)
	}
	if tx.commits != 0 {
		t.Fatalf("expected conflicting bootstrap to avoid commit, got %d", tx.commits)
	}
}

func TestEnsureDefaultRolesAndPermissions(t *testing.T) {
	tx := &bootstrapTx{}
	if err := EnsureDefaultRolesAndPermissions(context.Background(), tx); err != nil {
		t.Fatalf("expected EnsureDefaultRolesAndPermissions to succeed, got %v", err)
	}
	hasRoles, hasPermissions, hasSalaryGroups := false, false, false
	for _, sql := range tx.executed {
		if strings.Contains(sql, "INSERT INTO public.roles") {
			hasRoles = true
		}
		if strings.Contains(sql, "INSERT INTO public.role_permissions") {
			hasPermissions = true
		}
		if strings.Contains(sql, "salary_base_groups") {
			hasSalaryGroups = true
		}
	}
	if !hasRoles || !hasPermissions || hasSalaryGroups {
		t.Fatalf("unexpected catalog seed statements: roles=%v perms=%v retired_salary_groups=%v", hasRoles, hasPermissions, hasSalaryGroups)
	}

	roleSeed := tx.executed[0]
	for _, expected := range []string{
		"'admin', 'Administrator with full access', 'rose', TRUE, 'global'",
		"'hr_manager', 'HR Manager with employee and attendance control', 'purple', TRUE, 'global'",
		"'shift_manager', 'Shift Manager for schedule planning', 'blue', TRUE, 'global'",
		"'team_lead', 'Team Lead for daily attendance monitoring', 'green', TRUE, 'scoped'",
		"'employee', 'Standard employee with self-service access', 'gray', TRUE, 'scoped'",
	} {
		if !strings.Contains(roleSeed, expected) {
			t.Errorf("role seed missing expected scope definition %q", expected)
		}
	}

	adminSeed := tx.executed[1]
	for _, permission := range auth.AllPermissions() {
		grant := fmt.Sprintf("('admin', '%s')", permission)
		if !strings.Contains(adminSeed, grant) {
			t.Errorf("admin seed missing canonical permission %q", permission)
		}
	}

	for _, grant := range []string{
		"('hr_manager', 'schedules:activate')",
		"('admin', 'schedules:activate')",
		"('hr_manager', 'attendance:closure:close')",
		"('admin', 'attendance:closure:reopen')",
		"('shift_manager', 'schedules:publish:rotating')",
		"('team_lead', 'schedules:read')",
		"('employee', 'schedules:read')",
	} {
		if !strings.Contains(adminSeed, grant) {
			t.Errorf("role seed missing allowed lifecycle grant %q", grant)
		}
	}
	for _, grant := range []string{
		"('shift_manager', 'schedules:activate')",
		"('team_lead', 'schedules:activate')",
		"('employee', 'schedules:activate')",
		"('shift_manager', 'attendance:closure:close')",
		"('team_lead', 'attendance:closure:close')",
		"('employee', 'attendance:closure:close')",
		"('shift_manager', 'schedules:publish:office')",
		"('team_lead', 'schedules:publish:rotating')",
	} {
		if strings.Contains(adminSeed, grant) {
			t.Errorf("role seed contains forbidden lifecycle grant %q", grant)
		}
	}

	grants := regexp.MustCompile(`\('([^']+)', '([^']+)'\)`).FindAllStringSubmatch(adminSeed, -1)
	seen := make(map[string]bool, len(grants))
	for _, grant := range grants {
		key := grant[1] + ":" + grant[2]
		if seen[key] {
			t.Errorf("duplicate role permission seed %q", key)
		}
		seen[key] = true
	}
}
