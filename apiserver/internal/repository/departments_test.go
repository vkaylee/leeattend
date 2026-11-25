package repository

import (
	"context"
	"testing"

	"github.com/google/uuid"
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
}
