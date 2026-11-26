package repository

import (
	"context"
	"errors"
	"reflect"
	"strings"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/ids"
)

func TestComparePayrollImpactBatchRejectsDuplicateEmploymentIDs(t *testing.T) {
	first, second := uuid.New(), uuid.New()
	for _, values := range [][]uuid.UUID{{first, first}, {first, second, first}} {
		_, err := (PayrollRepository{}).ComparePayrollImpactBatch(context.Background(), api.PayrollImpactBatchRequest{
			EmploymentIDs: values,
			BeforeRunID:   uuid.New(),
			AfterRunID:    uuid.New(),
		})
		if !errors.Is(err, api.ErrPayrollImpactValidation) {
			t.Fatalf("duplicate employment IDs %v: error = %v, want validation", values, err)
		}
	}
}

func TestImpactStalenessForPolicyIgnoresLegacySnapshotWithoutPolicy(t *testing.T) {
	stale, reasons := impactStalenessForPolicy(nil, []byte(`{"kpi_inputs":{"actual":"1"}}`))
	if stale || len(reasons) != 0 {
		t.Fatalf("legacy snapshot should remain non-stale: stale=%t reasons=%v", stale, reasons)
	}
}

func TestImpactStalenessForPolicyDetectsChangedOrMissingPolicy(t *testing.T) {
	v1, v2 := uuid.New(), uuid.New()
	stale, reasons := impactStalenessForPolicy([]uuid.UUID{v1}, []byte(`{"policy":{"version_id":"bad"}}`))
	if !stale || len(reasons) != 1 || reasons[0] != "policy_version_invalid" {
		t.Fatalf("invalid policy snapshot should be stale: stale=%t reasons=%v", stale, reasons)
	}
	stale, reasons = impactStalenessForPolicy([]uuid.UUID{v1, v2}, []byte(`{"policy":{"version_id":"`+v1.String()+`"}}`))
	if !stale || reasons[0] != "policy_version_changed" {
		t.Fatalf("mid-period second policy should mark snapshot stale: stale=%t reasons=%v", stale, reasons)
	}
	stale, reasons = impactStalenessForPolicy([]uuid.UUID{v1}, []byte(`{"policy":{"version_id":"`+v1.String()+`"}}`))
	if stale || len(reasons) != 0 {
		t.Fatalf("matching policy should not be stale: stale=%t reasons=%v", stale, reasons)
	}
}

func TestPayrollInputRowStatus(t *testing.T) {
	status, message := payrollInputRowStatus(decimal.Zero)
	if status != "warning" || message != "zero metric value requires review" {
		t.Fatalf("zero value status = %q, message = %v", status, message)
	}
	status, message = payrollInputRowStatus(decimal.NewFromInt(1))
	if status != "accepted" || message != nil {
		t.Fatalf("positive value status = %q, message = %v", status, message)
	}
}

func TestEnsurePayrollPeriodOpen_RejectsLockedPeriod(t *testing.T) {
	tx := &payrollPeriodTestTx{}
	tx.queryRowFn = func(_ context.Context, sql string, _ ...any) pgx.Row {
		if strings.Contains(sql, "SELECT status FROM payroll_periods") {
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*string)) = "locked"
				return nil
			}}
		}
		return fakeRow{}
	}

	err := ensurePayrollPeriodOpen(context.Background(), tx, uuid.New(), 2026, 1)
	if !errors.Is(err, api.ErrPayrollPeriodConflict) {
		t.Fatalf("expected locked period conflict, got %v", err)
	}
	if tx.committed || tx.rolledBack {
		t.Fatalf("guard must not manage transaction lifecycle, committed=%t rolled_back=%t", tx.committed, tx.rolledBack)
	}
}

func TestEnsurePayrollPeriodOpen_RejectsInvalidPeriodBeforeWrite(t *testing.T) {
	execCalled := false
	tx := &payrollPeriodTestTx{}
	tx.execFn = func(_ context.Context, _ string, _ ...any) (pgconn.CommandTag, error) {
		execCalled = true
		return pgconn.NewCommandTag("INSERT 0 1"), nil
	}

	err := ensurePayrollPeriodOpen(context.Background(), tx, uuid.New(), 2026, 13)
	if !errors.Is(err, api.ErrPayrollPeriodValidation) {
		t.Fatalf("expected invalid period error, got %v", err)
	}
	if execCalled {
		t.Fatal("invalid period must not write payroll period")
	}
}

