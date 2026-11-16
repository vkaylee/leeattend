package repository

import (
	"context"
	"errors"
	"strings"
	"testing"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func TestDepartmentRepository_RequiresDatabase(t *testing.T) {
	repo := DepartmentRepository{}
	ctx := context.Background()
	deptID := uuid.New()

	if _, err := repo.ListDepartments(ctx, 1, 20); err == nil {
		t.Fatal("expected error when database is not configured")
	}
	if _, err := repo.GetDepartmentByID(ctx, deptID); err == nil {
		t.Fatal("expected error when database is not configured")
	}
	if _, err := repo.CreateDepartment(ctx, api.CreateDepartmentRequest{Name: "Engineering"}); err == nil {
		t.Fatal("expected error when database is not configured")
	}
	if _, err := repo.UpdateDepartment(ctx, deptID, api.UpdateDepartmentRequest{}); err == nil {
		t.Fatal("expected error when database is not configured")
	}
	if err := repo.DeleteDepartment(ctx, deptID); err == nil {
		t.Fatal("expected error when database is not configured")
	}
	if _, err := repo.GetDepartmentPayrollConfigs(ctx, deptID); err == nil {
		t.Fatal("expected error when database is not configured")
	}
	if _, err := repo.ListDepartmentPayrollReconciliation(ctx, 1, 50, ""); err == nil {
		t.Fatal("expected error when database is not configured")
	}
	if _, err := repo.SetDepartmentPayrollConfig(ctx, deptID, api.SetDepartmentPayrollConfigRequest{}, uuid.New()); err == nil {
		t.Fatal("expected error when database is not configured")
	}
}

func TestDepartmentRepository_ReconciliationQueryIsBoundedAndFilterable(t *testing.T) {
	var dataQuery, countQuery string
	db := &fakeDBTX{
		queryFn: func(_ context.Context, query string, args ...any) (pgx.Rows, error) {
			dataQuery = query
			if len(args) != 3 || args[0] != 20 || args[1] != 20 || args[2] != "value_mismatch" {
				t.Fatalf("unexpected reconciliation query args: %#v", args)
			}
			return &rowsStub{}, nil
		},
		queryRowFn: func(_ context.Context, query string, args ...any) pgx.Row {
			countQuery = query
			if len(args) != 1 || args[0] != "value_mismatch" {
				t.Fatalf("unexpected reconciliation count args: %#v", args)
			}
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*int64)) = 0
				return nil
			}}
		},
	}
	result, err := (DepartmentRepository{DB: db}).ListDepartmentPayrollReconciliation(context.Background(), 2, 20, "value_mismatch")
	if err != nil {
		t.Fatalf("list reconciliation: %v", err)
	}
	if result.Page != 2 || result.Limit != 20 || result.Total != 0 || len(result.Data) != 0 {
		t.Fatalf("unexpected reconciliation response: %+v", result)
	}
	if !strings.Contains(dataQuery, "LIMIT $1 OFFSET $2") || !strings.Contains(dataQuery, "WHERE reconciliation_status = $3") || !strings.Contains(dataQuery, "canonical_effective_from::text") {
		t.Fatalf("reconciliation query must be bounded, filterable, and type-stable: %s", dataQuery)
	}
	if !strings.Contains(countQuery, "WHERE reconciliation_status = $1") {
		t.Fatalf("reconciliation count query must apply filter: %s", countQuery)
	}
}

func TestDepartmentRepository_MissingDepartmentBehavior(t *testing.T) {
	ctx := context.Background()
	deptID := uuid.New()
	repo := DepartmentRepository{DB: &fakeDBTX{
		queryRowFn: func(_ context.Context, _ string, _ ...any) pgx.Row {
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*bool)) = false
				return nil
			}}
		},
	}}

	configs, err := repo.GetDepartmentPayrollConfigs(ctx, deptID)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if len(configs) != 0 {
		t.Fatalf("expected no configs for missing department, got %d", len(configs))
	}

	_, err = repo.SetDepartmentPayrollConfig(ctx, deptID, api.SetDepartmentPayrollConfigRequest{EffectiveDate: "2026-01-01"}, uuid.New())
	if !errors.Is(err, api.ErrDepartmentNotFound) {
		t.Fatalf("expected ErrDepartmentNotFound, got %v", err)
	}
}
