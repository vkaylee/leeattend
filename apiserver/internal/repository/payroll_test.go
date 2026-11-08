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

func TestPayrollRepository_ListPayslips_ScopeFree(t *testing.T) {
	queryExecuted := false
	db := &fakeDBTX{
		queryFn: func(_ context.Context, sql string, args ...any) (pgx.Rows, error) {
			queryExecuted = true
			if strings.Contains(strings.ToLower(sql), "workspace_id") {
				t.Fatalf("expected query to be scope-free without workspace_id: %s", sql)
			}
			if len(args) != 2 {
				t.Fatalf("expected 2 period args, got %d", len(args))
			}
			return &fakeRows{}, nil
		},
	}
	repo := PayrollRepository{DB: db}
	start := time.Date(2026, 1, 1, 0, 0, 0, 0, time.UTC)
	end := time.Date(2026, 2, 1, 0, 0, 0, 0, time.UTC)
	payslips, err := repo.ListPayslips(context.Background(), start, end)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if !queryExecuted {
		t.Fatal("query was not executed")
	}
	if len(payslips) != 0 {
		t.Fatalf("expected empty list, got %d", len(payslips))
	}
}

func TestPayrollRepository_ListEmployeePayslips_ScopeFree(t *testing.T) {
	queryExecuted := false
	empID := uuid.New()
	db := &fakeDBTX{
		queryFn: func(_ context.Context, sql string, args ...any) (pgx.Rows, error) {
			queryExecuted = true
			if strings.Contains(strings.ToLower(sql), "workspace_id") {
				t.Fatalf("expected query to be scope-free without workspace_id: %s", sql)
			}
			if !strings.Contains(sql, "FROM payslips p JOIN employments e ON e.id=p.employment_id WHERE e.employee_id=$1") {
				t.Fatalf("unexpected SQL join/filter: %s", sql)
			}
			if len(args) != 1 || args[0] != empID {
				t.Fatalf("expected single employeeID arg, got %v", args)
			}
			return &fakeRows{}, nil
		},
	}
	repo := PayrollRepository{DB: db}
	payslips, err := repo.ListEmployeePayslips(context.Background(), empID)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if !queryExecuted {
		t.Fatal("query was not executed")
	}
	if len(payslips) != 0 {
		t.Fatalf("expected empty list, got %d", len(payslips))
	}
}

func TestPayrollRepository_GetPayrollSettings_UsesConfigEntries(t *testing.T) {
	queryExecuted := false
	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, sql string, _ ...any) pgx.Row {
			if strings.Contains(sql, "SELECT timezone FROM organization_settings") {
				return fakeRow{scanFn: func(dest ...any) error {
					*(dest[0].(*string)) = "Asia/Ho_Chi_Minh"
					return nil
				}}
			}
			return fakeRow{}
		},
		queryFn: func(_ context.Context, sql string, args ...any) (pgx.Rows, error) {
			queryExecuted = true
			if !strings.Contains(sql, "FROM config_entries") {
				t.Fatalf("expected query from config_entries, got %s", sql)
			}
			if !strings.Contains(sql, "scope_type = 'organization' AND scope_id IS NULL") {
				t.Fatalf("expected organization scope with NULL scope_id, got %s", sql)
			}
			if strings.Contains(strings.ToLower(sql), "workspace_id") {
				t.Fatalf("unexpected workspace_id in query: %s", sql)
			}
			return &fakeRows{}, nil
		},
	}
	repo := PayrollRepository{DB: db}
	_, err := repo.GetPayrollSettings(context.Background())
	if err != api.ErrPayrollSettingsNotFound {
		t.Fatalf("expected ErrPayrollSettingsNotFound when no rows, got %v", err)
	}
	if !queryExecuted {
		t.Fatal("settings query was not executed")
	}
}