func TestPayrollRepository_CalculateEmployeePayroll_RejectsLockedPeriod(t *testing.T) {
	employeeID := uuid.New()
	employmentID := uuid.New()
	employerID := uuid.New()
	queriedAfterGuard := false
	tx := &payrollPeriodTestTx{}
	tx.queryRowFn = func(_ context.Context, sql string, _ ...any) pgx.Row {
		switch {
		case strings.Contains(sql, "SELECT EXISTS (SELECT 1 FROM employees"):
			return fakeRow{scanFn: func(dest ...any) error { *(dest[0].(*bool)) = true; return nil }}
		case strings.Contains(sql, "SELECT id, employer_id"):
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*uuid.UUID)) = employmentID
				*(dest[1].(*uuid.UUID)) = employerID
				return nil
			}}
		case strings.Contains(sql, "SELECT status FROM payroll_periods"):
			return fakeRow{scanFn: func(dest ...any) error { *(dest[0].(*string)) = "locked"; return nil }}
		default:
			return fakeRow{}
		}
	}
	tx.queryFn = func(_ context.Context, sql string, _ ...any) (pgx.Rows, error) {
		if strings.Contains(sql, "FROM employee_salary_components") {
			queriedAfterGuard = true
		}
		return &fakeRows{}, nil
	}

	_, err := (PayrollRepository{DB: tx}).CalculateEmployeePayroll(context.Background(), ids.EmployeeID(employeeID), 2026, 1, ids.UserID(uuid.New()))
	if !errors.Is(err, api.ErrPayrollPeriodConflict) {
		t.Fatalf("expected locked period conflict, got %v", err)
	}
	if queriedAfterGuard {
		t.Fatal("locked period must stop calculation before loading payroll inputs")
	}
	if tx.committed || !tx.rolledBack {
		t.Fatalf("expected rollback without commit, committed=%t rolled_back=%t", tx.committed, tx.rolledBack)
	}
}

func TestPayrollRepository_CalculateEmployeePayroll_ProtectsFinalizedPayslip(t *testing.T) {
	for _, status := range []string{"paid", "finalized"} {
		t.Run(status, func(t *testing.T) {
			tx := payrollCalculationFixture(uuid.New(), uuid.New(), uuid.New(), "open", status)
			_, err := (PayrollRepository{DB: tx}).CalculateEmployeePayroll(context.Background(), ids.EmployeeID(uuid.New()), 2026, 1, ids.UserID(uuid.New()))
			if !errors.Is(err, api.ErrPayslipConflict) {
				t.Fatalf("expected payslip conflict for %s payslip, got %v", status, err)
			}
			if tx.committed || !tx.rolledBack {
				t.Fatalf("expected rollback without commit, committed=%t rolled_back=%t", tx.committed, tx.rolledBack)
			}
		})
	}
}

func TestPayrollRepository_CalculateEmployeePayroll_PropagatesPayslipChildDeleteFailure(t *testing.T) {
	deleteErr := errors.New("child delete failed")
	tx := payrollCalculationFixture(uuid.New(), uuid.New(), uuid.New(), "open", "draft")
	tx.execFn = func(_ context.Context, sql string, _ ...any) (pgconn.CommandTag, error) {
		if strings.Contains(sql, "DELETE FROM payslip_segments") {
			return pgconn.CommandTag{}, deleteErr
		}
		return pgconn.NewCommandTag("INSERT 0 1"), nil
	}

	_, err := (PayrollRepository{DB: tx}).CalculateEmployeePayroll(context.Background(), ids.EmployeeID(uuid.New()), 2026, 1, ids.UserID(uuid.New()))
	if !errors.Is(err, deleteErr) {
		t.Fatalf("expected child delete error, got %v", err)
	}
	if tx.committed || !tx.rolledBack {
		t.Fatalf("expected rollback without commit, committed=%t rolled_back=%t", tx.committed, tx.rolledBack)
	}
}

