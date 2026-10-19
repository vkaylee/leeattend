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
)

func TestAttendancePolicyRepository_GetDefaultPolicyRules_Fallback(t *testing.T) {
	wsID := uuid.New()
	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, sql string, _ ...any) pgx.Row {
			if strings.Contains(sql, "FROM attendance_policy_versions") {
				return fakeRow{scanFn: func(_ ...any) error {
					return pgx.ErrNoRows
				}}
			}
			return fakeRow{}
		},
	}
	repo := AttendancePolicyRepository{
		TenantDB: func(_ context.Context, _ uuid.UUID) (DBTX, error) {
			return db, nil
		},
	}

	rules, err := repo.GetDefaultPolicyRules(context.Background(), wsID)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if rules.WorkUnitMinutes != 30 || rules.RoundingPolicy != "floor" || rules.EarlyArrivalStrategy != "block" {
		t.Fatalf("expected default rules fallback, got %+v", rules)
	}
}

func TestAttendancePolicyRepository_UpdateDefaultPolicyRules_Transaction(t *testing.T) {
	wsID := uuid.New()
	userID := uuid.New()
	policyID := uuid.New()

	tx := &fakeTx{}
	tx.queryRowFn = func(_ context.Context, sql string, _ ...any) pgx.Row {
		if strings.Contains(sql, "SELECT id FROM attendance_policies") {
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*uuid.UUID)) = policyID
				return nil
			}}
		}
		if strings.Contains(sql, "SELECT rules FROM attendance_policy_versions") {
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*[]byte)) = []byte(`{"work_unit_minutes": 30, "rounding_policy": "floor", "early_arrival_strategy": "block"}`)
				return nil
			}}
		}
		return fakeRow{}
	}
	tx.execFn = func(_ context.Context, _ string, _ ...any) (pgconn.CommandTag, error) {
		return pgconn.NewCommandTag("UPDATE 1"), nil
	}

	repo := AttendancePolicyRepository{
		BeginTx: func(_ context.Context, _ uuid.UUID) (TxBeginner, error) {
			return fakeTxBeginner{tx: tx}, nil
		},
	}

	newWorkUnit := 15
	newRounding := "round"
	validFrom := "2026-06-01"
	req := api.UpdateAttendanceRulesRequest{
		WorkUnitMinutes: &newWorkUnit,
		RoundingPolicy:  &newRounding,
		ValidFrom:       &validFrom,
	}

	res, err := repo.UpdateDefaultPolicyRules(context.Background(), wsID, userID, req)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if res.WorkUnitMinutes != 15 || res.RoundingPolicy != "round" {
		t.Fatalf("unexpected updated rules: %+v", res)
	}
	if !tx.committed {
		t.Fatal("expected transaction to be committed")
	}
}

func TestAttendancePolicyRepository_ListPolicyAssignments(t *testing.T) {
	wsID := uuid.New()
	assignID := uuid.New()
	targetID := uuid.New()
	now := time.Now().UTC()

	db := &fakeDBTX{
		queryFn: func(_ context.Context, _ string, _ ...any) (pgx.Rows, error) {
			return &rowsStub{
				data: [][]any{
					{assignID, "USER", targetID, now, (*time.Time)(nil), []byte(`{"work_unit_minutes": 30, "rounding_policy": "floor", "early_arrival_strategy": "block"}`)},
				},
			}, nil
		},
	}
	repo := AttendancePolicyRepository{
		TenantDB: func(_ context.Context, _ uuid.UUID) (DBTX, error) {
			return db, nil
		},
	}

	list, err := repo.ListPolicyAssignments(context.Background(), wsID)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if len(list) != 1 || list[0].TargetType != "USER" || list[0].TargetID != targetID.String() {
		t.Fatalf("unexpected assignments list: %+v", list)
	}
}

func TestAttendancePolicyRepository_UpsertPolicyAssignment(t *testing.T) {
	wsID := uuid.New()
	userID := uuid.New()
	targetID := uuid.New()
	policyID := uuid.New()
	assignID := uuid.New()

	tx := &fakeTx{}
	tx.queryRowFn = func(_ context.Context, sql string, _ ...any) pgx.Row {
		if strings.Contains(sql, "FROM attendance_policy_assignments") {
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*uuid.UUID)) = assignID
				*(dest[1].(*uuid.UUID)) = policyID
				return nil
			}}
		}
		return fakeRow{}
	}
	tx.execFn = func(_ context.Context, _ string, _ ...any) (pgconn.CommandTag, error) {
		return pgconn.NewCommandTag("INSERT 1"), nil
	}

	repo := AttendancePolicyRepository{
		BeginTx: func(_ context.Context, _ uuid.UUID) (TxBeginner, error) {
			return fakeTxBeginner{tx: tx}, nil
		},
	}

	req := api.UpsertPolicyAssignmentRequest{
		TargetType:           "USER",
		TargetID:             targetID,
		WorkUnitMinutes:      15,
		RoundingPolicy:       "ceil",
		EarlyArrivalStrategy: "clamp",
	}

	res, err := repo.UpsertPolicyAssignment(context.Background(), wsID, userID, req)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if res.ID != assignID.String() || res.Rules.WorkUnitMinutes != 15 || res.Rules.RoundingPolicy != "ceil" {
		t.Fatalf("unexpected upsert result: %+v", res)
	}
	if !tx.committed {
		t.Fatal("expected transaction commit")
	}
}