func TestPayrollRepository_UpdatePayrollSettings_OrganizationScope(t *testing.T) {
	insertExecuted := false
	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, sql string, _ ...any) pgx.Row {
			if strings.Contains(sql, "SELECT timezone FROM organization_settings") {
				return fakeRow{scanFn: func(dest ...any) error {
					*(dest[0].(*string)) = "Asia/Ho_Chi_Minh"
					return nil
				}}
			}
			if strings.Contains(sql, "SELECT id FROM config_entries") {
				return fakeRow{scanFn: func(dest ...any) error {
					return pgx.ErrNoRows
				}}
			}
			return fakeRow{}
		},
		queryFn: func(_ context.Context, _ string, _ ...any) (pgx.Rows, error) {
			return &fakeRows{}, nil
		},
		execFn: func(_ context.Context, sql string, args ...any) (pgconn.CommandTag, error) {
			if strings.Contains(sql, "INSERT INTO config_entries") {
				insertExecuted = true
				if !strings.Contains(sql, "'organization', NULL") {
					t.Fatalf("expected insert with 'organization', NULL scope, got %s", sql)
				}
				if strings.Contains(strings.ToLower(sql), "workspace_id") {
					t.Fatalf("unexpected workspace_id in SQL: %s", sql)
				}
			}
			return pgconn.NewCommandTag("INSERT 0 1"), nil
		},
	}
	repo := PayrollRepository{DB: db}
	mode := "fixed"
	fixedDays := decimal.NewFromInt(26)
	effDate := "2026-06-01"
	settings, err := repo.UpdatePayrollSettings(context.Background(), api.UpdatePayrollSettingsRequest{
		StdWorkingDayMode: &mode,
		FixedStdDays:      &fixedDays,
		EffectiveDate:     &effDate,
	})
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if !insertExecuted {
		t.Fatal("expected insert into config_entries")
	}
	if settings.StdWorkingDayMode != "fixed" || settings.FixedStdDays.String() != "26" {
		t.Fatalf("unexpected settings returned: %+v", settings)
	}
}

func TestPayrollRepository_SetEmployeePayrollConfig_EmploymentScope(t *testing.T) {
	empID := uuid.New()
	activeEmpID := uuid.New()
	actorID := uuid.New()
	upsertExecuted := false

	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, sql string, _ ...any) pgx.Row {
			if strings.Contains(sql, "SELECT EXISTS (SELECT 1 FROM employees") {
				return fakeRow{scanFn: func(dest ...any) error {
					*(dest[0].(*bool)) = true
					return nil
				}}
			}
			if strings.Contains(sql, "SELECT id FROM employments") {
				return fakeRow{scanFn: func(dest ...any) error {
					*(dest[0].(*uuid.UUID)) = activeEmpID
					return nil
				}}
			}
			if strings.Contains(sql, "INSERT INTO config_entries") {
				upsertExecuted = true
				if !strings.Contains(sql, "'employment'") {
					t.Fatalf("expected scope_type 'employment', got: %s", sql)
				}
				if !strings.Contains(sql, "ON CONFLICT (scope_type, scope_id, config_key, effective_from)") {
					t.Fatalf("expected ON CONFLICT clause matching unique constraint: %s", sql)
				}
				return fakeRow{scanFn: func(dest ...any) error {
					*(dest[0].(*uuid.UUID)) = uuid.New()
					*(dest[1].(*time.Time)) = time.Now().UTC()
					return nil
				}}
			}
			return fakeRow{}
		},
		execFn: func(_ context.Context, sql string, _ ...any) (pgconn.CommandTag, error) {
			return pgconn.NewCommandTag("INSERT 0 1"), nil
		},
	}
	repo := PayrollRepository{DB: db}
	cfg, err := repo.SetEmployeePayrollConfig(context.Background(), empID, api.SetEmployeePayrollConfigRequest{
		StdWorkingDays: decimal.NewFromInt(22),
		HoursPerDay:    decimal.NewFromInt(8),
		EffectiveDate:  "2026-06-01",
	}, actorID)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if !upsertExecuted {
		t.Fatal("expected upsert into config_entries")
	}
	if cfg.EmploymentID != activeEmpID {
		t.Fatalf("expected active employment ID %s, got %s", activeEmpID, cfg.EmploymentID)
	}
}

