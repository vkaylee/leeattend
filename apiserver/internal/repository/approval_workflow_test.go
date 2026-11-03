package repository

import (
	"context"
	"strings"
	"testing"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
)

type fakeRows struct {
	rows [][]any
	idx  int
}

func (f *fakeRows) Close()                                       {}
func (f *fakeRows) Err() error                                   { return nil }
func (f *fakeRows) CommandTag() pgconn.CommandTag                { return pgconn.CommandTag{} }
func (f *fakeRows) FieldDescriptions() []pgconn.FieldDescription { return nil }
func (f *fakeRows) Next() bool {
	if f.idx < len(f.rows) {
		f.idx++
		return true
	}
	return false
}
func (f *fakeRows) Scan(dest ...any) error {
	row := f.rows[f.idx-1]
	for i, v := range row {
		switch d := dest[i].(type) {
		case *uuid.UUID:
			*d = v.(uuid.UUID)
		case *string:
			*d = v.(string)
		case *int:
			*d = v.(int)
		}
	}
	return nil
}
func (f *fakeRows) Values() ([]any, error) { return nil, nil }
func (f *fakeRows) RawValues() [][]byte    { return nil }
func (f *fakeRows) Conn() *pgx.Conn        { return nil }

func TestApprovalWorkflowRepository_GetPendingTasksForUser_SystemDBRoles(t *testing.T) {
	wsID := uuid.New()
	userID := uuid.New()
	roleID := uuid.New()

	systemQueried := false
	systemDB := &fakeDBTX{
		queryFn: func(_ context.Context, sql string, _ ...any) (pgx.Rows, error) {
			if strings.Contains(sql, "FROM workspace_members") && strings.Contains(sql, "member_extra_roles") {
				systemQueried = true
				return &fakeRows{rows: [][]any{{roleID}}}, nil
			}
			return &fakeRows{}, nil
		},
	}

	tenantQueried := false
	tenantDB := &fakeDBTX{
		queryRowFn: func(_ context.Context, sql string, args ...any) pgx.Row {
			if strings.Contains(sql, "SELECT COUNT(*) FROM approval_workflow_tasks") {
				tenantQueried = true
				return fakeRow{scanFn: func(dest ...any) error {
					*(dest[0].(*int64)) = 0
					return nil
				}}
			}
			return fakeRow{}
		},
		queryFn: func(_ context.Context, _ string, _ ...any) (pgx.Rows, error) {
			return &fakeRows{}, nil
		},
	}

	repo := ApprovalWorkflowRepository{
		TenantDB: func(_ context.Context, _ uuid.UUID) (DBTX, error) {
			return tenantDB, nil
		},
		SystemDB: systemDB,
	}

	res, err := repo.GetPendingTasksForUser(context.Background(), wsID, userID, 1, 50)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if !systemQueried {
		t.Fatalf("expected SystemDB to be queried for workspace_members and member_extra_roles")
	}
	if !tenantQueried {
		t.Fatalf("expected TenantDB to be queried for pending approval tasks")
	}
	if res.Page != 1 || res.Limit != 50 {
		t.Fatalf("expected page=1, limit=50, got page=%d, limit=%d", res.Page, res.Limit)
	}
}
