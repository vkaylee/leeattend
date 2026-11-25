package repository

import (
	"context"
	"errors"
	"strings"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
)

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
		t.Fatalf("expected forbidden for non-member, got %v", err)
	}
}

func TestAttendancePolicyRepository_ListMyBehalfTargets(t *testing.T) {
	userID := uuid.New()
	empID := uuid.New()
	activeEmpID := uuid.New()
	deptName := "Engineering"
	now := time.Now().UTC()

	db := &fakeDBTX{
		queryFn: func(_ context.Context, _ string, _ ...any) (pgx.Rows, error) {
			return &rowsStub{
				data: [][]any{
					{empID, (*uuid.UUID)(nil), "Target Worker", "EMP01", []string{"core"}, now, now, &deptName, &activeEmpID, (*uuid.UUID)(nil), (*uuid.UUID)(nil)},
				},
			}, nil
		},
	}
	repo := AttendancePolicyRepository{DB: db}

	targets, err := repo.ListMyBehalfTargets(context.Background(), userID, "attendance_adjustment")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if len(targets) != 1 || targets[0].ID != empID || targets[0].FullName != "Target Worker" {
		t.Fatalf("unexpected targets: %+v", targets)
	}
}