func TestPayrollRepository_SimulateEmployeePayroll_PerformsZeroDBWrites(t *testing.T) {
	empID := uuid.New()
	employmentID := uuid.New()
	relID := uuid.New()
	employerID := uuid.New()
	deptID := uuid.New()
	execCalls := 0
	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, sql string, _ ...any) pgx.Row {
			if strings.Contains(sql, "FROM employment_segments") {
				return fakeRow{scanFn: func(dest ...any) error {
					*(dest[0].(*uuid.UUID)) = employmentID
					return nil
				}}
			}
			return fakeRow{}
		},
		queryFn: func(_ context.Context, sql string, _ ...any) (pgx.Rows, error) {
			if strings.Contains(sql, "SELECT id, relationship_id, employee_id, employer_id, department_id") {
				return &rowsStub{data: [][]any{{employmentID, relID, empID, employerID, &deptID}}}, nil
			}
			if strings.Contains(sql, "FROM employment_segments e") && strings.Contains(sql, "work_policy_memberships") {
				return &resolvedPolicyTestRows{empID: employmentID, relID: relID}, nil
			}
			return &fakeRows{}, nil
		},
		execFn: func(_ context.Context, _ string, _ ...any) (pgconn.CommandTag, error) {
			execCalls++
			return pgconn.NewCommandTag("OK"), nil
		},
	}

	resp, err := (PayrollRepository{DB: db}).SimulateEmployeePayroll(context.Background(), ids.EmployeeID(empID), 2026, 1, ids.UserID(uuid.New()))
	if err != nil {
		t.Fatalf("simulate payroll: %v", err)
	}
	if execCalls != 0 {
		t.Fatalf("expected 0 DB writes during simulation, got %d", execCalls)
	}
	if resp.EmploymentID != employmentID {
		t.Fatalf("unexpected employment ID: %s", resp.EmploymentID)
	}
}

func payrollCalculationFixture(employeeID, employmentID, employerID uuid.UUID, periodStatus, payslipStatus string) *payrollPeriodTestTx {
	relID := uuid.New()
	departmentID := uuid.New()
	tx := &payrollPeriodTestTx{}
	tx.queryRowFn = func(_ context.Context, sql string, _ ...any) pgx.Row {
		switch {
		case strings.Contains(sql, "SELECT EXISTS (SELECT 1 FROM employees"):
			return fakeRow{scanFn: func(dest ...any) error { *(dest[0].(*bool)) = true; return nil }}
		case strings.Contains(sql, "SELECT id, employer_id"):
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*uuid.UUID)) = employmentID
				*(dest[1].(*uuid.UUID)) = employerID
				return nil
			}}
		case strings.Contains(sql, "SELECT status FROM payroll_periods"):
			return fakeRow{scanFn: func(dest ...any) error { *(dest[0].(*string)) = periodStatus; return nil }}
		case strings.Contains(sql, "FROM payslips"):
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*uuid.UUID)) = uuid.New()
				*(dest[1].(*string)) = payslipStatus
				return nil
			}}
		default:
			return fakeRow{}
		}
	}
	tx.queryFn = func(_ context.Context, sql string, _ ...any) (pgx.Rows, error) {
		if strings.Contains(sql, "SELECT id, relationship_id, employee_id, employer_id, department_id") {
			return &rowsStub{data: [][]any{{employmentID, relID, employeeID, employerID, &departmentID}}}, nil
		}
		if strings.Contains(sql, "FROM employment_segments e") && strings.Contains(sql, "work_policy_memberships") {
			return &resolvedPolicyTestRows{empID: employmentID, relID: relID}, nil
		}
		return &fakeRows{}, nil
	}
	return tx
}

type resolvedPolicyTestRows struct {
	empID uuid.UUID
	relID uuid.UUID
	read  bool
}

func (r *resolvedPolicyTestRows) Close()                                       {}
func (r *resolvedPolicyTestRows) Err() error                                   { return nil }
func (r *resolvedPolicyTestRows) CommandTag() pgconn.CommandTag                { return pgconn.CommandTag{} }
func (r *resolvedPolicyTestRows) FieldDescriptions() []pgconn.FieldDescription { return nil }
func (r *resolvedPolicyTestRows) Values() ([]any, error)                       { return nil, nil }
func (r *resolvedPolicyTestRows) RawValues() [][]byte                          { return nil }
func (r *resolvedPolicyTestRows) Conn() *pgx.Conn                              { return nil }
func (r *resolvedPolicyTestRows) Next() bool {
	if r.read {
		return false
	}
	r.read = true
	return true
}
func (r *resolvedPolicyTestRows) Scan(dest ...any) error {
	if len(dest) != 38 {
		return errors.New("resolved policy row: unexpected column count")
	}
	effFrom := time.Date(2026, 1, 1, 0, 0, 0, 0, time.UTC)
	denom, days := "fixed", "22"
	vals := []any{
		r.empID, uuid.New(), r.relID, uuid.New(), "STD", "Standard", uuid.New(), 1, "published", int64(1),
		effFrom, (*time.Time)(nil), effFrom, (*time.Time)(nil), 480, 1, "none", "clamp", 0, false, 0, (*int)(nil),
		true, false, "aggregate", (*uuid.UUID)(nil), "monthly", &denom, &days, (*uuid.UUID)(nil), "standard_day",
		uuid.New(), uuid.New(), 1, (*int)(nil), (*string)(nil), (*string)(nil),
		[]byte(`[{"priority":10,"day_type":"any","days_of_week":127,"multiplier":"1.5"}]`),
	}
	return assignScanValues(dest, vals)
}