func TestAttendancePolicyRepository_DeletePolicyAssignment(t *testing.T) {
	wsID := uuid.New()
	assignID := uuid.New()

	var executedSQL string
	db := &fakeDBTX{
		execFn: func(_ context.Context, sql string, _ ...any) (pgconn.CommandTag, error) {
			executedSQL = sql
			return pgconn.NewCommandTag("UPDATE 1"), nil
		},
	}
	repo := AttendancePolicyRepository{
		TenantDB: func(_ context.Context, _ uuid.UUID) (DBTX, error) {
			return db, nil
		},
	}

	if err := repo.DeletePolicyAssignment(context.Background(), wsID, assignID); err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if !strings.Contains(executedSQL, "valid_to = NOW()") {
		t.Fatalf("expected soft close in query: %s", executedSQL)
	}
}

func TestAttendancePolicyRepository_BehalfAssignments_CRUD(t *testing.T) {
	wsID := uuid.New()
	assignee := uuid.New()
	targetEmp := uuid.New()
	behalfID := uuid.New()
	now := time.Now().UTC()

	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, sql string, _ ...any) pgx.Row {
			if strings.Contains(sql, "INSERT INTO behalf_assignments") {
				return fakeRow{scanFn: func(dest ...any) error {
					*(dest[0].(*uuid.UUID)) = behalfID
					*(dest[1].(*uuid.UUID)) = wsID
					*(dest[2].(*uuid.UUID)) = assignee
					*(dest[3].(**uuid.UUID)) = &targetEmp
					*(dest[4].(**uuid.UUID)) = nil
					*(dest[5].(*string)) = "attendance_adjustment"
					*(dest[6].(*time.Time)) = now
					*(dest[7].(*time.Time)) = now
					*(dest[8].(**uuid.UUID)) = nil
					assigneeName := "Assignee Lead"
					*(dest[9].(**string)) = &assigneeName
					assigneeCode := "LEAD01"
					*(dest[10].(**string)) = &assigneeCode
					targetName := "Target Worker"
					*(dest[11].(**string)) = &targetName
					targetCode := "EMP01"
					*(dest[12].(**string)) = &targetCode
					*(dest[13].(**string)) = nil
					return nil
				}}
			}
			return fakeRow{}
		},
		execFn: func(_ context.Context, sql string, _ ...any) (pgconn.CommandTag, error) {
			if strings.Contains(sql, "DELETE FROM behalf_assignments") {
				return pgconn.NewCommandTag("DELETE 1"), nil
			}
			return pgconn.NewCommandTag("OK"), nil
		},
		queryFn: func(_ context.Context, _ string, _ ...any) (pgx.Rows, error) {
			return &rowsStub{
				data: [][]any{
					{behalfID, wsID, assignee, targetEmp, (*uuid.UUID)(nil), "attendance_adjustment", now, now, (*uuid.UUID)(nil), (*string)(nil), (*string)(nil), (*string)(nil), (*string)(nil), (*string)(nil)},
				},
			}, nil
		},
	}
	repo := AttendancePolicyRepository{
		TenantDB: func(_ context.Context, _ uuid.UUID) (DBTX, error) {
			return db, nil
		},
	}

	// Create
	item, err := repo.CreateBehalfAssignment(context.Background(), wsID, api.CreateBehalfAssignmentRequest{
		AssigneeUserID:   assignee,
		TargetEmployeeID: &targetEmp,
	}, "attendance_adjustment")
	if err != nil {
		t.Fatalf("unexpected create error: %v", err)
	}
	if item.ID != behalfID || item.AssigneeName == nil || *item.AssigneeName != "Assignee Lead" {
		t.Fatalf("unexpected created item: %+v", item)
	}

	// List
	items, err := repo.ListBehalfAssignments(context.Background(), wsID, "attendance_adjustment")
	if err != nil || len(items) != 1 {
		t.Fatalf("unexpected list: %v, count: %d", err, len(items))
	}

	// Delete
	if err := repo.DeleteBehalfAssignment(context.Background(), wsID, behalfID); err != nil {
		t.Fatalf("unexpected delete error: %v", err)
	}
}

func TestAttendancePolicyRepository_ListMyBehalfTargets(t *testing.T) {
	wsID := uuid.New()
	userID := uuid.New()
	empID := uuid.New()
	activeEmpID := uuid.New()
	deptName := "Engineering"
	now := time.Now().UTC()

	db := &fakeDBTX{
		queryFn: func(_ context.Context, sql string, _ ...any) (pgx.Rows, error) {
			return &rowsStub{
				data: [][]any{
					{empID, wsID, (*uuid.UUID)(nil), "Target Worker", "EMP01", []string{"core"}, now, now, &deptName, &activeEmpID, (*uuid.UUID)(nil), (*uuid.UUID)(nil)},
				},
			}, nil
		},
	}
	repo := AttendancePolicyRepository{
		TenantDB: func(_ context.Context, _ uuid.UUID) (DBTX, error) {
			return db, nil
		},
	}

	targets, err := repo.ListMyBehalfTargets(context.Background(), wsID, userID, "attendance_adjustment")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if len(targets) != 1 || targets[0].ID != empID || targets[0].FullName != "Target Worker" {
		t.Fatalf("unexpected targets: %+v", targets)
	}
}
