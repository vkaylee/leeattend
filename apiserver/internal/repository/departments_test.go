package repository

import (
	"context"
	"errors"
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
	if _, err := repo.SetDepartmentPayrollConfig(ctx, deptID, api.SetDepartmentPayrollConfigRequest{}, uuid.New()); err == nil {
		t.Fatal("expected error when database is not configured")
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
