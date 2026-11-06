package standalone

import (
	"context"
	"strings"
	"testing"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
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
	workspaceID    uuid.UUID
	workspaceCount int
	userID         uuid.UUID
	userExists     bool
	roleID         uuid.UUID
	commits        int
	executed       []string
}

func (tx *bootstrapTx) Begin(context.Context) (pgx.Tx, error) { return tx, nil }
func (tx *bootstrapTx) Commit(context.Context) error          { tx.commits++; return nil }
func (tx *bootstrapTx) Rollback(context.Context) error        { return nil }
func (tx *bootstrapTx) Exec(_ context.Context, sql string, _ ...any) (pgconn.CommandTag, error) {
	tx.executed = append(tx.executed, sql)
	if strings.Contains(sql, "INSERT INTO workspaces") {
		tx.workspaceCount = 1
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
	case strings.Contains(sql, "count(*) FROM workspaces"):
		return bootstrapRow{value: tx.workspaceCount}
	case strings.Contains(sql, "SELECT id FROM workspaces"):
		return bootstrapRow{value: tx.workspaceID}
	case strings.Contains(sql, "SELECT id FROM users"):
		if !tx.userExists {
			return bootstrapRow{err: pgx.ErrNoRows}
		}
		return bootstrapRow{value: tx.userID}
	case strings.Contains(sql, "SELECT id FROM workspace_roles"):
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
	workspaceInserts, userInserts := 0, 0
	for _, sql := range tx.executed {
		if strings.Contains(sql, "INSERT INTO workspaces") {
			workspaceInserts++
		}
		if strings.Contains(sql, "INSERT INTO users") {
			userInserts++
		}
	}
	if workspaceInserts != 1 || userInserts != 1 {
		t.Fatalf("bootstrap was not idempotent: workspaces=%d users=%d", workspaceInserts, userInserts)
	}
}

func TestEnsureDefaultRolesAndPermissions(t *testing.T) {
	tx := &bootstrapTx{}
	if err := EnsureDefaultRolesAndPermissions(context.Background(), tx); err != nil {
		t.Fatalf("expected EnsureDefaultRolesAndPermissions to succeed, got %v", err)
	}
	hasWorkspaceRoles, hasPermissions, hasSalaryGroups := false, false, false
	for _, sql := range tx.executed {
		if strings.Contains(sql, "INSERT INTO public.workspace_roles") {
			hasWorkspaceRoles = true
		}
		if strings.Contains(sql, "INSERT INTO workspace_role_permissions") {
			hasPermissions = true
		}
		if strings.Contains(sql, "salary_base_groups") {
			hasSalaryGroups = true
		}
	}
	if !hasWorkspaceRoles || !hasPermissions || !hasSalaryGroups {
		t.Fatalf("missing catalog seed statements: workspace=%v perms=%v salary=%v", hasWorkspaceRoles, hasPermissions, hasSalaryGroups)
	}
}