func assignScanValues(dest, vals []any) error {
	for i, v := range vals {
		reflect.ValueOf(dest[i]).Elem().Set(reflect.ValueOf(v))
	}
	return nil
}

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
	if !queryExecuted || len(payslips) != 0 {
		t.Fatalf("unexpected payslips result")
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
			if len(args) != 1 || args[0] != empID {
				t.Fatalf("expected single employeeID arg, got %v", args)
			}
			return &fakeRows{}, nil
		},
	}
	repo := PayrollRepository{DB: db}
	payslips, err := repo.ListEmployeePayslips(context.Background(), ids.EmployeeID(empID))
	if err != nil || !queryExecuted || len(payslips) != 0 {
		t.Fatalf("unexpected error=%v executed=%t count=%d", err, queryExecuted, len(payslips))
	}
}

func TestPayrollRepository_AssignPayrollRule_OrganizationAssigneeNullID(t *testing.T) {
	ruleID := uuid.New()
	inserted := false
	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, sql string, args ...any) pgx.Row {
			if strings.Contains(sql, "SELECT EXISTS (SELECT 1 FROM payroll_rule_definitions") {
				return fakeRow{scanFn: func(dest ...any) error { *(dest[0].(*bool)) = true; return nil }}
			}
			if strings.Contains(sql, "INSERT INTO payroll_rule_assignments") {
				inserted = true
				if args[2] != "organization" || (args[3] != nil && args[3] != (*uuid.UUID)(nil)) {
					t.Fatalf("expected organization with nil assignee_id, got %v %v", args[2], args[3])
				}
				return fakeRow{scanFn: func(dest ...any) error { *(dest[0].(*time.Time)) = time.Now().UTC(); return nil }}
			}
			return fakeRow{}
		},
	}
	someID := uuid.New()
	assignment, err := (PayrollRepository{DB: db}).AssignPayrollRule(context.Background(), ids.PayrollRuleID(ruleID), api.AssignPayrollRuleRequest{
		AssigneeType: "organization", AssigneeID: &someID, EffectiveFrom: "2026-01-01",
	})
	if err != nil || !inserted || assignment.AssigneeID != nil {
		t.Fatalf("unexpected result: %+v err=%v", assignment, err)
	}
}

func TestPayrollRepository_AssignPayrollRule_DepartmentAssigneeWithID(t *testing.T) {
	ruleID := uuid.New()
	deptID := uuid.New()
	inserted := false
	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, sql string, args ...any) pgx.Row {
			if strings.Contains(sql, "SELECT EXISTS (SELECT 1 FROM payroll_rule_definitions") {
				return fakeRow{scanFn: func(dest ...any) error { *(dest[0].(*bool)) = true; return nil }}
			}
			if strings.Contains(sql, "INSERT INTO payroll_rule_assignments") {
				inserted = true
				return fakeRow{scanFn: func(dest ...any) error { *(dest[0].(*time.Time)) = time.Now().UTC(); return nil }}
			}
			return fakeRow{}
		},
	}
	assignment, err := (PayrollRepository{DB: db}).AssignPayrollRule(context.Background(), ids.PayrollRuleID(ruleID), api.AssignPayrollRuleRequest{
		AssigneeType: "department", AssigneeID: &deptID, EffectiveFrom: "2026-01-01",
	})
	if err != nil || !inserted || assignment.AssigneeID == nil || *assignment.AssigneeID != deptID {
		t.Fatalf("unexpected result: %+v err=%v", assignment, err)
	}
}
