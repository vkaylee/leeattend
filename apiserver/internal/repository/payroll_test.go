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
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/ids"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/workpolicy"
)

func TestCalculateHourlyPolicyPay(t *testing.T) {
	if got := calculateHourlyPolicyPay(decimal.RequireFromString("100"), 90); !got.Equal(decimal.RequireFromString("150")) {
		t.Fatalf("expected 150 hourly pay, got %s", got)
	}
	if got := calculateHourlyPolicyPay(decimal.Zero, 90); !got.IsZero() {
		t.Fatalf("zero rate must produce zero, got %s", got)
	}
}

func TestCalculateShiftPolicyPay(t *testing.T) {
	if got := calculateShiftPolicyPay(decimal.RequireFromString("75.50"), 3); !got.Equal(decimal.RequireFromString("226.50")) {
		t.Fatalf("expected 226.50 shift pay, got %s", got)
	}
	if got := calculateShiftPolicyPay(decimal.RequireFromString("75.50"), 0); !got.IsZero() {
		t.Fatalf("zero completed shifts must produce zero, got %s", got)
	}
}

func TestCalculatePolicyCreditsUsesExactSegmentMinutes(t *testing.T) {
	policyA := &api.WorkPolicyGroupVersion{
		Name:               "Policy A",
		MinutesPerCredit:   480,
		AttendancePolicyID: uuid.New(),
		WorkingPatternID:   uuid.New(),
	}
	policyB := &api.WorkPolicyGroupVersion{
		Name:               "Policy B",
		MinutesPerCredit:   450,
		AttendancePolicyID: uuid.New(),
		WorkingPatternID:   uuid.New(),
	}
	creditA, err := calculatePolicyCredits(policyA, 90)
	if err != nil {
		t.Fatalf("policy A credits: %v", err)
	}
	creditB, err := calculatePolicyCredits(policyB, 450)
	if err != nil {
		t.Fatalf("policy B credits: %v", err)
	}
	if got := creditA.Add(creditB); !got.Equal(decimal.RequireFromString("1.1875")) {
		t.Fatalf("segment credits = %s, want 1.1875", got)
	}
	if got, err := calculatePolicyCredits(nil, 90); err != nil || !got.IsZero() {
		t.Fatalf("legacy no-policy credits = %s, err=%v; want zero without error", got, err)
	}
}

func TestPayrollStandardDaysForPolicy(t *testing.T) {
	legacy := decimal.NewFromInt(26)
	if got, err := payrollStandardDaysForPolicy(nil, legacy, decimal.Zero); err != nil || !got.Equal(legacy) {
		t.Fatalf("legacy payroll should retain standard days: %s, %v", got, err)
	}

	days := decimal.NewFromInt(22)
	denominator := workpolicy.MonthlyDenominatorFixed
	policy := &api.WorkPolicyGroupVersion{
		PayMethod:           workpolicy.PayMethodMonthly,
		MonthlyDenominator:  &denominator,
		MonthlyStandardDays: &days,
	}
	if got, err := payrollStandardDaysForPolicy(policy, legacy, decimal.Zero); err != nil || !got.Equal(days) {
		t.Fatalf("fixed monthly policy should use configured days: %s, %v", got, err)
	}

	for name, invalid := range map[string]*api.WorkPolicyGroupVersion{
		"hourly": {PayMethod: workpolicy.PayMethodHourly},
		"shift":  {PayMethod: workpolicy.PayMethodShift},
	} {
		t.Run(name, func(t *testing.T) {
			if _, err := payrollStandardDaysForPolicy(invalid, legacy, decimal.Zero); err != api.ErrPayrollValidation {
				t.Fatalf("expected payroll validation error, got %v", err)
			}
		})
	}

	schedule := workpolicy.MonthlyDenominatorSchedule
	scheduledPolicy := &api.WorkPolicyGroupVersion{
		PayMethod:          workpolicy.PayMethodMonthly,
		MonthlyDenominator: &schedule,
	}
	if got, err := payrollStandardDaysForPolicy(scheduledPolicy, legacy, decimal.NewFromInt(21)); err != nil || !got.Equal(decimal.NewFromInt(21)) {
		t.Fatalf("schedule monthly policy should use scheduled workdays: %s, %v", got, err)
	}
	if _, err := payrollStandardDaysForPolicy(scheduledPolicy, legacy, decimal.Zero); err != api.ErrPayrollValidation {
		t.Fatalf("missing scheduled workdays should fail closed, got %v", err)
	}
}

func TestScheduledWorkdayCountInRange(t *testing.T) {
	counts := map[string]int{"2026-01-01": 2, "2026-01-02": 1, "2026-01-04": 1}
	got := scheduledWorkdayCountInRange(counts, time.Date(2026, 1, 1, 0, 0, 0, 0, time.UTC), time.Date(2026, 1, 3, 0, 0, 0, 0, time.UTC))
	if got != 2 {
		t.Fatalf("scheduled workday count = %d, want 2 distinct days", got)
	}
}

