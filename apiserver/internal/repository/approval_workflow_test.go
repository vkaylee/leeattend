package repository

import (
	"context"
	"strings"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/ids"
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
	userID := uuid.New()
	roleID := uuid.New()

	systemQueried := false
	systemDB := &fakeDBTX{
		queryFn: func(_ context.Context, sql string, _ ...any) (pgx.Rows, error) {
			if strings.Contains(sql, "FROM user_roles") && strings.Contains(sql, "user_extra_roles") {
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
				if strings.Contains(sql, "workspace_id") {
					t.Fatalf("query must not contain workspace_id: %s", sql)
				}
				tenantQueried = true
				return fakeRow{scanFn: func(dest ...any) error {
					*(dest[0].(*int64)) = 0
					return nil
				}}
			}
			return fakeRow{}
		},
		queryFn: func(_ context.Context, sql string, _ ...any) (pgx.Rows, error) {
			if strings.Contains(sql, "workspace_id") {
				t.Fatalf("query must not contain workspace_id: %s", sql)
			}
			return &fakeRows{}, nil
		},
	}

	repo := ApprovalWorkflowRepository{
		DB:       tenantDB,
		SystemDB: systemDB,
	}

	res, err := repo.GetPendingTasksForUser(context.Background(), ids.UserID(userID), 1, 50)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if !systemQueried {
		t.Fatalf("expected SystemDB to be queried for user_roles and user_extra_roles")
	}
	if !tenantQueried {
		t.Fatalf("expected TenantDB to be queried for pending approval tasks")
	}
	if res.Page != 1 || res.Limit != 50 {
		t.Fatalf("expected page=1, limit=50, got page=%d, limit=%d", res.Page, res.Limit)
	}
}

func TestApprovalWorkflowRepository_ProcessApproval_PreservesLockAndState(t *testing.T) {
	actorID := uuid.New()
	entityID := uuid.New()
	instanceID := uuid.New()
	stepOrder := 1
	stepName := "Initial Review"

	lockChecked := false
	logInserted := false
	instanceUpdated := false

	tx := &fakeTx{
		fakeDBTX: fakeDBTX{
			queryRowFn: func(_ context.Context, sql string, args ...any) pgx.Row {
				if strings.Contains(sql, "FOR UPDATE") && strings.Contains(sql, "approval_workflow_instances") {
					if strings.Contains(sql, "workspace_id") {
						t.Fatalf("lock query must not reference workspace_id: %s", sql)
					}
					lockChecked = true
					return fakeRow{scanFn: func(dest ...any) error {
						*(dest[0].(*uuid.UUID)) = instanceID
						*(dest[1].(*int)) = stepOrder
						*(dest[2].(**string)) = &stepName
						return nil
					}}
				}
				return fakeRow{}
			},
			execFn: func(_ context.Context, sql string, args ...any) (pgconn.CommandTag, error) {
				if strings.Contains(sql, "INSERT INTO approval_action_log") {
					if strings.Contains(sql, "workspace_id") {
						t.Fatalf("log insert must not reference workspace_id: %s", sql)
					}
					logInserted = true
				}
				if strings.Contains(sql, "UPDATE approval_workflow_instances SET status = $1") {
					if args[0] != "approved" {
						t.Fatalf("expected status approved, got %v", args[0])
					}
					instanceUpdated = true
				}
				return pgconn.NewCommandTag("UPDATE 1"), nil
			},
		},
	}

	repo := ApprovalWorkflowRepository{
		BeginTx: func(context.Context) (TxBeginner, error) {
			return fakeTxBeginner{tx: tx}, nil
		},
	}

	err := repo.ProcessApproval(context.Background(), ids.UserID(actorID), api.ProcessApprovalRequest{
		EntityType: "leave_request",
		EntityID:   entityID,
		Action:     "approve",
	})
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if !lockChecked {
		t.Fatalf("expected row lock check with FOR UPDATE")
	}
	if !logInserted {
		t.Fatalf("expected approval action log inserted")
	}
	if !instanceUpdated {
		t.Fatalf("expected approval workflow instance status updated")
	}
}

func TestApprovalWorkflowRepository_CreateAndGetWorkflow(t *testing.T) {
	creatorID := uuid.New()
	workflowID := uuid.New()
	now := time.Now().UTC()

	tx := &fakeTx{
		fakeDBTX: fakeDBTX{
			execFn: func(_ context.Context, sql string, args ...any) (pgconn.CommandTag, error) {
				if strings.Contains(sql, "workspace_id") {
					t.Fatalf("insert must not reference workspace_id: %s", sql)
				}
				return pgconn.NewCommandTag("INSERT 1"), nil
			},
		},
	}

	getDB := &fakeDBTX{
		queryRowFn: func(_ context.Context, sql string, args ...any) pgx.Row {
			if strings.Contains(sql, "workspace_id") {
				t.Fatalf("get query must not reference workspace_id: %s", sql)
			}
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*uuid.UUID)) = workflowID
				*(dest[1].(*string)) = "leave_request"
				*(dest[2].(*string)) = "Standard Leave"
				*(dest[3].(**string)) = nil
				*(dest[4].(*int)) = 1
				*(dest[5].(*bool)) = true
				*(dest[6].(**uuid.UUID)) = &creatorID
				*(dest[7].(*time.Time)) = now
				*(dest[8].(*time.Time)) = now
				return nil
			}}
		},
		queryFn: func(_ context.Context, sql string, args ...any) (pgx.Rows, error) {
			return &fakeRows{}, nil
		},
	}

	repo := ApprovalWorkflowRepository{
		DB: getDB,
		BeginTx: func(context.Context) (TxBeginner, error) {
			return fakeTxBeginner{tx: tx}, nil
		},
	}

	created, err := repo.CreateWorkflow(context.Background(), creatorID, api.CreateWorkflowRequest{
		EntityType: "leave_request",
		Name:       "Standard Leave",
		IsActive:   true,
		Steps: []api.WorkflowStepRequest{
			{
				StepOrder:      1,
				StepName:       "Manager",
				ApproverType:   "manager",
				ActionRequired: "approve",
			},
		},
	})
	if err != nil {
		t.Fatalf("unexpected create error: %v", err)
	}
	if created.Name != "Standard Leave" || len(created.Steps) != 1 {
		t.Fatalf("unexpected created workflow: %+v", created)
	}

	got, err := repo.GetWorkflow(context.Background(), workflowID)
	if err != nil {
		t.Fatalf("unexpected get error: %v", err)
	}
	if got.ID != workflowID || got.Name != "Standard Leave" {
		t.Fatalf("unexpected got workflow: %+v", got)
	}
}
