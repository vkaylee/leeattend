package repository

import (
	"context"
	"strings"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func TestDepartmentRepository_CreateAndGet(t *testing.T) {
	wsID, deptID := uuid.New(), uuid.New()
	createdAt := time.Date(2026, 1, 1, 0, 0, 0, 0, time.UTC)
	var execQuery string
	var execArgs []any
	var getQuery string
	var getArgs []any

	db := &fakeDBTX{
		execFn: func(_ context.Context, sql string, arguments ...any) (pgconn.CommandTag, error) {
			execQuery, execArgs = sql, arguments
			return pgconn.NewCommandTag("INSERT 0 1"), nil
		},
		queryRowFn: func(_ context.Context, sql string, args ...any) pgx.Row {
			getQuery, getArgs = sql, args
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*uuid.UUID)) = deptID
				*(dest[1].(*uuid.UUID)) = wsID
				*(dest[2].(*string)) = "Engineering"
				*(dest[3].(**string)) = nil
				*(dest[4].(**uuid.UUID)) = nil
				*(dest[5].(*time.Time)) = createdAt
				*(dest[6].(*time.Time)) = createdAt
				*(dest[7].(*int64)) = 5
				return nil
			}}
		},
	}
	repo := DepartmentRepository{TenantDB: func(context.Context, uuid.UUID) (DBTX, error) { return db, nil }}

	dept, err := repo.CreateDepartment(context.Background(), wsID, api.CreateDepartmentRequest{Name: "Engineering"})
	if err != nil {
		t.Fatalf("create department: %v", err)
	}
	if dept.ID != deptID || dept.Name != "Engineering" || dept.ActiveEmployeeCount != 5 {
		t.Fatalf("unexpected department: %+v", dept)
	}
	if !strings.Contains(execQuery, "INSERT INTO departments") || len(execArgs) < 5 || execArgs[1] != wsID {
		t.Fatalf("unexpected insert query/args: %s, %#v", execQuery, execArgs)
	}
	if !strings.Contains(getQuery, "WHERE d.workspace_id = $1 AND d.id = $2") || len(getArgs) != 2 || getArgs[0] != wsID {
		t.Fatalf("unexpected get query/args: %s, %#v", getQuery, getArgs)
	}
}

func TestDepartmentRepository_PayrollConfig(t *testing.T) {
	wsID, deptID, actorID := uuid.New(), uuid.New(), uuid.New()
	cfgID := uuid.New()
	createdAt := time.Date(2026, 1, 1, 0, 0, 0, 0, time.UTC)
	var upsertCount int

	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, sql string, args ...any) pgx.Row {
			if strings.Contains(sql, "SELECT EXISTS") {
				return fakeRow{scanFn: func(dest ...any) error {
					*(dest[0].(*bool)) = true
					return nil
				}}
			}
			upsertCount++
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*uuid.UUID)) = cfgID
				*(dest[1].(*time.Time)) = createdAt
				return nil
			}}
		},
	}
	repo := DepartmentRepository{TenantDB: func(context.Context, uuid.UUID) (DBTX, error) { return db, nil }}

	cfg, err := repo.SetDepartmentPayrollConfig(context.Background(), wsID, deptID, api.SetDepartmentPayrollConfigRequest{
		StdWorkingDays: decimal.NewFromFloat(22),
		HoursPerDay:    decimal.NewFromFloat(8),
		EffectiveDate:  "2026-01-01",
	}, actorID)
	if err != nil {
		t.Fatalf("set payroll config: %v", err)
	}
	if cfg.DepartmentID != deptID || cfg.EffectiveDate != "2026-01-01" {
		t.Fatalf("unexpected config: %+v", cfg)
	}
	if upsertCount != 2 {
		t.Fatalf("expected 2 config entry upserts, got %d", upsertCount)
	}
}