func TestResolveMonthlyOTQuotaPrefersCanonicalOverLegacy(t *testing.T) {
	canonical := decimal.RequireFromString("12")
	legacy := decimal.RequireFromString("20")
	if got := resolveMonthlyOTQuota(&canonical, &legacy); got == nil || !got.Equal(canonical) {
		t.Fatalf("canonical quota must win, got %v", got)
	}
	if got := resolveMonthlyOTQuota(nil, &legacy); got == nil || !got.Equal(legacy) {
		t.Fatalf("legacy quota must remain fallback, got %v", got)
	}
	if got := resolveMonthlyOTQuota(nil, nil); got != nil {
		t.Fatalf("missing quota must remain unset, got %v", got)
	}
}

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

func TestImpactSnapshotStalenessIgnoresLegacySnapshotWithoutPolicy(t *testing.T) {
	stale, reasons, err := impactSnapshotStaleness(context.Background(), &fakeDBTX{}, ids.EmploymentID(uuid.New()), 2026, 6, []byte(`{"kpi_inputs":{"actual":"1"}}`))
	if err != nil || stale || len(reasons) != 0 {
		t.Fatalf("legacy snapshot should remain non-stale: stale=%t reasons=%v err=%v", stale, reasons, err)
	}
}

func TestImpactSnapshotStalenessRejectsInvalidPolicyVersion(t *testing.T) {
	stale, reasons, err := impactSnapshotStaleness(context.Background(), &fakeDBTX{}, ids.EmploymentID(uuid.New()), 2026, 6, []byte(`{"policy":{"version_id":"bad"}}`))
	if err != nil || !stale || len(reasons) != 1 || reasons[0] != "policy_version_invalid" {
		t.Fatalf("invalid policy snapshot should be stale: stale=%t reasons=%v err=%v", stale, reasons, err)
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

func TestPayrollRepository_CalculateEmployeePayroll_UsesEmploymentDepartmentHistory(t *testing.T) {
	querySeen := false
	employeeID := uuid.New()
	employmentID := uuid.New()
	departmentID := uuid.New()
	employerID := uuid.New()
	tx := &payrollPeriodTestTx{}
	tx.queryRowFn = func(_ context.Context, sql string, _ ...any) pgx.Row {
		if strings.Contains(sql, "SELECT EXISTS (SELECT 1 FROM employees") {
			return fakeRow{scanFn: func(dest ...any) error { *(dest[0].(*bool)) = true; return nil }}
		}
		if strings.Contains(sql, "SELECT status FROM payroll_periods") {
			return fakeRow{scanFn: func(dest ...any) error { *(dest[0].(*string)) = "open"; return nil }}
		}
		if strings.Contains(sql, "FROM employments WHERE employee_id") {
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*uuid.UUID)) = employmentID
				*(dest[1].(**uuid.UUID)) = &departmentID
				*(dest[2].(*uuid.UUID)) = employerID
				return nil
			}}
		}
		if strings.Contains(sql, "INSERT INTO payslips") {
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*time.Time)) = time.Now().UTC()
				*(dest[1].(*time.Time)) = time.Now().UTC()
				return nil
			}}
		}
		return fakeRow{}
	}
	tx.queryFn = func(_ context.Context, sql string, _ ...any) (pgx.Rows, error) {
		if strings.Contains(sql, "FROM employee_salary_components") {
			return &fakeRows{}, nil
		}
		if strings.Contains(sql, "FROM labor_policy_memberships") {
			return &payrollPolicyRows{}, nil
		}
		if strings.Contains(sql, "FROM employments\nWHERE employee_id") {
			querySeen = true
			if strings.Contains(sql, "employee_department_history") || !strings.Contains(sql, "deleted_at IS NULL") {
				t.Fatalf("expected canonical employment department query, got %s", sql)
			}
			return &fakeRows{rows: [][]any{{departmentID, "2026-01-01"}}}, nil
		}
		return &fakeRows{}, nil
	}
	_, err := (PayrollRepository{DB: tx}).CalculateEmployeePayroll(context.Background(), ids.EmployeeID(employeeID), 2026, 1, ids.UserID(uuid.New()))
	if err != nil {
		t.Fatalf("calculate payroll: %v", err)
	}
	if !querySeen {
		t.Fatal("expected employment department history query")
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
	departmentID := uuid.New()
	queriedAfterGuard := false
	tx := &payrollPeriodTestTx{}
	tx.queryRowFn = func(_ context.Context, sql string, _ ...any) pgx.Row {
		switch {
		case strings.Contains(sql, "SELECT EXISTS (SELECT 1 FROM employees"):
			return fakeRow{scanFn: func(dest ...any) error { *(dest[0].(*bool)) = true; return nil }}
		case strings.Contains(sql, "FROM employments WHERE employee_id"):
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*uuid.UUID)) = employmentID
				*(dest[1].(**uuid.UUID)) = &departmentID
				*(dest[2].(*uuid.UUID)) = employerID
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

func payrollCalculationFixture(employeeID, employmentID, employerID uuid.UUID, periodStatus, payslipStatus string) *payrollPeriodTestTx {
	departmentID := uuid.New()
	tx := &payrollPeriodTestTx{}
	tx.queryRowFn = func(_ context.Context, sql string, _ ...any) pgx.Row {
		switch {
		case strings.Contains(sql, "SELECT EXISTS (SELECT 1 FROM employees"):
			return fakeRow{scanFn: func(dest ...any) error { *(dest[0].(*bool)) = true; return nil }}
		case strings.Contains(sql, "FROM employments WHERE employee_id"):
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*uuid.UUID)) = employmentID
				*(dest[1].(**uuid.UUID)) = &departmentID
				*(dest[2].(*uuid.UUID)) = employerID
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
		if strings.Contains(sql, "FROM labor_policy_memberships") {
			return &payrollPolicyRows{}, nil
		}
		return &fakeRows{}, nil
	}
	return tx
}

