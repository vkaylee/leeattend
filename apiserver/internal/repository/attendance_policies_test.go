package repository

import (
	"context"
	"errors"
	"strings"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
)

func TestAttendancePolicyRepository_GetDefaultPolicyRules_Fallback(t *testing.T) {
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
		DB: db,
	}

 	rules, err := repo.GetDefaultPolicyRules(context.Background())
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if rules.WorkUnitMinutes != 30 || rules.RoundingPolicy != "floor" || rules.EarlyArrivalStrategy != "block" {
		t.Fatalf("expected default rules fallback, got %+v", rules)
	}
}

func TestAttendancePolicyRepository_UpdateDefaultPolicyRules_Transaction(t *testing.T) {
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
 		BeginTx: func(_ context.Context) (TxBeginner, error) {
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

 	res, err := repo.UpdateDefaultPolicyRules(context.Background(), userID, req)
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
		DB: db,
	}

 	list, err := repo.ListPolicyAssignments(context.Background())
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if len(list) != 1 || list[0].TargetType != "USER" || list[0].TargetID != targetID.String() {
		t.Fatalf("unexpected assignments list: %+v", list)
	}
}

func TestAttendancePolicyRepository_UpsertPolicyAssignment(t *testing.T) {
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
 		BeginTx: func(_ context.Context) (TxBeginner, error) {
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

 	res, err := repo.UpsertPolicyAssignment(context.Background(), userID, req)
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
 	assignID := uuid.New()

	var executedSQL string
	db := &fakeDBTX{
		execFn: func(_ context.Context, sql string, _ ...any) (pgconn.CommandTag, error) {
			executedSQL = sql
			return pgconn.NewCommandTag("UPDATE 1"), nil
		},
	}
	repo := AttendancePolicyRepository{
		DB: db,
	}

 	if err := repo.DeletePolicyAssignment(context.Background(), assignID); err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if !strings.Contains(executedSQL, "valid_to = NOW()") {
		t.Fatalf("expected soft close in query: %s", executedSQL)
	}
}

 func TestAttendancePolicyRepository_BehalfAssignmentsRequireOrganizationMember(t *testing.T) {
 	assignee := uuid.New()
 	targetEmp := uuid.New()

	memberDB := &fakeDBTX{
		queryRowFn: func(_ context.Context, sql string, _ ...any) pgx.Row {
 			if !strings.Contains(sql, "user_roles") {
 				t.Fatalf("expected organization membership validation query, got %s", sql)
			}
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*bool)) = true
				return nil
			}}
		},
	}
	tenantDB := &fakeDBTX{
		queryRowFn: func(_ context.Context, sql string, _ ...any) pgx.Row {
			if !strings.Contains(sql, "INSERT INTO behalf_assignments") {
				t.Fatalf("expected assignment insert, got %s", sql)
			}
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*uuid.UUID)) = uuid.New()
				*(dest[1].(*uuid.UUID)) = assignee
				*(dest[2].(**uuid.UUID)) = &targetEmp
				*(dest[3].(**uuid.UUID)) = nil
				*(dest[4].(*string)) = "attendance_adjustment"
				*(dest[5].(*time.Time)) = time.Now().UTC()
				*(dest[6].(*time.Time)) = time.Now().UTC()
				*(dest[7].(**uuid.UUID)) = nil
				*(dest[8].(**string)) = nil
				*(dest[9].(**string)) = nil
				*(dest[10].(**string)) = nil
				return nil
			}}
		},
	}
	repo := AttendancePolicyRepository{
		SystemDB: memberDB,
		DB:       tenantDB,
	}
 	if _, err := repo.CreateBehalfAssignment(context.Background(), api.CreateBehalfAssignmentRequest{AssigneeUserID: assignee, TargetEmployeeID: &targetEmp}, "attendance_adjustment"); err != nil {
 		t.Fatalf("expected organization member to be assignable: %v", err)
	}

	nonMemberDB := &fakeDBTX{
		queryRowFn: func(_ context.Context, _ string, _ ...any) pgx.Row {
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*bool)) = false
				return nil
			}}
		},
	}
	repo.SystemDB = nonMemberDB
	var appErr *apperror.AppError
 	if _, err := repo.CreateBehalfAssignment(context.Background(), api.CreateBehalfAssignmentRequest{AssigneeUserID: assignee, TargetEmployeeID: &targetEmp}, "attendance_adjustment"); !errors.As(err, &appErr) || appErr.Code != apperror.CodeForbidden {
	}
}

func TestAttendancePolicyRepository_ListMyBehalfTargets(t *testing.T) {
	userID := uuid.New()
	empID := uuid.New()
	activeEmpID := uuid.New()
	deptName := "Engineering"
	now := time.Now().UTC()

	db := &fakeDBTX{
		queryFn: func(_ context.Context, sql string, _ ...any) (pgx.Rows, error) {
			return &rowsStub{
				data: [][]any{
 					{empID, (*uuid.UUID)(nil), "Target Worker", "EMP01", []string{"core"}, now, now, &deptName, &activeEmpID, (*uuid.UUID)(nil), (*uuid.UUID)(nil)},
				},
			}, nil
		},
	}
	repo := AttendancePolicyRepository{
		DB: db,
	}

 	targets, err := repo.ListMyBehalfTargets(context.Background(), userID, "attendance_adjustment")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if len(targets) != 1 || targets[0].ID != empID || targets[0].FullName != "Target Worker" {
		t.Fatalf("unexpected targets: %+v", targets)
	}
}