func TestPayrollRepository_AssignPayrollRule_OrganizationAssigneeNullID(t *testing.T) {
	ruleID := uuid.New()
	inserted := false

	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, sql string, args ...any) pgx.Row {
			if strings.Contains(sql, "SELECT EXISTS (SELECT 1 FROM payroll_rule_definitions") {
				return fakeRow{scanFn: func(dest ...any) error {
					*(dest[0].(*bool)) = true
					return nil
				}}
			}
			if strings.Contains(sql, "INSERT INTO payroll_rule_assignments") {
				inserted = true
				if strings.Contains(strings.ToLower(sql), "workspace_id") {
					t.Fatalf("unexpected workspace_id column in insert: %s", sql)
				}
				// args: [assignID, ruleID, req.AssigneeType, assigneeID, req.EffectiveFrom, req.EffectiveTo, overrideParam]
				if len(args) < 4 {
					t.Fatalf("expected at least 4 args, got %d", len(args))
				}
				if args[2] != "organization" {
					t.Fatalf("expected assignee_type 'organization', got %v", args[2])
				}
				if args[3] != nil && args[3] != (*uuid.UUID)(nil) {
					t.Fatalf("expected nil assignee_id for organization rule, got %v", args[3])
				}
				return fakeRow{scanFn: func(dest ...any) error {
					*(dest[0].(*time.Time)) = time.Now().UTC()
					return nil
				}}
			}
			return fakeRow{}
		},
	}
	repo := PayrollRepository{DB: db}
	someID := uuid.New()
	assignment, err := repo.AssignPayrollRule(context.Background(), ruleID, api.AssignPayrollRuleRequest{
		AssigneeType:  "organization",
		AssigneeID:    &someID, // Even if passed, repository must normalize to nil
		EffectiveFrom: "2026-01-01",
	})
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if !inserted {
		t.Fatal("assignment was not inserted")
	}
	if assignment.AssigneeType != "organization" {
		t.Fatalf("expected organization assignee_type, got %s", assignment.AssigneeType)
	}
	if assignment.AssigneeID != nil {
		t.Fatalf("expected nil AssigneeID in response, got %v", assignment.AssigneeID)
	}
}

func TestPayrollRepository_AssignPayrollRule_DepartmentAssigneeWithID(t *testing.T) {
	ruleID := uuid.New()
	deptID := uuid.New()
	inserted := false

	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, sql string, args ...any) pgx.Row {
			if strings.Contains(sql, "SELECT EXISTS (SELECT 1 FROM payroll_rule_definitions") {
				return fakeRow{scanFn: func(dest ...any) error {
					*(dest[0].(*bool)) = true
					return nil
				}}
			}
			if strings.Contains(sql, "INSERT INTO payroll_rule_assignments") {
				inserted = true
				if args[2] != "department" {
					t.Fatalf("expected assignee_type 'department', got %v", args[2])
				}
				if args[3] == nil || *(args[3].(*uuid.UUID)) != deptID {
					t.Fatalf("expected department ID %s, got %v", deptID, args[3])
				}
				return fakeRow{scanFn: func(dest ...any) error {
					*(dest[0].(*time.Time)) = time.Now().UTC()
					return nil
				}}
			}
			return fakeRow{}
		},
	}
	repo := PayrollRepository{DB: db}
	assignment, err := repo.AssignPayrollRule(context.Background(), ruleID, api.AssignPayrollRuleRequest{
		AssigneeType:  "department",
		AssigneeID:    &deptID,
		EffectiveFrom: "2026-01-01",
	})
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if !inserted {
		t.Fatal("assignment was not inserted")
	}
	if assignment.AssigneeType != "department" {
		t.Fatalf("expected department assignee_type, got %s", assignment.AssigneeType)
	}
	if assignment.AssigneeID == nil || *assignment.AssigneeID != deptID {
		t.Fatalf("expected AssigneeID %s, got %v", deptID, assignment.AssigneeID)
	}
}