type payrollPolicyRows struct {
	read bool
}

func (r *payrollPolicyRows) Close()                                       {}
func (r *payrollPolicyRows) Err() error                                   { return nil }
func (r *payrollPolicyRows) CommandTag() pgconn.CommandTag                { return pgconn.CommandTag{} }
func (r *payrollPolicyRows) FieldDescriptions() []pgconn.FieldDescription { return nil }
func (r *payrollPolicyRows) Next() bool {
	if r.read {
		return false
	}
	r.read = true
	return true
}
func (r *payrollPolicyRows) Scan(dest ...any) error {
	if len(dest) != 26 {
		return errors.New("payroll policy row: unexpected column count")
	}
	groupID := uuid.New()
	creditID := uuid.New()
	attendanceID := uuid.New()
	workingPatternID := uuid.New()
	payBasisID := uuid.New()
	overtimeID := uuid.Nil
	method := "monthly"
	denominator := "fixed"
	days := "26"
	policyFrom := "2026-01-01"
	membershipFrom := "2026-01-01"
	otAggregation := "per_segment"
	*(dest[0].(*uuid.UUID)) = uuid.New()
	*(dest[1].(*uuid.UUID)) = groupID
	*(dest[2].(*string)) = "Test policy"
	*(dest[3].(*uuid.UUID)) = creditID
	*(dest[4].(*string)) = "Test credit policy"
	*(dest[5].(*int)) = 480
	*(dest[6].(*uuid.UUID)) = attendanceID
	*(dest[7].(*uuid.UUID)) = workingPatternID
	*(dest[8].(*uuid.UUID)) = payBasisID
	*(dest[9].(*string)) = "Monthly"
	*(dest[10].(**string)) = &method
	*(dest[11].(**string)) = &denominator
	*(dest[12].(**string)) = &days
	*(dest[13].(**uuid.UUID)) = nil
	*(dest[14].(*uuid.UUID)) = overtimeID
	*(dest[15].(*string)) = policyFrom
	*(dest[16].(**string)) = nil
	*(dest[17].(*int)) = 0
	*(dest[18].(*bool)) = false
	*(dest[19].(*int)) = 0
	*(dest[20].(**int)) = nil
	*(dest[21].(*bool)) = false
	*(dest[22].(*bool)) = false
	*(dest[23].(*string)) = otAggregation
	*(dest[24].(*string)) = membershipFrom
	*(dest[25].(**string)) = nil
	return nil
}
func (r *payrollPolicyRows) Values() ([]any, error) { return nil, nil }
func (r *payrollPolicyRows) RawValues() [][]byte    { return nil }
func (r *payrollPolicyRows) Conn() *pgx.Conn        { return nil }

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
			if !strings.Contains(sql, "FROM payslips p JOIN employments e ON e.id=p.employment_id") || !strings.Contains(sql, "JOIN employee_tenures") || !strings.Contains(sql, "e.employee_id=$1") {
				t.Fatalf("unexpected SQL join/filter: %s", sql)
			}
			if len(args) != 1 || args[0] != empID {
				t.Fatalf("expected single employeeID arg, got %v", args)
			}
			return &fakeRows{}, nil
		},
	}
	repo := PayrollRepository{DB: db}
	payslips, err := repo.ListEmployeePayslips(context.Background(), ids.EmployeeID(empID))
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
	cfg, err := repo.SetEmployeePayrollConfig(context.Background(), ids.EmployeeID(empID), api.SetEmployeePayrollConfigRequest{
		StdWorkingDays: decimal.NewFromInt(22),
		HoursPerDay:    decimal.NewFromInt(8),
		EffectiveDate:  "2026-06-01",
	}, ids.UserID(actorID))
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
	assignment, err := repo.AssignPayrollRule(context.Background(), ids.PayrollRuleID(ruleID), api.AssignPayrollRuleRequest{
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
	assignment, err := repo.AssignPayrollRule(context.Background(), ids.PayrollRuleID(ruleID), api.AssignPayrollRuleRequest{
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
